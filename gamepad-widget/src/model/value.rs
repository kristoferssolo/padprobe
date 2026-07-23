/// The current state of a gamepad control.
///
/// Values are backend-neutral presentation data. Producers should generally
/// normalize axes and stick coordinates to `-1.0..=1.0`, and trigger values to
/// `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum ControlValue {
    /// A digital button.
    Button {
        /// Whether the button is currently pressed.
        pressed: bool,
    },
    /// A two-dimensional analog stick and its associated click button.
    Stick {
        /// The normalized horizontal position.
        x: f32,
        /// The normalized vertical position.
        y: f32,
        /// Whether the stick's click button is currently pressed.
        pressed: bool,
    },
    /// A unidirectional analog trigger.
    ///
    /// `None` represents an unavailable or unobserved value.
    Trigger {
        /// The normalized trigger position, or `None` when unavailable.
        value: Option<f32>,
    },
    /// A single signed analog axis.
    Axis {
        /// The normalized axis position.
        value: f32,
    },
}

impl ControlValue {
    const ANALOG_ACTIVE_THRESHOLD: f32 = 0.15;
    const TRIGGER_ACTIVE_THRESHOLD: f32 = 0.1;

    /// Creates a digital button value.
    #[must_use]
    #[inline]
    pub const fn button(pressed: bool) -> Self {
        Self::Button { pressed }
    }

    /// Creates a normalized two-dimensional stick value.
    #[must_use]
    #[inline]
    pub const fn stick(x: f32, y: f32, pressed: bool) -> Self {
        Self::Stick { x, y, pressed }
    }

    /// Creates an available normalized trigger value.
    #[must_use]
    #[inline]
    pub const fn trigger(value: f32) -> Self {
        Self::Trigger { value: Some(value) }
    }

    /// Creates a trigger value whose current position is unavailable.
    #[must_use]
    #[inline]
    pub const fn unavailable_trigger() -> Self {
        Self::Trigger { value: None }
    }

    /// Creates a normalized signed axis value.
    #[must_use]
    #[inline]
    pub const fn axis(value: f32) -> Self {
        Self::Axis { value }
    }

    /// Returns whether the value is active enough to highlight.
    ///
    /// Buttons and stick clicks are active while pressed. Triggers activate
    /// above `0.1`; sticks and signed axes activate beyond a `0.15` deadzone.
    #[must_use]
    #[inline]
    pub fn is_active(self) -> bool {
        match self {
            Self::Button { pressed } => pressed,
            Self::Trigger { value } => {
                value.is_some_and(|value| value > Self::TRIGGER_ACTIVE_THRESHOLD)
            }
            Self::Stick { x, y, pressed } => pressed || x.hypot(y) > Self::ANALOG_ACTIVE_THRESHOLD,
            Self::Axis { value } => value.abs() > Self::ANALOG_ACTIVE_THRESHOLD,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_constructors_preserve_input() {
        assert_eq!(
            ControlValue::button(true),
            ControlValue::Button { pressed: true }
        );
        assert_eq!(
            ControlValue::stick(0.5, -0.25, false),
            ControlValue::Stick {
                x: 0.5,
                y: -0.25,
                pressed: false,
            }
        );
        assert_eq!(
            ControlValue::trigger(0.75),
            ControlValue::Trigger { value: Some(0.75) }
        );
        assert_eq!(
            ControlValue::unavailable_trigger(),
            ControlValue::Trigger { value: None }
        );
        assert_eq!(ControlValue::axis(-0.5), ControlValue::Axis { value: -0.5 });
    }

    #[test]
    fn active_values_follow_widget_thresholds() {
        assert!(ControlValue::button(true).is_active());
        assert!(ControlValue::stick(0.2, 0.0, false).is_active());
        assert!(ControlValue::stick(0.0, 0.0, true).is_active());
        assert!(ControlValue::trigger(0.2).is_active());
        assert!(ControlValue::axis(-0.2).is_active());

        assert!(!ControlValue::button(false).is_active());
        assert!(!ControlValue::stick(0.1, 0.0, false).is_active());
        assert!(!ControlValue::trigger(0.1).is_active());
        assert!(!ControlValue::unavailable_trigger().is_active());
        assert!(!ControlValue::axis(0.1).is_active());
    }
}
