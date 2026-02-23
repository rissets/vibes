use anyhow::Result;
use rspotify::{
    model::{FullTrack, SearchResult, SearchType},
    prelude::*,
    AuthCodePkceSpotify,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Search {
    spotify: Arc<Mutex<AuthCodePkceSpotify>>,
}

impl Search {
    pub fn new(spotify: Arc<Mutex<AuthCodePkceSpotify>>) -> Self {
        Search { spotify }
    }

    pub async fn search_tracks(&self, query: &str, limit: u32) -> Result<Vec<FullTrack>> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        let sp = self.spotify.lock().await;
        let result = sp
            .search(
                query,
                SearchType::Track,
                None,
                None,
                Some(limit),
                None,
            )
            .await?;

        let tracks = match result {
            SearchResult::Tracks(page) => page.items,
            _ => vec![],
        };
        Ok(tracks)
    }
}
