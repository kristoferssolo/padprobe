use super::{App, input::stick_position};
use crate::analysis::StickSide;
use std::time::Instant;

impl App {
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

    pub fn cancel_drift_test(&mut self) {
        if self.drift_test.device_id().is_some() {
            self.drift_test.cancel();
            self.record_notice("Drift test cancelled");
        }
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
}
