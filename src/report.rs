use crate::{
    analysis::{ChecklistItem, DriftMetrics, RangeMetrics, TimingMetrics},
    app::{App, DeviceState},
};
use serde::Serialize;
use std::{
    fmt,
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, SystemTimeError},
};
use thiserror::Error;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub const REPORT_SCHEMA_VERSION: u32 = 1;
const RECENT_EVENT_LIMIT: usize = 50;
const WARNING_LIMIT: usize = 20;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("could not serialize the diagnostic report")]
    Serialize(#[from] serde_json::Error),
    #[error("system time is earlier than the Unix epoch")]
    InvalidSystemTime(#[from] SystemTimeError),
    #[error("could not write report file {path}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExportedReports {
    pub json: PathBuf,
    pub text: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticReport {
    pub schema_version: u32,
    pub application: ApplicationReport,
    pub device: Option<DeviceReport>,
    pub observations: ObservationReport,
    pub diagnostics: DiagnosticResults,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplicationReport {
    pub name: String,
    pub version: String,
    pub build_target: String,
    pub operating_system: String,
    pub kernel_version: Option<String>,
    pub generated_at: String,
    pub backend: String,
    pub session_duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceReport {
    pub backend_id: usize,
    pub name: String,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub uuid: String,
    pub connected: bool,
    pub mapping_source: String,
    pub power: String,
    pub rumble_supported: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ObservationReport {
    pub axes: Vec<AxisReport>,
    pub observed_buttons: Vec<String>,
    pub pressed_buttons: Vec<String>,
    pub unknown_controls: Vec<String>,
    pub warnings: Vec<String>,
    pub recent_events: Vec<ReportEvent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AxisReport {
    pub name: String,
    pub current: f32,
    pub minimum: f32,
    pub maximum: f32,
    pub changes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportEvent {
    pub elapsed_seconds: f64,
    pub device_id: Option<usize>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticResults {
    pub drift: Option<DriftMetrics>,
    pub range: Option<RangeMetrics>,
    pub control_checklist: Vec<ChecklistItem>,
    pub event_timing: Option<TimingMetrics>,
    pub rumble_test_result: Option<String>,
}

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

    /// Serializes the report as indented JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if a report field cannot be serialized.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    #[must_use]
    pub fn to_text(&self) -> String {
        let mut text = format!(
            "PadProbe diagnostic report\nSchema: {}\nGenerated: {}\nVersion: {}\nTarget: {}\nOS: {}{}\nBackend: {}\nSession duration: {:.2} s\n",
            self.schema_version,
            self.application.generated_at,
            self.application.version,
            self.application.build_target,
            self.application.operating_system,
            self.application
                .kernel_version
                .as_ref()
                .map_or_else(String::new, |kernel| format!(" ({kernel})")),
            self.application.backend,
            self.application.session_duration_seconds,
        );
        if let Some(device) = &self.device {
            append(
                &mut text,
                format_args!(
                    "\nDevice\nName: {}\nBackend ID: {}\nVID:PID: {}:{}\nUUID: {}\nConnected: {}\nMapping: {}\nPower: {}\nRumble reported: {}\n",
                    device.name,
                    device.backend_id,
                    format_optional_hex(device.vendor_id),
                    format_optional_hex(device.product_id),
                    device.uuid,
                    device.connected,
                    device.mapping_source,
                    device.power,
                    device.rumble_supported,
                ),
            );
        } else {
            text.push_str("\nDevice\nNo controller selected.\n");
        }
        text.push_str("\nObserved axes\n");
        for axis in &self.observations.axes {
            append(
                &mut text,
                format_args!(
                    "{}: current {:+.4}, range {:+.4}…{:+.4}, changes {}\n",
                    axis.name, axis.current, axis.minimum, axis.maximum, axis.changes
                ),
            );
        }
        append(
            &mut text,
            format_args!(
                "\nObserved buttons: {}\nPressed buttons: {}\nUnknown controls: {}\n",
                joined_or_none(&self.observations.observed_buttons),
                joined_or_none(&self.observations.pressed_buttons),
                joined_or_none(&self.observations.unknown_controls),
            ),
        );
        if !self.observations.warnings.is_empty() {
            text.push_str("\nWarnings\n");
            for warning in &self.observations.warnings {
                append(&mut text, format_args!("- {warning}\n"));
            }
        }
        if !self.observations.recent_events.is_empty() {
            text.push_str("\nRecent events\n");
            for event in &self.observations.recent_events {
                append(
                    &mut text,
                    format_args!("{:.3}  {}\n", event.elapsed_seconds, event.description),
                );
            }
        }
        append_diagnostics(&mut text, &self.diagnostics);
        text.push_str("\nLimitations\n");
        for limitation in &self.limitations {
            append(&mut text, format_args!("- {limitation}\n"));
        }
        text
    }
}

/// Captures and exports JSON and plain-text reports into `directory`.
///
/// Both files share a timestamped base name.
///
/// # Errors
///
/// Returns an error when the current time cannot be represented, the report
/// cannot be serialized, or either output file cannot be written.
pub fn export(app: &App, directory: &Path) -> Result<ExportedReports, ReportError> {
    let now = SystemTime::now();
    let timestamp = now.duration_since(SystemTime::UNIX_EPOCH)?.as_millis();
    let report = DiagnosticReport::capture(app, now);
    let base_name = format!("padprobe-report-{timestamp}");
    let json = directory.join(format!("{base_name}.json"));
    let text = directory.join(format!("{base_name}.txt"));
    write(&json, report.to_json()?.as_bytes())?;
    write(&text, report.to_text().as_bytes())?;
    Ok(ExportedReports { json, text })
}

fn write(path: &Path, contents: &[u8]) -> Result<(), ReportError> {
    fs::write(path, contents).map_err(|source| ReportError::Write {
        path: path.to_path_buf(),
        source,
    })
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

fn append_diagnostics(text: &mut String, diagnostics: &DiagnosticResults) {
    text.push_str("\nDiagnostic results\n");
    if let Some(drift) = &diagnostics.drift {
        append(
            text,
            format_args!(
                "Resting displacement: {} samples, mean ({:+.4}, {:+.4}), p95 radial {:.4}, max radial {:.4}, suggested deadzone {:.2}\n",
                drift.sample_count,
                drift.mean_x,
                drift.mean_y,
                drift.percentile_95_radial,
                drift.maximum_radial,
                drift.suggested_inner_deadzone,
            ),
        );
    }
    if let Some(range) = &diagnostics.range {
        append(
            text,
            format_args!(
                "Range: {} samples, angular coverage {:.1}%, circularity deviation {:.2}%\n",
                range.sample_count,
                range.angular_coverage_percent,
                range.circularity_deviation * 100.0,
            ),
        );
    }
    if !diagnostics.control_checklist.is_empty() {
        let observed = diagnostics
            .control_checklist
            .iter()
            .filter(|item| item.status == crate::analysis::ChecklistStatus::Observed)
            .count();
        append(
            text,
            format_args!(
                "Control checklist: {observed}/{} observed\n",
                diagnostics.control_checklist.len()
            ),
        );
    }
    if let Some(timing) = &diagnostics.event_timing {
        append(
            text,
            format_args!(
                "Observed event timing: {:.1} events/s, median {:.3} ms, p95 {:.3} ms\n",
                timing.events_per_second,
                timing.median_interval_ms,
                timing.percentile_95_interval_ms,
            ),
        );
    }
    if let Some(rumble) = &diagnostics.rumble_test_result {
        append(text, format_args!("Rumble: {rumble}\n"));
    }
}

fn append(text: &mut String, arguments: fmt::Arguments<'_>) {
    if text.write_fmt(arguments).is_err() {
        unreachable!("writing formatted data to a String cannot fail");
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

fn format_optional_hex(value: Option<u16>) -> String {
    value.map_or_else(|| "unknown".to_owned(), |value| format!("{value:04x}"))
}

fn joined_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_owned()
    } else {
        values.join(", ")
    }
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
    use crate::app::DeviceMetadata;
    use claims::assert_ok;
    use tempfile::tempdir;

    fn report() -> DiagnosticReport {
        let mut app = App::new();
        app.connect(
            3,
            DeviceMetadata {
                name: "Fixture".to_owned(),
                vendor_id: Some(0x1234),
                product_id: Some(0x5678),
                uuid: "fixture".to_owned(),
                mapping: "SDL mappings".to_owned(),
                power: "Wired".to_owned(),
                rumble_supported: true,
            },
        );
        DiagnosticReport::capture(&app, SystemTime::UNIX_EPOCH)
    }

    #[test]
    fn json_report_has_versioned_schema_and_device() {
        let report = report();
        let json = assert_ok!(report.to_json());

        assert!(json.contains("\"schema_version\": 1"));
        assert!(json.contains("\"name\": \"Fixture\""));
        assert!(json.contains("\"backend\": \"gilrs\""));
    }

    #[test]
    fn text_report_states_measurement_limits() {
        let text = report().to_text();

        assert!(text.contains("PadProbe diagnostic report"));
        assert!(text.contains("Observed event timing is not an exact hardware polling rate"));
    }

    #[test]
    fn text_report_includes_recent_warnings_and_events() {
        let mut report = report();
        report.observations.warnings = vec!["Controller disconnected".to_owned()];
        report.observations.recent_events = vec![ReportEvent {
            elapsed_seconds: 1.25,
            device_id: Some(3),
            description: "ButtonPressed(South)".to_owned(),
        }];

        let text = report.to_text();

        assert!(text.contains("Warnings\n- Controller disconnected"));
        assert!(text.contains("Recent events\n1.250  ButtonPressed(South)"));
    }

    #[test]
    fn export_writes_json_and_text_reports() {
        let directory = assert_ok!(tempdir());
        let mut app = App::new();
        app.connect(
            3,
            DeviceMetadata {
                name: "Fixture".to_owned(),
                vendor_id: None,
                product_id: None,
                uuid: "fixture".to_owned(),
                mapping: "driver".to_owned(),
                power: "Unknown".to_owned(),
                rumble_supported: false,
            },
        );

        let exported = assert_ok!(export(&app, directory.path()));

        assert!(exported.json.is_file());
        assert!(exported.text.is_file());
    }
}
