use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::state::{ActiveScreen, AppState};
use super::super::theme::*;

const NAV_ITEMS: &[(&str, &str, ActiveScreen)] = &[
    ("1", "󰍉  Search",      ActiveScreen::Search),
    ("2", "❤  Liked Songs", ActiveScreen::Library),
    ("3", "📋  Playlists",   ActiveScreen::Playlists),
    ("4", "🎵  Queue",       ActiveScreen::Queue),
    ("5", "🌊  Vibes",       ActiveScreen::Vibes),
];

pub fn render_sidebar(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(Span::styled(" 🎵 vibes ", title_style()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style(false))
        .style(normal_style());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // tagline
            Constraint::Length(7),  // nav items (5 items + 2 padding)
            Constraint::Length(1),  // separator
            Constraint::Min(0),    // now playing + animation
            Constraint::Length(3), // volume
        ])
        .split(inner);

    // ── Tagline ────────────────────────────────────────
    let tagline = Paragraph::new(Line::from(vec![
        Span::styled(" your terminal, your ", dim_style()),
        Span::styled("vibe", hot_pink_style()),
    ]));
    f.render_widget(tagline, chunks[0]);

    // ── Nav items ──────────────────────────────────────
    let items: Vec<ListItem> = NAV_ITEMS
        .iter()
        .map(|(key, label, screen)| {
            let is_active = &state.active_screen == screen;
            let prefix = if is_active { " ▶ " } else { "   " };
            
            // Clearer focus indicator with background color
            let style = if is_active { 
                Style::default().fg(BG).bg(HOT_PINK).add_modifier(Modifier::BOLD)
            } else { 
                normal_style()
            };
            
            let line = Line::from(vec![
                Span::styled(prefix, if is_active { Style::default().fg(BG).bg(HOT_PINK) } else { muted_style() }),
                Span::styled(format!("[{key}] {label}"), style),
                // Padding to fill the background block to the edge
                Span::styled(" ".repeat(area.width.saturating_sub(15) as usize), style)
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, chunks[1]);

    // ── Separator ──────────────────────────────────────
    let sep_width = chunks[2].width.saturating_sub(4) as usize;
    let sep = Paragraph::new(Line::from(Span::styled(
        format!("  {}", "─".repeat(sep_width)),
        muted_style(),
    )));
    f.render_widget(sep, chunks[2]);

    // ── Now Playing + Animation area ───────────────────
    render_now_playing_area(f, chunks[3], state);

    // ── Volume bar (bottom) ────────────────────────────
    render_volume(f, chunks[4], state);
}

const QUOTES: &[&str] = &[
    "\"Music is the universal language of mankind.\"\n  – Henry Wadsworth Longfellow",
    "\"Where words fail, music speaks.\"\n  – Hans Christian Andersen",
    "\"Without music, life would be a mistake.\"\n  – Friedrich Nietzsche",
    "\"Music gives a soul to the universe.\"\n  – Plato",
    "\"One good thing about music, when it hits you, you feel no pain.\"\n  – Bob Marley",
    "\"Music can change the world because it can change people.\"\n  – Bono",
    "\"I don't sing because I'm happy; I'm happy because I sing.\"\n  – William James",
    "\"Music is life itself.\"\n  – Louis Armstrong",
    "\"Life is like a beautiful melody, only the lyrics are messed up.\"\n  – Hans Christian Andersen",
    "\"No matter what you're going through, there's a song for that.\"",
];

fn render_now_playing_area(f: &mut Frame, area: Rect, state: &AppState) {
    if area.height < 3 {
        return;
    }

    let track = &state.current_track;

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // now playing info
            Constraint::Length(7), // animated animal
            Constraint::Min(0),    // quote
        ])
        .split(area);

    // ── Now Playing info ───────────────────────────────
    if !track.name.is_empty() {
        let play_icon = if track.is_playing { "▶" } else { "⏸" };
        let liked = if track.is_liked { " ❤" } else { "" };
        let title = truncate_str(&track.name, area.width.saturating_sub(6) as usize);
        let artist = truncate_str(
            &track.artists.join(", "),
            area.width.saturating_sub(4) as usize,
        );

        let info = Paragraph::new(vec![
            Line::from(Span::styled("  ♪ Now Playing", accent_style())),
            Line::from(vec![
                Span::styled(format!("  {play_icon} "), playing_style()),
                Span::styled(title, normal_style().add_modifier(ratatui::style::Modifier::BOLD)),
                Span::styled(liked, gold_style()),
            ]),
            Line::from(Span::styled(format!("    {artist}"), dim_style())),
            Line::from(Span::raw("")),
        ]);
        f.render_widget(info, sections[0]);
    } else {
        let empty = Paragraph::new(vec![
            Line::from(Span::styled("  ♪ Now Playing", accent_style())),
            Line::from(Span::styled("    No track", muted_style())),
            Line::from(Span::raw("")),
        ]);
        f.render_widget(empty, sections[0]);
    }

    // ── Animated Visualizer ────────────────────────────
    render_animal_visualizer(f, sections[1], state);

    // ── Quote ──────────────────────────────────────────
    if sections[2].height >= 2 {
        // Pick a stable quote based on tick / 100 so it changes every 8 seconds
        let quote_idx = ((state.eq_tick / 200) as usize) % QUOTES.len();
        let quote_text = QUOTES[quote_idx];
        
        // Format quote on a single line so Ratatui's Wrap can split it properly
        let mut lines = vec![Line::from(Span::raw(""))]; // top padding
        let formatted_quote = format!("   {}", quote_text.replace("\n", " "));
        lines.push(Line::from(Span::styled(
            formatted_quote,
            muted_style().add_modifier(ratatui::style::Modifier::ITALIC),
        )));
        
        // Apply text wrap so it doesn't get cut off
        let quote_para = Paragraph::new(lines)
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });
            
        f.render_widget(quote_para, sections[2]);
    }
}

