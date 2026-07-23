use super::{axis_value, diagnostic_block};
use crate::app::DeviceState;
use gamepad_widget::StickGauge;
use gilrs::Axis;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
};

pub(super) fn render(frame: &mut Frame<'_>, device: Option<&DeviceState>, area: Rect) {
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
    let metric = stick_metric(magnitude, edge_error(trace), area.width);
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

fn stick_metric(magnitude: f32, edge_error: Option<f64>, width: u16) -> String {
    let offset = magnitude * 100.0;
    match (edge_error, width) {
        (Some(error), 24..) => format!("offset {offset:.1}% · edge {error:.1}%"),
        (Some(error), 19..) => format!("off {offset:.1}% · err {error:.1}%"),
        _ if width >= 12 => format!("offset {offset:.1}%"),
        _ => format!("off {offset:.0}%"),
    }
}

fn edge_error(trace: &[(f64, f64)]) -> Option<f64> {
    let (sample_count, total_error) = trace
        .iter()
        .map(|(x, y)| x.hypot(*y))
        .filter(|radius| *radius >= 0.8)
        .fold((0.0, 0.0), |(count, total), radius| {
            (count + 1.0, total + (1.0 - radius).abs())
        });
    (sample_count >= 8.0).then(|| total_error / sample_count * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn edge_error_requires_enough_outer_samples() {
        assert_eq!(edge_error(&[(1.0, 0.0); 7]), None);
        assert!((edge_error(&[(0.9, 0.0); 8]).expect("enough samples") - 10.0).abs() < 1e-9);
    }

    #[rstest]
    #[case(30, "offset 0.5% · edge 2.0%")]
    #[case(20, "off 0.5% · err 2.0%")]
    #[case(17, "offset 0.5%")]
    fn stick_metric_adapts_to_available_width(#[case] width: u16, #[case] expected: &str) {
        assert_eq!(stick_metric(0.005, Some(2.0), width), expected);
    }
}
