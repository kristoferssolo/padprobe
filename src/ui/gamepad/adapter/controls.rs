#[cfg(test)]
use super::device;
#[cfg(test)]
use crate::app::AxisState;
use crate::app::DeviceState;
use gamepad_widget::prelude::*;
use gilrs::{Axis, Button};

pub(super) fn button_control(label: &str, device: &DeviceState, button: Button) -> Control {
    Control::new(
        label,
        ControlValue::Button {
            pressed: pressed(device, button),
        },
    )
}

pub(super) fn stick_control(
    label: &str,
    device: &DeviceState,
    x_axis: Axis,
    y_axis: Axis,
    button: Button,
) -> Control {
    Control::new(
        label,
        ControlValue::Stick {
            x: axis_value(device, x_axis),
            y: axis_value(device, y_axis),
            pressed: pressed(device, button),
        },
    )
}

pub(super) fn trigger_control(
    label: &str,
    device: &DeviceState,
    axis: Axis,
    button: Button,
) -> Control {
    let value = device
        .button_values
        .get(&button)
        .copied()
        .or_else(|| {
            device
                .axes
                .get(&axis)
                .map(|state| state.current.midpoint(1.0))
        })
        .or_else(|| device.buttons.get(&button).copied().map(f32::from))
        .unwrap_or_default();
    Control::new(label, ControlValue::Trigger { value: Some(value) })
}

pub(super) fn dpad_control(
    label: &str,
    device: &DeviceState,
    button: Button,
    axis: Axis,
    direction: f32,
) -> Control {
    let axis_pressed = device
        .axes
        .get(&axis)
        .is_some_and(|state| state.current * direction > 0.5);
    Control::new(
        label,
        ControlValue::Button {
            pressed: pressed(device, button) || axis_pressed,
        },
    )
}

pub(super) fn extra_controls(device: &DeviceState) -> ControlCluster {
    let mut extras =
        ControlCluster::new("Extra / unmapped").with_placement(ClusterPlacement::Extra);
    for (label, button) in [
        ("C", Button::C),
        ("Z", Button::Z),
        ("Unknown", Button::Unknown),
    ] {
        if device.buttons.contains_key(&button) {
            extras = extras.with_control(button_control(label, device, button));
        }
    }
    if let Some(axis) = device.axes.get(&Axis::Unknown) {
        extras = extras.with_control(Control::new(
            "Unknown axis",
            ControlValue::Axis {
                value: axis.current,
            },
        ));
    }
    extras
}

#[inline]
fn pressed(device: &DeviceState, button: Button) -> bool {
    device.buttons.get(&button).copied().unwrap_or(false)
}

#[inline]
fn axis_value(device: &DeviceState, axis: Axis) -> f32 {
    device.axes.get(&axis).map_or(0.0, |state| state.current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dpad_axes_are_adapted_as_buttons() {
        let mut device = device();
        device.axes.insert(
            Axis::DPadY,
            AxisState {
                current: 1.0,
                minimum: 0.0,
                maximum: 1.0,
                changes: 1,
            },
        );

        let control = dpad_control("Up", &device, Button::DPadUp, Axis::DPadY, 1.0);

        assert_eq!(control.value(), ControlValue::Button { pressed: true });
    }

    #[test]
    fn observed_unknown_controls_create_extra_cluster() {
        let mut device = device();
        device.buttons.insert(Button::Unknown, true);
        device.axes.insert(
            Axis::Unknown,
            AxisState {
                current: 0.4,
                minimum: 0.4,
                maximum: 0.4,
                changes: 1,
            },
        );

        let extras = extra_controls(&device);

        assert_eq!(extras.title(), "Extra / unmapped");
        assert_eq!(extras.controls().len(), 2);
    }

    #[test]
    fn trigger_uses_analog_button_value() {
        let mut device = device();
        device.button_values.insert(Button::LeftTrigger2, 0.37);

        let control = trigger_control("LT", &device, Axis::LeftZ, Button::LeftTrigger2);

        assert_eq!(control.value(), ControlValue::Trigger { value: Some(0.37) });
    }

    #[test]
    fn unobserved_trigger_defaults_to_zero() {
        let control = trigger_control("LT", &device(), Axis::LeftZ, Button::LeftTrigger2);

        assert_eq!(control.value(), ControlValue::Trigger { value: Some(0.0) });
    }
}
