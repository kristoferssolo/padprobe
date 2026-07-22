use super::layout::panel_block;
use crate::app::{App, EventEntry};
use ratatui::{Frame, layout::Rect, text::Line, widgets::Paragraph};

pub(super) fn render_events(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let state = if app.event_scroll_anchor.is_some() {
        "paused"
    } else {
        "live"
    };
    let device = if app.event_device_filter == crate::app::EventDeviceFilter::Selected {
        " · selected"
    } else {
        ""
    };
    let search = if app.event_search.is_empty() {
        String::new()
    } else {
        format!(" · /{}/", app.event_search)
    };
    let title = format!(
        " Recent events · {state} · {}{device}{search} · evicted {} ",
        app.event_kind_filter.label(),
        app.evicted_event_count
    );
    let visible_rows = usize::from(area.height.saturating_sub(2));
    let entries = app
        .events
        .iter()
        .filter(|entry| {
            app.event_scroll_anchor
                .is_none_or(|anchor| entry.sequence <= anchor)
        })
        .filter(|entry| app.event_is_visible(entry))
        .collect::<Vec<_>>();
    let entries = coalesced_entries(&entries);
    let end = entries.len().saturating_sub(app.event_scroll_offset);
    let start = end.saturating_sub(visible_rows);
    let lines = entries[start..end]
        .iter()
        .map(|entry| event_line(entry.entry, entry.count, area.width));
    frame.render_widget(
        Paragraph::new(lines.collect::<Vec<_>>()).block(panel_block(&title)),
        area,
    );
}

#[derive(Clone, Copy)]
struct DisplayEntry<'entry> {
    entry: &'entry EventEntry,
    count: usize,
}

fn coalesced_entries<'entry>(entries: &[&'entry EventEntry]) -> Vec<DisplayEntry<'entry>> {
    let mut display = Vec::<DisplayEntry<'_>>::with_capacity(entries.len());
    for entry in entries {
        let key = axis_event_key(&entry.description);
        if let Some(previous) = display.last_mut()
            && key.is_some()
            && key == axis_event_key(&previous.entry.description)
            && entry.device_id == previous.entry.device_id
        {
            previous.entry = entry;
            previous.count += 1;
            continue;
        }
        display.push(DisplayEntry { entry, count: 1 });
    }
    display
}

fn axis_event_key(description: &str) -> Option<&str> {
    description
        .strip_prefix("AxisChanged(")?
        .split_once(',')
        .map(|(axis, _)| axis)
}

fn event_line(entry: &EventEntry, count: usize, width: u16) -> Line<'static> {
    let seconds = entry.elapsed.as_secs();
    let millis = entry.elapsed.subsec_millis();
    let repeated = if count > 1 {
        format!(" ×{count}")
    } else {
        String::new()
    };
    if width < 40 {
        return Line::from(format!(
            "{seconds}.{millis:03}  {}{repeated}",
            entry.description
        ));
    }
    let source = entry
        .device_id
        .map_or_else(|| "app".to_owned(), |id| format!("gilrs:{id}"));
    Line::from(format!(
        "{seconds}.{millis:03}  {source}  {}{repeated}",
        entry.description,
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
            event_line(&entry, 1, 60)
                .to_string()
                .starts_with("5.359  gilrs:0")
        );
    }

    #[test]
    fn consecutive_axis_events_are_coalesced_by_control() {
        let first = EventEntry {
            sequence: 1,
            elapsed: Duration::from_millis(1),
            device_id: Some(0),
            description: "AxisChanged(LeftStickX, +0.100)".to_owned(),
        };
        let second = EventEntry {
            sequence: 2,
            elapsed: Duration::from_millis(2),
            device_id: Some(0),
            description: "AxisChanged(LeftStickX, +0.200)".to_owned(),
        };

        let display = coalesced_entries(&[&first, &second]);

        assert_eq!(display.len(), 1);
        assert_eq!(display[0].count, 2);
        assert_eq!(display[0].entry.sequence, 2);
    }
}
