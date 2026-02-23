use anyhow::Result;
use dotenvy::dotenv;

#[derive(Debug, Clone)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub redis_url: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv().ok(); // Try loading .env file, ignore if it doesn't exist (e.g. env vars set manually)

        Ok(Config {
            client_id: std::env::var("SPOTIFY_CLIENT_ID")
                .expect("SPOTIFY_CLIENT_ID is missing from .env or environment!"),
            client_secret: std::env::var("SPOTIFY_CLIENT_SECRET")
                .expect("SPOTIFY_CLIENT_SECRET is missing from .env or environment!"),
            redirect_uri: std::env::var("SPOTIFY_REDIRECT_URI")
                .unwrap_or_else(|_| "http://127.0.0.1:8989/login".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        })
    }
}
