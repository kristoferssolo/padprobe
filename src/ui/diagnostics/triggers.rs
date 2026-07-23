use super::{diagnostic_block, trigger_value};
use crate::app::DeviceState;
use gilrs::{Axis, Button};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Paragraph,
};

pub(super) fn render(frame: &mut Frame<'_>, device: Option<&DeviceState>, area: Rect) {
    let block = diagnostic_block(" Triggers · 0–1 ", Color::Magenta);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);
    let left = device.map_or(0.0, |device| {
        trigger_value(device, Axis::LeftZ, Button::LeftTrigger2)
    });
    let right = device.map_or(0.0, |device| {
        trigger_value(device, Axis::RightZ, Button::RightTrigger2)
    });

    render_trigger(frame, "LT / L2", left, halves[0]);
    render_trigger(frame, "RT / R2", right, halves[1]);
}

fn render_trigger(frame: &mut Frame<'_>, label: &str, value: f32, area: Rect) {
    const BAR_HEIGHT: usize = 6;
    const FILL_THRESHOLDS: [f32; BAR_HEIGHT] = [
        1.0 / 12.0,
        3.0 / 12.0,
        5.0 / 12.0,
        7.0 / 12.0,
        9.0 / 12.0,
        11.0 / 12.0,
    ];

    let value = value.clamp(0.0, 1.0);
    let filled = FILL_THRESHOLDS
        .into_iter()
        .filter(|threshold| value >= *threshold)
        .count();
    let mut lines = vec![Line::from("┌─────┐").alignment(Alignment::Center)];
    lines.extend((0..BAR_HEIGHT).map(|row| {
        let active = row >= BAR_HEIGHT.saturating_sub(filled);
        Line::styled(
            if active {
                "│█████│"
            } else {
                "│     │"
            },
            if active {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        )
        .alignment(Alignment::Center)
    }));
    lines.extend([
        Line::from("└─────┘").alignment(Alignment::Center),
        Line::styled(format!("{value:.2}"), Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center),
        Line::styled(
            label.to_owned(),
            Style::default().add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center),
    ]);
    frame.render_widget(Paragraph::new(lines), area);
}
