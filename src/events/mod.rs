use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum UserAction {
    Quit,
    ToggleHelp,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    Select,
    Back,
    TogglePlay,
    NextTrack,
    PrevTrack,
    VolumeUp,
    VolumeDown,
    LikeTrack,
    AddToQueue,
    OpenSearch,
    SearchInput(char),
    SearchBackspace,
    SearchSubmit,
    SwitchScreen(u8),
    SeekForward,
    SeekBackward,
    ToggleEQ,
}

pub fn map_key_to_action(key: KeyEvent, search_active: bool) -> Option<UserAction> {
    if search_active {
        return match key.code {
            KeyCode::Esc => Some(UserAction::Back),
            KeyCode::Enter => Some(UserAction::SearchSubmit),
            KeyCode::Backspace => Some(UserAction::SearchBackspace),
            KeyCode::Char(c) => Some(UserAction::SearchInput(c)),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('q') => Some(UserAction::Quit),
        KeyCode::Char('?') => Some(UserAction::ToggleHelp),
        KeyCode::Up | KeyCode::Char('k') => Some(UserAction::NavigateUp),
        KeyCode::Down | KeyCode::Char('j') => Some(UserAction::NavigateDown),
        KeyCode::Left => Some(UserAction::NavigateLeft),
        KeyCode::Char('h') => Some(UserAction::NavigateLeft),
        KeyCode::Right => Some(UserAction::NavigateRight),
        KeyCode::Enter => Some(UserAction::Select),
        KeyCode::Esc | KeyCode::Char('b') => Some(UserAction::Back),
        KeyCode::Char(' ') => Some(UserAction::TogglePlay),
        KeyCode::Char('n') => Some(UserAction::NextTrack),
        KeyCode::Char('p') => Some(UserAction::PrevTrack),
        KeyCode::Char('+') | KeyCode::Char('=') => Some(UserAction::VolumeUp),
        KeyCode::Char('-') => Some(UserAction::VolumeDown),
        KeyCode::Char('l') => Some(UserAction::LikeTrack),
        KeyCode::Char('a') => Some(UserAction::AddToQueue),
        KeyCode::Char('s') => Some(UserAction::OpenSearch),
        KeyCode::Char('1') => Some(UserAction::SwitchScreen(1)),
        KeyCode::Char('2') => Some(UserAction::SwitchScreen(2)),
        KeyCode::Char('3') => Some(UserAction::SwitchScreen(3)),
        KeyCode::Char('4') => Some(UserAction::SwitchScreen(4)),
        KeyCode::Char('5') => Some(UserAction::SwitchScreen(5)),
        KeyCode::Char('f') => Some(UserAction::SeekForward),
        KeyCode::Char('r') => Some(UserAction::SeekBackward),
        KeyCode::Char('e') => Some(UserAction::ToggleEQ),
        _ => None,
    }
}
