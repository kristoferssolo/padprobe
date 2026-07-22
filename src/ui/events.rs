use super::layout::panel_block;
use crate::app::{App, EventEntry};
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
    let lines = entries
        .into_iter()
        .skip(skip)
        .map(|entry| event_line(entry, area.width));
    frame.render_widget(
        Paragraph::new(lines.collect::<Vec<_>>()).block(panel_block(title)),
        area,
    );
}

fn event_line(entry: &EventEntry, width: u16) -> Line<'static> {
    let seconds = entry.elapsed.as_secs();
    let millis = entry.elapsed.subsec_millis();
    if width < 40 {
        return Line::from(format!("{seconds}.{millis:03}  {}", entry.description));
    }
    let source = entry
        .device_id
        .map_or_else(|| "app".to_owned(), |id| format!("gilrs:{id}"));
    Line::from(format!(
        "{seconds}.{millis:03}  {source}  {}",
        entry.description
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn event_line_has_no_leading_timestamp_padding() {
        let entry = EventEntry {
            sequence: 0,
            elapsed: Duration::from_millis(5_359),
            device_id: Some(0),
            description: "ButtonPressed(South)".to_owned(),
        };

        assert!(
            event_line(&entry, 60)
                .to_string()
                .starts_with("5.359  gilrs:0")
        );
    }
}
