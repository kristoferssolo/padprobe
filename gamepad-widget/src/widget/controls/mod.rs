mod stick;

use crate::{Control, ControlCluster, ControlValue};
use ratatui::{
    layout::Alignment,
    style::Style,
    text::{Line, Span},
};
use stick::stick_direction;
pub(super) use stick::stick_lines;

pub(super) fn control_span(
    control: &Control,
    idle_style: Style,
    active_style: Style,
) -> Span<'static> {
    let active = control.value().is_active();
    Span::styled(
        format!("{} {}", if active { "●" } else { "○" }, control.label()),
        if active { active_style } else { idle_style },
    )
}

pub(super) fn vertically_center(lines: Vec<Line<'static>>, height: u16) -> Vec<Line<'static>> {
    let padding = usize::from(height).saturating_sub(lines.len()) / 2;
    let mut centered = Vec::with_capacity(padding + lines.len());
    centered.resize_with(padding, Line::default);
    centered.extend(lines);
    centered
}

pub(super) fn control_lines(
    cluster: &ControlCluster,
    idle_style: Style,
    active_style: Style,
) -> Vec<Line<'static>> {
    cluster
        .controls()
        .iter()
        .map(|control| control_line(control, idle_style, active_style))
        .collect()
}

pub(super) fn diamond_lines(
    cluster: &ControlCluster,
    idle_style: Style,
    active_style: Style,
) -> Vec<Line<'static>> {
    let [north, west, east, south] = cluster.controls() else {
        return control_lines(cluster, idle_style, active_style);
    };

    vec![
        Line::from(button_span(north, idle_style, active_style)).alignment(Alignment::Center),
        Line::from(vec![
            button_span(west, idle_style, active_style),
            Span::raw("   "),
            button_span(east, idle_style, active_style),
        ])
        .alignment(Alignment::Center),
        Line::from(button_span(south, idle_style, active_style)).alignment(Alignment::Center),
    ]
}

fn button_span(control: &Control, idle_style: Style, active_style: Style) -> Span<'static> {
    let ControlValue::Button { pressed } = control.value() else {
        return Span::styled(control.label().to_owned(), idle_style);
    };
    Span::styled(
        format!("{} {}", if pressed { "●" } else { "○" }, control.label()),
        if pressed { active_style } else { idle_style },
    )
}

fn control_line(control: &Control, idle_style: Style, active_style: Style) -> Line<'static> {
    let control_value = control.value();
    let value = match control_value {
        ControlValue::Button { pressed } => if pressed { "●" } else { "○" }.to_owned(),
        ControlValue::Stick { x, y, .. } => {
            format!("{} x {x:+.2} y {y:+.2}", stick_direction(x, y))
        }
        ControlValue::Trigger { value } => trigger_bar(value),
        ControlValue::Axis { value } => format!("{value:+.3}"),
    };
    Line::styled(
        format!("{:<8} {value}", control.label()),
        if control_value.is_active() {
            active_style
        } else {
            idle_style
        },
    )
}

fn trigger_bar(value: Option<f32>) -> String {
    const WIDTH: usize = 5;
    let Some(value) = value else {
        return "[·····] n/a".to_owned();
    };
    let value = value.clamp(0.0, 1.0);
    let filled = [0.1, 0.3, 0.5, 0.7, 0.9]
        .into_iter()
        .filter(|threshold| value >= *threshold)
        .count();
    format!(
        "[{}{}] {value:.2}",
        "█".repeat(filled),
        "░".repeat(WIDTH - filled)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClusterPlacement;

    #[test]
    fn trigger_bar_handles_unknown_and_clamps_values() {
        assert_eq!(trigger_bar(None), "[·····] n/a");
        assert_eq!(trigger_bar(Some(-1.0)), "[░░░░░] 0.00");
        assert_eq!(trigger_bar(Some(1.5)), "[█████] 1.00");
    }

    #[test]
    fn controller_control_span_hides_exact_trigger_value() {
        let control = Control::new("LT", ControlValue::trigger(0.0));

        let span = control_span(&control, Style::default(), Style::default());

        assert_eq!(span.content, "○ LT");
    }

    #[test]
    fn diamond_places_cardinal_controls_on_three_rows() {
        let cluster = ControlCluster::new("Face")
            .with_placement(ClusterPlacement::Face)
            .with_controls([
                Control::new("North", ControlValue::button(false)),
                Control::new("West", ControlValue::button(false)),
                Control::new("East", ControlValue::button(true)),
                Control::new("South", ControlValue::button(false)),
            ]);

        let lines = diamond_lines(&cluster, Style::default(), Style::default());

        assert_eq!(lines[0].spans[0].content, "○ North");
        assert_eq!(lines[1].spans[0].content, "○ West");
        assert_eq!(lines[1].spans[2].content, "● East");
        assert_eq!(lines[2].spans[0].content, "○ South");
    }

    #[test]
    fn compact_controls_are_centered_in_tall_clusters() {
        let lines = vertically_center(
            vec![
                Line::from("north"),
                Line::from("middle"),
                Line::from("south"),
            ],
            9,
        );

        assert_eq!(lines.len(), 6);
        assert!(lines[0].spans.is_empty());
        assert_eq!(lines[3].spans[0].content, "north");
    }
}
