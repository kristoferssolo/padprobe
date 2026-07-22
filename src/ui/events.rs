use super::layout::panel_block;
use crate::app::App;
use ratatui::{Frame, layout::Rect, text::Line, widgets::Paragraph};

pub(super) fn render_events(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let title = if app.event_scroll_anchor.is_some() {
        " Recent events — scroll paused "
    } else {
        " Recent events "
    };
    let visible_rows = area.height.saturating_sub(2) as usize;
    let entries = app
        .events
        .iter()
        .filter(|entry| {
            app.event_scroll_anchor
                .is_none_or(|anchor| entry.sequence <= anchor)
        })
        .collect::<Vec<_>>();
    let skip = entries.len().saturating_sub(visible_rows);
    let lines = entries.into_iter().skip(skip).map(|entry| {
        let seconds = entry.elapsed.as_secs();
        let millis = entry.elapsed.subsec_millis();
        let source = entry
            .device_id
            .map_or_else(|| "app".to_owned(), |id| format!("gilrs:{id}"));
        Line::from(format!(
            "{seconds:>5}.{millis:03}  {source}  {}",
            entry.description
        ))
    });
    frame.render_widget(
        Paragraph::new(lines.collect::<Vec<_>>()).block(panel_block(title)),
        area,
    );
}
