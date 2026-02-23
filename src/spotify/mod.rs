use anyhow::Result;
use rspotify::{
    clients::{BaseClient, OAuthClient},
    scopes, AuthCodePkceSpotify, Config as SpotifyConfig, Credentials, OAuth,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::cache::Cache;
use crate::config::Config;
use self::auth::PkceChallenge;

pub mod auth;
pub mod library;
pub mod player;
pub mod queue;
pub mod search;
pub mod vibes;

const TOKEN_CACHE_KEY: &str = "vibes:spotify_token";

pub async fn build_spotify_client(
    config: &Config,
    cache: &Cache,
) -> Result<(Arc<Mutex<AuthCodePkceSpotify>>, Option<String>)> {
    let creds = Credentials::new(&config.client_id, &config.client_secret);

    let scopes = scopes!(
        "user-read-playback-state",
        "user-modify-playback-state",
        "user-read-currently-playing",
        "user-library-read",
        "user-library-modify",
        "playlist-read-private",
        "playlist-read-collaborative",
        "user-read-private",
        "user-read-email",
        "streaming"
    );

    let oauth = OAuth {
        redirect_uri: config.redirect_uri.clone(),
        scopes,
        ..Default::default()
    };

    let sp_config = SpotifyConfig {
        token_refreshing: true,
        ..Default::default()
    };

    let mut spotify = AuthCodePkceSpotify::with_config(creds, oauth, sp_config);

    // Try loading cached token from Redis
    if let Ok(Some(token_json)) = cache.get(TOKEN_CACHE_KEY).await {
        if let Ok(token) = serde_json::from_str::<rspotify::Token>(&token_json) {
            info!("Loaded cached token from Redis");
            *spotify.token.lock().await.unwrap() = Some(token.clone());
            
            // Attempt to refresh the token to ensure it's still valid
            match spotify.refetch_token().await {
                Ok(_) => {
                    let client = Arc::new(Mutex::new(spotify));
                    return Ok((client, None));
                }
                Err(e) => {
                    tracing::warn!("Failed to refresh cached token ({}), clearing cache and re-authenticating", e);
                    // Clear invalid token from memory and cache
                    *spotify.token.lock().await.unwrap() = None;
                    cache.delete(TOKEN_CACHE_KEY).await.ok();
                }
            }
        }
    }

    // Generate PKCE challenge and auth URL
    let pkce = PkceChallenge::new();
    let url = spotify.get_authorize_url(Some(pkce.challenge.len()))?;
    info!("Auth URL generated, opening browser...");

    Ok((Arc::new(Mutex::new(spotify)), Some(url)))
}

pub async fn complete_auth(
    spotify: Arc<Mutex<AuthCodePkceSpotify>>,
    code: &str,
    _verifier: &str,
    cache: &Cache,
) -> Result<()> {
    let sp = spotify.lock().await;
    sp.request_token(code).await?;

    // Cache the token
    let token_guard = sp.token.lock().await.unwrap();
    if let Some(ref token) = *token_guard {
        let token_json = serde_json::to_string(token)?;
        drop(token_guard);
        cache.set(TOKEN_CACHE_KEY, &token_json, Some(3600 * 24)).await.ok();
        info!("Token saved to Redis cache");
    }

    Ok(())
}
