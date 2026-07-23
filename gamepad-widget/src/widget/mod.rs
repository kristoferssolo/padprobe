mod controller;
mod controls;
mod layout;

use self::controller::render_controller_art;
use self::controls::{control_lines, diamond_lines, stick_lines, vertically_center};
use self::layout::{can_render_controller, grid_areas};
use crate::{ClusterPlacement, GamepadState, GamepadTheme};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph, Widget},
};

/// A responsive, backend-neutral gamepad overview widget.
///
/// In sufficiently large areas, controls with semantic
/// [`ClusterPlacement`] values are arranged in a controller-shaped layout.
/// Smaller areas and generic clusters fall back to a responsive grid.
#[derive(Debug, Clone, Copy)]
pub struct GamepadWidget<'state> {
    state: &'state GamepadState,
    layout: GamepadLayout,
    border_style: Style,
    idle_style: Style,
    active_style: Style,
}

/// The layout policy used by [`GamepadWidget`].
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum GamepadLayout {
    /// Uses the controller layout when all controls fit, otherwise a grid.
    #[default]
    Auto,
    /// Prefers the controller layout and safely falls back when it cannot fit.
    Controller,
    /// Always renders every cluster in a responsive grid.
    Grid,
}

impl<'state> GamepadWidget<'state> {
    /// Creates a widget that borrows the gamepad state to render.
    #[must_use]
    #[inline]
    pub fn new(state: &'state GamepadState) -> Self {
        let theme = GamepadTheme::default();
        Self {
            state,
            layout: GamepadLayout::default(),
            border_style: theme.border,
            idle_style: theme.idle,
            active_style: theme.active,
        }
    }

    /// Sets the layout policy.
    #[must_use]
    #[inline]
    pub const fn layout(mut self, layout: GamepadLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Applies a shared gamepad theme to the overview.
    #[must_use]
    #[inline]
    pub const fn theme(mut self, theme: GamepadTheme) -> Self {
        self.border_style = theme.border;
        self.idle_style = theme.idle;
        self.active_style = theme.active;
        self
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

        if self.layout != GamepadLayout::Grid && can_render_controller(area, clusters) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Control, ControlCluster, ControlValue};

    #[test]
    fn explicit_grid_layout_bypasses_controller_art() {
        let state = GamepadState::new([ControlCluster::new("Menu")
            .with_placement(ClusterPlacement::Menu)
            .with_control(Control::new("Start", ControlValue::button(false)))]);
        let area = Rect::new(0, 0, 70, 25);
        let mut buffer = Buffer::empty(area);

        GamepadWidget::new(&state)
            .layout(GamepadLayout::Grid)
            .render(area, &mut buffer);

        let symbols = buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(symbols.contains("Menu"));
        assert!(symbols.contains("Start"));
        assert!(symbols.contains('┌'));
    }
}
