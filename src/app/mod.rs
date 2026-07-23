mod device;
mod diagnostics;
mod event;
mod event_filter;
mod input;
mod navigation;
#[cfg(test)]
mod tests;
use crate::analysis::{ControlChecklist, DriftTest, RangeTest};
pub use device::{AxisState, DeviceMetadata, DeviceState, StickTrace};
use event::FIRST_EVENT_SEQUENCE;
pub use event::{EVENT_CAPACITY, EventEntry};
pub use event_filter::{EventDeviceFilter, EventKindFilter, EventSearchState};
#[cfg(test)]
use gilrs::Axis;
#[cfg(test)]
use input::{apply_button_value, update_stick_trace};
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

    pub fn connect(&mut self, id: usize, metadata: DeviceMetadata) {
        let name = metadata.name.clone();
        if let Some(device) = self.devices.get_mut(&id) {
            device.metadata = metadata;
            device.connected = true;
            device.clear_input_state();
        } else {
            self.devices.insert(id, DeviceState::new(metadata));
            self.device_order.push(id);
        }

        if self.selected_id.is_none() {
            self.selected_id = Some(id);
        }

        self.status = format!("{name} connected");
        self.push_event(Some(id), "Connected".to_owned());
    }

    pub fn disconnect(&mut self, id: usize) {
        let Some(device) = self.devices.get_mut(&id) else {
            return;
        };

        device.connected = false;
        device.clear_input_state();
        let name = device.metadata.name.clone();
        self.status = if self.selected_id == Some(id) {
            format!("Selected controller disconnected: {name}")
        } else {
            format!("{name} disconnected")
        };
        self.push_event(Some(id), "Disconnected".to_owned());
        if self.drift_test.device_id() == Some(id) {
            self.drift_test.cancel();
            self.record_notice_for(Some(id), "Drift test cancelled: controller disconnected");
        }
        if self.range_test.device_id() == Some(id) {
            self.range_test.cancel();
            self.record_notice_for(Some(id), "Range test cancelled: controller disconnected");
        }
        if self.control_checklist.device_id() == Some(id) {
            self.control_checklist.finish();
            self.record_notice_for(
                Some(id),
                "Control checklist interrupted: controller disconnected",
            );
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
