use super::*;
use claims::assert_some;

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
