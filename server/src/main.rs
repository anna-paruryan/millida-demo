//! Millida auth server.
//!
//! The only component that talks to the database. The desktop client
//! (Tauri) never sees DATABASE_URL — it calls this server over HTTPS.

use std::{env, net::SocketAddr, sync::LazyLock, time::Duration};

use argon2::{
    password_hash::{
        rand_core::{OsRng, RngCore},
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use axum::{
    extract::{DefaultBodyLimit, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use tracing::{error, info};

const TOKEN_BYTES: usize = 32; // 256-bit token -> 64 hex chars

// Used when the user does not exist, so a failed login takes the same time
// whether or not the username is registered (prevents user enumeration).
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    hash_password("dummy-password-for-timing").expect("dummy hash")
});

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

// ───────────────── ERRORS ─────────────────

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    /// Logs the real error on the server, returns a generic message to the client.
    fn internal(context: &str, err: impl std::fmt::Display) -> Self {
        error!("{context}: {err}");
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

type ApiResult<T> = Result<T, ApiError>;

// ───────────────── DTOs ─────────────────

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
    email: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    username: String,
    token: String,
}

#[derive(Serialize)]
struct MeResponse {
    username: String,
}

// ───────────────── HELPERS ─────────────────

fn validate_creds(u: &str, p: &str) -> Result<(), ApiError> {
    if !(3..=16).contains(&u.len())
        || !u.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(ApiError::bad_request(
            "Username: 3–16 chars, letters/digits/_ only",
        ));
    }

    // Upper bound protects the server from huge Argon2 inputs.
    if p.len() < 6 || p.len() > 128 {
        return Err(ApiError::bad_request(
            "Password must be 6–128 characters",
        ));
    }

    Ok(())
}

fn validate_email(email: &str) -> Result<(), ApiError> {
    let ok = (5..=254).contains(&email.len())
        && email.contains('@')
        && !email.starts_with('@')
        && !email.ends_with('@')
        && !email.chars().any(char::is_whitespace);

    if ok {
        Ok(())
    } else {
        Err(ApiError::bad_request("Invalid email"))
    }
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

fn generate_token() -> String {
    let mut bytes = [0u8; TOKEN_BYTES];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Only the SHA-256 of a token is stored, so a database leak
/// does not hand out working sessions.
fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn bearer_token(headers: &HeaderMap) -> ApiResult<String> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|t| t.len() == TOKEN_BYTES * 2 && t.chars().all(|c| c.is_ascii_hexdigit()))
        .ok_or_else(|| ApiError::unauthorized("Session expired, please log in again"))?;

    Ok(token.to_string())
}

async fn create_session(pool: &PgPool, user_id: i32) -> ApiResult<String> {
    let token = generate_token();

    sqlx::query(
        "INSERT INTO sessions (user_id, token_hash, expires_at)
         VALUES ($1, $2, NOW() + INTERVAL '1 day')",
    )
    .bind(user_id)
    .bind(hash_token(&token))
    .execute(pool)
    .await
    .map_err(|e| ApiError::internal("create session", e))?;

    Ok(token)
}

// ───────────────── HANDLERS ─────────────────

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    validate_creds(&req.username, &req.password)?;
    validate_email(&req.email)?;

    let password = req.password;
    let password_hash = tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|e| ApiError::internal("hash task", e))?
        .map_err(|e| ApiError::internal("hash password", e))?;

    let row = sqlx::query(
        "INSERT INTO users (username, password_hash, email)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(&req.username)
    .bind(&password_hash)
    .bind(&req.email)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            ApiError::new(StatusCode::CONFLICT, "Username or email already taken")
        }
        _ => ApiError::internal("insert user", e),
    })?;

    let id: i32 = row.get("id");
    let token = create_session(&state.pool, id).await?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse { username: req.username, token }),
    ))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    const INVALID: &str = "Invalid username or password";

    if validate_creds(&req.username, &req.password).is_err() {
        return Err(ApiError::unauthorized(INVALID));
    }

    let row = sqlx::query("SELECT id, password_hash FROM users WHERE username = $1")
        .bind(&req.username)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| ApiError::internal("select user", e))?;

    let (user_id, stored_hash) = match row {
        Some(r) => (Some(r.get::<i32, _>("id")), r.get::<String, _>("password_hash")),
        None => (None, DUMMY_HASH.clone()),
    };

    let password = req.password;
    let password_ok = tokio::task::spawn_blocking(move || verify_password(&password, &stored_hash))
        .await
        .map_err(|e| ApiError::internal("verify task", e))?;

    let user_id = match (user_id, password_ok) {
        (Some(id), true) => id,
        _ => return Err(ApiError::unauthorized(INVALID)),
    };

    let token = create_session(&state.pool, user_id).await?;

    Ok(Json(AuthResponse { username: req.username, token }))
}

async fn me(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<Json<MeResponse>> {
    let token = bearer_token(&headers)?;

    let row = sqlx::query(
        "SELECT u.username
         FROM sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = $1 AND s.expires_at > NOW()",
    )
    .bind(hash_token(&token))
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::internal("select session", e))?
    .ok_or_else(|| ApiError::unauthorized("Session expired, please log in again"))?;

    Ok(Json(MeResponse { username: row.get("username") }))
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<StatusCode> {
    let token = bearer_token(&headers)?;

    sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
        .bind(hash_token(&token))
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::internal("delete session", e))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn health() -> &'static str {
    "ok"
}

// ───────────────── APP ─────────────────

fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/register", post(register))
        .route("/api/login", post(login))
        .route("/api/me", get(me))
        .route("/api/logout", post(logout))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .with_state(state)
}

/// Removes expired sessions once an hour.
fn spawn_session_cleanup(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            match sqlx::query("DELETE FROM sessions WHERE expires_at <= NOW()")
                .execute(&pool)
                .await
            {
                Ok(r) if r.rows_affected() > 0 => {
                    info!("removed {} expired sessions", r.rows_affected())
                }
                Ok(_) => {}
                Err(e) => error!("session cleanup failed: {e}"),
            }
        }
    });
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    // Most hosting platforms pass the port via $PORT.
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    info!("connected to database");

    LazyLock::force(&DUMMY_HASH);
    spawn_session_cleanup(pool.clone());

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    info!("listening on {addr}");

    axum::serve(listener, app(AppState { pool }))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}
