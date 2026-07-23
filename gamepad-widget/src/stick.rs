use crate::GamepadTheme;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::Line,
    widgets::{
        Paragraph, Widget,
        canvas::{Canvas, Circle, Line as CanvasLine},
    },
};

/// A high-resolution analog-stick gauge rendered with Unicode Braille cells.
#[derive(Clone, Copy, Debug)]
pub struct StickGauge<'label> {
    label: &'label str,
    button: Option<(&'label str, bool)>,
    metric: Option<&'label str>,
    trace: &'label [(f64, f64)],
    x: f32,
    y: f32,
    gate_style: Style,
    marker_style: Style,
    trace_style: Style,
    value_style: Style,
}

impl<'label> StickGauge<'label> {
    /// Creates a gauge for a normalized stick position.
    #[must_use]
    #[inline]
    pub fn new(label: &'label str, x: f32, y: f32) -> Self {
        let theme = GamepadTheme::default();
        Self {
            label,
            button: None,
            metric: None,
            trace: &[],
            x: x.clamp(-1.0, 1.0),
            y: y.clamp(-1.0, 1.0),
            gate_style: theme.border,
            marker_style: theme.active,
            trace_style: theme.trace,
            value_style: theme.value,
        }
    }

    /// Applies a shared gamepad theme to the gauge.
    #[must_use]
    #[inline]
    pub const fn theme(mut self, theme: GamepadTheme) -> Self {
        self.gate_style = theme.border;
        self.marker_style = theme.active;
        self.trace_style = theme.trace;
        self.value_style = theme.value;
        self
    }

    /// Adds the stick-click control to the gauge summary.
    #[must_use]
    #[inline]
    pub const fn button(mut self, label: &'label str, pressed: bool) -> Self {
        self.button = Some((label, pressed));
        self
    }

    /// Adds an analysis metric below the stick values.
    #[must_use]
    #[inline]
    pub const fn metric(mut self, metric: &'label str) -> Self {
        self.metric = Some(metric);
        self
    }

    /// Adds normalized session points to the stick plot.
    #[must_use]
    #[inline]
    pub const fn trace(mut self, points: &'label [(f64, f64)]) -> Self {
        self.trace = points;
        self
    }

    /// Sets the style used for the gate and its crosshair.
    #[must_use]
    #[inline]
    pub const fn gate_style(mut self, style: Style) -> Self {
        self.gate_style = style;
        self
    }

    /// Sets the style used for the position marker.
    #[must_use]
    #[inline]
    pub const fn marker_style(mut self, style: Style) -> Self {
        self.marker_style = style;
        self
    }

    /// Sets the style used for the observed stick trace.
    #[must_use]
    #[inline]
    pub const fn trace_style(mut self, style: Style) -> Self {
        self.trace_style = style;
        self
    }

    /// Sets the style used for labels and numerical values.
    #[must_use]
    #[inline]
    pub const fn value_style(mut self, style: Style) -> Self {
        self.value_style = style;
        self
    }
}

impl Widget for StickGauge<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        Paragraph::new(Line::styled(
            self.label.to_uppercase(),
            self.value_style.add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center)
        .render(Rect::new(area.x, area.y, area.width, 1), buffer);

        let footer_height =
            u16::from(area.height >= 3).saturating_mul(2) + u16::from(self.metric.is_some());
        if let Some(plot) = gate_rect(area, footer_height) {
            render_gate(plot, buffer, self);
        }

        if footer_height == 0 {
            return;
        }

        let magnitude = self.x.hypot(self.y);
        let footer = Rect::new(
            area.x,
            area.bottom().saturating_sub(footer_height),
            area.width,
            footer_height,
        );
        let mut lines = vec![
            Line::styled(
                format!("x {:+.2}  y {:+.2}", self.x, self.y),
                self.value_style,
            )
            .alignment(Alignment::Center),
        ];
        if footer_height > 1 {
            let button = self.button.map_or_else(String::new, |(label, pressed)| {
                format!("  {label} {}", if pressed { "●" } else { "○" })
            });
            lines.push(
                Line::styled(format!("r {magnitude:.2}{button}"), self.value_style)
                    .alignment(Alignment::Center),
            );
        }
        if let Some(metric) = self.metric {
            lines.push(
                Line::styled(metric.to_owned(), self.trace_style).alignment(Alignment::Center),
            );
        }
        Paragraph::new(lines).render(footer, buffer);
    }
}

fn gate_rect(area: Rect, footer_height: u16) -> Option<Rect> {
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

fn render_gate(area: Rect, buffer: &mut Buffer, gauge: StickGauge<'_>) {
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
