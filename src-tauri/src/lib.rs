
use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

// TEMPORARY: move this secret to a trusted backend before production.
const JWT_SECRET: &[u8] = b"millida-demo-secret-change-me";

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    uid: i32,
    exp: usize,
}

#[derive(Serialize)]
struct AuthResult {
    username: String,
    token: String,
}

fn now() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize
}

fn validate_creds(u: &str, p: &str) -> Result<(), String> {
    if !(3..=16).contains(&u.len())
        || !u
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("Username: 3–16 chars, letters/digits/_ only".into());
    }

    if p.len() < 6 {
        return Err("Password must be at least 6 characters".into());
    }

    Ok(())
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

fn issue_token(username: &str, uid: i32) -> Result<String, String> {
    let claims = Claims {
        sub: username.to_string(),
        uid,
        exp: now() + 3600 * 24,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|e| e.to_string())
}

async fn insert_session(
    pool: &PgPool,
    user_id: i32,
    token: &str,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO sessions (user_id, jwt_token, expires_at)
         VALUES ($1, $2, NOW() + INTERVAL '1 day')",
    )
    .bind(user_id)
    .bind(token)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create session: {e}"))?;

    Ok(())
}

// ───────────────── REGISTER ─────────────────

#[tauri::command]
async fn secure_register(
    pool: State<'_, PgPool>,
    username: String,
    password: String,
    email: String,
) -> Result<AuthResult, String> {
    validate_creds(&username, &password)?;

    if email.len() < 5 || !email.contains('@') {
        return Err("Invalid email".into());
    }

    let password_for_hash = password.clone();

    let password_hash = tokio::task::spawn_blocking(move || {
        hash_password(&password_for_hash)
    })
    .await
    .map_err(|e| format!("Password hashing task failed: {e}"))??;

    let row = sqlx::query(
        "INSERT INTO users (username, password_hash, email)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(&username)
    .bind(&password_hash)
    .bind(&email)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| {
        format!("Registration failed. Username or email may already exist: {e}")
    })?;

    let id: i32 = row.get("id");

    let token = issue_token(&username, id)?;

    insert_session(pool.inner(), id, &token).await?;

    Ok(AuthResult { username, token })
}

// ───────────────── LOGIN ─────────────────

#[tauri::command]
async fn secure_login(
    pool: State<'_, PgPool>,
    username: String,
    password: String,
) -> Result<AuthResult, String> {
    validate_creds(&username, &password)?;

    let row = sqlx::query(
        "SELECT id, password_hash FROM users WHERE username = $1",
    )
    .bind(&username)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| format!("Database error: {e}"))?
    .ok_or_else(|| "Invalid username or password".to_string())?;

    let id: i32 = row.get("id");
    let password_hash: String = row.get("password_hash");

    let password_for_verify = password.clone();

    let is_valid = tokio::task::spawn_blocking(move || {
        verify_password(&password_for_verify, &password_hash)
    })
    .await
    .map_err(|e| format!("Password verification task failed: {e}"))?;

    if !is_valid {
        return Err("Invalid username or password".into());
    }

    let token = issue_token(&username, id)?;

    insert_session(pool.inner(), id, &token).await?;

    Ok(AuthResult { username, token })
}

// ───────────────── LAUNCH ─────────────────

#[tauri::command]
async fn secure_launch(
    server_ip: String,
    token: String,
) -> Result<String, String> {
    let data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map_err(|_| "Session expired, please log in again".to_string())?;

    let ok = !server_ip.is_empty()
        && server_ip.len() <= 64
        && server_ip
            .chars()
            .all(|c| {
                c.is_ascii_alphanumeric()
                    || c == '.'
                    || c == '-'
                    || c == ':'
            });

    if !ok {
        return Err("Invalid server address".into());
    }

    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;

    Ok(format!(
        "Launched {} as {}",
        server_ip,
        data.claims.sub
    ))
}

// ───────────────── SKIN ─────────────────

#[tauri::command]
fn validate_skin_upload(
    file_name: String,
    bytes: Vec<u8>,
) -> Result<String, String> {
    if file_name.contains("..")
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains('\0')
    {
        return Err("Unsafe file name".into());
    }

    if bytes.len() >= 1_048_576 {
        return Err("Skin must be < 1MB".into());
    }

    const PNG: [u8; 8] = [
        0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A,
    ];

    if bytes.len() < 24 || bytes[..8] != PNG {
        return Err("Not a PNG file".into());
    }

    let width = u32::from_be_bytes([
        bytes[16], bytes[17], bytes[18], bytes[19],
    ]);

    let height = u32::from_be_bytes([
        bytes[20], bytes[21], bytes[22], bytes[23],
    ]);

    if width != 64 || height != 64 {
        return Err(format!(
            "Skin must be 64x64, got {width}x{height}"
        ));
    }

    Ok(format!(
        "Skin OK (PNG, 64x64, {} B)",
        bytes.len()
    ))
}

// ───────────────── APP ─────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL not set");

    let pool = tauri::async_runtime::block_on(async {
        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to Neon")
    });

    println!("Connected to Neon");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(pool)
        .invoke_handler(tauri::generate_handler![
            secure_register,
            secure_login,
            secure_launch,
            validate_skin_upload
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
