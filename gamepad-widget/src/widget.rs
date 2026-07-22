use crate::{ClusterPlacement, Control, ControlCluster, ControlValue, GamepadState};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

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

        let areas = cluster_areas(area, clusters);
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
                ClusterPlacement::LeftStick | ClusterPlacement::RightStick => {
                    stick_lines(cluster, self.idle_style, self.active_style)
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

fn stick_lines(
    cluster: &ControlCluster,
    idle_style: Style,
    active_style: Style,
) -> Vec<Line<'static>> {
    let [control] = cluster.controls() else {
        return control_lines(cluster, idle_style, active_style);
    };
    let ControlValue::Stick { x, y, pressed } = control.value() else {
        return control_lines(cluster, idle_style, active_style);
    };
    let magnitude = x.hypot(y);
    let style = if pressed || magnitude > 0.15 {
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

fn cluster_areas(area: Rect, clusters: &[ControlCluster]) -> Vec<Rect> {
    controller_areas(area, clusters).unwrap_or_else(|| grid_areas(area, clusters.len()))
}

fn controller_areas(area: Rect, clusters: &[ControlCluster]) -> Option<Vec<Rect>> {
    const MIN_WIDTH: u16 = 88;
    const MIN_HEIGHT: u16 = 17;
    const TOP_HEIGHT: u16 = 5;
    const GAP: u16 = 1;
    const EXTRA_HEIGHT: u16 = 4;

    let extras = clusters
        .iter()
        .filter(|cluster| {
            matches!(
                cluster.placement(),
                ClusterPlacement::Flow | ClusterPlacement::Extra
            )
        })
        .count();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT || extras > 1 {
        return None;
    }

    let extra_height = if extras == 1 { EXTRA_HEIGHT + GAP } else { 0 };
    if area.height < MIN_HEIGHT + extra_height {
        return None;
    }
    let body_y = area.y + TOP_HEIGHT + GAP;
    let body_bottom = area.bottom().saturating_sub(extra_height);
    let body_height = body_bottom.saturating_sub(body_y);
    let shoulder_columns = equal_columns(area, 3);
    let body_columns = equal_columns(Rect::new(area.x, body_y, area.width, body_height), 4);

    clusters
        .iter()
        .map(|cluster| {
            let rect = match cluster.placement() {
                ClusterPlacement::LeftShoulder => Rect::new(
                    shoulder_columns[0].x,
                    area.y,
                    shoulder_columns[0].width,
                    TOP_HEIGHT,
                ),
                ClusterPlacement::Menu => Rect::new(
                    shoulder_columns[1].x,
                    area.y,
                    shoulder_columns[1].width,
                    TOP_HEIGHT,
                ),
                ClusterPlacement::RightShoulder => Rect::new(
                    shoulder_columns[2].x,
                    area.y,
                    shoulder_columns[2].width,
                    TOP_HEIGHT,
                ),
                ClusterPlacement::LeftStick => body_columns[0],
                ClusterPlacement::DPad => body_columns[1],
                ClusterPlacement::RightStick => body_columns[2],
                ClusterPlacement::Face => body_columns[3],
                ClusterPlacement::Flow | ClusterPlacement::Extra => Rect::new(
                    area.x,
                    area.bottom().saturating_sub(EXTRA_HEIGHT),
                    area.width,
                    EXTRA_HEIGHT,
                ),
            };
            (!rect.is_empty()).then_some(rect)
        })
        .collect()
}

fn equal_columns(area: Rect, count: usize) -> Vec<Rect> {
    let gaps = u16::try_from(count.saturating_sub(1)).unwrap_or(u16::MAX);
    let usable_width = area.width.saturating_sub(gaps);
    let count = u16::try_from(count).unwrap_or(1);
    let base_width = usable_width / count;

    (0..count)
        .map(|index| {
            let x = area.x + index * (base_width + 1);
            let width = if index + 1 == count {
                area.right().saturating_sub(x)
            } else {
                base_width
            };
            Rect::new(x, area.y, width, area.height)
        })
        .collect()
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
            .with_control(Control::new(
                "L3",
                ControlValue::Stick {
                    x: 0.3,
                    y: 0.4,
                    pressed: true,
                },
            ));

        let lines = stick_lines(&cluster, Style::default(), Style::default());

        assert_eq!(lines[7].spans[0].content, "x +0.30  y +0.40");
        assert_eq!(lines[8].spans[0].content, "r 0.50  L3 ●");
    }

    #[test]
    fn semantic_layout_matches_controller_geometry() {
        let clusters = [
            ControlCluster::new("Left stick").with_placement(ClusterPlacement::LeftStick),
            ControlCluster::new("D-pad").with_placement(ClusterPlacement::DPad),
            ControlCluster::new("Right stick").with_placement(ClusterPlacement::RightStick),
            ControlCluster::new("Face").with_placement(ClusterPlacement::Face),
        ];

        let areas = controller_areas(Rect::new(0, 0, 100, 20), &clusters)
            .expect("wide area should use controller layout");

        assert!(areas[0].x < areas[1].x);
        assert!(areas[1].x < areas[2].x);
        assert!(areas[2].x < areas[3].x);
        assert_eq!(areas[0].y, areas[1].y);
        assert_eq!(areas[1].y, areas[2].y);
        assert_eq!(areas[2].y, areas[3].y);
        assert_eq!(areas[0].height, areas[3].height);
    }

    #[test]
    fn semantic_layout_falls_back_in_small_areas() {
        let clusters =
            [ControlCluster::new("Left stick").with_placement(ClusterPlacement::LeftStick)];

        assert!(controller_areas(Rect::new(0, 0, 60, 12), &clusters).is_none());
    }

    #[test]
    fn semantic_layout_reserves_space_for_menu_controls() {
        let clusters = [ControlCluster::new("Menu").with_placement(ClusterPlacement::Menu)];

        let areas = controller_areas(Rect::new(0, 0, 100, 17), &clusters)
            .expect("standard area should use controller layout");

        assert_eq!(areas[0].height, 5);
    }

    #[test]
    fn semantic_layout_does_not_clip_extra_cluster() {
        let clusters = [
            ControlCluster::new("Left stick").with_placement(ClusterPlacement::LeftStick),
            ControlCluster::new("Extra").with_placement(ClusterPlacement::Extra),
        ];

        assert!(controller_areas(Rect::new(0, 0, 100, 17), &clusters).is_none());
        assert!(controller_areas(Rect::new(0, 0, 100, 22), &clusters).is_some());
    }
}
