use super::super::{GamepadWidget, controls::control_span};
use crate::{ControlCluster, ControlValue};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Color,
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Paragraph, Widget,
        canvas::{Canvas, Circle},
    },
};

pub(super) fn render_shoulder(
    cluster: Option<&ControlCluster>,
    center: u16,
    y: u16,
    buffer: &mut Buffer,
    widget: GamepadWidget<'_>,
) {
    let Some(cluster) = cluster else {
        return;
    };
    let [bumper, trigger] = cluster.controls() else {
        render_control_row(
            Some(cluster),
            Rect::new(center.saturating_sub(9), y + 3, 18, 1),
            buffer,
            widget,
        );
        return;
    };
    let trigger_style = if trigger.value().is_active() {
        widget.active_style
    } else {
        widget.border_style
    };
    let bumper_style = if bumper.value().is_active() {
        widget.active_style
    } else {
        widget.border_style
    };
    Paragraph::new(vec![
        Line::styled("╭───╮", trigger_style).alignment(Alignment::Center),
        Line::styled(
            format!("│{:^3}│", compact_label(trigger.label(), 3)),
            trigger_style,
        )
        .alignment(Alignment::Center),
        Line::styled("╰───╯", trigger_style).alignment(Alignment::Center),
        Line::styled("╭─────╮", bumper_style).alignment(Alignment::Center),
        Line::styled(
            format!("│{:^5}│", compact_label(bumper.label(), 5)),
            bumper_style,
        )
        .alignment(Alignment::Center),
        Line::styled("╰─────╯", bumper_style).alignment(Alignment::Center),
    ])
    .render(Rect::new(center.saturating_sub(4), y, 9, 6), buffer);
}

fn compact_label(label: &str, width: usize) -> String {
    label.chars().take(width).collect()
}

pub(super) fn render_control_row(
    cluster: Option<&ControlCluster>,
    area: Rect,
    buffer: &mut Buffer,
    widget: GamepadWidget<'_>,
) {
    let Some(cluster) = cluster else {
        return;
    };
    let spans = cluster
        .controls()
        .iter()
        .enumerate()
        .flat_map(|(index, control)| {
            (index > 0)
                .then(|| Span::raw("  "))
                .into_iter()
                .chain(std::iter::once(control_span(
                    control,
                    widget.idle_style,
                    widget.active_style,
                )))
        })
        .collect::<Vec<_>>();
    Paragraph::new(Line::from(spans).alignment(Alignment::Center)).render(area, buffer);
}

pub(super) fn render_art_stick(
    cluster: Option<&ControlCluster>,
    center: u16,
    y: u16,
    buffer: &mut Buffer,
    widget: GamepadWidget<'_>,
) {
    let Some(cluster) = cluster else {
        return;
    };
    let [control] = cluster.controls() else {
        return;
    };
    let value = control.value();
    let ControlValue::Stick {
        x,
        y: axis_y,
        pressed,
    } = value
    else {
        return;
    };
    let style = if value.is_active() {
        widget.active_style
    } else {
        widget.idle_style
    };
    let circle = Rect::new(center.saturating_sub(4), y, 8, 4);
    let gate_color = widget.border_style.fg.unwrap_or(Color::Reset);
    let marker_color = style.fg.unwrap_or(Color::Reset);
    Canvas::default()
        .marker(Marker::Braille)
        .x_bounds([-1.1, 1.1])
        .y_bounds([-1.1, 1.1])
        .paint(|context| {
            context.draw(&Circle::new(0.0, 0.0, 1.0, gate_color));
            context.layer();
            context.draw(&Circle::new(
                f64::from(x.clamp(-1.0, 1.0)) * 0.75,
                f64::from(axis_y.clamp(-1.0, 1.0)) * 0.75,
                0.08,
                marker_color,
            ));
        })
        .render(circle, buffer);
    Paragraph::new(
        Line::styled(
            format!("{} {}", control.label(), if pressed { "●" } else { "○" }),
            style,
        )
        .alignment(Alignment::Center),
    )
    .render(Rect::new(center.saturating_sub(5), y + 4, 10, 1), buffer);
}

pub(super) fn render_art_diamond(
    cluster: Option<&ControlCluster>,
    center: u16,
    y: u16,
    buffer: &mut Buffer,
    widget: GamepadWidget<'_>,
) {
    let Some(cluster) = cluster else {
        return;
    };
    Paragraph::new(super::super::controls::diamond_lines(
        cluster,
        widget.idle_style,
        widget.active_style,
    ))
    .render(Rect::new(center.saturating_sub(9), y, 18, 3), buffer);
}
