mod device;
#[cfg(test)]
mod tests;
use crate::analysis::{
    ControlChecklist, DriftTest, RangeTest, StickSide, TimingMetrics, TimingSample,
};
pub use device::{AxisState, DeviceMetadata, DeviceState, StickTrace};
use gilrs::{Axis, EventType};
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

pub const EVENT_CAPACITY: usize = 256;
const FIRST_EVENT_SEQUENCE: u64 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AppTab {
    #[default]
    Dashboard,
    Drift,
    Range,
    Controls,
    Timing,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EventKindFilter {
    #[default]
    All,
    Buttons,
    Axes,
    System,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EventDeviceFilter {
    #[default]
    All,
    Selected,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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

impl AppTab {
    const ALL: [Self; 5] = [
        Self::Dashboard,
        Self::Drift,
        Self::Range,
        Self::Controls,
        Self::Timing,
    ];

    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Drift => "Drift",
            Self::Range => "Range",
            Self::Controls => "Controls",
            Self::Timing => "Timing",
        }
    }
}

#[derive(Clone, Debug)]
pub struct EventEntry {
    pub sequence: u64,
    pub elapsed: Duration,
    pub device_id: Option<usize>,
    pub description: String,
}

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

    pub fn apply_controller_event(&mut self, id: usize, event: &EventType) {
        let Some(device) = self.devices.get_mut(&id) else {
            return;
        };

        match event {
            EventType::ButtonPressed(button, _) => {
                apply_button_value(device, *button, 1.0);
            }
            EventType::ButtonReleased(button, _) => {
                apply_button_value(device, *button, 0.0);
            }
            EventType::ButtonChanged(button, value, _) => {
                apply_button_value(device, *button, *value);
            }
            EventType::AxisChanged(axis, value, _) => {
                device
                    .axes
                    .entry(*axis)
                    .and_modify(|state| state.update(*value))
                    .or_insert_with(|| AxisState::new(*value));
                update_stick_trace(device, *axis);
            }
            _ => {}
        }

        if let EventType::AxisChanged(axis, _, _) = event
            && axis_matches_stick(*axis, self.range_test.side())
        {
            let position = stick_position(device, self.range_test.side());
            self.range_test.record(id, position);
        }
        if let Some(key) = checklist_event_key(event) {
            self.control_checklist.observe(id, &key);
        }

        self.push_event(Some(id), format_event(event));
    }

    #[inline]
    pub fn select_next(&mut self) {
        self.move_selection(1);
    }

    #[inline]
    pub fn select_previous(&mut self) {
        self.move_selection(-1);
    }

    #[inline]
    pub const fn open_device_selector(&mut self) {
        self.device_selector_visible = true;
    }

    #[inline]
    pub const fn close_device_selector(&mut self) {
        self.device_selector_visible = false;
    }

    pub fn toggle_event_scroll(&mut self) {
        self.event_scroll_anchor = self
            .event_scroll_anchor
            .is_none()
            .then(|| self.events.back().map_or(0, |entry| entry.sequence));
        self.event_scroll_offset = 0;
    }

    pub fn scroll_events_older(&mut self) {
        if self.event_scroll_anchor.is_none() {
            self.event_scroll_anchor = Some(self.events.back().map_or(0, |entry| entry.sequence));
        }
        self.event_scroll_offset = self
            .event_scroll_offset
            .saturating_add(1)
            .min(self.events.len().saturating_sub(1));
    }

    pub const fn scroll_events_newer(&mut self) {
        self.event_scroll_offset = self.event_scroll_offset.saturating_sub(1);
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
        self.event_scroll_anchor = None;
        self.event_scroll_offset = 0;
        self.evicted_event_count = 0;
        "Event history cleared".clone_into(&mut self.status);
    }

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

    pub fn record_notice(&mut self, description: impl Into<String>) {
        self.record_notice_for(self.selected_id, description);
    }

    pub fn record_notice_for(&mut self, device_id: Option<usize>, description: impl Into<String>) {
        let description = description.into();
        self.status.clone_from(&description);
        self.push_event(device_id, description);
    }

    pub fn reset_selected_observations(&mut self) {
        let Some(id) = self.selected_id else {
            self.record_notice("No controller selected");
            return;
        };
        let Some(device) = self.devices.get_mut(&id) else {
            self.record_notice("Selected controller is unavailable");
            return;
        };

        device.reset_observations();
        self.record_notice_for(Some(id), "Session observations reset");
    }

    pub fn select_next_tab(&mut self) {
        self.move_tab(1);
    }

    pub fn select_previous_tab(&mut self) {
        self.move_tab(-1);
    }

    pub fn select_drift_stick(&mut self, side: StickSide) {
        self.drift_test.select_side(side);
        self.status = format!("{} stick selected for drift test", side.label());
    }

    pub fn start_drift_test(&mut self, now: Instant) {
        let Some((id, device)) = self.selected_device() else {
            self.record_notice("Drift test unavailable: no controller selected");
            return;
        };
        if !device.connected {
            self.record_notice_for(Some(id), "Drift test unavailable: controller disconnected");
            return;
        }
        self.drift_test.start(id, now);
        self.record_notice_for(
            Some(id),
            format!(
                "{} stick drift test starting — release the stick",
                self.drift_test.side().label()
            ),
        );
    }

    pub fn select_range_stick(&mut self, side: StickSide) {
        self.range_test.select_side(side);
        self.status = format!("{} stick selected for range test", side.label());
    }

    pub fn toggle_range_test(&mut self) {
        if self.range_test.is_recording() {
            let completed = self.range_test.finish();
            self.record_notice(if completed {
                "Stick range test completed"
            } else {
                "Range test needs movement around the outer edge"
            });
            return;
        }
        let Some((id, device)) = self.selected_device() else {
            self.record_notice("Range test unavailable: no controller selected");
            return;
        };
        if !device.connected {
            self.record_notice_for(Some(id), "Range test unavailable: controller disconnected");
            return;
        }
        self.range_test.start(id);
        self.record_notice_for(
            Some(id),
            format!(
                "Recording {} stick range — trace the outer edge, then press s",
                self.range_test.side().label()
            ),
        );
    }

    pub fn cancel_range_test(&mut self) {
        if self.range_test.device_id().is_some() {
            self.range_test.cancel();
            self.record_notice("Range test cancelled");
        }
    }

    pub fn start_control_checklist(&mut self) {
        let Some((id, device)) = self.selected_device() else {
            self.record_notice("Control checklist unavailable: no controller selected");
            return;
        };
        if !device.connected {
            self.record_notice_for(
                Some(id),
                "Control checklist unavailable: controller disconnected",
            );
            return;
        }
        let observed = device
            .buttons
            .keys()
            .map(|button| format!("button:{button:?}"))
            .chain(device.axes.keys().map(|axis| format!("axis:{axis:?}")))
            .collect::<Vec<_>>();
        self.control_checklist.start(id, observed);
        self.record_notice_for(Some(id), "Control checklist started");
    }

    pub fn finish_control_checklist(&mut self) {
        if self.control_checklist.is_active() {
            self.control_checklist.finish();
            self.record_notice("Control checklist finished");
        }
    }

    pub fn cancel_drift_test(&mut self) {
        if self.drift_test.device_id().is_some() {
            self.drift_test.cancel();
            self.record_notice("Drift test cancelled");
        }
    }

    pub fn update_diagnostics(&mut self, now: Instant) {
        let side = self.drift_test.side();
        let position = self.drift_test.device_id().and_then(|id| {
            self.devices
                .get(&id)
                .filter(|device| device.connected)
                .map(|device| stick_position(device, side))
        });
        if self.drift_test.tick(now, position) {
            self.record_notice("Drift test completed");
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

    fn move_selection(&mut self, direction: isize) {
        let connected = self
            .device_order
            .iter()
            .copied()
            .filter(|id| self.devices.get(id).is_some_and(|device| device.connected))
            .collect::<Vec<_>>();

        if connected.is_empty() {
            return;
        }

        let current = self
            .selected_id
            .and_then(|id| connected.iter().position(|candidate| *candidate == id));
        let next = match (current, direction) {
            (Some(0) | None, -1) => connected.len() - 1,
            (Some(index), -1) => index - 1,
            (Some(index), 1) => (index + 1) % connected.len(),
            (None, 1) => 0,
            _ => return,
        };

        self.selected_id = Some(connected[next]);
        self.status = format!("Selected {}", self.devices[&connected[next]].metadata.name);
    }

    fn move_tab(&mut self, direction: isize) {
        let current = AppTab::ALL
            .iter()
            .position(|tab| *tab == self.active_tab)
            .unwrap_or_default();
        let next = match direction {
            -1 if current == 0 => AppTab::ALL.len() - 1,
            -1 => current - 1,
            1 => (current + 1) % AppTab::ALL.len(),
            _ => current,
        };
        self.active_tab = AppTab::ALL[next];
        self.status = format!("{} diagnostics", self.active_tab.title());
    }

    fn push_event(&mut self, device_id: Option<usize>, description: String) {
        if self.events.len() == EVENT_CAPACITY {
            self.events.pop_front();
            self.evicted_event_count += 1;
        }
        self.events.push_back(EventEntry {
            sequence: self.next_event_sequence,
            elapsed: self.started_at.elapsed(),
            device_id,
            description,
        });
        self.next_event_sequence += 1;
    }
}

fn stick_position(device: &DeviceState, side: StickSide) -> (f32, f32) {
    let (x_axis, y_axis) = match side {
        StickSide::Left => (Axis::LeftStickX, Axis::LeftStickY),
        StickSide::Right => (Axis::RightStickX, Axis::RightStickY),
    };
    (
        device.axes.get(&x_axis).map_or(0.0, |axis| axis.current),
        device.axes.get(&y_axis).map_or(0.0, |axis| axis.current),
    )
}

const fn axis_matches_stick(axis: Axis, side: StickSide) -> bool {
    matches!(
        (axis, side),
        (Axis::LeftStickX | Axis::LeftStickY, StickSide::Left)
            | (Axis::RightStickX | Axis::RightStickY, StickSide::Right)
    )
}

fn checklist_event_key(event: &EventType) -> Option<String> {
    match event {
        EventType::ButtonPressed(button, _) => Some(format!("button:{button:?}")),
        EventType::ButtonChanged(button, value, _) if *value > 0.5 => {
            Some(format!("button:{button:?}"))
        }
        EventType::AxisChanged(axis, value, _) if value.abs() >= 0.5 => {
            Some(format!("axis:{axis:?}"))
        }
        _ => None,
    }
}

#[inline]
fn update_stick_trace(device: &mut DeviceState, changed_axis: Axis) {
    let (trace, x_axis, y_axis) = match changed_axis {
        Axis::LeftStickX | Axis::LeftStickY => (
            &mut device.left_stick_trace,
            Axis::LeftStickX,
            Axis::LeftStickY,
        ),
        Axis::RightStickX | Axis::RightStickY => (
            &mut device.right_stick_trace,
            Axis::RightStickX,
            Axis::RightStickY,
        ),
        _ => return,
    };
    let x = device.axes.get(&x_axis).map_or(0.0, |state| state.current);
    let y = device.axes.get(&y_axis).map_or(0.0, |state| state.current);
    trace.push(x, y);
}

#[inline]
fn apply_button_value(device: &mut DeviceState, button: gilrs::Button, value: f32) {
    device.buttons.insert(button, value > 0.5);
    device.button_values.insert(button, value);
}

fn format_event(event: &EventType) -> String {
    match event {
        EventType::ButtonPressed(button, _) => format!("ButtonPressed({button:?})"),
        EventType::ButtonRepeated(button, _) => format!("ButtonRepeated({button:?})"),
        EventType::ButtonReleased(button, _) => format!("ButtonReleased({button:?})"),
        EventType::ButtonChanged(button, value, _) => {
            format!("ButtonChanged({button:?}, {value:.3})")
        }
        EventType::AxisChanged(axis, value, _) => format!("AxisChanged({axis:?}, {value:+.3})"),
        EventType::Dropped => "Dropped (backend queue overflow)".to_owned(),
        EventType::Connected => "Connected".to_owned(),
        EventType::Disconnected => "Disconnected".to_owned(),
        _ => format!("{event:?}"),
    }
}
