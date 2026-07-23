mod metrics;
mod session;

pub use metrics::DriftMetrics;
use serde::Serialize;
pub use session::{DriftTest, DriftView};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize)]
pub enum StickSide {
    #[default]
    Left,
    Right,
}

impl StickSide {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
}
