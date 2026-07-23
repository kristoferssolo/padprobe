use super::*;
use crate::app::{App, DeviceMetadata};
use claims::assert_ok;
use std::time::SystemTime;
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
