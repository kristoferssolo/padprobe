use super::*;
use crate::app::{AxisState, DeviceMetadata, StickTrace};
use std::collections::HashMap;

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

    let state = gamepad_state(&device);

    assert_eq!(
        state.clusters()[5].controls()[0].value(),
        ControlValue::Button { pressed: true }
    );
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

    let state = gamepad_state(&device);

    assert_eq!(
        state.clusters().last().map(ControlCluster::title),
        Some("Extra / unmapped")
    );
    assert_eq!(
        state
            .clusters()
            .last()
            .map(|cluster| cluster.controls().len()),
        Some(2)
    );
}

#[test]
fn trigger_uses_analog_button_value() {
    let mut device = device();
    device.button_values.insert(Button::LeftTrigger2, 0.37);

    let state = gamepad_state(&device);

    assert_eq!(
        state.clusters()[0].controls()[1].value(),
        ControlValue::Trigger { value: Some(0.37) }
    );
}

#[test]
fn unobserved_trigger_defaults_to_zero() {
    let state = gamepad_state(&device());

    assert_eq!(
        state.clusters()[0].controls()[1].value(),
        ControlValue::Trigger { value: Some(0.0) }
    );
}

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
