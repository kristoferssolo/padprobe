use super::App;
#[cfg(test)]
use super::metadata;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum AppTab {
    #[default]
    Dashboard,
    Drift,
    Range,
    Controls,
    Timing,
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

impl App {
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

    pub fn select_next_tab(&mut self) {
        self.move_tab(1);
    }

    pub fn select_previous_tab(&mut self) {
        self.move_tab(-1);
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn device_selector_visibility_is_explicit() {
        let mut app = App::new();

        app.open_device_selector();
        assert!(app.device_selector_visible);

        app.close_device_selector();
        assert!(!app.device_selector_visible);
    }

    #[test]
    fn tab_navigation_wraps_in_both_directions() {
        let mut app = App::new();

        app.select_previous_tab();
        assert_eq!(app.active_tab, AppTab::Timing);
        app.select_next_tab();
        assert_eq!(app.active_tab, AppTab::Dashboard);
    }
}
