//! Backend-neutral gamepad state and a responsive Ratatui widget.
#![deny(missing_docs)]

mod model;
mod stick;
mod theme;
mod widget;

pub use model::{ClusterPlacement, Control, ControlCluster, ControlValue, GamepadState};
pub use stick::StickGauge;
pub use theme::GamepadTheme;
pub use widget::GamepadWidget;

pub mod prelude {
    //! Convenient imports for constructing and rendering gamepad state.

    pub use crate::{
        ClusterPlacement, Control, ControlCluster, ControlValue, GamepadState, GamepadTheme,
        GamepadWidget, StickGauge,
    };
}
