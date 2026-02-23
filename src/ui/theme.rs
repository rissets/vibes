use ratatui::style::{Color, Modifier, Style};

// ─── Color Palette ───────────────────────────────────────────────────────────
pub const BG:          Color = Color::Rgb(13,  13,  17);
pub const SURFACE:     Color = Color::Rgb(28,  28,  40);
pub const SURFACE_SEL: Color = Color::Rgb(40,  35,  65);

pub const PRIMARY:     Color = Color::Rgb(155, 93,  229); // electric violet
pub const ACCENT:      Color = Color::Rgb(0,   245, 255); // neon cyan
pub const HOT_PINK:    Color = Color::Rgb(241, 91,  181); // hot pink
pub const NEON_GREEN:  Color = Color::Rgb(0,   187, 249); // neon blue-green
pub const GOLD:        Color = Color::Rgb(255, 210, 63);  // gold/liked

pub const TEXT:        Color = Color::Rgb(220, 220, 235);
pub const TEXT_DIM:    Color = Color::Rgb(140, 140, 160);
pub const TEXT_MUTED:  Color = Color::Rgb(80,  80,  100);

pub const BORDER:      Color = Color::Rgb(50,  45,  80);
pub const BORDER_FOCUSED: Color = PRIMARY;

pub const ERROR:       Color = Color::Rgb(255, 90,  90);

// ─── Styles ──────────────────────────────────────────────────────────────────
pub fn title_style() -> Style {
    Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)
}

pub fn accent_style() -> Style {
    Style::default().fg(ACCENT)
}

pub fn selected_style() -> Style {
    Style::default()
        .bg(SURFACE_SEL)
        .fg(ACCENT)
        .add_modifier(Modifier::BOLD)
}

pub fn normal_style() -> Style {
    Style::default().fg(TEXT)
}

pub fn dim_style() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn muted_style() -> Style {
    Style::default().fg(TEXT_MUTED)
}

pub fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(BORDER_FOCUSED)
    } else {
        Style::default().fg(BORDER)
    }
}

pub fn playing_style() -> Style {
    Style::default().fg(NEON_GREEN).add_modifier(Modifier::BOLD)
}

pub fn hot_pink_style() -> Style {
    Style::default().fg(HOT_PINK).add_modifier(Modifier::BOLD)
}

pub fn gold_style() -> Style {
    Style::default().fg(GOLD)
}

pub fn error_style() -> Style {
    Style::default().fg(ERROR).add_modifier(Modifier::BOLD)
}

pub fn header_style() -> Style {
    Style::default()
        .fg(BG)
        .bg(PRIMARY)
        .add_modifier(Modifier::BOLD)
}
