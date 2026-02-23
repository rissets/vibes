use anyhow::Result;
use rspotify::{
    model::{Market, SearchType, SearchResult, FullTrack},
    prelude::*,
    AuthCodePkceSpotify,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::app::state::VibesMood;

pub struct Vibes {
    spotify: Arc<Mutex<AuthCodePkceSpotify>>,
}

impl Vibes {
    pub fn new(spotify: Arc<Mutex<AuthCodePkceSpotify>>) -> Self {
        Vibes { spotify }
    }

    /// Since Spotify deprecated the Recommendations API (Nov 2024),
    /// we use search with mood-appropriate keywords + genres instead.
    pub async fn get_recommendations(&self, mood: &VibesMood) -> Result<Vec<FullTrack>> {
        let sp = self.spotify.lock().await;

        // Build a mood-based search query
        let query = match mood {
            VibesMood::Chill => "genre:chill lo-fi relaxing",
            VibesMood::Hype  => "genre:edm hype energy bass",
            VibesMood::Focus => "genre:classical focus study ambient",
            VibesMood::Happy => "genre:pop happy upbeat feel good",
            VibesMood::Dark  => "genre:metal dark heavy intense",
        };

        let result = sp
            .search(
                query,
                SearchType::Track,
                Some(Market::FromToken),
                None,  // include_external
                Some(30),
                Some(0),
            )
            .await?;

        if let SearchResult::Tracks(page) = result {
            Ok(page.items)
        } else {
            Ok(vec![])
        }
    }
}
