mod device;
mod events;
mod input;
mod navigation;

use super::*;

fn metadata(name: &str) -> DeviceMetadata {
    DeviceMetadata {
        name: name.to_owned(),
        vendor_id: None,
        product_id: None,
        uuid: "00".repeat(16),
        mapping: "none".to_owned(),
        power: "Unknown".to_owned(),
        rumble_supported: false,
    }
}
