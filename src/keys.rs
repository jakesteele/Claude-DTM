use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    FocusNext,
    FocusPrev,
    SwapNext,
    SwapPrev,
    ZoomMaster,
    ShrinkMaster,
    GrowMaster,
    IncMasterCount,
    DecMasterCount,
    LayoutMasterStack,
    LayoutMonocle,
    LayoutGrid,
    NewSession,
    KillSession,
    PauseSession,
    ResumeSession,
    EnterPane,
    ExitPane,
    FocusPane(usize),
    SearchSession,
    ToggleZoom,
    Quit,
    DialogConfirm,
    DialogCancel,
    DialogInput(char),
    DialogBackspace,
    DialogUp,
    DialogDown,
    PassThrough(KeyEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    PaneFocused,
    Dialog,
}

pub fn map_key(key: KeyEvent, mode: InputMode) -> Option<Action> {
    match mode {
        InputMode::PaneFocused => {
            if key.code == KeyCode::Esc {
                Some(Action::ExitPane)
            } else {
                Some(Action::PassThrough(key))
            }
        }
        InputMode::Dialog => match key.code {
            KeyCode::Enter => Some(Action::DialogConfirm),
            KeyCode::Esc => Some(Action::DialogCancel),
            KeyCode::Backspace => Some(Action::DialogBackspace),
            KeyCode::Up => Some(Action::DialogUp),
            KeyCode::Down => Some(Action::DialogDown),
            KeyCode::Tab => Some(Action::DialogInput('\t')),
            KeyCode::Char(c) => Some(Action::DialogInput(c)),
            _ => None,
        },
        InputMode::Normal => {
            let shift = key.modifiers.contains(KeyModifiers::SHIFT);
            match key.code {
                KeyCode::Char('j') if !shift => Some(Action::FocusNext),
                KeyCode::Char('k') if !shift => Some(Action::FocusPrev),
                KeyCode::Char('J') | KeyCode::Char('j') if shift => Some(Action::SwapNext),
                KeyCode::Char('K') | KeyCode::Char('k') if shift => Some(Action::SwapPrev),
                KeyCode::Enter => Some(Action::EnterPane),
                KeyCode::Char('z') => Some(Action::ToggleZoom),
                KeyCode::Char('h') => Some(Action::ShrinkMaster),
                KeyCode::Char('l') => Some(Action::GrowMaster),
                KeyCode::Char('i') => Some(Action::IncMasterCount),
                KeyCode::Char('d') => Some(Action::DecMasterCount),
                KeyCode::Char('t') => Some(Action::LayoutMasterStack),
                KeyCode::Char('m') => Some(Action::LayoutMonocle),
                KeyCode::Char('g') => Some(Action::LayoutGrid),
                KeyCode::Char('n') => Some(Action::NewSession),
                KeyCode::Char('q') if !shift => Some(Action::KillSession),
                KeyCode::Char('p') => Some(Action::PauseSession),
                KeyCode::Char('r') => Some(Action::ResumeSession),
                KeyCode::Char('s') => Some(Action::SearchSession),
                KeyCode::Esc => Some(Action::ExitPane),
                KeyCode::Char('Q') | KeyCode::Char('q') if shift => Some(Action::Quit),
                KeyCode::Char(c @ '1'..='9') => {
                    Some(Action::FocusPane(c.to_digit(10).unwrap() as usize - 1))
                }
                _ => None,
            }
        }
    }
}
