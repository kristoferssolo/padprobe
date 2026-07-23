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
fn centered_stick_uses_idle_marker() {
    assert_eq!(stick_direction(0.0, 0.0), '·');
    assert_eq!(stick_direction(0.8, 0.8), '↗');
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

#[test]
fn stick_plot_moves_marker_with_direction() {
    let centered = stick_plot(0.0, 0.0);
    let upper_right = stick_plot(1.0, 1.0);

    assert_eq!(centered[3].chars().nth(6), Some('·'));
    assert_eq!(upper_right[1].chars().nth(10), Some('●'));
}

#[test]
fn stick_summary_reports_strength_and_click() {
    let cluster = ControlCluster::new("Left stick")
        .with_placement(ClusterPlacement::LeftStick)
        .with_control(Control::new("L3", ControlValue::stick(0.3, 0.4, true)));

    let lines = stick_lines(&cluster, Style::default(), Style::default());

    assert_eq!(lines[7].spans[0].content, "x +0.30  y +0.40");
    assert_eq!(lines[8].spans[0].content, "r 0.50  L3 ●");
}
