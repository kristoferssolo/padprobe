use super::super::GamepadWidget;
use crate::{ControlCluster, ControlValue};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Color,
    symbols::Marker,
    text::Line,
    widgets::{
        Paragraph, Widget,
        canvas::{Canvas, Circle},
    },
};

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
