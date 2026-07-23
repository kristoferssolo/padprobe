use super::super::{GamepadWidget, controls::control_span};
use crate::ControlCluster;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClusterPlacement, Control, ControlValue, GamepadState};
    use ratatui::style::Color;

    #[test]
    fn shoulder_art_separates_trigger_and_bumper() {
        let cluster = ControlCluster::new("Left shoulder")
            .with_placement(ClusterPlacement::LeftShoulder)
            .with_controls([
                Control::new("LB", ControlValue::button(false)),
                Control::new("LT", ControlValue::trigger(0.5)),
            ]);
        let state = GamepadState::default();
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
