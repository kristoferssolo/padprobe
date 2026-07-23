use super::*;
use claims::assert_some;

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
