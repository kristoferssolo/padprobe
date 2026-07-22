//! Backend-neutral gamepad state and a responsive Ratatui widget.

mod model;
mod widget;

pub use model::{ClusterPlacement, Control, ControlCluster, ControlValue, GamepadState};
pub use widget::GamepadWidget;
