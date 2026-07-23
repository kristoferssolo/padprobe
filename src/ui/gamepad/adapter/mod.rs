mod controls;

use self::controls::{
    button_control, dpad_control, extra_controls, stick_control, trigger_control,
};
use crate::app::DeviceState;
#[cfg(test)]
use crate::app::{DeviceMetadata, StickTrace};
use gamepad_widget::prelude::*;
use gilrs::{Axis, Button};
#[cfg(test)]
use std::collections::HashMap;

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

#[cfg(test)]
fn device() -> DeviceState {
    DeviceState {
        metadata: DeviceMetadata {
            name: "fixture".to_owned(),
            vendor_id: None,
            product_id: None,
            uuid: String::new(),
            mapping: "fixture".to_owned(),
            power: "unknown".to_owned(),
            rumble_supported: false,
        },
        connected: true,
        buttons: HashMap::new(),
        button_values: HashMap::new(),
        axes: HashMap::new(),
        left_stick_trace: StickTrace::default(),
        right_stick_trace: StickTrace::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_are_grouped_by_controller_role() {
        let state = gamepad_state(&device());
        let placements = state
            .clusters()
            .iter()
            .map(ControlCluster::placement)
            .collect::<Vec<_>>();

        assert_eq!(
            placements,
            [
                ClusterPlacement::LeftShoulder,
                ClusterPlacement::Menu,
                ClusterPlacement::RightShoulder,
                ClusterPlacement::LeftStick,
                ClusterPlacement::Face,
                ClusterPlacement::DPad,
                ClusterPlacement::RightStick,
            ]
        );
    }
}
