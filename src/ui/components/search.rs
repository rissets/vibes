use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::state::AppState;
use super::super::theme::*;

pub fn render_search(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // search input
            Constraint::Min(0),    // results
        ])
        .split(area);

    // ── Search input box ──────────────────────────────────────────────────
    let input_focused = state.search.is_searching;
    let cursor = if input_focused && (state.eq_tick / 5) % 2 == 0 { "│" } else { "" };
    let input_block = Block::default()
        .title(Span::styled(" 󰍉 Search Spotify ", title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style(input_focused))
        .style(normal_style().bg(BG_ALT));

    let input_text = Paragraph::new(Line::from(vec![
        Span::styled(" ", muted_style()),
        Span::styled(state.search.query.clone(), accent_style()),
        Span::styled(cursor, hot_pink_style()),
    ]))
    .block(input_block);
    f.render_widget(input_text, chunks[0]);

    // ── Results ───────────────────────────────────────────────────────────
    if state.search.tracks.is_empty() {
        let placeholder = if state.search.query.is_empty() {
            "  Press [s] to search, type a query, then Enter..."
        } else if state.search.is_searching {
            "  Searching..."
        } else {
            "  No results found."
        };
        let para = Paragraph::new(Line::from(Span::styled(placeholder, muted_style())))
            .block(
                Block::default()
                    .title(Span::styled(" Results ", dim_style()))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style(false))
                    .style(normal_style().bg(BG)),
            );
        f.render_widget(para, chunks[1]);
        return;
    }

    let selected = state.search.selected_track;
    let items: Vec<ListItem> = state
        .search
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let is_sel = i == selected;
            let num = format!("{:>3}. ", i + 1);
            let title = track.name.clone();
            let artist = track
                .artists
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let album = track.album.name.clone();
            let dur_ms = track.duration.num_milliseconds() as u32;
            let dur_s = dur_ms / 1000;
            let dur = format!("{}:{:02}", dur_s / 60, dur_s % 60);

            let line = if is_sel {
                Line::from(vec![
                    Span::styled("▶ ", playing_style()),
                    Span::styled(title, selected_style()),
                    Span::styled(" — ", muted_style()),
                    Span::styled(artist, dim_style()),
                    Span::styled(format!("  {dur}"), muted_style()),
                ])
            } else {
                Line::from(vec![
                    Span::styled(num, muted_style()),
                    Span::styled(title, normal_style()),
                    Span::styled(" — ", muted_style()),
                    Span::styled(artist, dim_style()),
                    Span::styled(format!("  {album}  {dur}"), muted_style()),
                ])
            };

            if is_sel {
                ListItem::new(line).style(selected_style())
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" Results ({}) ", state.search.tracks.len()),
                    title_style(),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style(!input_focused))
                .style(normal_style().bg(BG)),
        )
        .highlight_style(selected_style());

    f.render_widget(list, chunks[1]);
}
