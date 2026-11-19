// Desktop client. It has NO database access and NO secrets:
// all auth goes through the Millida auth server over HTTPS.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;
use tauri::State;

// Public server address, baked in at build time:
//   MILLIDA_API_URL=https://auth.example.com cargo tauri build
// The URL is not a secret, so embedding it is fine.
const API_URL: &str = match option_env!("MILLIDA_API_URL") {
    Some(url) => url,
    None => "http://127.0.0.1:8080",
};

struct Api {
    http: reqwest::Client,
}

impl Api {
    fn url(path: &str) -> String {
        format!("{}{}", API_URL.trim_end_matches('/'), path)
    }
}

#[derive(Serialize, Deserialize)]
struct AuthResult {
    username: String,
    token: String,
}

#[derive(Deserialize)]
struct MeResponse {
    username: String,
}

#[derive(Deserialize)]
struct ErrorBody {
    error: String,
}

fn network_error(_: reqwest::Error) -> String {
    "Cannot reach the server. Check your internet connection.".into()
}

/// Turns a server response into Ok(T) or a human-readable error.
async fn read_json<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T, String> {
    let status = resp.status();

    if status.is_success() {
        return resp
            .json::<T>()
            .await
            .map_err(|_| "Unexpected server response".to_string());
    }

    Err(match resp.json::<ErrorBody>().await {
        Ok(body) => body.error,
        Err(_) => format!("Server error ({})", status.as_u16()),
    })
}

// Same rules as on the server; checked here only for instant feedback.
fn validate_creds(u: &str, p: &str) -> Result<(), String> {
    if !(3..=16).contains(&u.len())
        || !u
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("Username: 3–16 chars, letters/digits/_ only".into());
    }

    if p.len() < 6 || p.len() > 128 {
        return Err("Password must be 6–128 characters".into());
    }

    Ok(())
}

// ───────────────── REGISTER ─────────────────

#[tauri::command]
async fn secure_register(
    api: State<'_, Api>,
    username: String,
    password: String,
    email: String,
) -> Result<AuthResult, String> {
    validate_creds(&username, &password)?;

    if email.len() < 5 || !email.contains('@') {
        return Err("Invalid email".into());
    }

    let resp = api
        .http
        .post(Api::url("/api/register"))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
            "email": email,
        }))
        .send()
        .await
        .map_err(network_error)?;

    read_json(resp).await
}

// ───────────────── LOGIN ─────────────────

#[tauri::command]
async fn secure_login(
    api: State<'_, Api>,
    username: String,
    password: String,
) -> Result<AuthResult, String> {
    validate_creds(&username, &password)?;

    let resp = api
        .http
        .post(Api::url("/api/login"))
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .map_err(network_error)?;

    read_json(resp).await
}

// ───────────────── LOGOUT ─────────────────

#[tauri::command]
async fn secure_logout(api: State<'_, Api>, token: String) -> Result<(), String> {
    let resp = api
        .http
        .post(Api::url("/api/logout"))
        .bearer_auth(&token)
        .send()
        .await
        .map_err(network_error)?;

    // 401 means the session is already gone — that's fine for logout.
    if resp.status().is_success() || resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        Ok(())
    } else {
        Err(format!("Logout failed ({})", resp.status().as_u16()))
    }
}

// ───────────────── LAUNCH ─────────────────

#[tauri::command]
async fn secure_launch(
    api: State<'_, Api>,
    server_ip: String,
    token: String,
) -> Result<String, String> {
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

    // The server checks the session; the client cannot fake this.
    let resp = api
        .http
        .get(Api::url("/api/me"))
        .bearer_auth(&token)
        .send()
        .await
        .map_err(network_error)?;

    let me: MeResponse = read_json(resp).await?;

    tokio::time::sleep(Duration::from_millis(1200)).await;

    Ok(format!(
        "Launched {} as {}",
        server_ip,
        me.username
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
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        // Release builds refuse plain http, so tokens never travel unencrypted.
        .https_only(!cfg!(debug_assertions))
        .build()
        .expect("Failed to create HTTP client");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Api { http })
        .invoke_handler(tauri::generate_handler![
            secure_register,
            secure_login,
            secure_logout,
            secure_launch,
            validate_skin_upload
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}