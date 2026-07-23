mod expected;

use self::expected::EXPECTED_CONTROLS;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize)]
pub enum ChecklistStatus {
    #[default]
    Pending,
    Observed,
    Skipped,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ChecklistItem {
    pub key: String,
    pub label: String,
    pub status: ChecklistStatus,
    pub activation_count: u64,
    pub unexpected: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ControlChecklist {
    device_id: Option<usize>,
    items: Vec<ChecklistItem>,
    selected: usize,
    active: bool,
}

impl ControlChecklist {
    #[must_use]
    pub const fn device_id(&self) -> Option<usize> {
        self.device_id
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    #[must_use]
    pub const fn selected(&self) -> usize {
        self.selected
    }

    #[must_use]
    pub fn items(&self) -> &[ChecklistItem] {
        &self.items
    }

    pub fn start(&mut self, device_id: usize, observed: impl IntoIterator<Item = String>) {
        self.device_id = Some(device_id);
        self.items = EXPECTED_CONTROLS
            .iter()
            .map(|(key, label)| ChecklistItem {
                key: (*key).to_owned(),
                label: (*label).to_owned(),
                status: ChecklistStatus::Pending,
                activation_count: 0,
                unexpected: false,
            })
            .collect();
        for key in observed {
            self.ensure_item(&key, true);
        }
        self.selected = 0;
        self.active = true;
    }

    pub fn observe(&mut self, device_id: usize, key: &str) {
        if !self.active || self.device_id != Some(device_id) {
            return;
        }
        let index = self.ensure_item(key, true);
        let item = &mut self.items[index];
        item.activation_count += 1;
        item.status = ChecklistStatus::Observed;
    }

    pub const fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub const fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = if self.selected == 0 {
            self.items.len() - 1
        } else {
            self.selected - 1
        };
    }

    pub fn skip_selected(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected)
            && item.status == ChecklistStatus::Pending
        {
            item.status = ChecklistStatus::Skipped;
        }
    }

    pub const fn finish(&mut self) {
        self.active = false;
    }

    pub const fn cancel(&mut self) {
        self.device_id = None;
        self.active = false;
    }

    #[must_use]
    pub fn counts(&self) -> (usize, usize, usize) {
        self.items
            .iter()
            .fold((0, 0, 0), |(observed, pending, skipped), item| {
                match item.status {
                    ChecklistStatus::Observed => (observed + 1, pending, skipped),
                    ChecklistStatus::Pending => (observed, pending + 1, skipped),
                    ChecklistStatus::Skipped => (observed, pending, skipped + 1),
                }
            })
    }

    fn ensure_item(&mut self, key: &str, unexpected: bool) -> usize {
        if let Some(index) = self.items.iter().position(|item| item.key == key) {
            return index;
        }
        self.items.push(ChecklistItem {
            key: key.to_owned(),
            label: display_label(key),
            status: ChecklistStatus::Pending,
            activation_count: 0,
            unexpected,
        });
        self.items.len() - 1
    }
}

fn display_label(key: &str) -> String {
    key.split_once(':')
        .map_or(key, |(_, control)| control)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_activation_updates_one_item() {
        let mut checklist = ControlChecklist::default();
        checklist.start(1, []);

        checklist.observe(1, "button:South");
        checklist.observe(1, "button:South");

        let south = &checklist.items()[0];
        assert_eq!(south.status, ChecklistStatus::Observed);
        assert_eq!(south.activation_count, 2);
        assert_eq!(
            checklist
                .items()
                .iter()
                .filter(|item| item.key == "button:South")
                .count(),
            1
        );
    }

    #[test]
    fn unexpected_control_is_retained() {
        let mut checklist = ControlChecklist::default();
        checklist.start(1, []);

        checklist.observe(1, "button:Unknown");

        assert!(
            checklist
                .items()
                .iter()
                .any(|item| item.key == "button:Unknown" && item.unexpected)
        );
    }
}
