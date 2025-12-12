use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Application keybinding actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Help,
    Search,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    GoToTop,
    GoToBottom,
    Select,
    Back,
    Forward,
    ToggleCollapse,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    OpenInBrowser,
    ViewComments,
    SaveBookmark,
    CopyUrl,
    NextResult,
    PrevResult,
    SwitchTab,
    Refresh,
    NextArticle,
    PrevArticle,
    None,
}

/// Map keyboard events to actions
pub fn map_key_event(event: KeyEvent, vim_mode: bool) -> Action {
    match (event.code, event.modifiers) {
        // Quit
        (KeyCode::Char('q'), KeyModifiers::NONE) => Action::Quit,
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Action::Quit,
        (KeyCode::Char('q'), KeyModifiers::SUPER) => Action::Quit, // Cmd+Q

        // Help
        (KeyCode::Char('?'), KeyModifiers::NONE) => Action::Help,
        (KeyCode::F(1), KeyModifiers::NONE) => Action::Help,

        // Search
        (KeyCode::Char('/'), KeyModifiers::NONE) => Action::Search,
        (KeyCode::Char('f'), KeyModifiers::SUPER) => Action::Search, // Cmd+F

        // Tab switching
        (KeyCode::Tab, KeyModifiers::NONE) => Action::SwitchTab,
        (KeyCode::BackTab, KeyModifiers::SHIFT) => Action::SwitchTab, // Shift+Tab

        // macOS-style Back navigation (Cmd+[ or Cmd+Left or Backspace or Esc)
        (KeyCode::Esc, KeyModifiers::NONE) => Action::Back,
        (KeyCode::Char('['), KeyModifiers::SUPER) => Action::Back, // Cmd+[
        (KeyCode::Left, KeyModifiers::SUPER) => Action::Back,      // Cmd+Left
        (KeyCode::Backspace, KeyModifiers::NONE) => Action::Back,  // Backspace
        (KeyCode::Left, KeyModifiers::ALT) => Action::Back,        // Alt+Left (Option+Left)

        // macOS-style Forward navigation (Cmd+] or Cmd+Right)
        (KeyCode::Char(']'), KeyModifiers::SUPER) => Action::Forward, // Cmd+]
        (KeyCode::Right, KeyModifiers::SUPER) => Action::Forward,     // Cmd+Right
        (KeyCode::Right, KeyModifiers::ALT) => Action::Forward,       // Alt+Right (Option+Right)

        // Refresh
        (KeyCode::Char('r'), KeyModifiers::NONE) => Action::Refresh,
        (KeyCode::Char('r'), KeyModifiers::SUPER) => Action::Refresh, // Cmd+R
        (KeyCode::F(5), KeyModifiers::NONE) => Action::Refresh,       // F5

        // Navigation - Arrow keys
        (KeyCode::Up, KeyModifiers::NONE) => Action::NavigateUp,
        (KeyCode::Down, KeyModifiers::NONE) => Action::NavigateDown,
        (KeyCode::Left, KeyModifiers::NONE) => Action::NavigateLeft,
        (KeyCode::Right, KeyModifiers::NONE) => Action::NavigateRight,
        (KeyCode::Enter, KeyModifiers::NONE) => Action::Select,
        (KeyCode::Char(' '), KeyModifiers::NONE) => Action::ToggleCollapse,

        // Quick article navigation ([ and ] without modifier)
        (KeyCode::Char('['), KeyModifiers::NONE) => Action::PrevArticle,
        (KeyCode::Char(']'), KeyModifiers::NONE) => Action::NextArticle,

        // Vim-style navigation (only if vim_mode is enabled)
        (KeyCode::Char('j'), KeyModifiers::NONE) if vim_mode => Action::NavigateDown,
        (KeyCode::Char('k'), KeyModifiers::NONE) if vim_mode => Action::NavigateUp,
        (KeyCode::Char('h'), KeyModifiers::NONE) if vim_mode => Action::NavigateLeft,
        (KeyCode::Char('l'), KeyModifiers::NONE) if vim_mode => Action::NavigateRight,
        (KeyCode::Char('g'), KeyModifiers::NONE) if vim_mode => Action::GoToTop,
        (KeyCode::Char('G'), KeyModifiers::SHIFT) if vim_mode => Action::GoToBottom,

        // Home/End keys
        (KeyCode::Home, KeyModifiers::NONE) => Action::GoToTop,
        (KeyCode::End, KeyModifiers::NONE) => Action::GoToBottom,

        // Scrolling
        (KeyCode::Char('d'), KeyModifiers::NONE) if vim_mode => Action::PageDown,
        (KeyCode::Char('u'), KeyModifiers::NONE) if vim_mode => Action::PageUp,
        (KeyCode::PageDown, KeyModifiers::NONE) => Action::PageDown,
        (KeyCode::PageUp, KeyModifiers::NONE) => Action::PageUp,
        (KeyCode::Down, KeyModifiers::SUPER) => Action::PageDown, // Cmd+Down
        (KeyCode::Up, KeyModifiers::SUPER) => Action::PageUp,     // Cmd+Up

        // Actions
        (KeyCode::Char('o'), KeyModifiers::NONE) => Action::OpenInBrowser,
        (KeyCode::Char('o'), KeyModifiers::SUPER) => Action::OpenInBrowser, // Cmd+O
        (KeyCode::Char('c'), KeyModifiers::NONE) => Action::ViewComments,
        (KeyCode::Char('s'), KeyModifiers::NONE) => Action::SaveBookmark,
        (KeyCode::Char('y'), KeyModifiers::NONE) => Action::CopyUrl,
        (KeyCode::Char('c'), KeyModifiers::SUPER) => Action::CopyUrl, // Cmd+C copies URL

        // Search navigation
        (KeyCode::Char('n'), KeyModifiers::NONE) => Action::NextResult,
        (KeyCode::Char('N'), KeyModifiers::SHIFT) => Action::PrevResult,
        (KeyCode::Char('g'), KeyModifiers::SUPER) => Action::NextResult, // Cmd+G (find next)

        _ => Action::None,
    }
}

/// Get keybinding help text
pub fn get_help_text(vim_mode: bool) -> Vec<(&'static str, &'static str)> {
    let mut bindings = vec![
        ("Global", ""),
        ("q / Cmd+Q", "Quit application"),
        ("? / F1", "Show help"),
        ("/ / Cmd+F", "Search"),
        ("Tab", "Switch panes"),
        ("r / Cmd+R / F5", "Refresh"),
        ("", ""),
        ("Navigation", ""),
        ("↑/↓", "Move up/down"),
        ("←/→", "Switch panes"),
        ("Enter", "Open article"),
        ("[ / ]", "Prev/Next article"),
        ("Home/End", "Go to top/bottom"),
        ("", ""),
        ("Back/Forward", ""),
        ("Esc / ⌫", "Go back"),
        ("Cmd+[ / Opt+←", "Go back"),
        ("Cmd+] / Opt+→", "Go forward"),
        ("", ""),
        ("Article View", ""),
        ("o / Cmd+O", "Open in browser"),
        ("c", "View comments"),
        ("Cmd+C / y", "Copy URL"),
        ("PgUp/PgDn", "Scroll page"),
        ("", ""),
        ("Search", ""),
        ("n / Cmd+G", "Next result"),
        ("N", "Previous result"),
    ];

    if vim_mode {
        bindings.extend(vec![
            ("", ""),
            ("Vim Mode", ""),
            ("j/k", "Move down/up"),
            ("h/l", "Left/Right"),
            ("g/G", "Top/Bottom"),
            ("d/u", "Page down/up"),
        ]);
    }

    bindings
}
