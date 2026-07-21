use super::*;

fn metadata(name: &str) -> DeviceMetadata {
    DeviceMetadata {
        name: name.to_owned(),
        vendor_id: None,
        product_id: None,
        uuid: "00".repeat(16),
        mapping: "none".to_owned(),
        power: "Unknown".to_owned(),
        rumble_supported: false,
    }
}

#[test]
fn selects_first_connected_device() {
    let mut app = App::new();
    app.connect(4, metadata("first"));
    app.connect(8, metadata("second"));

    assert_eq!(app.selected_id, Some(4));
}

#[test]
fn keeps_disconnected_device_selected() {
    let mut app = App::new();
    app.connect(4, metadata("first"));
    app.connect(8, metadata("second"));

    app.disconnect(4);

    assert_eq!(app.selected_id, Some(4));
    assert!(!app.devices[&4].connected);
}

#[test]
fn navigation_explicitly_leaves_disconnected_selection() {
    let mut app = App::new();
    app.connect(4, metadata("first"));
    app.connect(8, metadata("second"));
    app.disconnect(4);

    app.select_next();

    assert_eq!(app.selected_id, Some(8));
}

#[test]
fn axis_updates_preserve_observed_range() {
    let mut state = AxisState::new(0.2);
    state.update(-0.7);
    state.update(0.5);

    assert_eq!(state.current, 0.5);
    assert_eq!(state.minimum, -0.7);
    assert_eq!(state.maximum, 0.5);
    assert_eq!(state.changes, 3);
}

#[test]
fn event_history_evicts_oldest_entry() {
    let mut app = App::new();
    app.connect(1, metadata("controller"));

    for index in 0..EVENT_CAPACITY {
        app.push_event(Some(1), format!("event {index}"));
    }

    assert_eq!(app.events.len(), EVENT_CAPACITY);
    assert_eq!(app.events.front().unwrap().description, "event 0");
}

#[test]
fn pausing_events_captures_current_sequence() {
    let mut app = App::new();
    app.connect(1, metadata("controller"));

    app.toggle_event_scroll();
    let anchor = app.event_scroll_anchor;
    app.push_event(Some(1), "later event".to_owned());

    assert!(anchor.is_some());
    assert_eq!(app.event_scroll_anchor, anchor);
    assert!(app.events.back().unwrap().sequence > anchor.unwrap());
}

#[test]
fn device_selector_visibility_is_explicit() {
    let mut app = App::new();

    app.open_device_selector();
    assert!(app.device_selector_visible);

    app.close_device_selector();
    assert!(!app.device_selector_visible);
}
