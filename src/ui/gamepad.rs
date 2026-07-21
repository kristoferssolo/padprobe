use gilrs::Axis;
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
    let muted = Style::default().fg(Color::DarkGray);
    let active = Style::default().fg(Color::Cyan);
    let lines = vec![
        Line::styled(
            format!("       ╭─ LT:{left_trigger} ─╮          ╭─ RT:{right_trigger} ─╮"),
            if left_trigger != '·' || right_trigger != '·' {
                active
            } else {
                muted
            },
        ),
        Line::styled("       ╰── LB ──╯          ╰── RB ──╯", muted),
        Line::styled("      ╭──────────────────────────────╮", muted),
        Line::styled(
            format!("   ╭──╯   LS:{left_stick}     ◦    ◦      N    ╰──╮"),
            if left_stick == '●' { muted } else { active },
        ),
        Line::styled("  ╱             ╭─┬─╮         W   E     ╲", muted),
        Line::styled(" │              ├─┼─┤           S         │", muted),
        Line::styled(
            format!("  ╲             ╰─┴─╯      RS:{right_stick}         ╱"),
            if right_stick == '●' { muted } else { active },
        ),
        Line::styled("   ╰──╮         ╭────────────╮       ╭──╯", muted),
        Line::styled("      ╰─────────╯            ╰───────╯", muted),
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
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
}
