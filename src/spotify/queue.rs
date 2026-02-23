use anyhow::Result;
use rspotify::{
    model::{FullTrack, PlayableItem, TrackId},
    prelude::*,
    AuthCodePkceSpotify,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Queue {
    spotify: Arc<Mutex<AuthCodePkceSpotify>>,
}

impl Queue {
    pub fn new(spotify: Arc<Mutex<AuthCodePkceSpotify>>) -> Self {
        Queue { spotify }
    }

    pub async fn get_queue(&self) -> Result<Vec<FullTrack>> {
        let sp = self.spotify.lock().await;
        let queue = sp.current_user_queue().await?;
        let tracks: Vec<FullTrack> = queue
            .queue
            .into_iter()
            .filter_map(|item| {
                if let PlayableItem::Track(t) = item {
                    Some(t)
                } else {
                    None
                }
            })
            .collect();
        Ok(tracks)
    }

    pub async fn add_to_queue(&self, track_uri: &str) -> Result<()> {
        let sp = self.spotify.lock().await;
        let track_id = TrackId::from_uri(track_uri)?;
        sp.add_item_to_queue(PlayableId::Track(track_id), None).await?;
        Ok(())
    }
}
