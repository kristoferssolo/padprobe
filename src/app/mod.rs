mod device;

pub use device::{AxisState, DeviceMetadata, DeviceState};

use gilrs::EventType;
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

pub const EVENT_CAPACITY: usize = 256;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Focus {
    #[default]
    Devices,
    LiveState,
    Events,
}

impl Focus {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Devices => Self::LiveState,
            Self::LiveState => Self::Events,
            Self::Events => Self::Devices,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Devices => Self::Events,
            Self::LiveState => Self::Devices,
            Self::Events => Self::LiveState,
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
    pub focus: Focus,
    pub event_scroll_anchor: Option<u64>,
    pub help_visible: bool,
    pub should_quit: bool,
    pub status: String,
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
            focus: Focus::default(),
            event_scroll_anchor: None,
            help_visible: false,
            should_quit: false,
            status: "gilrs backend ready".to_owned(),
            next_event_sequence: 0,
        }
    }

    pub fn connect(&mut self, id: usize, metadata: DeviceMetadata) {
        let name = metadata.name.clone();
        if let Some(device) = self.devices.get_mut(&id) {
            device.metadata = metadata;
            device.connected = true;
            device.buttons.clear();
            device.axes.clear();
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
        device
            .buttons
            .values_mut()
            .for_each(|pressed| *pressed = false);
        let name = device.metadata.name.clone();
        self.status = if self.selected_id == Some(id) {
            format!("Selected controller disconnected: {name}")
        } else {
            format!("{name} disconnected")
        };
        self.push_event(Some(id), "Disconnected".to_owned());
    }

    pub fn apply_controller_event(&mut self, id: usize, event: &EventType) {
        let Some(device) = self.devices.get_mut(&id) else {
            return;
        };

        match event {
            EventType::ButtonPressed(button, _) => {
                device.buttons.insert(*button, true);
            }
            EventType::ButtonReleased(button, _) => {
                device.buttons.insert(*button, false);
            }
            EventType::ButtonChanged(button, value, _) => {
                device.buttons.insert(*button, *value > 0.5);
            }
            EventType::AxisChanged(axis, value, _) => {
                device
                    .axes
                    .entry(*axis)
                    .and_modify(|state| state.update(*value))
                    .or_insert_with(|| AxisState::new(*value));
            }
            _ => {}
        }

        self.push_event(Some(id), format_event(event));
    }

    pub fn select_next(&mut self) {
        self.move_selection(1);
    }

    pub fn select_previous(&mut self) {
        self.move_selection(-1);
    }

    pub fn toggle_event_scroll(&mut self) {
        self.event_scroll_anchor = self
            .event_scroll_anchor
            .is_none()
            .then(|| self.events.back().map_or(0, |entry| entry.sequence));
    }

    pub fn record_notice(&mut self, description: impl Into<String>) {
        self.record_notice_for(self.selected_id, description);
    }

    pub fn record_notice_for(&mut self, device_id: Option<usize>, description: impl Into<String>) {
        let description = description.into();
        self.status.clone_from(&description);
        self.push_event(device_id, description);
    }

    #[must_use]
    pub fn selected_device(&self) -> Option<(usize, &DeviceState)> {
        let id = self.selected_id?;
        self.devices.get(&id).map(|device| (id, device))
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
            (Some(0), -1) | (None, -1) => connected.len() - 1,
            (Some(index), -1) => index - 1,
            (Some(index), 1) => (index + 1) % connected.len(),
            (None, 1) => 0,
            _ => return,
        };

        self.selected_id = Some(connected[next]);
        self.status = format!("Selected {}", self.devices[&connected[next]].metadata.name);
    }

    fn push_event(&mut self, device_id: Option<usize>, description: String) {
        if self.events.len() == EVENT_CAPACITY {
            self.events.pop_front();
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn selects_first_connected_device() {
        let mut app = App::new();
        app.connect(4, metadata("first"));
        app.connect(8, metadata("second"));

        assert_eq!(app.selected_id, Some(4));
    }

    #[test]
    fn keeps_disconnected_device_selected() {
        let mut app = App::new();
        app.connect(4, metadata("first"));
        app.connect(8, metadata("second"));

        app.disconnect(4);

        assert_eq!(app.selected_id, Some(4));
        assert!(!app.devices[&4].connected);
    }

    #[test]
    fn navigation_explicitly_leaves_disconnected_selection() {
        let mut app = App::new();
        app.connect(4, metadata("first"));
        app.connect(8, metadata("second"));
        app.disconnect(4);

        app.select_next();

        assert_eq!(app.selected_id, Some(8));
    }

    #[test]
    fn axis_updates_preserve_observed_range() {
        let mut state = AxisState::new(0.2);
        state.update(-0.7);
        state.update(0.5);

        assert_eq!(state.current, 0.5);
        assert_eq!(state.minimum, -0.7);
        assert_eq!(state.maximum, 0.5);
        assert_eq!(state.changes, 3);
    }

    #[test]
    fn event_history_evicts_oldest_entry() {
        let mut app = App::new();
        app.connect(1, metadata("controller"));

        for index in 0..EVENT_CAPACITY {
            app.push_event(Some(1), format!("event {index}"));
        }

        assert_eq!(app.events.len(), EVENT_CAPACITY);
        assert_eq!(app.events.front().unwrap().description, "event 0");
    }

    #[test]
    fn pausing_events_captures_current_sequence() {
        let mut app = App::new();
        app.connect(1, metadata("controller"));

        app.toggle_event_scroll();
        let anchor = app.event_scroll_anchor;
        app.push_event(Some(1), "later event".to_owned());

        assert!(anchor.is_some());
        assert_eq!(app.event_scroll_anchor, anchor);
        assert!(app.events.back().unwrap().sequence > anchor.unwrap());
    }
}
