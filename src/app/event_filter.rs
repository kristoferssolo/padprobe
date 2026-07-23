use super::{App, EventEntry};
use crate::analysis::{TimingMetrics, TimingSample};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum EventKindFilter {
    #[default]
    All,
    Buttons,
    Axes,
    System,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum EventDeviceFilter {
    #[default]
    All,
    Selected,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum EventSearchState {
    #[default]
    Closed,
    Open,
}

impl EventKindFilter {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Buttons => "buttons",
            Self::Axes => "axes",
            Self::System => "system",
        }
    }

    const fn next(self) -> Self {
        match self {
            Self::All => Self::Buttons,
            Self::Buttons => Self::Axes,
            Self::Axes => Self::System,
            Self::System => Self::All,
        }
    }
}

impl App {
    pub fn cycle_event_kind_filter(&mut self) {
        self.event_kind_filter = self.event_kind_filter.next();
        self.status = format!("Event filter: {}", self.event_kind_filter.label());
    }

    pub fn toggle_event_device_filter(&mut self) {
        self.event_device_filter = match self.event_device_filter {
            EventDeviceFilter::All => EventDeviceFilter::Selected,
            EventDeviceFilter::Selected => EventDeviceFilter::All,
        };
        if self.event_device_filter == EventDeviceFilter::Selected {
            "Event filter: selected controller only"
        } else {
            "Event filter: all controllers"
        }
        .clone_into(&mut self.status);
    }

    #[must_use]
    pub fn event_is_visible(&self, entry: &EventEntry) -> bool {
        let device_matches = self.event_device_filter == EventDeviceFilter::All
            || entry.device_id == self.selected_id;
        let kind_matches = match self.event_kind_filter {
            EventKindFilter::All => true,
            EventKindFilter::Buttons => entry.description.starts_with("Button"),
            EventKindFilter::Axes => entry.description.starts_with("Axis"),
            EventKindFilter::System => {
                !entry.description.starts_with("Button") && !entry.description.starts_with("Axis")
            }
        };
        let search_matches = self.event_search.is_empty()
            || entry
                .description
                .to_lowercase()
                .contains(&self.event_search.to_lowercase());
        device_matches && kind_matches && search_matches
    }

    #[must_use]
    pub fn selected_event_timing(&self) -> Option<TimingMetrics> {
        let selected_id = self.selected_id?;
        let samples = self
            .events
            .iter()
            .filter(|entry| entry.device_id == Some(selected_id))
            .filter(|entry| {
                entry.description.starts_with("Button") || entry.description.starts_with("Axis")
            })
            .map(|entry| TimingSample {
                elapsed_seconds: entry.elapsed.as_secs_f64(),
                signature: entry.description.clone(),
            })
            .collect::<Vec<_>>();
        TimingMetrics::calculate(&samples)
    }
}
