use std::slice::Iter;

/// A snapshot of the controls presented by a gamepad.
///
/// Controls are organized into [`ControlCluster`] values so renderers can
/// preserve familiar physical groupings without depending on a specific
/// controller model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GamepadState {
    clusters: Vec<ControlCluster>,
}

impl GamepadState {
    /// Creates a state snapshot from an ordered collection of control clusters.
    ///
    /// Cluster order is retained and is used when a renderer falls back to a
    /// generic flowing layout.
    #[must_use]
    pub fn new(clusters: impl IntoIterator<Item = ControlCluster>) -> Self {
        clusters.into_iter().collect()
    }

    /// Returns the control clusters in their original order.
    #[must_use]
    #[inline]
    pub fn clusters(&self) -> &[ControlCluster] {
        &self.clusters
    }

    /// Returns the number of control clusters in this snapshot.
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.clusters.len()
    }

    /// Returns `true` when this snapshot contains no control clusters.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.clusters.is_empty()
    }

    /// Iterates over the control clusters in their original order.
    #[inline]
    pub fn iter(&self) -> Iter<'_, ControlCluster> {
        self.clusters.iter()
    }
}

impl FromIterator<ControlCluster> for GamepadState {
    fn from_iter<T: IntoIterator<Item = ControlCluster>>(iter: T) -> Self {
        Self {
            clusters: iter.into_iter().collect(),
        }
    }
}

impl Extend<ControlCluster> for GamepadState {
    fn extend<T: IntoIterator<Item = ControlCluster>>(&mut self, iter: T) {
        self.clusters.extend(iter);
    }
}

impl From<Vec<ControlCluster>> for GamepadState {
    #[inline]
    fn from(clusters: Vec<ControlCluster>) -> Self {
        Self { clusters }
    }
}

impl<'state> IntoIterator for &'state GamepadState {
    type Item = &'state ControlCluster;
    type IntoIter = Iter<'state, ControlCluster>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A named group of related gamepad controls.
///
/// A cluster can identify its approximate physical [`ClusterPlacement`] while
/// retaining its insertion order for generic or unknown layouts.
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
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

/// The current state of a gamepad control.
///
/// Values are backend-neutral presentation data. Producers should generally
/// normalize axes and stick coordinates to `-1.0..=1.0`, and trigger values to
/// `0.0..=1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_preserves_cluster_order() {
        let state = GamepadState::new([
            ControlCluster::new("Left").with_control(Control::new(
                "Stick",
                ControlValue::Stick {
                    x: 0.5,
                    y: -0.25,
                    pressed: false,
                },
            )),
            ControlCluster::new("Face"),
        ]);

        assert_eq!(state.clusters()[0].title(), "Left");
        assert_eq!(state.clusters()[1].title(), "Face");
        assert_eq!(state.clusters()[0].controls()[0].label(), "Stick");
    }

    #[test]
    fn cluster_placement_defaults_to_flow() {
        let cluster = ControlCluster::new("Generic");

        assert_eq!(cluster.placement(), ClusterPlacement::Flow);
    }

    #[test]
    fn cluster_supports_bulk_control_building() {
        let mut cluster = ControlCluster::new("Face").with_controls([
            Control::new("North", ControlValue::Button { pressed: false }),
            Control::new("East", ControlValue::Button { pressed: true }),
        ]);
        cluster.extend([Control::new(
            "South",
            ControlValue::Button { pressed: false },
        )]);

        assert_eq!(cluster.controls().len(), 3);
        assert_eq!(cluster.controls()[0].label(), "North");
        assert_eq!(cluster.controls()[1].label(), "East");
        assert_eq!(cluster.controls()[2].label(), "South");
    }

    #[test]
    fn state_supports_collection_operations() {
        let mut state = ["Left", "Face"]
            .into_iter()
            .map(ControlCluster::new)
            .collect::<GamepadState>();
        state.extend([ControlCluster::new("Extra")]);

        assert_eq!(state.len(), 3);
        assert!(!state.is_empty());
        assert_eq!(
            (&state)
                .into_iter()
                .map(ControlCluster::title)
                .collect::<Vec<_>>(),
            ["Left", "Face", "Extra"]
        );
    }

    #[test]
    fn state_converts_from_cluster_vector_without_reordering() {
        let state = GamepadState::from(vec![
            ControlCluster::new("Menu"),
            ControlCluster::new("D-pad"),
        ]);

        assert_eq!(state.clusters()[0].title(), "Menu");
        assert_eq!(state.clusters()[1].title(), "D-pad");
    }
}
