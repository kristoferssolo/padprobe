use std::collections::HashMap;

use gilrs::{Axis, Button, Gamepad, MappingSource};

#[derive(Clone, Debug)]
pub struct DeviceMetadata {
    pub name: String,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub uuid: String,
    pub mapping: String,
    pub power: String,
    pub rumble_supported: bool,
}

impl DeviceMetadata {
    #[must_use]
    pub fn from_gamepad(gamepad: &Gamepad<'_>) -> Self {
        Self {
            name: gamepad.name().to_owned(),
            vendor_id: gamepad.vendor_id(),
            product_id: gamepad.product_id(),
            uuid: format_uuid(gamepad.uuid()),
            mapping: match gamepad.mapping_source() {
                MappingSource::SdlMappings => "SDL mappings",
                MappingSource::Driver => "driver",
                MappingSource::None => "none",
            }
            .to_owned(),
            power: format!("{:?}", gamepad.power_info()),
            rumble_supported: gamepad.is_ff_supported(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisState {
    pub current: f32,
    pub minimum: f32,
    pub maximum: f32,
    pub changes: u64,
}

impl AxisState {
    pub(super) fn new(value: f32) -> Self {
        Self {
            current: value,
            minimum: value,
            maximum: value,
            changes: 1,
        }
    }

    pub(super) fn update(&mut self, value: f32) {
        self.current = value;
        self.minimum = self.minimum.min(value);
        self.maximum = self.maximum.max(value);
        self.changes += 1;
    }
}

#[derive(Clone, Debug)]
pub struct DeviceState {
    pub metadata: DeviceMetadata,
    pub connected: bool,
    pub buttons: HashMap<Button, bool>,
    pub axes: HashMap<Axis, AxisState>,
}

impl DeviceState {
    pub(super) fn new(metadata: DeviceMetadata) -> Self {
        Self {
            metadata,
            connected: true,
            buttons: HashMap::new(),
            axes: HashMap::new(),
        }
    }
}

fn format_uuid(bytes: [u8; 16]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
