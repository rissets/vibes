use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame,
};
use rspotify::model::PlayableItem;

use crate::app::state::AppState;
use super::super::theme::*;

pub fn render_playlists(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(area);

    render_playlist_list(f, chunks[0], state);
    render_playlist_tracks(f, chunks[1], state);
}

fn render_playlist_list(f: &mut Frame, area: Rect, state: &AppState) {
    if state.playlists.is_loading {
        let para = Paragraph::new(Line::from(Span::styled("  ⠋ Loading playlists...", dim_style())))
            .block(make_block(" 📋 Playlists ", true));
        f.render_widget(para, area);
        return;
    }

    let selected = state.playlists.selected_playlist;
    let items: Vec<ListItem> = state
        .playlists
        .playlists
        .iter()
        .enumerate()
        .map(|(i, pl)| {
            let is_sel = i == selected;
            let name = pl.name.clone();
            let count = pl.tracks.total;
            let icon = if is_sel { "▶" } else { " " };
            let line = Line::from(vec![
                Span::styled(format!("{icon} "), if is_sel { playing_style() } else { muted_style() }),
                Span::styled(name, if is_sel { selected_style() } else { normal_style() }),
                Span::styled(format!("  {count}"), muted_style()),
            ]);
            if is_sel {
                ListItem::new(line).style(selected_style())
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items).block(make_block(
        &format!(" 📋 Playlists ({}) ", state.playlists.playlists.len()),
        !state.playlists.viewing_tracks,
    ));
    f.render_widget(list, area);
}

fn render_playlist_tracks(f: &mut Frame, area: Rect, state: &AppState) {
    let playlist_name = state
        .playlists
        .playlists
        .get(state.playlists.selected_playlist)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Playlist".to_string());

    if state.playlists.playlist_tracks.is_empty() {
        let msg = if state.playlists.is_loading {
            "  ⠋ Loading tracks..."
        } else {
            "  Select a playlist to see its tracks (Enter)"
        };
        let para = Paragraph::new(Line::from(Span::styled(msg, muted_style())))
            .block(make_block(&format!(" 🎵 {playlist_name} "), false));
        f.render_widget(para, area);
        return;
    }

    let selected = state.playlists.selected_track;
    let rows: Vec<Row> = state
        .playlists
        .playlist_tracks
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            if let Some(PlayableItem::Track(track)) = &item.track {
                let is_sel = i == selected;
                let dur_ms = track.duration.num_milliseconds() as u32;
                let secs = dur_ms / 1000;
                let dur = format!("{}:{:02}", secs / 60, secs % 60);
                let artist = track
                    .artists
                    .iter()
                    .map(|a| a.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                let num = if is_sel { "▶".to_string() } else { format!("{}", i + 1) };
                let style = if is_sel { selected_style() } else { normal_style() };
                Some(
                    Row::new(vec![
                        Cell::from(num)
                            .style(if is_sel { playing_style() } else { muted_style() }),
                        Cell::from(track.name.clone()).style(style.clone()),
                        Cell::from(artist).style(dim_style()),
                        Cell::from(dur).style(muted_style()),
                    ])
                    .style(style),
                )
            } else {
                None
            }
        })
        .collect();

    let header = Row::new(vec![
        Cell::from("#").style(header_style()),
        Cell::from("Title").style(header_style()),
        Cell::from("Artist").style(header_style()),
        Cell::from("Dur").style(header_style()),
    ]);

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Percentage(45),
            Constraint::Percentage(40),
            Constraint::Length(7),
        ],
    )
    .header(header)
    .block(make_block(
        &format!(" 🎵 {} ", playlist_name),
        state.playlists.viewing_tracks,
    ))
    .row_highlight_style(selected_style());

    f.render_widget(table, area);
}

fn make_block(title: &str, focused: bool) -> Block<'static> {
    Block::default()
        .title(Span::styled(title.to_string(), title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style(focused))
        .style(normal_style().bg(BG))
}
