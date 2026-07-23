#[cfg(test)]
use super::report_fixture;
use super::{
    ApplicationReport, AxisReport, DeviceReport, DiagnosticReport, DiagnosticResults,
    ObservationReport, REPORT_SCHEMA_VERSION, ReportEvent,
};
use crate::app::{App, DeviceState};
use std::time::SystemTime;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const RECENT_EVENT_LIMIT: usize = 50;
const WARNING_LIMIT: usize = 20;

impl DiagnosticReport {
    #[must_use]
    pub fn capture(app: &App, generated_at: SystemTime) -> Self {
        let selected = app.selected_device();
        let selected_id = selected.map(|(id, _)| id);
        let device = selected.map(|(id, device)| DeviceReport::from_device(id, device));
        let observations = selected.map_or_else(ObservationReport::default, |(_, device)| {
            ObservationReport::capture(app, device, selected_id)
        });
        let rumble_test_result = app
            .events
            .iter()
            .rev()
            .find(|entry| entry.description.to_lowercase().contains("rumble"))
            .map(|entry| entry.description.clone());

        Self {
            schema_version: REPORT_SCHEMA_VERSION,
            application: ApplicationReport {
                name: "PadProbe".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                build_target: format!(
                    "{}-{}",
                    std::env::consts::ARCH,
                    std::env::consts::OS
                ),
                operating_system: std::env::consts::OS.to_owned(),
                kernel_version: kernel_version(),
                generated_at: format_timestamp(generated_at),
                backend: "gilrs".to_owned(),
                session_duration_seconds: app.elapsed().as_secs_f64(),
            },
            device,
            observations,
            diagnostics: DiagnosticResults {
                drift: app.drift_test.result().cloned(),
                range: app.range_test.result().cloned(),
                control_checklist: app.control_checklist.items().to_vec(),
                event_timing: app.selected_event_timing(),
                rumble_test_result,
            },
            limitations: vec![
                "Values are normalized input reported through gilrs.".to_owned(),
                "Driver mappings, configured deadzones, Steam Input, transport, and OS scheduling can affect observations.".to_owned(),
                "Observed event timing is not an exact hardware polling rate.".to_owned(),
                "Diagnostic measurements do not by themselves identify physical wear or hardware failure.".to_owned(),
            ],
        }
    }
}

impl DeviceReport {
    fn from_device(id: usize, device: &DeviceState) -> Self {
        Self {
            backend_id: id,
            name: device.metadata.name.clone(),
            vendor_id: device.metadata.vendor_id,
            product_id: device.metadata.product_id,
            uuid: device.metadata.uuid.clone(),
            connected: device.connected,
            mapping_source: device.metadata.mapping.clone(),
            power: device.metadata.power.clone(),
            rumble_supported: device.metadata.rumble_supported,
        }
    }
}

impl ObservationReport {
    fn capture(app: &App, device: &DeviceState, selected_id: Option<usize>) -> Self {
        let mut axes = device
            .axes
            .iter()
            .map(|(axis, state)| AxisReport {
                name: format!("{axis:?}"),
                current: state.current,
                minimum: state.minimum,
                maximum: state.maximum,
                changes: state.changes,
            })
            .collect::<Vec<_>>();
        axes.sort_unstable_by(|left, right| left.name.cmp(&right.name));
        let mut observed_buttons = device
            .buttons
            .keys()
            .map(|button| format!("{button:?}"))
            .collect::<Vec<_>>();
        observed_buttons.sort_unstable();
        let mut pressed_buttons = device
            .buttons
            .iter()
            .filter(|(_, pressed)| **pressed)
            .map(|(button, _)| format!("{button:?}"))
            .collect::<Vec<_>>();
        pressed_buttons.sort_unstable();
        let unknown_controls = app
            .control_checklist
            .items()
            .iter()
            .filter(|item| item.unexpected)
            .map(|item| item.label.clone())
            .collect();
        let selected_events = app
            .events
            .iter()
            .filter(|entry| entry.device_id == selected_id)
            .collect::<Vec<_>>();
        let warnings = selected_events
            .iter()
            .rev()
            .filter(|entry| is_warning(&entry.description))
            .take(WARNING_LIMIT)
            .map(|entry| entry.description.clone())
            .collect();
        let recent_events = selected_events
            .iter()
            .rev()
            .take(RECENT_EVENT_LIMIT)
            .rev()
            .map(|entry| ReportEvent {
                elapsed_seconds: entry.elapsed.as_secs_f64(),
                device_id: entry.device_id,
                description: entry.description.clone(),
            })
            .collect();

        Self {
            axes,
            observed_buttons,
            pressed_buttons,
            unknown_controls,
            warnings,
            recent_events,
        }
    }
}

fn format_timestamp(timestamp: SystemTime) -> String {
    OffsetDateTime::from(timestamp)
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".to_owned())
}

fn kernel_version() -> Option<String> {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|version| version.trim().to_owned())
        .filter(|version| !version.is_empty())
}

fn is_warning(description: &str) -> bool {
    let lowercase = description.to_lowercase();
    [
        "unavailable",
        "unsupported",
        "disconnected",
        "cancelled",
        "could not",
        "dropped",
    ]
    .iter()
    .any(|needle| lowercase.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_ok;

    #[test]
    fn json_report_has_versioned_schema_and_device() {
        let report = report_fixture();
        let json = assert_ok!(report.to_json());

        assert!(json.contains("\"schema_version\": 1"));
        assert!(json.contains("\"name\": \"Fixture\""));
        assert!(json.contains("\"backend\": \"gilrs\""));
    }
}
