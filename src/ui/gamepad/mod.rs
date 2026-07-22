mod adapter;

use self::adapter::gamepad_state;
use crate::app::DeviceState;
use gamepad_widget::prelude::*;
use ratatui::{Frame, layout::Rect};

pub(super) fn render_gamepad(frame: &mut Frame<'_>, device: &DeviceState, area: Rect) {
    let state = gamepad_state(device);
    frame.render_widget(GamepadWidget::new(&state), area);
}
