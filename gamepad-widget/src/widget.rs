use crate::{ClusterPlacement, Control, ControlCluster, ControlValue, GamepadState};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Block, Borders, Paragraph, Widget,
        canvas::{Canvas, Circle},
    },
};

/// A responsive, backend-neutral gamepad overview widget.
///
/// In sufficiently large areas, controls with semantic
/// [`ClusterPlacement`] values are arranged in a controller-shaped layout.
/// Smaller areas and generic clusters fall back to a responsive grid.
#[derive(Clone, Copy, Debug)]
pub struct GamepadWidget<'state> {
    state: &'state GamepadState,
    border_style: Style,
    idle_style: Style,
    active_style: Style,
}

impl<'state> GamepadWidget<'state> {
    /// Creates a widget that borrows the gamepad state to render.
    #[must_use]
    #[inline]
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

    /// Sets the style used for the controller outline and cluster borders.
    #[must_use]
    #[inline]
    pub const fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Sets the style used for controls that are not active.
    #[must_use]
    #[inline]
    pub const fn idle_style(mut self, style: Style) -> Self {
        self.idle_style = style;
        self
    }

    /// Sets the style used for pressed buttons and active analog controls.
    #[must_use]
    #[inline]
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

        if can_render_controller(area, clusters) {
            render_controller_art(area, buffer, self);
            return;
        }

        for (cluster, area) in clusters.iter().zip(grid_areas(area, clusters.len())) {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", cluster.title()))
                .border_style(self.border_style);
            let inner = block.inner(area);
            block.render(area, buffer);
            let lines = match cluster.placement() {
                ClusterPlacement::DPad | ClusterPlacement::Face => vertically_center(
                    diamond_lines(cluster, self.idle_style, self.active_style),
                    inner.height,
                ),
                ClusterPlacement::LeftStick | ClusterPlacement::RightStick => {
                    stick_lines(cluster, self.idle_style, self.active_style)
                }
                _ => control_lines(cluster, self.idle_style, self.active_style),
            };
            Paragraph::new(lines).render(inner, buffer);
        }
    }
}

fn render_controller_art(area: Rect, buffer: &mut Buffer, widget: GamepadWidget<'_>) {
    let width = area.width.min(70);
    let height = area.height.min(25);
    let art = Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    );
    render_shell(art, buffer, widget.border_style);

    let left_center = art.x + art.width / 4;
    let right_center = art.x + art.width * 3 / 4;
    render_shoulder(
        cluster_at(widget.state, ClusterPlacement::LeftShoulder),
        left_center,
        art.y,
        buffer,
        widget,
    );
    render_shoulder(
        cluster_at(widget.state, ClusterPlacement::RightShoulder),
        right_center,
        art.y,
        buffer,
        widget,
    );
    render_control_row(
        cluster_at(widget.state, ClusterPlacement::Menu),
        Rect::new(art.x + art.width / 4, art.y + 9, art.width / 2, 1),
        buffer,
        widget,
    );
    render_art_stick(
        cluster_at(widget.state, ClusterPlacement::LeftStick),
        left_center,
        art.y + 10,
        buffer,
        widget,
    );
    render_art_diamond(
        cluster_at(widget.state, ClusterPlacement::Face),
        right_center,
        art.y + 10,
        buffer,
        widget,
    );
    render_art_diamond(
        cluster_at(widget.state, ClusterPlacement::DPad),
        left_center,
        art.y + 16,
        buffer,
        widget,
    );
    render_art_stick(
        cluster_at(widget.state, ClusterPlacement::RightStick),
        right_center,
        art.y + 16,
        buffer,
        widget,
    );
}

