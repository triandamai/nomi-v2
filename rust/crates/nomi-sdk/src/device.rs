//! This device: the id the app makes the first time it runs and keeps from then on (in the SDK's
//! data directory), and what it says about itself in the person's device list. The id names every
//! request (`X-Device-Id`) and is the MQTT client id of the realtime connection.

use std::path::Path;

use serde::Serialize;

use crate::error::NomiError;

const DEVICE_ID_FILE: &str = "device_id";
const UUID_BYTES: usize = 16;
/// Byte 6's high nibble holds the UUID version (4: random).
const VERSION_BYTE: usize = 6;
const VERSION_4: u8 = 0x40;
/// Byte 8's top bits hold the variant (RFC 4122).
const VARIANT_BYTE: usize = 8;
const VARIANT_RFC4122: u8 = 0x80;
const LOW_NIBBLE: u8 = 0x0f;
const LOW_SIX_BITS: u8 = 0x3f;
/// Where the dashes go in a UUID's 32 hex digits.
const DASHES_AFTER: [usize; 4] = [8, 12, 16, 20];
const UUID_TEXT_LEN: usize = 36;

/// What a device tells the person's device list about itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, uniffi::Record)]
pub struct DeviceInfo {
    /// `android`, `ios`, `macos`, `windows` or `linux`.
    pub platform: String,
    /// What the person sees: "Pixel 9", "Trian's MacBook Pro".
    pub name: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub app_version: String,
}

/// The device id kept in `data_dir`, made now if this is the first run (or it was unreadable).
pub(crate) fn load_or_create(data_dir: &Path) -> Result<String, NomiError> {
    let path = data_dir.join(DEVICE_ID_FILE);
    if let Ok(saved) = std::fs::read_to_string(&path) {
        let saved = saved.trim();
        if is_uuid(saved) {
            return Ok(saved.to_owned());
        }
    }
    let id = new_uuid()?;
    std::fs::write(&path, &id).map_err(|error| NomiError::Storage {
        reason: format!("can't save the device id in `{}`: {error}", path.display()),
    })?;
    Ok(id)
}

/// A random (version 4) UUID, from the same secure source the TLS stack uses.
fn new_uuid() -> Result<String, NomiError> {
    let mut bytes = [0u8; UUID_BYTES];
    rustls::crypto::ring::default_provider()
        .secure_random
        .fill(&mut bytes)
        .map_err(|_| NomiError::Internal {
            reason: "no secure randomness for the device id".into(),
        })?;
    bytes[VERSION_BYTE] = (bytes[VERSION_BYTE] & LOW_NIBBLE) | VERSION_4;
    bytes[VARIANT_BYTE] = (bytes[VARIANT_BYTE] & LOW_SIX_BITS) | VARIANT_RFC4122;
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    let mut id = String::with_capacity(UUID_TEXT_LEN);
    for (index, digit) in hex.chars().enumerate() {
        if DASHES_AFTER.contains(&index) {
            id.push('-');
        }
        id.push(digit);
    }
    Ok(id)
}

fn is_uuid(text: &str) -> bool {
    text.len() == UUID_TEXT_LEN
        && text.chars().enumerate().all(|(index, c)| {
            if DASHES_AFTER
                .iter()
                .enumerate()
                .any(|(n, after)| index == after + n)
            {
                c == '-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_run_makes_an_id_and_later_runs_keep_it() {
        let dir = tempfile::tempdir().unwrap();
        let first = load_or_create(dir.path()).unwrap();
        assert!(is_uuid(&first), "{first}");
        assert_eq!(&first[14..15], "4", "a random UUID");
        assert_eq!(load_or_create(dir.path()).unwrap(), first);
    }

    #[test]
    fn an_unreadable_id_is_replaced() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(DEVICE_ID_FILE), "garbage").unwrap();
        assert!(is_uuid(&load_or_create(dir.path()).unwrap()));
    }

    #[test]
    fn two_devices_get_different_ids() {
        assert_ne!(new_uuid().unwrap(), new_uuid().unwrap());
    }
}
