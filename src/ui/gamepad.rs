use gilrs::{Axis, Button};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
};

use crate::app::DeviceState;

pub(super) fn render_gamepad(frame: &mut Frame<'_>, device: &DeviceState, area: Rect) {
    let left_stick = stick_direction(
        axis_value(device, Axis::LeftStickX),
        axis_value(device, Axis::LeftStickY),
    );
    let right_stick = stick_direction(
        axis_value(device, Axis::RightStickX),
        axis_value(device, Axis::RightStickY),
    );
    let left_trigger = trigger_level(device.axes.get(&Axis::LeftZ).map(|axis| axis.current));
    let right_trigger = trigger_level(device.axes.get(&Axis::RightZ).map(|axis| axis.current));
    let lt = pressed(device, Button::LeftTrigger2);
    let rt = pressed(device, Button::RightTrigger2);
    let lb = pressed(device, Button::LeftTrigger);
    let rb = pressed(device, Button::RightTrigger);
    let l3 = pressed(device, Button::LeftThumb);
    let r3 = pressed(device, Button::RightThumb);
    let select = pressed(device, Button::Select);
    let mode = pressed(device, Button::Mode);
    let start = pressed(device, Button::Start);
    let north = pressed(device, Button::North);
    let west = pressed(device, Button::West);
    let east = pressed(device, Button::East);
    let south = pressed(device, Button::South);
    let c = pressed(device, Button::C);
    let z = pressed(device, Button::Z);
    let dpad_up = dpad_pressed(device, Button::DPadUp, Axis::DPadY, 1.0);
    let dpad_down = dpad_pressed(device, Button::DPadDown, Axis::DPadY, -1.0);
    let dpad_left = dpad_pressed(device, Button::DPadLeft, Axis::DPadX, -1.0);
    let dpad_right = dpad_pressed(device, Button::DPadRight, Axis::DPadX, 1.0);
    let muted = Style::default().fg(Color::DarkGray);
    let active = Style::default().fg(Color::Cyan);
    let lines = vec![
        Line::styled(
            format!(
                "       ╭─ {}:{left_trigger} ─╮          ╭─ {}:{right_trigger} ─╮",
                button_label("LT", lt),
                button_label("RT", rt)
            ),
            if lt || rt || trigger_active(device, Axis::LeftZ, Axis::RightZ) {
                active
            } else {
                muted
            },
        ),
        Line::styled(
            format!(
                "       ╰── {} ──╯          ╰── {} ──╯",
                button_label("LB", lb),
                button_label("RB", rb)
            ),
            if lb || rb { active } else { muted },
        ),
        Line::styled("      ╭──────────────────────────────╮", muted),
        Line::styled(
            format!(
                "   ╭──╯  {}:{left_stick}  {} {} {}   {}   ╰──╮",
                button_label("L3", l3),
                button_label("SE", select),
                button_label("MO", mode),
                button_label("ST", start),
                button_label("N", north),
            ),
            if l3 || select || mode || start || north || left_stick != '●' {
                active
            } else {
                muted
            },
        ),
        Line::styled(
            format!(
                "  ╱              {}          {}   {}      ╲",
                dpad_symbol(dpad_up, '△', '▲'),
                button_label("W", west),
                button_label("E", east),
            ),
            if dpad_up || west || east {
                active
            } else {
                muted
            },
        ),
        Line::styled(
            format!(
                " │            {} + {}       {} {} {}      │",
                dpad_symbol(dpad_left, '◁', '◀'),
                dpad_symbol(dpad_right, '▷', '▶'),
                button_label("S", south),
                button_label("C", c),
                button_label("Z", z),
            ),
            if dpad_left || dpad_right || south || c || z {
                active
            } else {
                muted
            },
        ),
        Line::styled(
            format!(
                "  ╲              {}       {}:{right_stick}        ╱",
                dpad_symbol(dpad_down, '▽', '▼'),
                button_label("R3", r3),
            ),
            if dpad_down || r3 || right_stick != '●' {
                active
            } else {
                muted
            },
        ),
        Line::styled("   ╰──╮         ╭────────────╮       ╭──╯", muted),
        Line::styled("      ╰─────────╯            ╰───────╯", muted),
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

fn pressed(device: &DeviceState, button: Button) -> bool {
    device.buttons.get(&button).copied().unwrap_or(false)
}

fn button_label(label: &str, active: bool) -> String {
    if active {
        label.to_owned()
    } else {
        label.to_ascii_lowercase()
    }
}

fn dpad_symbol(active: bool, inactive: char, pressed: char) -> char {
    if active { pressed } else { inactive }
}

fn dpad_pressed(device: &DeviceState, button: Button, axis: Axis, direction: f32) -> bool {
    pressed(device, button)
        || device
            .axes
            .get(&axis)
            .is_some_and(|state| state.current * direction > 0.5)
}

fn trigger_active(device: &DeviceState, left: Axis, right: Axis) -> bool {
    [left, right]
        .into_iter()
        .filter_map(|axis| device.axes.get(&axis))
        .any(|state| state.current > -0.7)
}

fn axis_value(device: &DeviceState, axis: Axis) -> f32 {
    device.axes.get(&axis).map_or(0.0, |state| state.current)
}

fn stick_direction(x: f32, y: f32) -> char {
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

fn trigger_level(value: Option<f32>) -> char {
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
