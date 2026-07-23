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

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered_symbols(area: Rect, gauge: StickGauge<'_>) -> String {
        let mut buffer = Buffer::empty(area);
        gauge.render(area, &mut buffer);
        buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect()
    }

    #[test]
    fn gauge_uses_braille_cells_for_the_gate() {
        let symbols = rendered_symbols(Rect::new(0, 0, 18, 11), StickGauge::new("Left", 0.0, 0.0));

        assert!(
            symbols
                .chars()
                .filter(|symbol| ('\u{2800}'..='\u{28ff}').contains(symbol))
                .count()
                > 20
        );
        assert!(!symbols.trim().is_empty());
    }

    #[test]
    fn larger_area_produces_a_larger_gate() {
        let small = rendered_symbols(Rect::new(0, 0, 10, 8), StickGauge::new("Stick", 0.0, 0.0));
        let large = rendered_symbols(Rect::new(0, 0, 20, 12), StickGauge::new("Stick", 0.0, 0.0));
        let count_braille = |symbols: &str| {
            symbols
                .chars()
                .filter(|symbol| ('\u{2800}'..='\u{28ff}').contains(symbol))
                .count()
        };

        assert!(count_braille(&large) > count_braille(&small));
    }

    #[test]
    fn marker_moves_with_the_stick_position() {
        let area = Rect::new(0, 0, 18, 11);
        let mut centered = Buffer::empty(area);
        let mut upper_right = Buffer::empty(area);

        StickGauge::new("Stick", 0.0, 0.0).render(area, &mut centered);
        StickGauge::new("Stick", 1.0, 1.0).render(area, &mut upper_right);
        let marker = |buffer: &Buffer| {
            buffer
                .content()
                .iter()
                .position(|cell| cell.fg == Color::Cyan)
        };

        assert_ne!(marker(&centered), marker(&upper_right));
    }

    #[test]
    fn resting_marker_uses_the_canvas_center() {
        let area = Rect::new(0, 0, 18, 11);
        let mut buffer = Buffer::empty(area);

        StickGauge::new("Stick", 0.0, 0.0).render(area, &mut buffer);
        let center = 5 * usize::from(area.width) + 9;
        let marker_is_centered = buffer
            .content()
            .iter()
            .enumerate()
            .any(|(index, cell)| index == center && cell.fg == Color::Cyan);

        assert!(marker_is_centered);
    }

    #[test]
    fn gate_uses_two_columns_per_row() {
        let wide = gate_rect(Rect::new(0, 0, 30, 12), 2).expect("wide gate should fit");
        let tall = gate_rect(Rect::new(0, 0, 30, 30), 2).expect("tall gate should fit");

        assert_eq!(wide.width, wide.height * 2);
        assert_eq!(tall.width, tall.height * 2);
        assert_eq!(tall, Rect::new(0, 1, 30, 15));
    }

    #[test]
    fn gauge_renders_observed_trace_and_metric() {
        let area = Rect::new(0, 0, 20, 12);
        let trace = [(-0.5, -0.5), (0.5, 0.5)];
        let mut buffer = Buffer::empty(area);

        StickGauge::new("Stick", 0.5, 0.5)
            .trace(&trace)
            .metric("offset 70.7%")
            .render(area, &mut buffer);
        let rendered = buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();

        assert!(buffer.content().iter().any(|cell| cell.fg == Color::Green));
        assert!(rendered.contains("offset 70.7%"));
    }
}
