use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{ClusterPlacement, Control, ControlCluster, ControlValue, GamepadState};

#[derive(Clone, Copy, Debug)]
pub struct GamepadWidget<'state> {
    state: &'state GamepadState,
    border_style: Style,
    idle_style: Style,
    active_style: Style,
}

impl<'state> GamepadWidget<'state> {
    #[must_use]
    pub fn new(state: &'state GamepadState) -> Self {
        Self {
            state,
            border_style: Style::default().fg(Color::DarkGray),
            idle_style: Style::default(),
            active_style: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        }
    }

    #[must_use]
    pub const fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    #[must_use]
    pub const fn idle_style(mut self, style: Style) -> Self {
        self.idle_style = style;
        self
    }

    #[must_use]
    pub const fn active_style(mut self, style: Style) -> Self {
        self.active_style = style;
        self
    }
}

impl Widget for GamepadWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let clusters = self.state.clusters();
        if area.is_empty() || clusters.is_empty() {
            return;
        }

        let areas = grid_areas(area, clusters.len());
        for (cluster, area) in clusters.iter().zip(areas) {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", cluster.title()))
                .border_style(self.border_style);
            let inner = block.inner(area);
            block.render(area, buffer);
            let lines = match cluster.placement() {
                ClusterPlacement::DPad | ClusterPlacement::Face => {
                    diamond_lines(cluster, self.idle_style, self.active_style)
                }
                _ => control_lines(cluster, self.idle_style, self.active_style),
            };
            Paragraph::new(lines).render(inner, buffer);
        }
    }
}

fn control_lines(
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

fn diamond_lines(
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
    let (value, active) = match control.value() {
        ControlValue::Button { pressed } => (if pressed { "●" } else { "○" }.to_owned(), pressed),
        ControlValue::Stick { x, y, pressed } => (
            format!("{} x {x:+.2} y {y:+.2}", stick_direction(x, y)),
            pressed || x.hypot(y) > 0.15,
        ),
        ControlValue::Trigger { value } => {
            (trigger_bar(value), value.is_some_and(|value| value > 0.1))
        }
        ControlValue::Axis { value } => (format!("{value:+.3}"), value.abs() > 0.15),
    };
    Line::styled(
        format!("{:<8} {value}", control.label()),
        if active { active_style } else { idle_style },
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

fn grid_areas(area: Rect, item_count: usize) -> Vec<Rect> {
    let columns = match area.width {
        89.. => item_count.min(3),
        59.. => item_count.min(2),
        _ => 1,
    };
    let rows = item_count.div_ceil(columns);
    let column_gap = usize::from(columns > 1);
    let row_gap = usize::from(rows > 1);
    let usable_width =
        usize::from(area.width).saturating_sub(column_gap * columns.saturating_sub(1));
    let usable_height = usize::from(area.height).saturating_sub(row_gap * rows.saturating_sub(1));
    let column_width = usable_width / columns;
    let row_height = usable_height / rows;

    (0..item_count)
        .map(|index| {
            let column = index % columns;
            let row = index / columns;
            let x = area.x.saturating_add(
                u16::try_from(column * (column_width + column_gap)).unwrap_or(u16::MAX),
            );
            let y = area
                .y
                .saturating_add(u16::try_from(row * (row_height + row_gap)).unwrap_or(u16::MAX));
            let width = if column + 1 == columns {
                area.right().saturating_sub(x)
            } else {
                u16::try_from(column_width).unwrap_or(u16::MAX)
            };
            let height = if row + 1 == rows {
                area.bottom().saturating_sub(y)
            } else {
                u16::try_from(row_height).unwrap_or(u16::MAX)
            };
            Rect::new(x, y, width, height)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_adapts_column_count_to_width() {
        let narrow = grid_areas(Rect::new(0, 0, 30, 30), 6);
        let medium = grid_areas(Rect::new(0, 0, 60, 15), 6);
        let wide = grid_areas(Rect::new(0, 0, 90, 10), 6);

        assert_eq!(narrow[1].x, narrow[0].x);
        assert!(medium[1].x > medium[0].x);
        assert!(wide[2].x > wide[1].x);
    }

    #[test]
    fn trigger_bar_handles_unknown_and_clamps_values() {
        assert_eq!(trigger_bar(None), "[·····] n/a");
        assert_eq!(trigger_bar(Some(-1.0)), "[░░░░░] 0.00");
        assert_eq!(trigger_bar(Some(1.5)), "[█████] 1.00");
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
            .with_control(Control::new(
                "North",
                ControlValue::Button { pressed: false },
            ))
            .with_control(Control::new(
                "West",
                ControlValue::Button { pressed: false },
            ))
            .with_control(Control::new("East", ControlValue::Button { pressed: true }))
            .with_control(Control::new(
                "South",
                ControlValue::Button { pressed: false },
            ));

        let lines = diamond_lines(&cluster, Style::default(), Style::default());

        assert_eq!(lines[0].spans[0].content, "○ North");
        assert_eq!(lines[1].spans[0].content, "○ West");
        assert_eq!(lines[1].spans[2].content, "● East");
        assert_eq!(lines[2].spans[0].content, "○ South");
    }
}
