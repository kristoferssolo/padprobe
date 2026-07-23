use super::DiagnosticReport;
use crate::app::App;
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, SystemTimeError},
};
use thiserror::Error;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::DeviceMetadata;
    use claims::assert_ok;
    use tempfile::tempdir;

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
