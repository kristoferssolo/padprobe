mod render;
#[cfg(test)]
mod tests;

use self::render::{gate_rect, render_gate};
use crate::GamepadTheme;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Paragraph, Widget},
};

/// A high-resolution analog-stick gauge rendered with Unicode Braille cells.
#[derive(Debug, Clone, Copy)]
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
