use ratatui::style::{Color, Modifier, Style};

/// Styles shared by the gamepad overview and analog-stick gauge.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct GamepadTheme {
    /// Controller outlines, cluster borders, and stick gates.
    pub border: Style,
    /// Controls that are currently inactive.
    pub idle: Style,
    /// Pressed buttons and active analog controls.
    pub active: Style,
    /// Observed analog-stick traces and analysis metrics.
    pub trace: Style,
    /// Labels and numerical values.
    pub value: Style,
}

impl Default for GamepadTheme {
    fn default() -> Self {
        Self {
            border: Style::default().fg(Color::DarkGray),
            idle: Style::default(),
            active: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            trace: Style::default().fg(Color::Green),
            value: Style::default(),
        }
    }
}

impl GamepadTheme {
    /// Returns a theme that does not depend on terminal colors.
    #[must_use]
    pub fn monochrome() -> Self {
        Self {
            border: Style::default().add_modifier(Modifier::DIM),
            idle: Style::default(),
            active: Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED),
            trace: Style::default().add_modifier(Modifier::UNDERLINED),
            value: Style::default(),
        }
    }
}
