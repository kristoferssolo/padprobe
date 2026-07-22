use super::*;
use claims::assert_some;

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
fn disconnect_clears_stale_input_state() {
    let mut app = App::new();
    app.connect(4, metadata("controller"));
    let device = assert_some!(app.devices.get_mut(&4));
    apply_button_value(device, gilrs::Button::LeftTrigger2, 1.0);
    device.axes.insert(Axis::LeftStickX, AxisState::new(0.75));
    update_stick_trace(device, Axis::LeftStickX);

    app.disconnect(4);

    let device = &app.devices[&4];
    assert!(device.buttons.is_empty());
    assert!(device.button_values.is_empty());
    assert!(device.axes.is_empty());
    assert!(device.left_stick_trace.points().is_empty());
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

    assert!((state.current - 0.5).abs() < f32::EPSILON);
    assert!((state.minimum - -0.7).abs() < f32::EPSILON);
    assert!((state.maximum - 0.5).abs() < f32::EPSILON);
    assert_eq!(state.changes, 3);
}

#[test]
fn session_reset_preserves_live_input_and_clears_observations() {
    let mut app = App::new();
    app.connect(1, metadata("controller"));
    let device = assert_some!(app.devices.get_mut(&1));
    device.axes.insert(Axis::LeftStickX, AxisState::new(-0.4));
    assert_some!(device.axes.get_mut(&Axis::LeftStickX)).update(0.25);
    device.buttons.insert(gilrs::Button::South, true);
    update_stick_trace(device, Axis::LeftStickX);

    app.reset_selected_observations();

    let device = &app.devices[&1];
    let axis = device.axes[&Axis::LeftStickX];
    assert!((axis.current - 0.25).abs() < f32::EPSILON);
    assert!((axis.minimum - 0.25).abs() < f32::EPSILON);
    assert!((axis.maximum - 0.25).abs() < f32::EPSILON);
    assert_eq!(axis.changes, 0);
    assert!(device.buttons[&gilrs::Button::South]);
    assert!(device.left_stick_trace.points().is_empty());
}

#[test]
fn analog_button_values_are_preserved() {
    let mut app = App::new();
    app.connect(1, metadata("controller"));
    let device = assert_some!(app.devices.get_mut(&1));

    apply_button_value(device, gilrs::Button::LeftTrigger2, 0.37);

    let value = app.devices[&1].button_values[&gilrs::Button::LeftTrigger2];
    assert!((value - 0.37).abs() < f32::EPSILON);
}

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

#[test]
fn device_selector_visibility_is_explicit() {
    let mut app = App::new();

    app.open_device_selector();
    assert!(app.device_selector_visible);

    app.close_device_selector();
    assert!(!app.device_selector_visible);
}

#[test]
fn tab_navigation_wraps_in_both_directions() {
    let mut app = App::new();

    app.select_previous_tab();
    assert_eq!(app.active_tab, AppTab::Timing);
    app.select_next_tab();
    assert_eq!(app.active_tab, AppTab::Dashboard);
}

#[test]
fn stick_trace_records_paired_axis_positions() {
    let mut app = App::new();
    app.connect(1, metadata("controller"));
    let device = assert_some!(app.devices.get_mut(&1));
    device.axes.insert(Axis::LeftStickX, AxisState::new(0.5));
    update_stick_trace(device, Axis::LeftStickX);
    device.axes.insert(Axis::LeftStickY, AxisState::new(-0.25));
    update_stick_trace(device, Axis::LeftStickY);

    assert_eq!(device.left_stick_trace.points(), [(0.5, 0.0), (0.5, -0.25)]);
}

#[test]
fn stick_trace_remains_bounded() {
    let mut trace = StickTrace::default();

    for index in 0_u16..512 {
        trace.push(f32::from(index), 0.0);
    }

    assert!(trace.points().len() <= 256);
    assert_eq!(trace.points().last(), Some(&(511.0, 0.0)));
}
