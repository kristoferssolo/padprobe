use super::*;
use claims::assert_some;

#[test]
fn event_history_evicts_oldest_entry() {
    let mut app = App::new();
    app.connect(1, metadata("controller"));

    for index in 0..EVENT_CAPACITY {
        app.push_event(Some(1), format!("event {index}"));
    }

    assert_eq!(app.events.len(), EVENT_CAPACITY);
    assert_eq!(assert_some!(app.events.front()).description, "event 0");
    assert_eq!(app.evicted_event_count, 1);
}

#[test]
fn event_filters_combine_kind_device_and_search() {
    let mut app = App::new();
    app.connect(1, metadata("first"));
    app.connect(2, metadata("second"));
    app.selected_id = Some(1);
    app.event_kind_filter = EventKindFilter::Axes;
    app.event_device_filter = EventDeviceFilter::Selected;
    app.event_search = "leftstick".to_owned();
    let visible = EventEntry {
        sequence: 1,
        elapsed: Duration::ZERO,
        device_id: Some(1),
        description: "AxisChanged(LeftStickX, +0.100)".to_owned(),
    };
    let other_device = EventEntry {
        device_id: Some(2),
        ..visible.clone()
    };

    assert!(app.event_is_visible(&visible));
    assert!(!app.event_is_visible(&other_device));
}

#[test]
fn pausing_events_captures_current_sequence() {
    let mut app = App::new();
    app.connect(1, metadata("controller"));

    app.toggle_event_scroll();
    let anchor = assert_some!(app.event_scroll_anchor);
    app.push_event(Some(1), "later event".to_owned());

    assert_eq!(app.event_scroll_anchor, Some(anchor));
    assert!(assert_some!(app.events.back()).sequence > anchor);
}

#[test]
fn pausing_an_empty_log_hides_later_events() {
    let mut app = App::new();

    app.toggle_event_scroll();
    app.push_event(None, "later event".to_owned());

    let anchor = assert_some!(app.event_scroll_anchor);
    assert!(
        app.events
            .back()
            .is_some_and(|event| event.sequence > anchor)
    );
}
