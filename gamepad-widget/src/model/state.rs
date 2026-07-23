use super::ControlCluster;
use std::slice::Iter;

/// A snapshot of the controls presented by a gamepad.
///
/// Controls are organized into [`ControlCluster`] values so renderers can
/// preserve familiar physical groupings without depending on a specific
/// controller model.
#[derive(Debug, Clone, Default, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Control, ControlValue};

    #[test]
    fn state_preserves_cluster_order() {
        let state = GamepadState::new([
            ControlCluster::new("Left").with_control(Control::new(
                "Stick",
                ControlValue::stick(0.5, -0.25, false),
            )),
            ControlCluster::new("Face"),
        ]);

        assert_eq!(state.clusters()[0].title(), "Left");
        assert_eq!(state.clusters()[1].title(), "Face");
        assert_eq!(state.clusters()[0].controls()[0].label(), "Stick");
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
