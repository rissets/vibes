use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};
use strum::IntoEnumIterator;

use crate::app::state::{AppState, VibesMood};
use super::super::theme::*;

const MOOD_DESCS: &[&str] = &[
    "Lo-fi beats, ambient sounds, slow tempo",
    "High energy, bass drops, dance tracks",
    "Instrumental, minimal vocals, concentration",
    "Uplifting, positive vibes, sing-along",
    "Deep, moody, atmospheric sounds",
];

const EQ_CHARS: &[&str] = &["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

fn bar_char(height: u8) -> &'static str {
    let idx = ((height as usize).saturating_sub(1)).min(EQ_CHARS.len() - 1);
    EQ_CHARS[idx]
}

pub fn render_vibes(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(0),
        ])
        .split(area);

    render_mood_panel(f, chunks[0], state);
    render_recommendations(f, chunks[1], state);
}

fn render_mood_panel(f: &mut Frame, area: Rect, state: &AppState) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Mood selector
    let moods: Vec<ListItem> = VibesMood::iter()
        .enumerate()
        .map(|(i, mood)| {
            let is_sel = i == state.vibes.selected_mood;
            let desc = MOOD_DESCS[i];
            let line = Line::from(vec![
                Span::styled(if is_sel { "▶ " } else { "  " }, if is_sel { playing_style() } else { muted_style() }),
                Span::styled(format!("[{}] ", i + 1), muted_style()),
                Span::styled(mood.to_string(), if is_sel { hot_pink_style() } else { normal_style() }),
            ]);
            if is_sel {
                ListItem::new(vec![
                    line,
                    Line::from(vec![
                        Span::styled("    ", dim_style()),
                        Span::styled(desc.to_string(), dim_style()),
                    ]),
                ])
                .style(selected_style())
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let mood_list = List::new(moods).block(
        Block::default()
            .title(Span::styled(" 🌊 Select Your Vibe ", title_style()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style(true))
            .style(normal_style()),
    );
    f.render_widget(mood_list, cols[0]);

    // EQ Visualization
    let colors = [
        ACCENT, PRIMARY, HOT_PINK, NEON_GREEN, ACCENT, PRIMARY,
        HOT_PINK, NEON_GREEN, ACCENT, PRIMARY, HOT_PINK, NEON_GREEN,
    ];
    let eq_spans: Vec<Span> = state
        .eq_bars
        .iter()
        .enumerate()
        .map(|(i, &h)| {
            Span::styled(
                format!("{} ", bar_char(h)),
                ratatui::style::Style::default().fg(colors[i % colors.len()]),
            )
        })
        .collect();

    let eq_lines = vec![
        Line::from(Span::styled("  Equalizer", dim_style())),
        Line::from(eq_spans),
        Line::from(Span::styled("  ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔", muted_style())),
    ];

    let eq_block = Paragraph::new(eq_lines)
        .block(
            Block::default()
                .title(Span::styled(" ≋ Live Equalizer ", title_style()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style(false))
                .style(normal_style()),
        )
        .alignment(Alignment::Center);
    f.render_widget(eq_block, cols[1]);
}

fn render_recommendations(f: &mut Frame, area: Rect, state: &AppState) {
    if state.vibes.is_loading {
        let para = Paragraph::new(Line::from(Span::styled(
            "  ✨ Generating your vibe recommendations...",
            dim_style(),
        )))
        .block(make_block(" ✨ Recommendations ", true));
        f.render_widget(para, area);
        return;
    }

    if state.vibes.recommendations.is_empty() {
        let para = Paragraph::new(vec![
            Line::from(Span::styled("  Select a mood above and press", muted_style())),
            Line::from(Span::styled("  Enter to generate recommendations!", accent_style())),
        ])
        .block(make_block(" ✨ Recommendations ", false));
        f.render_widget(para, area);
        return;
    }

    let selected = state.vibes.selected_track;
    let items: Vec<ListItem> = state
        .vibes
        .recommendations
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let is_sel = i == selected;
            let artist = track
                .artists
                .iter()
                .map(|a| a.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let dur_ms = track.duration.num_milliseconds() as u32;
            let secs = dur_ms / 1000;
            let dur = format!("{}:{:02}", secs / 60, secs % 60);
            let prefix = if is_sel {
                "▶ ".to_string()
            } else {
                format!("{:>2}. ", i + 1)
            };
            let line = Line::from(vec![
                Span::styled(prefix, if is_sel { playing_style() } else { muted_style() }),
                Span::styled(track.name.clone(), if is_sel { selected_style() } else { normal_style() }),
                Span::styled(" — ", muted_style()),
                Span::styled(artist, dim_style()),
                Span::styled(format!("  {dur}"), muted_style()),
            ]);
            if is_sel {
                ListItem::new(line).style(selected_style())
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items).block(make_block(
        &format!(" ✨ Recommendations ({}) ", state.vibes.recommendations.len()),
        true,
    ));
    f.render_widget(list, area);
}

fn make_block(title: &str, focused: bool) -> Block<'static> {
    Block::default()
        .title(Span::styled(title.to_string(), title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style(focused))
        .style(normal_style().bg(BG))
}
