mod controls;
mod shell;
mod stick;

use self::controls::{render_art_diamond, render_control_row, render_shoulder};
use self::shell::render_shell;
use self::stick::render_art_stick;
use super::GamepadWidget;
use crate::{ClusterPlacement, ControlCluster, GamepadState};
use ratatui::{buffer::Buffer, layout::Rect};

pub(super) fn render_controller_art(area: Rect, buffer: &mut Buffer, widget: GamepadWidget<'_>) {
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

#[inline]
fn cluster_at(state: &GamepadState, placement: ClusterPlacement) -> Option<&ControlCluster> {
    state
        .clusters()
        .iter()
        .find(|cluster| cluster.placement() == placement)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Control, ControlValue};
    use ratatui::widgets::Widget;

    #[test]
    fn semantic_overview_renders_state_only_controller_art() {
        let state = GamepadState::new([ControlCluster::new("Left stick")
            .with_placement(ClusterPlacement::LeftStick)
            .with_control(Control::new("L3", ControlValue::stick(0.0, 0.0, false)))]);
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
}
