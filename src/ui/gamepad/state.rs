use gilrs::{Axis, Button};

use crate::app::DeviceState;

pub(super) fn pressed(device: &DeviceState, button: Button) -> bool {
    device.buttons.get(&button).copied().unwrap_or(false)
}

pub(super) fn button_label(label: &str, active: bool) -> String {
    if active {
        label.to_owned()
    } else {
        label.to_ascii_lowercase()
    }
}

pub(super) fn dpad_symbol(active: bool, inactive: char, pressed: char) -> char {
    if active { pressed } else { inactive }
}

pub(super) fn dpad_pressed(
    device: &DeviceState,
    button: Button,
    axis: Axis,
    direction: f32,
) -> bool {
    pressed(device, button)
        || device
            .axes
            .get(&axis)
            .is_some_and(|state| state.current * direction > 0.5)
}

pub(super) fn trigger_active(device: &DeviceState, left: Axis, right: Axis) -> bool {
    [left, right]
        .into_iter()
        .filter_map(|axis| device.axes.get(&axis))
        .any(|state| state.current > -0.7)
}

pub(super) fn axis_value(device: &DeviceState, axis: Axis) -> f32 {
    device.axes.get(&axis).map_or(0.0, |state| state.current)
}

pub(super) fn stick_direction(x: f32, y: f32) -> char {
    const THRESHOLD: f32 = 0.25;
    match (x, y) {
        (x, y) if x < -THRESHOLD && y > THRESHOLD => '↖',
        (x, y) if x > THRESHOLD && y > THRESHOLD => '↗',
        (x, y) if x < -THRESHOLD && y < -THRESHOLD => '↙',
        (x, y) if x > THRESHOLD && y < -THRESHOLD => '↘',
        (x, _) if x < -THRESHOLD => '←',
        (x, _) if x > THRESHOLD => '→',
        (_, y) if y > THRESHOLD => '↑',
        (_, y) if y < -THRESHOLD => '↓',
        _ => '●',
    }
}

pub(super) fn trigger_level(value: Option<f32>) -> char {
    let Some(value) = value else {
        return '·';
    };
    let normalized = (value + 1.0) / 2.0;
    match normalized {
        value if value < 0.15 => '░',
        value if value < 0.5 => '▒',
        value if value < 0.85 => '▓',
        _ => '█',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stick_direction_uses_deadzone() {
        assert_eq!(stick_direction(0.1, -0.1), '●');
        assert_eq!(stick_direction(0.7, 0.8), '↗');
        assert_eq!(stick_direction(-0.8, 0.0), '←');
    }

    #[test]
    fn trigger_level_distinguishes_unknown_and_range() {
        assert_eq!(trigger_level(None), '·');
        assert_eq!(trigger_level(Some(-1.0)), '░');
        assert_eq!(trigger_level(Some(0.0)), '▓');
        assert_eq!(trigger_level(Some(1.0)), '█');
    }

    #[test]
    fn inactive_button_labels_are_visibly_distinct() {
        assert_eq!(button_label("LT", false), "lt");
        assert_eq!(button_label("LT", true), "LT");
        assert_eq!(dpad_symbol(false, '△', '▲'), '△');
        assert_eq!(dpad_symbol(true, '△', '▲'), '▲');
    }
}
