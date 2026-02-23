use ratatui::{
    layout::{Constraint, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::state::AppState;
use super::super::theme::*;

pub fn render_library(f: &mut Frame, area: Rect, state: &AppState) {
    if state.library.is_loading {
        let para =
            Paragraph::new(Line::from(Span::styled("  ⠋ Loading liked songs...", dim_style())))
                .block(make_block(" ❤  Liked Songs ", true));
        f.render_widget(para, area);
        return;
    }

    if state.library.liked_songs.is_empty() {
        let para = Paragraph::new(Line::from(Span::styled(
            "  No liked songs yet. Open Spotify and like some tracks!",
            muted_style(),
        )))
        .block(make_block(" ❤  Liked Songs ", false));
        f.render_widget(para, area);
        return;
    }

    let selected = state.library.selected;
    let rows: Vec<Row> = state
        .library
        .liked_songs
        .iter()
        .enumerate()
        .map(|(i, saved)| {
            let track = &saved.track;
            let is_sel = i == selected;
            let num = if is_sel { "▶".to_string() } else { format!("{:>3}", i + 1) };
            let title = track.name.clone();
            let artist = track
                .artists
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let album = track.album.name.clone();
            let dur_ms = track.duration.num_milliseconds() as u32;
            let secs = dur_ms / 1000;
            let dur = format!("{}:{:02}", secs / 60, secs % 60);

            let style = if is_sel { selected_style() } else { normal_style() };
            Row::new(vec![
                Cell::from(num).style(if is_sel { playing_style() } else { muted_style() }),
                Cell::from(title).style(style.clone()),
                Cell::from(artist).style(dim_style()),
                Cell::from(album).style(muted_style()),
                Cell::from(dur).style(muted_style()),
            ])
            .style(style)
        })
        .collect();

    let header = Row::new(vec![
        Cell::from(" # ").style(header_style()),
        Cell::from("Title").style(header_style()),
        Cell::from("Artist").style(header_style()),
        Cell::from("Album").style(header_style()),
        Cell::from("Dur").style(header_style()),
    ])
    .height(1);

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(30),
            Constraint::Length(7),
        ],
    )
    .header(header)
    .block(make_block(
        &format!(" ❤  Liked Songs ({}) ", state.library.liked_songs.len()),
        true,
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
