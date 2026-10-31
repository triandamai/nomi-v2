//! The Nomi client engine.
//!
//! Kotlin (Android, iOS, desktop) reaches this crate only through the types exported here with
//! UniFFI: [`NomiSdk`] for the lifecycle, [`EventSubscription`] for the typed event stream,
//! [`SdkConfig`] / [`SdkEvent`] / [`NomiError`] as the stable public models.

uniffi::setup_scaffolding!();

mod client;
mod config;
mod error;
mod event;
mod logging;

pub use client::{Diagnostics, NomiSdk};
pub use config::{LogLevel, SdkConfig, DEFAULT_EVENT_BUFFER_CAPACITY};
pub use error::NomiError;
pub use event::{EventSubscription, LogRecord, SdkEvent, SdkState};

/// The SDK's own version, as Cargo knows it.
#[uniffi::export]
pub fn sdk_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// A configuration with every optional setting at its default.
#[uniffi::export]
pub fn default_sdk_config(base_url: String, data_dir: String) -> SdkConfig {
    SdkConfig {
        base_url,
        data_dir,
        log_level: LogLevel::Info,
        event_buffer_capacity: DEFAULT_EVENT_BUFFER_CAPACITY,
    }
}
