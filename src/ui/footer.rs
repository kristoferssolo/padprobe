use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::app::{App, Focus};

pub(super) fn render_footer(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let controls = if app.focus == Focus::Events {
        "q quit | r rumble | tab focus | p pause events | ? help"
    } else {
        "q quit | r rumble | ↑↓/jk select | tab focus | ? help"
    };
    let width = area.width as usize;
    let status_room = width.saturating_sub(controls.len() + 3);
    let status = if status_room > 8 {
        format!(" | {:.status_room$}", app.status)
    } else {
        String::new()
    };
    frame.render_widget(
        Paragraph::new(format!("{controls}{status}")).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}
