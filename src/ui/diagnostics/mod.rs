mod raw;
mod sticks;
mod triggers;

use crate::app::{App, DeviceState};
use gilrs::{Axis, Button};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders},
};

pub(super) use raw::render_raw_data;

pub(super) fn render_primary_diagnostics(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);
    sticks::render(
        frame,
        app.selected_device().map(|(_, device)| device),
        columns[0],
    );
    triggers::render(
        frame,
        app.selected_device().map(|(_, device)| device),
        columns[1],
    );
}

fn diagnostic_block(title: &str, color: Color) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Line::styled(title, Style::default().fg(color)))
}

#[inline]
fn axis_value(device: &DeviceState, axis: Axis) -> f32 {
    device.axes.get(&axis).map_or(0.0, |state| state.current)
}

#[inline]
fn trigger_value(device: &DeviceState, axis: Axis, button: Button) -> f32 {
    device
        .button_values
        .get(&button)
        .copied()
        .or_else(|| {
            device
                .axes
                .get(&axis)
                .map(|state| state.current.midpoint(1.0))
        })
        .unwrap_or_default()
}
