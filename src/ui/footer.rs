use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};

pub(super) fn render_footer(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let controls = if app.event_search_state == crate::app::EventSearchState::Open {
        "event filter: type text | Enter apply | Esc clear"
    } else {
        "q quit | ⇥ tab | d devices | x reset | r rumble | p pause | / filter | ? help"
    };
    let width = usize::from(area.width);
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