fn render_shell(area: Rect, buffer: &mut Buffer, style: Style) {
    let left = area.x;
    let right = area.right().saturating_sub(1);
    let top = area.y + 6;
    draw_horizontal(buffer, left + 11, right - 9, top, style);
    buffer[(left + 10, top)].set_char('╭').set_style(style);
    buffer[(right - 10, top)].set_char('╮').set_style(style);

    draw_horizontal(buffer, left + 7, left + 10, top + 1, style);
    draw_horizontal(buffer, right - 9, right - 6, top + 1, style);
    buffer[(left + 6, top + 1)].set_char('╭').set_style(style);
    buffer[(left + 10, top + 1)].set_char('╯').set_style(style);
    buffer[(right - 10, top + 1)].set_char('╰').set_style(style);
    buffer[(right - 6, top + 1)].set_char('╮').set_style(style);

    draw_horizontal(buffer, left + 4, left + 6, top + 2, style);
    draw_horizontal(buffer, right - 5, right - 3, top + 2, style);
    buffer[(left + 3, top + 2)].set_char('╭').set_style(style);
    buffer[(left + 6, top + 2)].set_char('╯').set_style(style);
    buffer[(right - 6, top + 2)].set_char('╰').set_style(style);
    buffer[(right - 3, top + 2)].set_char('╮').set_style(style);

    buffer[(left + 2, top + 3)].set_char('╱').set_style(style);
    buffer[(right - 2, top + 3)].set_char('╲').set_style(style);
    buffer[(left + 1, top + 4)].set_char('╱').set_style(style);
    buffer[(right - 1, top + 4)].set_char('╲').set_style(style);
    for y in top + 5..top + 14 {
        buffer[(left + 1, y)].set_char('│').set_style(style);
        buffer[(right - 1, y)].set_char('│').set_style(style);
    }

    buffer[(left + 1, top + 14)].set_char('╲').set_style(style);
    buffer[(right - 1, top + 14)].set_char('╱').set_style(style);
    buffer[(left + 2, top + 15)].set_char('╲').set_style(style);
    buffer[(right - 2, top + 15)].set_char('╱').set_style(style);
    draw_horizontal(buffer, left + 19, right - 18, top + 15, style);
    buffer[(left + 18, top + 15)].set_char('╭').set_style(style);
    buffer[(right - 18, top + 15)]
        .set_char('╮')
        .set_style(style);

    buffer[(left + 3, top + 16)].set_char('╲').set_style(style);
    buffer[(left + 17, top + 16)].set_char('╱').set_style(style);
    buffer[(right - 17, top + 16)]
        .set_char('╲')
        .set_style(style);
    buffer[(right - 3, top + 16)].set_char('╱').set_style(style);
    buffer[(left + 4, top + 17)].set_char('╲').set_style(style);
    buffer[(left + 16, top + 17)].set_char('╱').set_style(style);
    buffer[(right - 16, top + 17)]
        .set_char('╲')
        .set_style(style);
    buffer[(right - 4, top + 17)].set_char('╱').set_style(style);

    buffer[(left + 5, top + 18)].set_char('╰').set_style(style);
    buffer[(left + 15, top + 18)].set_char('╯').set_style(style);
    buffer[(right - 15, top + 18)]
        .set_char('╰')
        .set_style(style);
    buffer[(right - 5, top + 18)].set_char('╯').set_style(style);
    draw_horizontal(buffer, left + 6, left + 15, top + 18, style);
    draw_horizontal(buffer, right - 14, right - 5, top + 18, style);
}

fn draw_horizontal(buffer: &mut Buffer, start: u16, end: u16, y: u16, style: Style) {
    for x in start..end {
        buffer[(x, y)].set_char('─').set_style(style);
    }
}

#[inline]
fn cluster_at(state: &GamepadState, placement: ClusterPlacement) -> Option<&ControlCluster> {
    state
        .clusters()
        .iter()
        .find(|cluster| cluster.placement() == placement)
}

