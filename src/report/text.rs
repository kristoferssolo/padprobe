use super::{DiagnosticReport, DiagnosticResults};
use crate::analysis::ChecklistStatus;
use std::fmt::{self, Write as _};

impl DiagnosticReport {
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
            .filter(|item| item.status == ChecklistStatus::Observed)
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
