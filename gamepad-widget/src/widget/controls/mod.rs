#[cfg(test)]
mod tests;

use crate::{Control, ControlCluster, ControlValue};
use ratatui::{
    layout::Alignment,
    style::Style,
    text::{Line, Span},
};

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

pub(super) fn stick_lines(
    cluster: &ControlCluster,
    idle_style: Style,
    active_style: Style,
) -> Vec<Line<'static>> {
    let [control] = cluster.controls() else {
        return control_lines(cluster, idle_style, active_style);
    };
    let value = control.value();
    let ControlValue::Stick { x, y, pressed } = value else {
        return control_lines(cluster, idle_style, active_style);
    };
    let magnitude = x.hypot(y);
    let style = if value.is_active() {
        active_style
    } else {
        idle_style
    };
    let mut lines = stick_plot(x, y)
        .into_iter()
        .map(|line| Line::styled(line, style).alignment(Alignment::Center))
        .collect::<Vec<_>>();
    lines.push(Line::styled(format!("x {x:+.2}  y {y:+.2}"), style).alignment(Alignment::Center));
    lines.push(
        Line::styled(
            format!(
                "r {magnitude:.2}  {} {}",
                control.label(),
                if pressed { "●" } else { "○" }
            ),
            style,
        )
        .alignment(Alignment::Center),
    );
    lines
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "clamped stick coordinates map to fixed, in-bounds plot indices"
)]
fn stick_plot(x: f32, y: f32) -> Vec<String> {
    let mut rows = [
        "  ╭───────╮  ".chars().collect::<Vec<_>>(),
        " ╱         ╲ ".chars().collect(),
        "│           │".chars().collect(),
        "│           │".chars().collect(),
        "│           │".chars().collect(),
        " ╲         ╱ ".chars().collect(),
        "  ╰───────╯  ".chars().collect(),
    ];
    let column = usize::try_from(6_i32 + (x.clamp(-1.0, 1.0) * 4.0).round() as i32).unwrap_or(6);
    let row = usize::try_from(3_i32 - (y.clamp(-1.0, 1.0) * 2.0).round() as i32).unwrap_or(3);
    rows[row][column] = if x.hypot(y) > 0.05 { '●' } else { '·' };

    rows.into_iter()
        .map(|characters| characters.into_iter().collect())
        .collect()
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
        _ => '·',
    }
}
