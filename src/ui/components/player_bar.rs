use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::state::AppState;
use super::super::theme::*;

/// Block characters for vertical bar heights (8 levels)
const BAR_BLOCKS: &[&str] = &[" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

fn bar_block(height: u8, row_from_bottom: u8) -> &'static str {
    // For a given bar height (1-12) and row (0=bottom),
    // return full block if height > row, else empty
    if height > row_from_bottom {
        "█"
    } else {
        " "
    }
}

fn bar_color(height: u8, row_from_bottom: u8) -> ratatui::style::Color {
    let level = row_from_bottom;
    if height <= row_from_bottom {
        SURFACE // invisible
    } else if level >= 9 {
        ERROR      // red peak
    } else if level >= 6 {
        HOT_PINK   // hot zone
    } else if level >= 3 {
        PRIMARY    // mid purple
    } else {
        ACCENT     // cyan base
    }
}

pub fn render_player_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style(true))
        .style(normal_style());

    let inner = block.inner(area);
    f.render_widget(block, area);

    if state.eq_expanded {
        render_expanded(f, inner, state);
    } else {
        render_compact(f, inner, state);
    }
}

/// Compact player bar (5 lines) — track info + inline EQ + progress
fn render_compact(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // track info
            Constraint::Percentage(45), // EQ + progress
            Constraint::Percentage(25), // controls
        ])
        .split(area);

    // ── Track info ──────────────────────────────────────────────────
    render_track_info(f, chunks[0], state);

    // ── Center: EQ + progress ──────────────────────────────────────
    let center_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // EQ bars (single row)
            Constraint::Length(1), // progress gauge
            Constraint::Min(0),   // time label
        ])
        .split(chunks[1]);

    // Single-row EQ
    let eq_spans: Vec<Span> = state.eq_bars.iter().map(|&h| {
        let ch = BAR_BLOCKS[(h as usize).clamp(0, 8)];
        let color = if h >= 9 { ERROR } else if h >= 6 { HOT_PINK } else if h >= 3 { PRIMARY } else { ACCENT };
        Span::styled(ch, ratatui::style::Style::default().fg(color))
    }).collect();
    let eq_line = Line::from(eq_spans);
    f.render_widget(Paragraph::new(eq_line).alignment(Alignment::Center), center_chunks[0]);

    // Progress gauge
    let progress_pct = (state.current_track.progress_percent() * 100.0) as u16;
    let gauge = Gauge::default()
        .gauge_style(ratatui::style::Style::default().fg(PRIMARY).bg(SURFACE))
        .percent(progress_pct)
        .label("");
    f.render_widget(gauge, center_chunks[1]);

    // Time label
    let time_label = Paragraph::new(Line::from(Span::styled(
        state.current_track.progress_formatted(),
        dim_style(),
    ))).alignment(Alignment::Center);
    f.render_widget(time_label, center_chunks[2]);

    // ── Controls ───────────────────────────────────────────────────
    let controls = Paragraph::new(vec![
        Line::from(Span::styled("⏮ p  ⏸ spc  ⏭ n", dim_style())),
        Line::from(Span::styled("+ vol -   e EQ   ? help", muted_style())),
    ]).alignment(Alignment::Right);
    f.render_widget(controls, chunks[2]);
}

/// Expanded player bar (12 lines) — big vertical EQ + track + progress
fn render_expanded(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // track info
            Constraint::Percentage(55), // big EQ
            Constraint::Percentage(20), // controls
        ])
        .split(area);

    // ── Track info (left) ───────────────────────────────────────────
    render_track_info(f, chunks[0], state);

    // ── Vertical EQ visualization (center) ──────────────────────────
    let eq_area = chunks[1];
    let eq_rows = eq_area.height.min(12) as u8; // max 12 rows of bars
    let center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(eq_rows as u16),  // EQ bars
            Constraint::Length(1),                // progress gauge
            Constraint::Min(0),                   // time label
        ])
        .split(eq_area);

    // Render vertical bars: each row from top (high) to bottom (low)
    let bar_count = state.eq_bars.len().min(center[0].width as usize);
    for row in 0..eq_rows {
        let row_from_bottom = eq_rows.saturating_sub(1 + row);
        let mut spans: Vec<Span> = Vec::with_capacity(bar_count * 2);
        for i in 0..bar_count {
            let h = state.eq_bars[i];
            let color = bar_color(h, row_from_bottom);
            let ch = bar_block(h, row_from_bottom);
            spans.push(Span::styled(ch, ratatui::style::Style::default().fg(color)));
            spans.push(Span::styled(" ", ratatui::style::Style::default())); // spacing
        }
        let y = center[0].y + row as u16;
        if y < center[0].y + center[0].height {
            let line_area = Rect::new(center[0].x, y, center[0].width, 1);
            f.render_widget(Paragraph::new(Line::from(spans)).alignment(Alignment::Center), line_area);
        }
    }

    // Progress gauge
    let progress_pct = (state.current_track.progress_percent() * 100.0) as u16;
    let gauge = Gauge::default()
        .gauge_style(ratatui::style::Style::default().fg(PRIMARY).bg(SURFACE))
        .percent(progress_pct)
        .label("");
    f.render_widget(gauge, center[1]);

    // Time label
    let time_label = Paragraph::new(Line::from(Span::styled(
        state.current_track.progress_formatted(),
        dim_style(),
    ))).alignment(Alignment::Center);
    f.render_widget(time_label, center[2]);

    // ── Controls (right) ────────────────────────────────────────────
    // Right-aligning with uniform padding so the icons line up cleanly
    let controls = Paragraph::new(vec![
        Line::from(Span::styled("  ⏮ p", dim_style())),
        Line::from(Span::styled("⏸ spc", dim_style())),
        Line::from(Span::styled("  ⏭ n", dim_style())),
        Line::from(Span::raw("")),
        Line::from(Span::styled("+ vol -", muted_style())),
        Line::from(Span::styled("e min EQ", accent_style())),
        Line::from(Span::styled(" ? help", muted_style())),
        Line::from(Span::styled(" q quit", muted_style())),
    ]).alignment(Alignment::Right);
    
    // We render in a vertically centered block within the right chunk
    let vertical_pad = chunks[2].height.saturating_sub(8) / 2;
    let right_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_pad),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(chunks[2]);
        
    f.render_widget(controls, right_chunk[1]);
}

fn render_track_info(f: &mut Frame, area: Rect, state: &AppState) {
    let track = &state.current_track;
    let liked_icon = if track.is_liked { "❤ " } else { "♡ " };
    let liked_style = if track.is_liked { gold_style() } else { muted_style() };
    let play_icon = if track.is_playing { "▶" } else { "⏸" };

    let title_display = state.get_display_title(area.width.saturating_sub(6) as usize);
    let artist = track.artists.join(", ");
    let album = &track.album;

    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!("{play_icon} "), playing_style()),
            Span::styled(title_display, normal_style().add_modifier(ratatui::style::Modifier::BOLD)),
            Span::styled(format!(" {liked_icon}"), liked_style),
        ]),
        Line::from(Span::styled(
            if artist.is_empty() { "—".to_string() } else { artist },
            dim_style(),
        )),
    ];

    // Show album in expanded mode if there's space
    if area.height >= 4 && !album.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("💿 {album}"),
            muted_style(),
        )));
    }

    let info_para = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(info_para, area);
}
