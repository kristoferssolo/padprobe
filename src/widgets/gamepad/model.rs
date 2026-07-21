#[derive(Clone, Debug, Default, PartialEq)]
pub struct GamepadState {
    clusters: Vec<ControlCluster>,
}

impl GamepadState {
    #[must_use]
    pub fn new(clusters: impl IntoIterator<Item = ControlCluster>) -> Self {
        Self {
            clusters: clusters.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn clusters(&self) -> &[ControlCluster] {
        &self.clusters
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlCluster {
    title: String,
    controls: Vec<Control>,
}

impl ControlCluster {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            controls: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_control(mut self, control: Control) -> Self {
        self.controls.push(control);
        self
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn controls(&self) -> &[Control] {
        &self.controls
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Control {
    label: String,
    value: ControlValue,
}

impl Control {
    pub fn new(label: impl Into<String>, value: ControlValue) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }

    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[must_use]
    pub const fn value(&self) -> ControlValue {
        self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlValue {
    Button { pressed: bool },
    Stick { x: f32, y: f32, pressed: bool },
    Trigger { value: Option<f32> },
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
}
