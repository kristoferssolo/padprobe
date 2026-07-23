use crate::app::DeviceState;
use gamepad_widget::prelude::*;
use gilrs::{Axis, Button};

pub(super) fn gamepad_state(device: &DeviceState) -> GamepadState {
    let mut clusters = vec![
        ControlCluster::new("Left shoulder")
            .with_placement(ClusterPlacement::LeftShoulder)
            .with_control(button_control("LB", device, Button::LeftTrigger))
            .with_control(trigger_control(
                "LT",
                device,
                Axis::LeftZ,
                Button::LeftTrigger2,
            )),
        ControlCluster::new("Menu")
            .with_placement(ClusterPlacement::Menu)
            .with_control(button_control("Select", device, Button::Select))
            .with_control(button_control("Mode", device, Button::Mode))
            .with_control(button_control("Start", device, Button::Start)),
        ControlCluster::new("Right shoulder")
            .with_placement(ClusterPlacement::RightShoulder)
            .with_control(button_control("RB", device, Button::RightTrigger))
            .with_control(trigger_control(
                "RT",
                device,
                Axis::RightZ,
                Button::RightTrigger2,
            )),
        ControlCluster::new("Left stick")
            .with_placement(ClusterPlacement::LeftStick)
            .with_control(stick_control(
                "L3",
                device,
                Axis::LeftStickX,
                Axis::LeftStickY,
                Button::LeftThumb,
            )),
        ControlCluster::new("Face buttons")
            .with_placement(ClusterPlacement::Face)
            .with_control(button_control("North", device, Button::North))
            .with_control(button_control("West", device, Button::West))
            .with_control(button_control("East", device, Button::East))
            .with_control(button_control("South", device, Button::South)),
        ControlCluster::new("D-pad")
            .with_placement(ClusterPlacement::DPad)
            .with_control(dpad_control("Up", device, Button::DPadUp, Axis::DPadY, 1.0))
            .with_control(dpad_control(
                "Left",
                device,
                Button::DPadLeft,
                Axis::DPadX,
                -1.0,
            ))
            .with_control(dpad_control(
                "Right",
                device,
                Button::DPadRight,
                Axis::DPadX,
                1.0,
            ))
            .with_control(dpad_control(
                "Down",
                device,
                Button::DPadDown,
                Axis::DPadY,
                -1.0,
            )),
        ControlCluster::new("Right stick")
            .with_placement(ClusterPlacement::RightStick)
            .with_control(stick_control(
                "R3",
                device,
                Axis::RightStickX,
                Axis::RightStickY,
                Button::RightThumb,
            )),
    ];

    let extras = extra_controls(device);
    if !extras.controls().is_empty() {
        clusters.push(extras);
    }

    GamepadState::new(clusters)
}

fn button_control(label: &str, device: &DeviceState, button: Button) -> Control {
    Control::new(
        label,
        ControlValue::Button {
            pressed: pressed(device, button),
        },
    )
}

fn stick_control(
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

fn trigger_control(label: &str, device: &DeviceState, axis: Axis, button: Button) -> Control {
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

fn dpad_control(
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

fn extra_controls(device: &DeviceState) -> ControlCluster {
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
mod tests;
