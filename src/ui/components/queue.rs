use ratatui::{
    layout::{Constraint, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::state::AppState;
use super::super::theme::*;

pub fn render_queue(f: &mut Frame, area: Rect, state: &AppState) {
    if state.queue.is_loading {
        let para = Paragraph::new(Line::from(Span::styled("  ⠋ Loading queue...", dim_style())))
            .block(make_block(" 🎵 Queue ", true));
        f.render_widget(para, area);
        return;
    }

    if state.queue.tracks.is_empty() {
        let para = Paragraph::new(Line::from(Span::styled(
            "  Queue is empty. Press [a] on any track to add it.",
            muted_style(),
        )))
        .block(make_block(" 🎵 Queue ", false));
        f.render_widget(para, area);
        return;
    }

    let selected = state.queue.selected;
    let rows: Vec<Row> = state
        .queue
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
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
            let prefix = if i == 0 {
                "Next ▶".to_string()
            } else {
                format!("{}", i + 1)
            };
            let style = if is_sel {
                selected_style()
            } else if i == 0 {
                accent_style()
            } else {
                normal_style()
            };
            let num_style = if is_sel {
                playing_style()
            } else if i == 0 {
                accent_style()
            } else {
                muted_style()
            };
            Row::new(vec![
                Cell::from(prefix).style(num_style),
                Cell::from(track.name.clone()).style(style.clone()),
                Cell::from(artist).style(dim_style()),
                Cell::from(dur).style(muted_style()),
            ])
            .style(style)
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
            Constraint::Length(7),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Length(7),
        ],
    )
    .header(header)
    .block(make_block(
        &format!(" 🎵 Queue ({} tracks) ", state.queue.tracks.len()),
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
