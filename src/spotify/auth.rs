use anyhow::{Context, Result};
use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::info;

fn generate_code_verifier() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..64).map(|_| rng.gen_range(0u8..=255u8)).collect();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
}

fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

#[allow(dead_code)]
pub struct AuthResult {
    pub code: String,
    pub state: String,
}

/// Starts a local HTTP server on port 8989 waiting for the Spotify redirect
pub async fn wait_for_auth_code() -> Result<AuthResult> {
    let listener = TcpListener::bind("127.0.0.1:8989")
        .await
        .context("Failed to bind port 8989 for OAuth redirect")?;

    info!("Waiting for Spotify auth redirect on http://127.0.0.1:8989/login ...");

    let (mut stream, _) = listener.accept().await?;
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);

    // Parse the GET line: GET /login?code=...&state=... HTTP/1.1
    let query = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|path| path.split('?').nth(1))
        .unwrap_or("");

    let params: HashMap<&str, &str> = query
        .split('&')
        .filter_map(|kv| {
            let mut parts = kv.splitn(2, '=');
            Some((parts.next()?, parts.next()?))
        })
        .collect();

    let code = params.get("code").context("No code in redirect")?.to_string();
    let state = params.get("state").unwrap_or(&"").to_string();

    // Send a nice response page
    let body = r#"<!DOCTYPE html>
<html>
<head>
  <style>
    body { background: #0D0D0D; color: #00F5FF; font-family: monospace; 
           display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }
    .card { text-align: center; border: 1px solid #9B5DE5; padding: 40px; border-radius: 12px; }
    h1 { color: #9B5DE5; }
    p { color: #aaa; }
  </style>
</head>
<body>
  <div class="card">
    <h1>🎵 vibes</h1>
    <p>Authentication successful! You can close this tab.</p>
    <p style="color:#00F5FF">Return to your terminal ✨</p>
  </div>
</body>
</html>"#;

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    Ok(AuthResult { code, state })
}

pub struct PkceChallenge {
    pub verifier: String,
    pub challenge: String,
}

impl PkceChallenge {
    pub fn new() -> Self {
        let verifier = generate_code_verifier();
        let challenge = generate_code_challenge(&verifier);
        PkceChallenge { verifier, challenge }
    }
}
