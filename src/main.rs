mod app;
mod cache;
mod config;
mod events;
mod spotify;
mod ui;
#[cfg(test)]
mod tests;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, sync::Arc};
use tracing::error;
use tracing_subscriber::{fmt, EnvFilter};

use crate::{
    app::App,
    cache::Cache,
    config::Config,
};

#[tokio::main]
async fn main() -> Result<()> {
    // ── Logging setup ────────────────────────────────────────────────────────
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("vibes=info"));
    // Write logs to file so they don't corrupt the TUI
    let log_file = std::fs::File::create("/tmp/vibes.log").ok();
    if let Some(file) = log_file {
        fmt()
            .with_env_filter(filter)
            .with_writer(std::sync::Mutex::new(file))
            .init();
    }

    // ── Load config ──────────────────────────────────────────────────────────
    let config = Config::load()?;
    let redis_url = config.redis_url.clone();

    // ── Try Redis (optional — app works without it) ──────────────────────────
    let cache = Arc::new(match Cache::new(&redis_url) {
        Ok(c) => {
            if c.ping().await {
                tracing::info!("Redis connected at {redis_url}");
                c
            } else {
                tracing::warn!("Redis not reachable — token caching disabled");
                // Use a no-op cache key prefix so it silently fails
                Cache::new("redis://127.0.0.1:0").unwrap_or_else(|_| Cache::new("redis://127.0.0.1:6379").unwrap())
            }
        }
        Err(_) => {
            tracing::warn!("Redis unavailable — running without token cache");
            Cache::new("redis://127.0.0.1:6379").unwrap()
        }
    });

    // ── Terminal setup ────────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ── Panic hook to restore terminal on crash ──────────────────────────────
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        disable_raw_mode().ok();
        execute!(io::stdout(), LeaveAlternateScreen).ok();
        original_hook(panic_info);
    }));

    // ── Run the app ──────────────────────────────────────────────────────────
    let result = {
        let mut app = App::new(config, cache).await?;
        app.run(&mut terminal).await
    };

    // ── Restore terminal ─────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        error!("App error: {e:?}");
        eprintln!("\n\x1b[31mvibes crashed:\x1b[0m {e}");
        eprintln!("Check /tmp/vibes.log for details");
    }

    Ok(())
}
