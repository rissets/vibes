use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::state::AppState;
use super::super::theme::*;

pub fn render_help(f: &mut Frame, area: Rect, _state: &AppState) {
    // Center the popup
    let popup_area = centered_rect(60, 80, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(" ❓ Keybindings ", title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(border_style(true))
        .style(normal_style());

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .margin(1)
        .split(inner);

    let left = vec![
        Line::from(Span::styled("  Navigation", hot_pink_style().add_modifier(ratatui::style::Modifier::BOLD))),
        Line::from(Span::raw("")),
        key_line("↑ / k", "Move up"),
        key_line("↓ / j", "Move down"),
        key_line("Enter", "Select / Play"),
        key_line("Esc / b", "Back"),
        key_line("1-5", "Switch screen"),
        Line::from(Span::raw("")),
        Line::from(Span::styled("  Playback", hot_pink_style().add_modifier(ratatui::style::Modifier::BOLD))),
        Line::from(Span::raw("")),
        key_line("Space", "Pause / Resume"),
        key_line("n", "Next track"),
        key_line("p", "Previous track"),
        key_line("f / →", "Seek forward"),
        key_line("r / ←", "Seek backward"),
        key_line("+ / =", "Volume up"),
        key_line("-", "Volume down"),
    ];

    let right = vec![
        Line::from(Span::styled("  Library", hot_pink_style().add_modifier(ratatui::style::Modifier::BOLD))),
        Line::from(Span::raw("")),
        key_line("l", "Like / Unlike track"),
        key_line("a", "Add to queue"),
        key_line("s", "Open search"),
        Line::from(Span::raw("")),
        Line::from(Span::styled("  Screens", hot_pink_style().add_modifier(ratatui::style::Modifier::BOLD))),
        Line::from(Span::raw("")),
        key_line("[1]", "Search"),
        key_line("[2]", "Liked Songs"),
        key_line("[3]", "Playlists"),
        key_line("[4]", "Queue"),
        key_line("[5]", "Vibes"),
        Line::from(Span::raw("")),
        key_line("?", "Toggle this help"),
        key_line("q", "Quit"),
    ];

    f.render_widget(Paragraph::new(left), cols[0]);
    f.render_widget(Paragraph::new(right), cols[1]);
}

fn key_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ", muted_style()),
        Span::styled(format!("{key:<10}"), accent_style()),
        Span::styled(desc.to_string(), normal_style()),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
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
        .split(popup_layout[1])[1]
}
