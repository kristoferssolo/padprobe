use crate::app::{App, DeviceState};
use gilrs::{Axis, Button};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub(super) fn render_primary_diagnostics(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    render_sticks(
        frame,
        app.selected_device().map(|(_, device)| device),
        columns[0],
    );
    render_triggers(
        frame,
        app.selected_device().map(|(_, device)| device),
        columns[1],
    );
}

pub(super) fn render_raw_data(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let block = diagnostic_block(" Raw mapped data ", Color::Blue);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some((_, device)) = app.selected_device() else {
        frame.render_widget(Paragraph::new("No controller selected."), inner);
        return;
    };

    let mut pressed = device
        .buttons
        .iter()
        .filter(|(_, pressed)| **pressed)
        .map(|(button, _)| format!("{button:?}"))
        .collect::<Vec<_>>();
    pressed.sort_unstable();
    let pressed = if pressed.is_empty() {
        "none".to_owned()
    } else {
        pressed.join(", ")
    };
    let lines = vec![
        Line::styled(
            format!("gilrs · {}", device.metadata.mapping),
            Style::default().fg(Color::DarkGray),
        ),
        Line::from(""),
        Line::styled("AXES", Style::default().add_modifier(Modifier::BOLD)),
        axis_pair(
            "LX",
            axis_value(device, Axis::LeftStickX),
            "LY",
            axis_value(device, Axis::LeftStickY),
        ),
        axis_pair(
            "RX",
            axis_value(device, Axis::RightStickX),
            "RY",
            axis_value(device, Axis::RightStickY),
        ),
        axis_pair(
            "LZ",
            axis_value(device, Axis::LeftZ),
            "RZ",
            axis_value(device, Axis::RightZ),
        ),
        Line::from(""),
        Line::styled("BUTTONS", Style::default().add_modifier(Modifier::BOLD)),
        Line::from(format!(
            "observed {} · pressed {}",
            device.buttons.len(),
            device.buttons.values().filter(|pressed| **pressed).count()
        )),
        Line::from(pressed),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_sticks(frame: &mut Frame<'_>, device: Option<&DeviceState>, area: Rect) {
    let block = diagnostic_block(" Analog sticks ", Color::Cyan);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);
    let (left_x, left_y, right_x, right_y) = device.map_or((0.0, 0.0, 0.0, 0.0), |device| {
        (
            axis_value(device, Axis::LeftStickX),
            axis_value(device, Axis::LeftStickY),
            axis_value(device, Axis::RightStickX),
            axis_value(device, Axis::RightStickY),
        )
    });

    render_stick(frame, "LEFT", left_x, left_y, halves[0]);
    render_stick(frame, "RIGHT", right_x, right_y, halves[1]);
}

fn render_stick(frame: &mut Frame<'_>, label: &str, x: f32, y: f32, area: Rect) {
    let magnitude = x.hypot(y);
    let style = if magnitude > 0.15 {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let mut lines = vec![
        Line::styled(
            label.to_owned(),
            Style::default().add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center),
    ];
    lines.extend(
        stick_plot(x, y)
            .into_iter()
            .map(|line| Line::styled(line, style).alignment(Alignment::Center)),
    );
    lines.push(Line::styled(format!("{x:+.2}, {y:+.2}"), style).alignment(Alignment::Center));
    lines.push(Line::styled(format!("r {magnitude:.2}"), style).alignment(Alignment::Center));
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_triggers(frame: &mut Frame<'_>, device: Option<&DeviceState>, area: Rect) {
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
    let value = value.clamp(0.0, 1.0);
    let filled = (value * BAR_HEIGHT as f32).round() as usize;
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

fn diagnostic_block(title: &str, color: Color) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Line::styled(title, Style::default().fg(color)))
}

fn stick_plot(x: f32, y: f32) -> Vec<String> {
    let mut rows = [
        "╭─────╮".chars().collect::<Vec<_>>(),
        "│     │".chars().collect(),
        "│     │".chars().collect(),
        "│     │".chars().collect(),
        "╰─────╯".chars().collect(),
    ];
    let column = usize::try_from(3_i32 + (x.clamp(-1.0, 1.0) * 2.0).round() as i32).unwrap_or(3);
    let row = usize::try_from(2_i32 - y.clamp(-1.0, 1.0).round() as i32).unwrap_or(2);
    rows[row][column] = if x.hypot(y) > 0.05 { '●' } else { '·' };
    rows.into_iter()
        .map(|characters| characters.into_iter().collect())
        .collect()
}

fn axis_value(device: &DeviceState, axis: Axis) -> f32 {
    device.axes.get(&axis).map_or(0.0, |state| state.current)
}

fn trigger_value(device: &DeviceState, axis: Axis, button: Button) -> f32 {
    device
        .button_values
        .get(&button)
        .copied()
        .or_else(|| {
            device
                .axes
                .get(&axis)
                .map(|state| (state.current + 1.0) / 2.0)
        })
        .unwrap_or_default()
}

fn axis_pair(left_label: &str, left: f32, right_label: &str, right: f32) -> Line<'static> {
    Line::from(format!(
        "{left_label:<2} {left:+.2}   {right_label:<2} {right:+.2}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stick_plot_moves_marker() {
        assert_eq!(stick_plot(0.0, 0.0)[2].chars().nth(3), Some('·'));
        assert_eq!(stick_plot(1.0, 1.0)[1].chars().nth(5), Some('●'));
    }
}
