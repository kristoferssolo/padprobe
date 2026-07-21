use gilrs::{Axis, Button};

use crate::{
    app::DeviceState,
    widgets::gamepad::{Control, ControlCluster, ControlValue, GamepadState},
};

pub(super) fn gamepad_state(device: &DeviceState) -> GamepadState {
    let mut clusters = vec![
        ControlCluster::new("Left controls")
            .with_control(stick_control(
                "Stick",
                device,
                Axis::LeftStickX,
                Axis::LeftStickY,
                Button::LeftThumb,
            ))
            .with_control(button_control("L3", device, Button::LeftThumb))
            .with_control(button_control("LB", device, Button::LeftTrigger))
            .with_control(trigger_control(
                "LT",
                device,
                Axis::LeftZ,
                Button::LeftTrigger2,
            )),
        ControlCluster::new("D-pad")
            .with_control(dpad_control("Up", device, Button::DPadUp, Axis::DPadY, 1.0))
            .with_control(dpad_control(
                "Down",
                device,
                Button::DPadDown,
                Axis::DPadY,
                -1.0,
            ))
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
            )),
        ControlCluster::new("Menu")
            .with_control(button_control("Select", device, Button::Select))
            .with_control(button_control("Mode", device, Button::Mode))
            .with_control(button_control("Start", device, Button::Start)),
        ControlCluster::new("Face buttons")
            .with_control(button_control("North", device, Button::North))
            .with_control(button_control("West", device, Button::West))
            .with_control(button_control("East", device, Button::East))
            .with_control(button_control("South", device, Button::South)),
        ControlCluster::new("Right controls")
            .with_control(stick_control(
                "Stick",
                device,
                Axis::RightStickX,
                Axis::RightStickY,
                Button::RightThumb,
            ))
            .with_control(button_control("R3", device, Button::RightThumb))
            .with_control(button_control("RB", device, Button::RightTrigger))
            .with_control(trigger_control(
                "RT",
                device,
                Axis::RightZ,
                Button::RightTrigger2,
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
        .axes
        .get(&axis)
        .map(|state| (state.current + 1.0) / 2.0)
        .or_else(|| {
            device
                .buttons
                .contains_key(&button)
                .then(|| f32::from(pressed(device, button)))
        });
    Control::new(label, ControlValue::Trigger { value })
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
    let mut extras = ControlCluster::new("Extra / unmapped");
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

fn pressed(device: &DeviceState, button: Button) -> bool {
    device.buttons.get(&button).copied().unwrap_or(false)
}

fn axis_value(device: &DeviceState, axis: Axis) -> f32 {
    device.axes.get(&axis).map_or(0.0, |state| state.current)
}
