use crate::app::{App, DeviceState};
use gamepad_widget::StickGauge;
use gilrs::{Axis, Button};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

const NUMBERED_BUTTONS: [Button; 19] = [
    Button::South,
    Button::East,
    Button::North,
    Button::West,
    Button::C,
    Button::Z,
    Button::LeftTrigger,
    Button::LeftTrigger2,
    Button::RightTrigger,
    Button::RightTrigger2,
    Button::Select,
    Button::Start,
    Button::Mode,
    Button::LeftThumb,
    Button::RightThumb,
    Button::DPadUp,
    Button::DPadDown,
    Button::DPadLeft,
    Button::DPadRight,
];

pub(super) fn render_primary_diagnostics(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
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
    let mut lines = vec![
        Line::styled(
            format!("gilrs · {}", device.metadata.mapping),
            Style::default().fg(Color::DarkGray),
        ),
        Line::styled("AXES −1…+1", Style::default().add_modifier(Modifier::BOLD)),
    ];
    lines.extend(
        [
            ("LX", Axis::LeftStickX),
            ("LY", Axis::LeftStickY),
            ("RX", Axis::RightStickX),
            ("RY", Axis::RightStickY),
        ]
        .map(|(label, axis)| signed_axis_bar(label, axis_value(device, axis), inner.width)),
    );
    lines.push(Line::styled(
        "BUTTONS",
        Style::default().add_modifier(Modifier::BOLD),
    ));
    lines.extend(button_grid(device, inner.width));
    lines.push(Line::from(format!("Pressed: {pressed}")));
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
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

    if let Some(device) = device {
        render_stick(
            frame,
            "LEFT",
            left_x,
            left_y,
            device.left_stick_trace.points(),
            halves[0],
        );
        render_stick(
            frame,
            "RIGHT",
            right_x,
            right_y,
            device.right_stick_trace.points(),
            halves[1],
        );
    } else {
        render_stick(frame, "LEFT", left_x, left_y, &[], halves[0]);
        render_stick(frame, "RIGHT", right_x, right_y, &[], halves[1]);
    }
}

fn render_stick(
    frame: &mut Frame<'_>,
    label: &str,
    x: f32,
    y: f32,
    trace: &[(f64, f64)],
    area: Rect,
) {
    let magnitude = x.hypot(y);
    let metric = edge_error(trace).map_or_else(
        || format!("observed offset {:.1}%", magnitude * 100.0),
        |error| format!("offset {:.1}% · edge {error:.1}%", magnitude * 100.0),
    );
    let value_style = if magnitude > 0.15 {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    frame.render_widget(
        StickGauge::new(label, x, y)
            .trace(trace)
            .metric(&metric)
            .gate_style(Style::default().fg(Color::DarkGray))
            .marker_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .trace_style(Style::default().fg(Color::Green))
            .value_style(value_style),
        area,
    );
}

fn edge_error(trace: &[(f64, f64)]) -> Option<f64> {
    let (sample_count, total_error) = trace
        .iter()
        .map(|(x, y)| x.hypot(*y))
        .filter(|radius| *radius >= 0.8)
        .fold((0_usize, 0.0), |(count, total), radius| {
            (count + 1, total + (1.0 - radius).abs())
        });
    (sample_count >= 8).then(|| total_error / sample_count as f64 * 100.0)
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

fn signed_axis_bar(label: &str, value: f32, width: u16) -> Line<'static> {
    let bar_width = usize::from(width.saturating_sub(18).clamp(7, 17));
    let value = value.clamp(-1.0, 1.0);
    let position = (((value + 1.0) / 2.0) * (bar_width.saturating_sub(1)) as f32).round() as usize;
    let mut bar = vec!['─'; bar_width];
    bar[bar_width / 2] = '┼';
    bar[position] = '●';
    Line::from(format!(
        "{label:<2} −1 [{}] +1 {value:+.2}",
        bar.into_iter().collect::<String>()
    ))
}

fn button_grid(device: &DeviceState, width: u16) -> Vec<Line<'static>> {
    const CELL_WIDTH: u16 = 5;
    let columns = usize::from((width / CELL_WIDTH).max(1));
    NUMBERED_BUTTONS
        .chunks(columns)
        .enumerate()
        .map(|(row, buttons)| {
            let spans = buttons
                .iter()
                .enumerate()
                .flat_map(|(column, button)| {
                    let index = row * columns + column;
                    let pressed = device.buttons.get(button).copied().unwrap_or(false);
                    (column > 0)
                        .then(|| Span::raw(" "))
                        .into_iter()
                        .chain(std::iter::once(Span::styled(
                            format!("[{index:>2}]"),
                            if pressed {
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            },
                        )))
                })
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_error_requires_enough_outer_samples() {
        assert_eq!(edge_error(&[(1.0, 0.0); 7]), None);
        assert!((edge_error(&[(0.9, 0.0); 8]).expect("enough samples") - 10.0).abs() < 1e-9);
    }

    #[test]
    fn signed_axis_bar_marks_center_and_extremes() {
        assert!(signed_axis_bar("LX", -1.0, 40).to_string().contains("[●"));
        assert!(signed_axis_bar("LX", 0.0, 40).to_string().contains('●'));
        assert!(signed_axis_bar("LX", 1.0, 40).to_string().contains("●]"));
    }

    #[test]
    fn button_grid_wraps_numbered_cells() {
        let mut app = App::new();
        app.connect(
            0,
            crate::app::DeviceMetadata {
                name: "fixture".to_owned(),
                vendor_id: None,
                product_id: None,
                uuid: String::new(),
                mapping: "test".to_owned(),
                power: "unknown".to_owned(),
                rumble_supported: false,
            },
        );
        let device = app.devices.get_mut(&0).expect("device should exist");
        device.buttons.insert(Button::South, true);

        let lines = button_grid(device, 20);

        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0].spans[0].content, "[ 0]");
        assert_eq!(lines[0].spans[0].style.fg, Some(Color::Cyan));
    }
}