fn render_shoulder(
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
    let trigger_style = if control_active(trigger) {
        widget.active_style
    } else {
        widget.border_style
    };
    let bumper_style = if control_active(bumper) {
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

fn render_control_row(
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

fn render_art_stick(
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
    let ControlValue::Stick {
        x,
        y: axis_y,
        pressed,
    } = control.value()
    else {
        return;
    };
    let style = if pressed || x.hypot(axis_y) > 0.15 {
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

fn render_art_diamond(
    cluster: Option<&ControlCluster>,
    center: u16,
    y: u16,
    buffer: &mut Buffer,
    widget: GamepadWidget<'_>,
) {
    let Some(cluster) = cluster else {
        return;
    };
    Paragraph::new(diamond_lines(
        cluster,
        widget.idle_style,
        widget.active_style,
    ))
    .render(Rect::new(center.saturating_sub(9), y, 18, 3), buffer);
}

fn control_span(control: &Control, idle_style: Style, active_style: Style) -> Span<'static> {
    let active = control_active(control);
    Span::styled(
        format!("{} {}", if active { "●" } else { "○" }, control.label()),
        if active { active_style } else { idle_style },
    )
}

#[inline]
fn control_active(control: &Control) -> bool {
    match control.value() {
        ControlValue::Button { pressed } => pressed,
        ControlValue::Trigger { value } => value.is_some_and(|value| value > 0.1),
        ControlValue::Stick { x, y, pressed } => pressed || x.hypot(y) > 0.15,
        ControlValue::Axis { value } => value.abs() > 0.15,
    }
}

fn vertically_center(lines: Vec<Line<'static>>, height: u16) -> Vec<Line<'static>> {
    let padding = usize::from(height).saturating_sub(lines.len()) / 2;
    let mut centered = Vec::with_capacity(padding + lines.len());
    centered.resize_with(padding, Line::default);
    centered.extend(lines);
    centered
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

#[inline]
fn can_render_controller(area: Rect, clusters: &[ControlCluster]) -> bool {
    const MIN_WIDTH: u16 = 48;
    const MIN_HEIGHT: u16 = 25;

    area.width >= MIN_WIDTH
        && area.height >= MIN_HEIGHT
        && clusters.iter().enumerate().all(|(index, cluster)| {
            let placement = cluster.placement();
            !matches!(placement, ClusterPlacement::Flow | ClusterPlacement::Extra)
                && cluster_fits_controller_art(cluster)
                && !clusters[..index]
                    .iter()
                    .any(|previous| previous.placement() == placement)
        })
}

#[inline]
fn cluster_fits_controller_art(cluster: &ControlCluster) -> bool {
    match cluster.placement() {
        ClusterPlacement::LeftStick | ClusterPlacement::RightStick => {
            matches!(
                cluster.controls(),
                [control] if matches!(control.value(), ControlValue::Stick { .. })
            )
        }
        ClusterPlacement::DPad | ClusterPlacement::Face => cluster.controls().len() <= 4,
        ClusterPlacement::Flow
        | ClusterPlacement::LeftShoulder
        | ClusterPlacement::Menu
        | ClusterPlacement::RightShoulder
        | ClusterPlacement::Extra => true,
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
    fn controller_control_span_hides_exact_trigger_value() {
        let control = Control::new("LT", ControlValue::Trigger { value: Some(0.0) });

        let span = control_span(&control, Style::default(), Style::default());

        assert_eq!(span.content, "○ LT");
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
    fn compact_controls_are_centered_in_tall_clusters() {
        let lines = vertically_center(
            vec![
                Line::from("north"),
                Line::from("middle"),
                Line::from("south"),
            ],
            9,
        );

        assert_eq!(lines.len(), 6);
        assert!(lines[0].spans.is_empty());
        assert_eq!(lines[3].spans[0].content, "north");
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
    fn semantic_layout_falls_back_in_small_areas() {
        let clusters =
            [ControlCluster::new("Left stick").with_placement(ClusterPlacement::LeftStick)];

        assert!(!can_render_controller(Rect::new(0, 0, 60, 12), &clusters));
    }

    #[test]
    fn semantic_layout_accepts_standard_placements() {
        let clusters = [ControlCluster::new("Menu").with_placement(ClusterPlacement::Menu)];

        assert!(can_render_controller(Rect::new(0, 0, 100, 25), &clusters));
    }

    #[test]
    fn semantic_layout_does_not_clip_extra_cluster() {
        let clusters = [
            ControlCluster::new("Left stick").with_placement(ClusterPlacement::LeftStick),
            ControlCluster::new("Extra").with_placement(ClusterPlacement::Extra),
        ];

        assert!(!can_render_controller(Rect::new(0, 0, 100, 25), &clusters));
    }

    #[test]
    fn semantic_layout_does_not_hide_duplicate_placements() {
        let clusters = [
            ControlCluster::new("First").with_placement(ClusterPlacement::LeftStick),
            ControlCluster::new("Second").with_placement(ClusterPlacement::LeftStick),
        ];

        assert!(!can_render_controller(Rect::new(0, 0, 100, 25), &clusters));
    }

    #[test]
    fn semantic_layout_does_not_hide_malformed_stick_cluster() {
        let clusters = [ControlCluster::new("Left stick")
            .with_placement(ClusterPlacement::LeftStick)
            .with_control(Control::new(
                "unexpected",
                ControlValue::Button { pressed: false },
            ))];

        assert!(!can_render_controller(Rect::new(0, 0, 100, 25), &clusters));
    }

    #[test]
    fn semantic_overview_renders_state_only_controller_art() {
        let state = GamepadState::new([ControlCluster::new("Left stick")
            .with_placement(ClusterPlacement::LeftStick)
            .with_control(Control::new(
                "L3",
                ControlValue::Stick {
                    x: 0.0,
                    y: 0.0,
                    pressed: false,
                },
            ))]);
        let area = Rect::new(0, 0, 70, 25);
        let mut buffer = Buffer::empty(area);

        GamepadWidget::new(&state).render(area, &mut buffer);

        let symbols = buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(symbols.contains("L3 ○"));
        assert!(
            symbols
                .chars()
                .any(|symbol| ('\u{2800}'..='\u{28ff}').contains(&symbol))
        );
        assert!(!symbols.contains("x +0.00"));
        assert!(symbols.contains('─'));
        assert!(!symbols.contains('┌'));
    }

    #[test]
    fn controller_shell_has_shoulders_body_notch_and_grips() {
        let area = Rect::new(0, 0, 70, 25);
        let mut buffer = Buffer::empty(area);

        render_shell(area, &mut buffer, Style::default());

        assert_eq!(buffer[(10, 6)].symbol(), "╭");
        assert_eq!(buffer[(59, 6)].symbol(), "╮");
        assert_eq!(buffer[(1, 11)].symbol(), "│");
        assert_eq!(buffer[(18, 21)].symbol(), "╭");
        assert_eq!(buffer[(51, 21)].symbol(), "╮");
        assert_eq!(buffer[(5, 24)].symbol(), "╰");
        assert_eq!(buffer[(15, 24)].symbol(), "╯");
        assert_eq!(buffer[(54, 24)].symbol(), "╰");
        assert_eq!(buffer[(64, 24)].symbol(), "╯");
    }

    #[test]
    fn shoulder_art_separates_trigger_and_bumper() {
        let cluster = ControlCluster::new("Left shoulder")
            .with_placement(ClusterPlacement::LeftShoulder)
            .with_control(Control::new("LB", ControlValue::Button { pressed: false }))
            .with_control(Control::new(
                "LT",
                ControlValue::Trigger { value: Some(0.5) },
            ));
        let state = GamepadState::new([]);
        let widget = GamepadWidget::new(&state);
        let area = Rect::new(0, 0, 9, 6);
        let mut buffer = Buffer::empty(area);

        render_shoulder(Some(&cluster), 4, 0, &mut buffer, widget);
        let symbols = buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();

        assert!(symbols.contains("LT"));
        assert!(symbols.contains("LB"));
        assert!(buffer.content().iter().any(|cell| cell.fg == Color::Cyan));
        assert!(
            buffer
                .content()
                .iter()
                .any(|cell| cell.fg == Color::DarkGray)
        );
    }
}
