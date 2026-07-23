use super::App;
use std::time::Duration;

pub const EVENT_CAPACITY: usize = 256;
pub(super) const FIRST_EVENT_SEQUENCE: u64 = 1;

#[derive(Debug, Clone)]
pub struct EventEntry {
    pub sequence: u64,
    pub elapsed: Duration,
    pub device_id: Option<usize>,
    pub description: String,
}

impl App {
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

    pub fn record_notice(&mut self, description: impl Into<String>) {
        self.record_notice_for(self.selected_id, description);
    }

    pub fn record_notice_for(&mut self, device_id: Option<usize>, description: impl Into<String>) {
        let description = description.into();
        self.status.clone_from(&description);
        self.push_event(device_id, description);
    }

    pub(super) fn push_event(&mut self, device_id: Option<usize>, description: String) {
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