fn render_animal_visualizer(f: &mut Frame, area: Rect, state: &AppState) {
    if area.height < 6 || area.width < 15 {
        return; // Need space for the animal
    }

    let is_playing = state.current_track.is_playing;
    
    // Animate based on eq_tick
    let frame = if is_playing { (state.eq_tick / 3) % 4 } else { 0 };

    // Switch between cat and monkey every 15 seconds
    let show_monkey = (state.eq_tick / 400) % 2 != 0;

    let animal_art = if show_monkey {
        if !is_playing && state.current_track.name.is_empty() { // Sleeping
            vec![
                "               ",
                "   zZz         ",
                "      __       ",
                "   _ (--) _    ",
                "  /(_|__|_)_\\  ",
            ]
        } else if !is_playing { // Awake but paused
            vec![
                "               ",
                "      __       ",
                "   _ (oo) _    ",
                "  /(_|__|_)_\\  ",
                "   ---(((---(((---",
            ]
        } else { // Vibing monkey
            match frame {
                0 => vec![
                    "      ♪        ",
                    "      __       ",
                    "   _ (^o) _    ",
                    "  \\(_|__|_)_/  ",
                    "   ---(((---(((---",
                ],
                1 => vec![
                    "           ♪   ",
                    "      __       ",
                    "   _ (o^) _    ",
                    "  /(_|__|_)_\\  ",
                    "   ---(((---(((---",
                ],
                2 => vec![
                    "      ♪        ",
                    "      __       ",
                    "   _ (--) _    ",
                    "  \\(_|__|_)_/  ",
                    "   ---(((---(((---",
                ],
                _ => vec![
                    "           ♪   ",
                    "      __       ",
                    "   _ (oo) _    ",
                    "  /(_|__|_)_\\  ",
                    "   ---(((---(((---",
                ],
            }
        }
    } else {
        if !is_playing && state.current_track.name.is_empty() {
            // Sleeping cat (idle)
            vec![
                "               ",
                "   zZz         ",
                "      |\\__/,|  ",
                "    _ |o o  |  ",
                "   -(((---(((---",
            ]
        } else if !is_playing {
            // Awake but paused
            vec![
                "               ",
                "      |\\__/,|  ",
                "      |o o  |  ",
                "      ( T   )  ",
                "   ---(((---(((---",
            ]
        } else {
            // Vibing cat animation frames
            match frame {
                0 => vec![
                    "      ♪        ",
                    "      |\\__/,|  ",
                    "    _ |^ ^  |  ",
                    "     \\( T   )  ",
                    "   ---(((---(((---",
                ],
                1 => vec![
                    "           ♪   ",
                    "      |\\__/,|  ",
                    "    _ |> <  |  ",
                    "     /( T   )  ",
                    "   ---(((---(((---",
                ],
                2 => vec![
                    "      ♪        ",
                    "      |\\__/,|  ",
                    "    _ |- -  |  ",
                    "     \\( T   )  ",
                    "   ---(((---(((---",
                ],
                _ => vec![
                    "           ♪   ",
                    "      |\\__/,|  ",
                    "    _ |^ ^  |  ",
                    "     /( T   )  ",
                    "   ---(((---(((---",
                ],
            }
        }
    };

    // Center the animal vertically
    let vertical_padding = area.height.saturating_sub(6) / 2;
    let mut lines = vec![Line::from(Span::raw("")); vertical_padding as usize];
    
    // Add animal lines with some color
    for (i, line) in animal_art.iter().enumerate() {
        let color = if is_playing && i == 0 { 
            // Color the music notes
            if frame % 2 == 0 { HOT_PINK } else { ACCENT }
        } else {
            PRIMARY
        };
        lines.push(Line::from(Span::styled(format!("  {line}"), Style::default().fg(color))));
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn render_volume(f: &mut Frame, area: Rect, state: &AppState) {
    let vol = state.volume;
    let bar_width = area.width.saturating_sub(12) as usize; // Extra space for alignment
    let filled = ((vol as f64 / 100.0) * bar_width as f64).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    // Center the volume bar in the area
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2), // Left padding
            Constraint::Min(0),    // Volume content
            Constraint::Length(2), // Right padding
        ])
        .split(area);

    let vol_content = chunks_for_volume(vol, filled, empty);
    
    let para = Paragraph::new(vol_content).alignment(Alignment::Left);
    f.render_widget(para, layout[1]);
}

fn chunks_for_volume(vol: u8, filled: usize, empty: usize) -> Vec<Line<'static>> {
    let vol_line = Line::from(vec![
        Span::styled(" 🔊 ", accent_style()),
        Span::styled("█".repeat(filled), playing_style()),
        Span::styled("░".repeat(empty), muted_style()),
        Span::styled(format!(" {:3}%", vol), dim_style()),
    ]);

    vec![
        Line::from(Span::raw("")), // Top padding
        vol_line,
    ]
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
