use super::ControlValue;

/// A named group of related gamepad controls.
///
/// A cluster can identify its approximate physical [`ClusterPlacement`] while
/// retaining its insertion order for generic or unknown layouts.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlCluster {
    title: String,
    controls: Vec<Control>,
    placement: ClusterPlacement,
}

impl ControlCluster {
    /// Creates an empty cluster that uses [`ClusterPlacement::Flow`].
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            controls: Vec::new(),
            placement: ClusterPlacement::Flow,
        }
    }

    /// Sets the cluster's approximate physical placement.
    #[must_use]
    #[inline]
    pub const fn with_placement(mut self, placement: ClusterPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Appends a control to the cluster.
    #[must_use]
    pub fn with_control(mut self, control: Control) -> Self {
        self.controls.push(control);
        self
    }

    /// Appends controls from an iterator to the cluster.
    #[must_use]
    pub fn with_controls(mut self, controls: impl IntoIterator<Item = Control>) -> Self {
        self.extend(controls);
        self
    }

    /// Returns the cluster's display title.
    #[must_use]
    #[inline]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the controls in insertion order.
    #[must_use]
    #[inline]
    pub fn controls(&self) -> &[Control] {
        &self.controls
    }

    /// Returns the cluster's approximate physical placement.
    #[must_use]
    #[inline]
    pub const fn placement(&self) -> ClusterPlacement {
        self.placement
    }
}

impl Extend<Control> for ControlCluster {
    fn extend<T: IntoIterator<Item = Control>>(&mut self, iter: T) {
        self.controls.extend(iter);
    }
}

/// An approximate physical location for a control cluster.
///
/// Placements let [`crate::GamepadWidget`] arrange common controls like a
/// familiar gamepad while [`Self::Flow`] and [`Self::Extra`] accommodate
/// generic or vendor-specific controls.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum ClusterPlacement {
    /// No physical location is known; place the cluster in normal flow order.
    #[default]
    Flow,
    /// The left shoulder, bumper, or trigger area.
    LeftShoulder,
    /// The central menu, system, or auxiliary button area.
    Menu,
    /// The right shoulder, bumper, or trigger area.
    RightShoulder,
    /// The left analog-stick area.
    LeftStick,
    /// The primary face-button area.
    Face,
    /// The directional-pad area.
    DPad,
    /// The right analog-stick area.
    RightStick,
    /// An additional control group without a standard gamepad location.
    Extra,
}

/// A labeled gamepad input and its current value.
#[derive(Debug, Clone, PartialEq)]
pub struct Control {
    label: String,
    value: ControlValue,
}

impl Control {
    /// Creates a control with a display label and current value.
    pub fn new(label: impl Into<String>, value: ControlValue) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }

    /// Returns the control's display label.
    #[must_use]
    #[inline]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the control's current value.
    #[must_use]
    #[inline]
    pub const fn value(&self) -> ControlValue {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_placement_defaults_to_flow() {
        let cluster = ControlCluster::new("Generic");

        assert_eq!(cluster.placement(), ClusterPlacement::Flow);
    }

    #[test]
    fn cluster_supports_bulk_control_building() {
        let mut cluster = ControlCluster::new("Face").with_controls([
            Control::new("North", ControlValue::button(false)),
            Control::new("East", ControlValue::button(true)),
        ]);
        cluster.extend([Control::new("South", ControlValue::button(false))]);

        assert_eq!(cluster.controls().len(), 3);
        assert_eq!(cluster.controls()[0].label(), "North");
        assert_eq!(cluster.controls()[1].label(), "East");
        assert_eq!(cluster.controls()[2].label(), "South");
    }
}
