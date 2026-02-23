pub mod state;

use anyhow::Result;
use crossterm::event::{Event, EventStream};
use rspotify::prelude::Id;
use futures::StreamExt;
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, sync::mpsc, time};
use tracing::{info, warn};

use crate::{
    app::state::{ActiveScreen, AppState, CurrentTrack, Notification},
    cache::Cache,
    config::Config,
    events::{map_key_to_action, UserAction},
    spotify::{
        build_spotify_client, complete_auth,
        auth::wait_for_auth_code,
        library::Library,
        player::Player,
        queue::Queue,
        search::Search,
        vibes::Vibes,
    },
};

const TICK_MS: u64 = 80;         // UI tick (animations, EQ bars) — slightly faster
const SLOW_TICK_MS: u64 = 2000;  // Playback polling — less aggressive

pub struct App {
    pub state: AppState,
    config: Config,
    cache: Arc<Cache>,
}

impl App {
    pub async fn new(config: Config, cache: Arc<Cache>) -> Result<Self> {
        Ok(App {
            state: AppState::default(),
            config,
            cache,
        })
    }

    pub async fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut ratatui::Terminal<B>,
    ) -> Result<()> {
        // ── Spotify Auth ─────────────────────────────────────────────────────
        let (spotify_arc, auth_url) = build_spotify_client(&self.config, &self.cache).await?;

        if let Some(ref url) = auth_url {
            self.state.auth_url = Some(url.clone());
            // Open browser
            if let Err(e) = open::that(url) {
                warn!("Could not open browser: {e}");
            }

            // Draw auth screen first
            terminal.draw(|f| crate::ui::render(f, &self.state))?;

            // Generate PKCE verifier (we need to store it)
            let pkce = crate::spotify::auth::PkceChallenge::new();
            // Wait for the redirect
            let auth_result = wait_for_auth_code().await?;
            complete_auth(spotify_arc.clone(), &auth_result.code, &pkce.verifier, &self.cache).await?;
        }

        self.state.is_authenticated = true;
        self.state.auth_url = None;
        self.state.spotify = Some(spotify_arc.clone());
        self.state.set_notification(Notification::info("Connected to Spotify ✓"));
        info!("Authenticated successfully");

        // ── Load initial data (in background) ────────────────────────────────
        self.load_playlists(spotify_arc.clone()).await;
        self.load_library(spotify_arc.clone()).await;

        // ── Background playback channel ──────────────────────────────────────
        let (pb_tx, mut pb_rx) = mpsc::channel::<CurrentTrack>(4);

        // ── Main event loop ───────────────────────────────────────────────────
        let mut tick_interval = time::interval(Duration::from_millis(TICK_MS));
        let mut slow_interval = time::interval(Duration::from_millis(SLOW_TICK_MS));
        let mut event_stream = EventStream::new();

        loop {
            // Draw
            terminal.draw(|f| crate::ui::render(f, &self.state))?;

            // Wait for next event
            tokio::select! {
                _ = tick_interval.tick() => {
                    self.state.update_eq_bars();
                    self.state.tick_ticker();
                    self.state.tick_notification();
                    // Auto-increment progress for smooth bar movement
                    if self.state.current_track.is_playing {
                        self.state.current_track.progress_ms =
                            (self.state.current_track.progress_ms + TICK_MS as u32)
                                .min(self.state.current_track.duration_ms);
                    }
                }
                _ = slow_interval.tick() => {
                    // Fire-and-forget: spawn background task to poll playback
                    let sp = spotify_arc.clone();
                    let tx = pb_tx.clone();
                    tokio::spawn(async move {
                        let player = Player::new(sp);
                        if let Ok(Some(ct)) = player.get_current_playback().await {
                            let _ = tx.send(ct).await;
                        }
                    });
                }
                Some(ct) = pb_rx.recv() => {
                    // Sync volume from Spotify device
                    if let Some(vol) = ct.device_volume {
                        self.state.volume = vol;
                    }
                    self.state.current_track = ct;
                }
                maybe_event = event_stream.next() => {
                    if let Some(Ok(Event::Key(key))) = maybe_event {
                        let search_active = self.state.search.is_searching;
                        if let Some(action) = map_key_to_action(key, search_active) {
                            self.handle_action(action, spotify_arc.clone()).await;
                        }
                    }
                }
            }

            if self.state.should_quit {
                break;
            }
        }

        Ok(())
    }

    // ── Action handler ────────────────────────────────────────────────────────
    async fn handle_action(&mut self, action: UserAction, spotify: Arc<Mutex<rspotify::AuthCodePkceSpotify>>) {
        match action {
            UserAction::Quit => {
                self.state.should_quit = true;
            }
            UserAction::ToggleHelp => {
                self.state.show_help = !self.state.show_help;
            }
            UserAction::SwitchScreen(n) => {
                self.state.show_help = false;
                match n {
                    1 => { self.state.navigate_to(ActiveScreen::Search); self.state.search.is_searching = false; }
                    2 => { self.state.navigate_to(ActiveScreen::Library); self.load_library(spotify.clone()).await; }
                    3 => { self.state.navigate_to(ActiveScreen::Playlists); self.load_playlists(spotify.clone()).await; }
                    4 => { self.state.navigate_to(ActiveScreen::Queue); self.load_queue(spotify.clone()).await; }
                    5 => { self.state.navigate_to(ActiveScreen::Vibes); }
                    _ => {}
                }
            }
            UserAction::OpenSearch => {
                self.state.navigate_to(ActiveScreen::Search);
                self.state.search.is_searching = true;
            }
            UserAction::Back => {
                if self.state.search.is_searching {
                    self.state.search.is_searching = false;
                } else if self.state.playlists.viewing_tracks {
                    self.state.playlists.viewing_tracks = false;
                } else if self.state.show_help {
                    self.state.show_help = false;
                }
            }
            UserAction::SearchInput(c) => {
                self.state.search.query.push(c);
            }
            UserAction::SearchBackspace => {
                self.state.search.query.pop();
            }
            UserAction::SearchSubmit => {
                self.state.search.is_searching = false;
                if !self.state.search.query.is_empty() {
                    self.do_search(spotify.clone()).await;
                }
            }
            UserAction::NavigateUp => self.navigate_up(),
            UserAction::NavigateDown => self.navigate_down(),
            UserAction::NavigateLeft => {
                if self.state.active_screen == ActiveScreen::Playlists && self.state.playlists.viewing_tracks {
                    self.state.playlists.viewing_tracks = false;
                }
            }
            UserAction::NavigateRight => {
                if self.state.active_screen == ActiveScreen::Playlists && !self.state.playlists.viewing_tracks {
                    self.state.playlists.viewing_tracks = true;
                }
            }
            UserAction::Select => self.handle_select(spotify.clone()).await,
            UserAction::TogglePlay => {
                let is_playing = self.state.current_track.is_playing;
                self.state.current_track.is_playing = !is_playing; // Optimistic UI update
                let msg = if is_playing { "Paused" } else { "Resumed" };
                self.state.set_notification(Notification::info(msg));
                
                let sp = spotify.clone();
                tokio::spawn(async move {
                    let player = Player::new(sp);
                    let _ = player.toggle_playback(is_playing).await;
                });
            }
            UserAction::NextTrack => {
                self.state.set_notification(Notification::info("Next track ▶▶"));
                self.state.current_track.name = "Loading next track...".to_string(); // Optimistic feedback
                self.state.current_track.artists = vec![];
                self.state.current_track.progress_ms = 0;
                
                let sp = spotify.clone();
                tokio::spawn(async move {
                    let player = Player::new(sp);
                    let _ = player.next_track().await;
                    // Usually Spotify takes a moment to process this
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                });
            }
            UserAction::PrevTrack => {
                self.state.set_notification(Notification::info("Previous track ◀◀"));
                self.state.current_track.name = "Loading previous track...".to_string(); // Optimistic feedback
                self.state.current_track.artists = vec![];
                self.state.current_track.progress_ms = 0;
                
                let sp = spotify.clone();
                tokio::spawn(async move {
                    let player = Player::new(sp);
                    let _ = player.previous_track().await;
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                });
            }
            UserAction::VolumeUp => {
                let new_vol = (self.state.volume as u16 + 5).min(100) as u8;
                self.state.volume = new_vol;
                self.state.set_notification(Notification::info(format!("Volume: {new_vol}%")));
                let sp = spotify.clone();
                tokio::spawn(async move {
                    let player = Player::new(sp);
                    let _ = player.set_volume(new_vol).await;
                });
            }
            UserAction::VolumeDown => {
                let new_vol = self.state.volume.saturating_sub(5);
                self.state.volume = new_vol;
                self.state.set_notification(Notification::info(format!("Volume: {new_vol}%")));
                let sp = spotify.clone();
                tokio::spawn(async move {
                    let player = Player::new(sp);
                    let _ = player.set_volume(new_vol).await;
                });
            }
            UserAction::LikeTrack => {
                if let Some(ref id) = self.state.current_track.id.clone() {
                    let player = Player::new(spotify.clone());
                    if self.state.current_track.is_liked {
                        if player.remove_track(id).await.is_ok() {
                            self.state.current_track.is_liked = false;
                            self.state.set_notification(Notification::info("Removed from Liked Songs"));
                        }
                    } else if player.save_track(id).await.is_ok() {
                        self.state.current_track.is_liked = true;
                        self.state.set_notification(Notification::info("❤ Added to Liked Songs"));
                    }
                }
            }
            UserAction::AddToQueue => {
                self.handle_add_to_queue(spotify.clone()).await;
            }
            UserAction::SeekForward => {
                let new_pos = (self.state.current_track.progress_ms + 10_000)
                    .min(self.state.current_track.duration_ms);
                self.state.current_track.progress_ms = new_pos;
                let sp = spotify.clone();
                tokio::spawn(async move {
                    let player = Player::new(sp);
                    let _ = player.seek(new_pos).await;
                });
            }
            UserAction::SeekBackward => {
                let new_pos = self.state.current_track.progress_ms.saturating_sub(10_000);
                self.state.current_track.progress_ms = new_pos;
                let sp = spotify.clone();
                tokio::spawn(async move {
                    let player = Player::new(sp);
                    let _ = player.seek(new_pos).await;
                });
            }
            UserAction::ToggleEQ => {
                self.state.eq_expanded = !self.state.eq_expanded;
                let mode = if self.state.eq_expanded { "Expanded" } else { "Compact" };
                self.state.set_notification(Notification::info(format!("EQ: {mode}")));
            }

        }
    }

    // ── Navigation helpers ────────────────────────────────────────────────────
    fn navigate_up(&mut self) {
        match self.state.active_screen {
            ActiveScreen::Search => {
                if self.state.search.selected_track > 0 {
                    self.state.search.selected_track -= 1;
                }
            }
            ActiveScreen::Library => {
                if self.state.library.selected > 0 {
                    self.state.library.selected -= 1;
                }
            }
            ActiveScreen::Playlists => {
                if self.state.playlists.viewing_tracks {
                    if self.state.playlists.selected_track > 0 {
                        self.state.playlists.selected_track -= 1;
                    }
                } else if self.state.playlists.selected_playlist > 0 {
                    self.state.playlists.selected_playlist -= 1;
                }
            }
            ActiveScreen::Queue => {
                if self.state.queue.selected > 0 {
                    self.state.queue.selected -= 1;
                }
            }
            ActiveScreen::Vibes => {
                if self.state.vibes.selected_mood > 0 && !self.state.vibes.recommendations.is_empty() {
                    // In track list
                    if self.state.vibes.selected_track > 0 {
                        self.state.vibes.selected_track -= 1;
                    }
                } else if self.state.vibes.selected_mood > 0 {
                    self.state.vibes.selected_mood -= 1;
                }
            }

        }
    }

    fn navigate_down(&mut self) {
        match self.state.active_screen {
            ActiveScreen::Search => {
                let max = self.state.search.tracks.len().saturating_sub(1);
                if self.state.search.selected_track < max {
                    self.state.search.selected_track += 1;
                }
            }
            ActiveScreen::Library => {
                let max = self.state.library.liked_songs.len().saturating_sub(1);
                if self.state.library.selected < max {
                    self.state.library.selected += 1;
                }
            }
            ActiveScreen::Playlists => {
                if self.state.playlists.viewing_tracks {
                    let max = self.state.playlists.playlist_tracks.len().saturating_sub(1);
                    if self.state.playlists.selected_track < max {
                        self.state.playlists.selected_track += 1;
                    }
                } else {
                    let max = self.state.playlists.playlists.len().saturating_sub(1);
                    if self.state.playlists.selected_playlist < max {
                        self.state.playlists.selected_playlist += 1;
                    }
                }
            }
            ActiveScreen::Queue => {
                let max = self.state.queue.tracks.len().saturating_sub(1);
                if self.state.queue.selected < max {
                    self.state.queue.selected += 1;
                }
            }
            ActiveScreen::Vibes => {
                if !self.state.vibes.recommendations.is_empty() {
                    let max = self.state.vibes.recommendations.len().saturating_sub(1);
                    if self.state.vibes.selected_track < max {
                        self.state.vibes.selected_track += 1;
                    }
                } else {
                    let max = 4; // 5 moods
                    if self.state.vibes.selected_mood < max {
                        self.state.vibes.selected_mood += 1;
                    }
                }
            }

        }
    }

    // ── Select handler ────────────────────────────────────────────────────────
    async fn handle_select(&mut self, spotify: Arc<Mutex<rspotify::AuthCodePkceSpotify>>) {
        match self.state.active_screen.clone() {
            ActiveScreen::Search => {
                let current_idx = self.state.search.selected_track;
                let uris: Vec<String> = self.state.search.tracks.iter()
                    .skip(current_idx)
                    .filter_map(|t| t.id.as_ref().map(|id| id.uri()))
                    .take(50)
                    .collect();
                
                if let Some(track) = self.state.search.tracks.get(current_idx) {
                    if !uris.is_empty() {
                        let player = Player::new(spotify.clone());
                        let uri_refs: Vec<&str> = uris.iter().map(|s| s.as_str()).collect();
                        match player.play_tracks(uri_refs).await {
                            Ok(_) => self.state.set_notification(Notification::info(format!("Playing: {}", track.name))),
                            Err(e) => self.state.set_notification(Notification::error(format!("{e}"))),
                        }
                    }
                }
            }
            ActiveScreen::Library => {
                let current_idx = self.state.library.selected;
                let uris: Vec<String> = self.state.library.liked_songs.iter()
                    .skip(current_idx)
                    .filter_map(|s| s.track.id.as_ref().map(|id| id.uri()))
                    .take(50)
                    .collect();

                if let Some(saved) = self.state.library.liked_songs.get(current_idx) {
                    if !uris.is_empty() {
                        let player = Player::new(spotify.clone());
                        let uri_refs: Vec<&str> = uris.iter().map(|s| s.as_str()).collect();
                        match player.play_tracks(uri_refs).await {
                            Ok(_) => self.state.set_notification(Notification::info(format!("Playing: {}", saved.track.name))),
                            Err(e) => self.state.set_notification(Notification::error(format!("{e}"))),
                        }
                    }
                }
            }
            ActiveScreen::Playlists => {
                if !self.state.playlists.viewing_tracks {
                    // Enter playlist and load tracks
                    self.state.playlists.viewing_tracks = true;
                    self.state.playlists.selected_track = 0;
                    let playlist_id = self.state.playlists.playlists
                        .get(self.state.playlists.selected_playlist)
                        .and_then(|p| p.id.to_string().into());
                    if let Some(pid) = playlist_id {
                        self.load_playlist_tracks(spotify.clone(), pid).await;
                    }
                } else {
                    // Play selected track
                    use rspotify::model::PlayableItem;
                    let current_idx = self.state.playlists.selected_track;
                    let uris: Vec<String> = self.state.playlists.playlist_tracks.iter()
                        .skip(current_idx)
                        .filter_map(|item| {
                            if let Some(PlayableItem::Track(ref track)) = item.track {
                                track.id.as_ref().map(|id| id.uri())
                            } else {
                                None
                            }
                        })
                        .take(50)
                        .collect();

                    if let Some(item) = self.state.playlists.playlist_tracks.get(current_idx) {
                        if !uris.is_empty() {
                            let player = Player::new(spotify.clone());
                            let uri_refs: Vec<&str> = uris.iter().map(|s| s.as_str()).collect();
                            match player.play_tracks(uri_refs).await {
                                Ok(_) => {
                                    if let Some(PlayableItem::Track(ref track)) = item.track {
                                        self.state.set_notification(Notification::info(format!("Playing: {}", track.name)));
                                    }
                                }
                                Err(e) => self.state.set_notification(Notification::error(format!("{e}"))),
                            }
                        }
                    }
                }
            }
            ActiveScreen::Vibes => {
                if self.state.vibes.recommendations.is_empty() {
                    // Load recommendations for selected mood
                    use strum::IntoEnumIterator;
                    use crate::app::state::VibesMood;
                    let moods: Vec<VibesMood> = VibesMood::iter().collect();
                    if let Some(mood) = moods.get(self.state.vibes.selected_mood) {
                        self.load_vibes(spotify.clone(), mood.clone()).await;
                    }
                } else {
                    // Play selected recommendation
                    let current_idx = self.state.vibes.selected_track;
                    let uris: Vec<String> = self.state.vibes.recommendations.iter()
                        .skip(current_idx)
                        .filter_map(|t| t.id.as_ref().map(|id| id.uri()))
                        .take(50)
                        .collect();

                    if let Some(track) = self.state.vibes.recommendations.get(current_idx) {
                        if !uris.is_empty() {
                            let player = Player::new(spotify.clone());
                            let uri_refs: Vec<&str> = uris.iter().map(|s| s.as_str()).collect();
                            match player.play_tracks(uri_refs).await {
                                Ok(_) => self.state.set_notification(Notification::info(format!("Playing: {}", track.name))),
                                Err(e) => self.state.set_notification(Notification::error(format!("{e}"))),
                            }
                        }
                    }
                }
            }

            _ => {}
        }
    }

    async fn handle_add_to_queue(&mut self, spotify: Arc<Mutex<rspotify::AuthCodePkceSpotify>>) {
        let uri = match self.state.active_screen {
            ActiveScreen::Search => self.state.search.tracks
                .get(self.state.search.selected_track)
                .and_then(|t| t.id.as_ref().map(|id| id.uri())),
            ActiveScreen::Library => self.state.library.liked_songs
                .get(self.state.library.selected)
                .and_then(|s| s.track.id.as_ref().map(|id| id.uri())),
            ActiveScreen::Vibes => self.state.vibes.recommendations
                .get(self.state.vibes.selected_track)
                .and_then(|t| t.id.as_ref().map(|id| id.uri())),
            _ => None,
        };
        if let Some(uri) = uri {
            let queue = Queue::new(spotify.clone());
            match queue.add_to_queue(&uri).await {
                Ok(_) => self.state.set_notification(Notification::info("Added to queue ✓")),
                Err(e) => self.state.set_notification(Notification::error(format!("{e}"))),
            }
        }
    }

    // ── Spotify data loaders ──────────────────────────────────────────────────
    #[allow(dead_code)]
    async fn poll_playback(&mut self, spotify: Arc<Mutex<rspotify::AuthCodePkceSpotify>>) {
        let player = Player::new(spotify.clone());
        match player.get_current_playback().await {
            Ok(Some(mut ct)) => {
                // Check if liked
                if let Some(ref id) = ct.id.clone() {
                    let player2 = Player::new(spotify.clone());
                    ct.is_liked = player2.is_track_saved(id).await.unwrap_or(false);
                }
                self.state.current_track = ct;
            }
            Ok(None) => {}
            Err(e) => {
                warn!("Playback poll error: {e}");
            }
        }
    }

    async fn do_search(&mut self, spotify: Arc<Mutex<rspotify::AuthCodePkceSpotify>>) {
        let query = self.state.search.query.clone();
        self.state.search.is_searching = true;
        let searcher = Search::new(spotify.clone());
        match searcher.search_tracks(&query, 50).await {
            Ok(tracks) => {
                self.state.search.tracks = tracks;
                self.state.search.selected_track = 0;
                self.state.search.is_searching = false;
                self.state.set_notification(Notification::info(format!(
                    "Found {} tracks", self.state.search.tracks.len()
                )));
            }
            Err(e) => {
                self.state.search.is_searching = false;
                self.state.set_notification(Notification::error(format!("Search failed: {e}")));
            }
        }
    }

    async fn load_library(&mut self, spotify: Arc<Mutex<rspotify::AuthCodePkceSpotify>>) {
        if !self.state.library.liked_songs.is_empty() { return; }
        self.state.library.is_loading = true;
        let lib = Library::new(spotify.clone());
        match lib.get_liked_songs(200).await {
            Ok(songs) => {
                self.state.library.liked_songs = songs;
                self.state.library.is_loading = false;
            }
            Err(e) => {
                self.state.library.is_loading = false;
                warn!("Library load error: {e}");
            }
        }
    }

    async fn load_playlists(&mut self, spotify: Arc<Mutex<rspotify::AuthCodePkceSpotify>>) {
        if !self.state.playlists.playlists.is_empty() { return; }
        self.state.playlists.is_loading = true;
        let lib = Library::new(spotify.clone());
        match lib.get_user_playlists().await {
            Ok(pls) => {
                self.state.playlists.playlists = pls;
                self.state.playlists.is_loading = false;
            }
            Err(e) => {
                self.state.playlists.is_loading = false;
                warn!("Playlists load error: {e}");
            }
        }
    }

    async fn load_playlist_tracks(&mut self, spotify: Arc<Mutex<rspotify::AuthCodePkceSpotify>>, playlist_id: String) {
        self.state.playlists.is_loading = true;
        self.state.playlists.playlist_tracks.clear();
        let lib = Library::new(spotify.clone());
        match lib.get_playlist_tracks(&playlist_id).await {
            Ok(tracks) => {
                self.state.playlists.playlist_tracks = tracks;
                self.state.playlists.is_loading = false;
            }
            Err(e) => {
                self.state.playlists.is_loading = false;
                warn!("Playlist tracks load error: {e}");
            }
        }
    }

    async fn load_queue(&mut self, spotify: Arc<Mutex<rspotify::AuthCodePkceSpotify>>) {
        self.state.queue.is_loading = true;
        let q = Queue::new(spotify.clone());
        match q.get_queue().await {
            Ok(tracks) => {
                self.state.queue.tracks = tracks;
                self.state.queue.is_loading = false;
            }
            Err(e) => {
                self.state.queue.is_loading = false;
                warn!("Queue load error: {e}");
            }
        }
    }

    async fn load_vibes(&mut self, spotify: Arc<Mutex<rspotify::AuthCodePkceSpotify>>, mood: crate::app::state::VibesMood) {
        self.state.vibes.is_loading = true;
        self.state.vibes.recommendations.clear();
        self.state.vibes.selected_track = 0;
        let v = Vibes::new(spotify.clone());
        match v.get_recommendations(&mood).await {
            Ok(tracks) => {
                self.state.vibes.recommendations = tracks;
                self.state.vibes.is_loading = false;
                self.state.set_notification(Notification::info(format!("Generated {} recommendations", self.state.vibes.recommendations.len())));
            }
            Err(e) => {
                self.state.vibes.is_loading = false;
                self.state.set_notification(Notification::error(format!("Vibes error: {e}")));
            }
        }
    }
}
