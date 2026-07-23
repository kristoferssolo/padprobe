use super::StickGauge;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    symbols::Marker,
    widgets::{
        Widget,
        canvas::{Canvas, Circle, Line as CanvasLine},
    },
};

pub(super) fn gate_rect(area: Rect, footer_height: u16) -> Option<Rect> {
    let available_height = area.height.saturating_sub(1 + footer_height);
    let plot_height = available_height.min(area.width / 2);
    if plot_height < 2 {
        return None;
    }
    let plot_width = plot_height * 2;
    Some(Rect::new(
        area.x + area.width.saturating_sub(plot_width) / 2,
        area.y + 1,
        plot_width,
        plot_height,
    ))
}

pub(super) fn render_gate(area: Rect, buffer: &mut Buffer, gauge: StickGauge<'_>) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let gate_color = gauge.gate_style.fg.unwrap_or(Color::Reset);
    let marker_color = gauge.marker_style.fg.unwrap_or(Color::Reset);
    let trace_color = gauge.trace_style.fg.unwrap_or(Color::Reset);
    Canvas::default()
        .marker(Marker::Braille)
        .x_bounds([-1.1, 1.1])
        .y_bounds([-1.1, 1.1])
        .paint(|context| {
            context.draw(&Circle::new(0.0, 0.0, 1.0, gate_color));
            context.draw(&CanvasLine::new(-1.0, 0.0, 1.0, 0.0, gate_color));
            context.draw(&CanvasLine::new(0.0, -1.0, 0.0, 1.0, gate_color));
            for points in gauge.trace.windows(2) {
                let [(x1, y1), (x2, y2)] = points else {
                    continue;
                };
                context.draw(&CanvasLine::new(*x1, *y1, *x2, *y2, trace_color));
            }
            context.layer();
            context.draw(&Circle::new(
                f64::from(gauge.x.clamp(-1.0, 1.0)) * 0.82,
                f64::from(gauge.y.clamp(-1.0, 1.0)) * 0.82,
                0.06,
                marker_color,
            ));
        })
        .render(area, buffer);
}
