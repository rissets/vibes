use anyhow::Result;
use rspotify::{
    model::{PlaylistId, SavedTrack, SimplifiedPlaylist, PlaylistItem},
    prelude::*,
    AuthCodePkceSpotify,
};
use futures::{StreamExt, TryStreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Library {
    spotify: Arc<Mutex<AuthCodePkceSpotify>>,
}

impl Library {
    pub fn new(spotify: Arc<Mutex<AuthCodePkceSpotify>>) -> Self {
        Library { spotify }
    }

    pub async fn get_liked_songs(&self, limit: u32) -> Result<Vec<SavedTrack>> {
        let sp = self.spotify.lock().await;
        let stream = sp.current_user_saved_tracks(None); // Removed Market::FromToken
        let tracks: Vec<SavedTrack> = stream
            .take(limit as usize)
            .try_collect::<Vec<_>>()
            .await?; // Proper error propagation
        Ok(tracks)
    }

    pub async fn get_user_playlists(&self) -> Result<Vec<SimplifiedPlaylist>> {
        let sp = self.spotify.lock().await;
        let stream = sp.current_user_playlists();
        let playlists: Vec<SimplifiedPlaylist> = stream
            .try_collect()
            .await?;
        Ok(playlists)
    }

    pub async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<PlaylistItem>> {
        let sp = self.spotify.lock().await;
        let pid = PlaylistId::from_id(playlist_id)?;
        let stream = sp.playlist_items(pid, None, None); // Removed Market::FromToken
        let items: Vec<PlaylistItem> = stream
            .try_collect()
            .await?;
        Ok(items)
    }
}
