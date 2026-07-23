use super::{App, DeviceMetadata, DeviceState};

impl App {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AxisState, input::apply_button_value, input::update_stick_trace, metadata};
    use claims::assert_some;
    use gilrs::{Axis, Button};

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
    fn disconnect_clears_stale_input_state() {
        let mut app = App::new();
        app.connect(4, metadata("controller"));
        let device = assert_some!(app.devices.get_mut(&4));
        apply_button_value(device, Button::LeftTrigger2, 1.0);
        device.axes.insert(Axis::LeftStickX, AxisState::new(0.75));
        update_stick_trace(device, Axis::LeftStickX);

        app.disconnect(4);

        let device = &app.devices[&4];
        assert!(device.buttons.is_empty());
        assert!(device.button_values.is_empty());
        assert!(device.axes.is_empty());
        assert!(device.left_stick_trace.points().is_empty());
    }
}
