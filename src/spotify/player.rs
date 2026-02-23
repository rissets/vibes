use anyhow::{anyhow, Result};
use rspotify::{
    model::{
        AdditionalType, Market, PlayableItem, TrackId,
    },
    prelude::*,
    AuthCodePkceSpotify,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::app::state::CurrentTrack;

pub struct Player {
    spotify: Arc<Mutex<AuthCodePkceSpotify>>,
}

impl Player {
    pub fn new(spotify: Arc<Mutex<AuthCodePkceSpotify>>) -> Self {
        Player { spotify }
    }

    pub async fn get_current_playback(&self) -> Result<Option<CurrentTrack>> {
        let sp = self.spotify.lock().await;
        let additional = [AdditionalType::Track];
        let playback = sp
            .current_playback(Some(Market::FromToken), Some(additional.iter()))
            .await?;

        if let Some(ctx) = playback {
            let device_vol = ctx.device.volume_percent.map(|v| v.clamp(0, 100) as u8);
            if let Some(PlayableItem::Track(track)) = ctx.item {
                let track_id = track.id.as_ref().map(|id| id.to_string());
                let is_playing = ctx.is_playing;
                let progress_ms = ctx.progress.map(|p| p.num_milliseconds() as u32).unwrap_or(0);

                let ct = CurrentTrack {
                    id: track_id,
                    name: track.name.clone(),
                    artists: track.artists.iter().map(|a| a.name.clone()).collect(),
                    album: track.album.name.clone(),
                    duration_ms: track.duration.num_milliseconds() as u32,
                    progress_ms,
                    is_playing,
                    is_liked: false,
                    album_art_url: track.album.images.first().map(|i| i.url.clone()),
                    device_volume: device_vol,
                };
                return Ok(Some(ct));
            }
        }
        Ok(None)
    }

    /// Get the first available device ID, or return an error with helpful message
    async fn get_device_id(&self) -> Result<String> {
        let sp = self.spotify.lock().await;
        let devices = sp.device().await?;

        // Try to find an active device first, then any device
        if let Some(dev) = devices.iter().find(|d| d.is_active) {
            return Ok(dev.id.clone().unwrap_or_default());
        }
        if let Some(dev) = devices.first() {
            return Ok(dev.id.clone().unwrap_or_default());
        }

        Err(anyhow!(
            "No Spotify device found! Open Spotify on your phone, desktop, or web browser first."
        ))
    }

    // Replaced `play_track` with `play_tracks` to support Queue context

    pub async fn play_tracks(&self, uris: Vec<&str>) -> Result<()> {
        let device_id = self.get_device_id().await?;
        let sp = self.spotify.lock().await;

        let mut playable_ids = Vec::new();
        for uri in uris {
            // Silently ignore invalid URIs
            if let Ok(id) = TrackId::from_uri(uri) {
                playable_ids.push(PlayableId::Track(id));
            }
        }

        sp.start_uris_playback(
            playable_ids,
            Some(&device_id),
            None,
            None,
        )
        .await?;
        info!("Playing multiple tracks on device {device_id}");
        Ok(())
    }

    pub async fn pause(&self) -> Result<()> {
        let sp = self.spotify.lock().await;
        sp.pause_playback(None).await?;
        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        let sp = self.spotify.lock().await;
        sp.resume_playback(None, None).await?;
        Ok(())
    }

    pub async fn toggle_playback(&self, is_playing: bool) -> Result<()> {
        if is_playing {
            self.pause().await
        } else {
            self.resume().await
        }
    }

    pub async fn next_track(&self) -> Result<()> {
        let sp = self.spotify.lock().await;
        sp.next_track(None).await?;
        Ok(())
    }

    pub async fn previous_track(&self) -> Result<()> {
        let sp = self.spotify.lock().await;
        sp.previous_track(None).await?;
        Ok(())
    }

    pub async fn seek(&self, position_ms: u32) -> Result<()> {
        use chrono::TimeDelta;
        let pos = TimeDelta::milliseconds(position_ms as i64);
        let sp = self.spotify.lock().await;
        sp.seek_track(pos, None).await?;
        Ok(())
    }

    pub async fn set_volume(&self, volume: u8) -> Result<()> {
        let sp = self.spotify.lock().await;
        sp.volume(volume, None).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn is_track_saved(&self, track_id: &str) -> Result<bool> {
        let sp = self.spotify.lock().await;
        let id = TrackId::from_id(track_id)?;
        let results = sp.current_user_saved_tracks_contains([id]).await?;
        Ok(results.into_iter().next().unwrap_or(false))
    }

    pub async fn save_track(&self, track_id: &str) -> Result<()> {
        let sp = self.spotify.lock().await;
        let id = TrackId::from_id(track_id)?;
        sp.current_user_saved_tracks_add([id]).await?;
        Ok(())
    }

    pub async fn remove_track(&self, track_id: &str) -> Result<()> {
        let sp = self.spotify.lock().await;
        let id = TrackId::from_id(track_id)?;
        sp.current_user_saved_tracks_delete([id]).await?;
        Ok(())
    }
}
