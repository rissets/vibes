pub mod components;
pub mod theme;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::state::{ActiveScreen, AppState};
use self::theme::*;
use self::components::{
    help::render_help,
    library::render_library,
    player_bar::render_player_bar,
    playlists::render_playlists,
    queue::render_queue,
    search::render_search,
    sidebar::render_sidebar,
    vibes_screen::render_vibes,
};

/// Root render function — called every frame
pub fn render(f: &mut Frame, state: &AppState) {
    let size = f.area();

    // ── Outer layout: content + player bar ──────────────────────────────
    let player_height = if state.eq_expanded { 15 } else { 5 };
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),                     // top: sidebar + main
            Constraint::Length(player_height),       // bottom: player bar
        ])
        .split(size);

    // ── Top: sidebar + content ───────────────────────────────────────────
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(24), // sidebar
            Constraint::Min(0),     // main content
        ])
        .split(main_chunks[0]);

    // Render sidebar
    render_sidebar(f, top_chunks[0], state);

    // Render main content based on active screen
    match &state.active_screen {
        ActiveScreen::Search    => render_search(f, top_chunks[1], state),
        ActiveScreen::Library   => render_library(f, top_chunks[1], state),
        ActiveScreen::Playlists => render_playlists(f, top_chunks[1], state),
        ActiveScreen::Queue     => render_queue(f, top_chunks[1], state),
        ActiveScreen::Vibes     => render_vibes(f, top_chunks[1], state),
    }

    // Render player bar
    render_player_bar(f, main_chunks[1], state);

    // ── Auth screen overlay (if not authenticated) ────────────────────────
    if !state.is_authenticated {
        render_auth_overlay(f, size, state);
    }

    // ── Help overlay ─────────────────────────────────────────────────────
    if state.show_help {
        render_help(f, size, state);
    }

    // ── Notification toast ────────────────────────────────────────────────
    if let Some(ref notif) = state.notification {
        render_notification(f, size, notif.is_error, &notif.message);
    }
}

fn render_auth_overlay(f: &mut Frame, area: Rect, state: &AppState) {
    let popup = centered_rect(70, 50, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(" 🎵 vibes — Spotify Auth ", title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(border_style(true))
        .style(normal_style().bg(BG_ALT));

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let lines = if let Some(ref url) = state.auth_url {
        vec![
            Line::from(Span::raw("")),
            Line::from(Span::styled("  Opening browser for Spotify login...", accent_style())),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  If the browser didn't open, visit:", dim_style())),
            Line::from(Span::raw("")),
            Line::from(Span::styled(format!("  {url}"), hot_pink_style())),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  Waiting for authorization...", dim_style())),
            Line::from(Span::raw("")),
            Line::from(Span::styled("  ⠋ Listening on http://127.0.0.1:8989/login", muted_style())),
        ]
    } else {
        vec![
            Line::from(Span::raw("")),
            Line::from(Span::styled("  Connecting to Spotify...", accent_style())),
        ]
    };

    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Left),
        inner,
    );
}

fn render_notification(f: &mut Frame, area: Rect, is_error: bool, message: &str) {
    let toast_width = message.len().min(60) as u16 + 4;
    let toast_area = Rect {
        x: area.width.saturating_sub(toast_width + 2),
        y: area.height.saturating_sub(8),
        width: toast_width,
        height: 3,
    };

    f.render_widget(Clear, toast_area);

    let style = if is_error { error_style() } else { playing_style() };
    let icon = if is_error { "✖ " } else { "✔ " };

    let para = Paragraph::new(Line::from(vec![
        Span::styled(icon, style.clone()),
        Span::styled(message.to_string(), style),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(style),
    )
    .alignment(Alignment::Left);

    f.render_widget(para, toast_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}
