mod adapter;

use padprobe_gamepad_widget::GamepadWidget;
use ratatui::{Frame, layout::Rect};

use crate::app::DeviceState;

use self::adapter::gamepad_state;

pub(super) fn render_gamepad(frame: &mut Frame<'_>, device: &DeviceState, area: Rect) {
    let state = gamepad_state(device);
    frame.render_widget(GamepadWidget::new(&state), area);
}
