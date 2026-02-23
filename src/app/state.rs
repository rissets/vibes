use rspotify::{
    model::{FullTrack, SimplifiedPlaylist, SavedTrack, PlaylistItem},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use rspotify::AuthCodePkceSpotify;

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveScreen {
    Search,
    Library,
    Playlists,
    Queue,
    Vibes,
}

impl Default for ActiveScreen {
    fn default() -> Self {
        ActiveScreen::Search
    }
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct CurrentTrack {
    pub id: Option<String>,
    pub name: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_ms: u32,
    pub progress_ms: u32,
    pub is_playing: bool,
    pub is_liked: bool,
    pub album_art_url: Option<String>,
    pub device_volume: Option<u8>,
}

impl CurrentTrack {
    pub fn progress_percent(&self) -> f64 {
        if self.duration_ms == 0 {
            return 0.0;
        }
        (self.progress_ms as f64 / self.duration_ms as f64).clamp(0.0, 1.0)
    }

    pub fn progress_formatted(&self) -> String {
        let secs = self.progress_ms / 1000;
        let dur_secs = self.duration_ms / 1000;
        format!(
            "{}:{:02} / {}:{:02}",
            secs / 60,
            secs % 60,
            dur_secs / 60,
            dur_secs % 60
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub tracks: Vec<FullTrack>,
    pub selected_track: usize,
    pub is_searching: bool,
}

#[derive(Debug, Clone, Default)]
pub struct LibraryState {
    pub liked_songs: Vec<SavedTrack>,
    pub selected: usize,
    pub is_loading: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PlaylistsState {
    pub playlists: Vec<SimplifiedPlaylist>,
    pub selected_playlist: usize,
    pub playlist_tracks: Vec<PlaylistItem>,
    pub selected_track: usize,
    pub viewing_tracks: bool,
    pub is_loading: bool,
}

#[derive(Debug, Clone, Default)]
pub struct QueueState {
    pub tracks: Vec<FullTrack>,
    pub selected: usize,
    pub is_loading: bool,
}

#[derive(Debug, Clone, PartialEq, strum_macros::Display, strum_macros::EnumIter)]
pub enum VibesMood {
    #[strum(to_string = "🌊 Chill")]
    Chill,
    #[strum(to_string = "⚡ Hype")]
    Hype,
    #[strum(to_string = "🎯 Focus")]
    Focus,
    #[strum(to_string = "✨ Happy")]
    Happy,
    #[strum(to_string = "🌑 Dark")]
    Dark,
}

#[derive(Debug, Clone, Default)]
pub struct VibesState {
    pub selected_mood: usize,
    pub recommendations: Vec<FullTrack>,
    pub selected_track: usize,
    pub is_loading: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Notification {
    pub message: String,
    pub remaining_ticks: u8,
    pub is_error: bool,
}

impl Notification {
    pub fn info(msg: impl Into<String>) -> Self {
        Notification { message: msg.into(), remaining_ticks: 30, is_error: false }
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Notification { message: msg.into(), remaining_ticks: 40, is_error: true }
    }
}

pub struct AppState {
    pub active_screen: ActiveScreen,
    pub previous_screen: Option<ActiveScreen>,
    pub current_track: CurrentTrack,
    pub volume: u8,
    pub search: SearchState,
    pub library: LibraryState,
    pub playlists: PlaylistsState,
    pub queue: QueueState,
    pub vibes: VibesState,
    pub notification: Option<Notification>,
    pub show_help: bool,
    pub should_quit: bool,
    pub eq_bars: [u8; 24],
    pub eq_tick: u64,
    pub eq_expanded: bool,
    pub ticker_offset: usize,
    pub ticker_tick: u64,
    pub spotify: Option<Arc<Mutex<AuthCodePkceSpotify>>>,
    pub is_authenticated: bool,
    pub auth_url: Option<String>,
    #[allow(dead_code)]
    pub cached_device_id: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            active_screen: ActiveScreen::Search,
            previous_screen: None,
            current_track: CurrentTrack::default(),
            volume: 50,
            search: SearchState::default(),
            library: LibraryState::default(),
            playlists: PlaylistsState::default(),
            queue: QueueState::default(),
            vibes: VibesState::default(),
            notification: None,
            show_help: false,
            should_quit: false,
            eq_bars: [4, 6, 8, 5, 7, 9, 4, 6, 8, 5, 7, 6, 4, 8, 5, 7, 9, 3, 6, 8, 5, 7, 4, 6],
            eq_tick: 0,
            eq_expanded: false,
            ticker_offset: 0,
            ticker_tick: 0,
            spotify: None,
            is_authenticated: false,
            auth_url: None,
            cached_device_id: None,
        }
    }
}

impl AppState {
    pub fn navigate_to(&mut self, screen: ActiveScreen) {
        if self.active_screen != screen {
            self.previous_screen = Some(self.active_screen.clone());
            self.active_screen = screen;
        }
    }

    pub fn set_notification(&mut self, n: Notification) {
        self.notification = Some(n);
    }

    pub fn tick_notification(&mut self) {
        if let Some(ref mut n) = self.notification {
            if n.remaining_ticks > 0 {
                n.remaining_ticks -= 1;
            } else {
                self.notification = None;
            }
        }
    }

    pub fn update_eq_bars(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        self.eq_tick += 1;
        if self.current_track.is_playing {
            for bar in self.eq_bars.iter_mut() {
                let delta: i8 = rng.gen_range(-3..=3);
                *bar = (*bar as i8 + delta).clamp(1, 12) as u8;
            }
        } else {
            for bar in self.eq_bars.iter_mut() {
                if *bar > 1 {
                    *bar -= 1;
                }
            }
        }
    }

    pub fn tick_ticker(&mut self) {
        self.ticker_tick += 1;
        if self.ticker_tick % 5 == 0 {
            let len = self.current_track.name.len().max(1);
            self.ticker_offset = (self.ticker_offset + 1) % len;
        }
    }

    pub fn get_display_title(&self, max_width: usize) -> String {
        let title = &self.current_track.name;
        if title.len() <= max_width {
            return title.clone();
        }
        let padded = format!("{title}   ");
        let chars: Vec<char> = padded.chars().collect();
        let offset = self.ticker_offset % chars.len();
        let visible: String = chars[offset..]
            .iter()
            .chain(chars[..offset].iter())
            .take(max_width)
            .collect();
        visible
    }
}
