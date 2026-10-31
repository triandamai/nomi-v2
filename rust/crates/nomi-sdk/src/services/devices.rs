//! This device in the person's device list (`routes/devices.rs`).

use reqwest::Method;
use serde::Serialize;

use super::Services;
use crate::device::DeviceInfo;
use crate::error::NomiError;

#[derive(Serialize)]
struct Registration<'a> {
    device_id: &'a str,
    #[serde(flatten)]
    info: &'a DeviceInfo,
}

impl Services {
    /// Adds this device to the person's list, or brings its details up to date.
    pub async fn register_device(
        &self,
        device_id: &str,
        info: &DeviceInfo,
    ) -> Result<(), NomiError> {
        self.call(
            Method::POST,
            &["devices"],
            Some(&Registration { device_id, info }),
        )
        .await
    }
}
