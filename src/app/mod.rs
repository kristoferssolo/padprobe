mod connection;
mod device;
mod diagnostics;
mod event;
mod event_filter;
mod input;
mod navigation;
use crate::analysis::{ControlChecklist, DriftTest, RangeTest};
pub use device::{AxisState, DeviceMetadata, DeviceState, StickTrace};
use event::FIRST_EVENT_SEQUENCE;
pub use event::{EVENT_CAPACITY, EventEntry};
pub use event_filter::{EventDeviceFilter, EventKindFilter, EventSearchState};
pub use navigation::AppTab;
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct App {
    started_at: Instant,
    pub devices: HashMap<usize, DeviceState>,
    pub device_order: Vec<usize>,
    pub selected_id: Option<usize>,
    pub events: VecDeque<EventEntry>,
    pub event_scroll_anchor: Option<u64>,
    pub event_scroll_offset: usize,
    pub event_kind_filter: EventKindFilter,
    pub event_device_filter: EventDeviceFilter,
    pub event_search: String,
    pub event_search_state: EventSearchState,
    pub evicted_event_count: u64,
    pub device_selector_visible: bool,
    pub help_visible: bool,
    pub active_tab: AppTab,
    pub should_quit: bool,
    pub status: String,
    pub drift_test: DriftTest,
    pub range_test: RangeTest,
    pub control_checklist: ControlChecklist,
    next_event_sequence: u64,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            devices: HashMap::new(),
            device_order: Vec::new(),
            selected_id: None,
            events: VecDeque::with_capacity(EVENT_CAPACITY),
            event_scroll_anchor: None,
            event_scroll_offset: 0,
            event_kind_filter: EventKindFilter::default(),
            event_device_filter: EventDeviceFilter::default(),
            event_search: String::new(),
            event_search_state: EventSearchState::default(),
            evicted_event_count: 0,
            device_selector_visible: false,
            help_visible: false,
            active_tab: AppTab::default(),
            should_quit: false,
            status: "gilrs backend ready".to_owned(),
            drift_test: DriftTest::default(),
            range_test: RangeTest::default(),
            control_checklist: ControlChecklist::default(),
            next_event_sequence: FIRST_EVENT_SEQUENCE,
        }
    }

    #[must_use]
    #[inline]
    pub fn selected_device(&self) -> Option<(usize, &DeviceState)> {
        let id = self.selected_id?;
        self.devices.get(&id).map(|device| (id, device))
    }

    #[must_use]
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }
}

#[cfg(test)]
fn metadata(name: &str) -> DeviceMetadata {
    DeviceMetadata {
        name: name.to_owned(),
        vendor_id: None,
        product_id: None,
        uuid: "00".repeat(16),
        mapping: "none".to_owned(),
        power: "Unknown".to_owned(),
        rumble_supported: false,
    }
}
