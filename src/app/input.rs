#[cfg(test)]
use super::metadata;
use super::{App, AxisState, DeviceState};
use crate::analysis::StickSide;
#[cfg(test)]
use claims::assert_some;
use gilrs::{Axis, Button, EventType};

impl App {
    pub fn apply_controller_event(&mut self, id: usize, event: &EventType) {
        let Some(device) = self.devices.get_mut(&id) else {
            return;
        };

        match event {
            EventType::ButtonPressed(button, _) => {
                apply_button_value(device, *button, 1.0);
            }
            EventType::ButtonReleased(button, _) => {
                apply_button_value(device, *button, 0.0);
            }
            EventType::ButtonChanged(button, value, _) => {
                apply_button_value(device, *button, *value);
            }
            EventType::AxisChanged(axis, value, _) => {
                device
                    .axes
                    .entry(*axis)
                    .and_modify(|state| state.update(*value))
                    .or_insert_with(|| AxisState::new(*value));
                update_stick_trace(device, *axis);
            }
            _ => {}
        }

        if let EventType::AxisChanged(axis, _, _) = event
            && axis_matches_stick(*axis, self.range_test.side())
        {
            let position = stick_position(device, self.range_test.side());
            self.range_test.record(id, position);
        }
        if let Some(key) = checklist_event_key(event) {
            self.control_checklist.observe(id, &key);
        }

        self.push_event(Some(id), format_event(event));
    }
}

pub(super) fn stick_position(device: &DeviceState, side: StickSide) -> (f32, f32) {
    let (x_axis, y_axis) = match side {
        StickSide::Left => (Axis::LeftStickX, Axis::LeftStickY),
        StickSide::Right => (Axis::RightStickX, Axis::RightStickY),
    };
    (
        device.axes.get(&x_axis).map_or(0.0, |axis| axis.current),
        device.axes.get(&y_axis).map_or(0.0, |axis| axis.current),
    )
}

const fn axis_matches_stick(axis: Axis, side: StickSide) -> bool {
    matches!(
        (axis, side),
        (Axis::LeftStickX | Axis::LeftStickY, StickSide::Left)
            | (Axis::RightStickX | Axis::RightStickY, StickSide::Right)
    )
}

fn checklist_event_key(event: &EventType) -> Option<String> {
    match event {
        EventType::ButtonPressed(button, _) => Some(format!("button:{button:?}")),
        EventType::ButtonChanged(button, value, _) if *value > 0.5 => {
            Some(format!("button:{button:?}"))
        }
        EventType::AxisChanged(axis, value, _) if value.abs() >= 0.5 => {
            Some(format!("axis:{axis:?}"))
        }
        _ => None,
    }
}

#[inline]
pub(super) fn update_stick_trace(device: &mut DeviceState, changed_axis: Axis) {
    let (trace, x_axis, y_axis) = match changed_axis {
        Axis::LeftStickX | Axis::LeftStickY => (
            &mut device.left_stick_trace,
            Axis::LeftStickX,
            Axis::LeftStickY,
        ),
        Axis::RightStickX | Axis::RightStickY => (
            &mut device.right_stick_trace,
            Axis::RightStickX,
            Axis::RightStickY,
        ),
        _ => return,
    };
    let x = device.axes.get(&x_axis).map_or(0.0, |state| state.current);
    let y = device.axes.get(&y_axis).map_or(0.0, |state| state.current);
    trace.push(x, y);
}

#[inline]
pub(super) fn apply_button_value(device: &mut DeviceState, button: Button, value: f32) {
    device.buttons.insert(button, value > 0.5);
    device.button_values.insert(button, value);
}

fn format_event(event: &EventType) -> String {
    match event {
        EventType::ButtonPressed(button, _) => format!("ButtonPressed({button:?})"),
        EventType::ButtonRepeated(button, _) => format!("ButtonRepeated({button:?})"),
        EventType::ButtonReleased(button, _) => format!("ButtonReleased({button:?})"),
        EventType::ButtonChanged(button, value, _) => {
            format!("ButtonChanged({button:?}, {value:.3})")
        }
        EventType::AxisChanged(axis, value, _) => format!("AxisChanged({axis:?}, {value:+.3})"),
        EventType::Dropped => "Dropped (backend queue overflow)".to_owned(),
        EventType::Connected => "Connected".to_owned(),
        EventType::Disconnected => "Disconnected".to_owned(),
        _ => format!("{event:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analog_button_values_are_preserved() {
        let mut app = App::new();
        app.connect(1, metadata("controller"));
        let device = assert_some!(app.devices.get_mut(&1));

        apply_button_value(device, Button::LeftTrigger2, 0.37);

        let value = app.devices[&1].button_values[&Button::LeftTrigger2];
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
}
