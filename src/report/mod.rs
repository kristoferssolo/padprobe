mod capture;
mod export;
#[cfg(test)]
mod tests;
mod text;

use crate::analysis::{ChecklistItem, DriftMetrics, RangeMetrics, TimingMetrics};
pub use export::{ExportedReports, ReportError, export};
use serde::Serialize;

pub const REPORT_SCHEMA_VERSION: u32 = 1;

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
    /// Serializes the report as indented JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if a report field cannot be serialized.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}
