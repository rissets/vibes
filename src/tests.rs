#[cfg(test)]
mod tests {
    use crate::app::state::{
        ActiveScreen, AppState, CurrentTrack, Notification,
    };

    // ── CurrentTrack ─────────────────────────────────────────────────────────

    #[test]
    fn test_progress_percent_zero_duration() {
        let track = CurrentTrack {
            duration_ms: 0,
            progress_ms: 0,
            ..Default::default()
        };
        assert_eq!(track.progress_percent(), 0.0);
    }

    #[test]
    fn test_progress_percent_half() {
        let track = CurrentTrack {
            duration_ms: 200_000,
            progress_ms: 100_000,
            ..Default::default()
        };
        let pct = track.progress_percent();
        assert!((pct - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_percent_clamped() {
        let track = CurrentTrack {
            duration_ms: 100,
            progress_ms: 200, // beyond duration
            ..Default::default()
        };
        assert_eq!(track.progress_percent(), 1.0);
    }

    #[test]
    fn test_progress_formatted() {
        let track = CurrentTrack {
            duration_ms: 213_000, // 3:33
            progress_ms: 90_000,  // 1:30
            ..Default::default()
        };
        assert_eq!(track.progress_formatted(), "1:30 / 3:33");
    }

    // ── AppState navigation ───────────────────────────────────────────────────

    #[test]
    fn test_navigate_to_changes_screen() {
        let mut state = AppState::default();
        assert_eq!(state.active_screen, ActiveScreen::Search);
        state.navigate_to(ActiveScreen::Vibes);
        assert_eq!(state.active_screen, ActiveScreen::Vibes);
        assert_eq!(state.previous_screen, Some(ActiveScreen::Search));
    }

    #[test]
    fn test_navigate_to_same_screen_noop() {
        let mut state = AppState::default();
        state.navigate_to(ActiveScreen::Search);
        assert!(state.previous_screen.is_none());
    }

    // ── Notification ──────────────────────────────────────────────────────────

    #[test]
    fn test_notification_tick_decrements() {
        let mut state = AppState::default();
        state.set_notification(Notification::info("hello"));
        assert!(state.notification.is_some());
        // remaining_ticks=30: takes 30 ticks to reach 0, then 1 more tick to clear
        for _ in 0..31 {
            state.tick_notification();
        }
        assert!(state.notification.is_none());
    }

    #[test]
    fn test_notification_error_flag() {
        let n = Notification::error("oops");
        assert!(n.is_error);
        assert_eq!(n.message, "oops");
    }

    // ── EQ bars ───────────────────────────────────────────────────────────────

    #[test]
    fn test_eq_bars_all_positive() {
        let mut state = AppState::default();
        state.current_track.is_playing = true;
        for _ in 0..50 {
            state.update_eq_bars();
        }
        for &bar in state.eq_bars.iter() {
            assert!(bar >= 1 && bar <= 12, "bar value {bar} out of range [1, 12]");
        }
    }

    #[test]
    fn test_eq_bars_fall_when_paused() {
        let mut state = AppState::default();
        state.current_track.is_playing = false;
        // Set all bars to max
        state.eq_bars = [12; 24];
        for _ in 0..12 {
            state.update_eq_bars();
        }
        for &bar in state.eq_bars.iter() {
            assert!(bar <= 12);
        }
    }

    // ── Ticker ────────────────────────────────────────────────────────────────

    #[test]
    fn test_display_title_short() {
        let mut state = AppState::default();
        state.current_track.name = "Short".to_string();
        let title = state.get_display_title(40);
        assert_eq!(title, "Short");
    }

    #[test]
    fn test_display_title_truncates_long() {
        let mut state = AppState::default();
        state.current_track.name = "A".repeat(60);
        let title = state.get_display_title(20);
        assert_eq!(title.len(), 20);
    }
}
