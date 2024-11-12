use ratatui::style::{Color, Modifier, Style};

// Base styles
pub const BASE_STYLE: Style = Style::new().fg(Color::White);
pub const HIGHLIGHT_STYLE: Style = Style::new().fg(Color::Black).bg(Color::Cyan);
pub const BOLD_STYLE: Style = Style::new().add_modifier(Modifier::BOLD);

// Title styles
pub const TITLE_STYLE: Style = Style::new()
    .fg(Color::Cyan)
    .add_modifier(Modifier::BOLD);

// Menu styles
pub const MENU_STYLE: Style = Style::new().fg(Color::White);
pub const MENU_HIGHLIGHT_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::Cyan)
    .add_modifier(Modifier::BOLD);

// Status styles
pub const STATUS_LABEL_STYLE: Style = Style::new().fg(Color::Gray);
pub const STATUS_VALUE_STYLE: Style = Style::new().fg(Color::Green);
pub const STATUS_WARNING_STYLE: Style = Style::new().fg(Color::Yellow);

// Log styles
pub const LOG_ERROR_STYLE: Style = Style::new().fg(Color::Red);
pub const LOG_SUCCESS_STYLE: Style = Style::new().fg(Color::Green);
pub const LOG_DEBUG_STYLE: Style = Style::new().fg(Color::Yellow);
pub const LOG_INFO_STYLE: Style = Style::new().fg(Color::White);

// Border styles
pub const BORDER_STYLE: Style = Style::new().fg(Color::DarkGray);
pub const ACTIVE_BORDER_STYLE: Style = Style::new().fg(Color::Cyan);

pub const AUTOMATE_STYLE: Style = Style::new()
    .fg(Color::Red)
    .add_modifier(Modifier::BOLD);

pub const AUTOMATE_SELECTED_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::LightRed)
    .add_modifier(Modifier::BOLD);
