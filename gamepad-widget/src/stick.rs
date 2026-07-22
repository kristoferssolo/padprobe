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
    x: f32,
    y: f32,
    gate_style: Style,
    marker_style: Style,
    value_style: Style,
}

impl<'label> StickGauge<'label> {
    /// Creates a gauge for a normalized stick position.
    #[must_use]
    pub fn new(label: &'label str, x: f32, y: f32) -> Self {
        Self {
            label,
            button: None,
            x: x.clamp(-1.0, 1.0),
            y: y.clamp(-1.0, 1.0),
            gate_style: Style::default().fg(Color::DarkGray),
            marker_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            value_style: Style::default(),
        }
    }

    /// Adds the stick-click control to the gauge summary.
    #[must_use]
    pub const fn button(mut self, label: &'label str, pressed: bool) -> Self {
        self.button = Some((label, pressed));
        self
    }

    /// Sets the style used for the gate and its crosshair.
    #[must_use]
    pub const fn gate_style(mut self, style: Style) -> Self {
        self.gate_style = style;
        self
    }

    /// Sets the style used for the position marker.
    #[must_use]
    pub const fn marker_style(mut self, style: Style) -> Self {
        self.marker_style = style;
        self
    }

    /// Sets the style used for labels and numerical values.
    #[must_use]
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

        let footer_height = u16::from(area.height >= 3).saturating_mul(2);
        let plot_height = area.height.saturating_sub(1 + footer_height);
        if plot_height > 0 {
            let plot_width = area.width.min(plot_height.saturating_mul(2));
            let plot = Rect::new(
                area.x + area.width.saturating_sub(plot_width) / 2,
                area.y + 1,
                plot_width,
                plot_height,
            );
            render_gate(
                plot,
                buffer,
                self.x,
                self.y,
                self.gate_style,
                self.marker_style,
            );
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
        Paragraph::new(lines).render(footer, buffer);
    }
}

fn render_gate(
    area: Rect,
    buffer: &mut Buffer,
    x: f32,
    y: f32,
    gate_style: Style,
    marker_style: Style,
) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let gate_color = gate_style.fg.unwrap_or(Color::Reset);
    Canvas::default()
        .marker(Marker::Braille)
        .x_bounds([-1.1, 1.1])
        .y_bounds([-1.1, 1.1])
        .paint(|context| {
            context.draw(&Circle::new(0.0, 0.0, 1.0, gate_color));
            context.draw(&CanvasLine::new(-1.0, 0.0, 1.0, 0.0, gate_color));
            context.draw(&CanvasLine::new(0.0, -1.0, 0.0, 1.0, gate_color));
        })
        .render(area, buffer);

    let marker_x = f32::from(area.width.saturating_sub(1)) * (0.5 + x.clamp(-1.0, 1.0) * 0.4);
    let marker_y = f32::from(area.height.saturating_sub(1)) * (0.5 - y.clamp(-1.0, 1.0) * 0.4);
    buffer[(
        area.x + marker_x.round() as u16,
        area.y + marker_y.round() as u16,
    )]
        .set_char('●')
        .set_style(marker_style);
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
        assert!(symbols.contains('●'));
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
                .position(|cell| cell.symbol() == "●")
        };

        assert_ne!(marker(&centered), marker(&upper_right));
    }
}
