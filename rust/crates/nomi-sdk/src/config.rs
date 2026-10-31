use std::path::PathBuf;

use url::Url;

use crate::error::NomiError;

/// Events a subscriber may fall behind by before it is told some were dropped.
pub const DEFAULT_EVENT_BUFFER_CAPACITY: u32 = 256;
const MIN_EVENT_BUFFER_CAPACITY: u32 = 16;
const MAX_EVENT_BUFFER_CAPACITY: u32 = 65_536;
const ALLOWED_SCHEMES: [&str; 2] = ["https", "http"];

/// How the host app sets the SDK up.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct SdkConfig {
    /// The Nomi backend, e.g. `https://api-nomi.trian.space`.
    pub base_url: String,
    /// An absolute directory the SDK owns: the app's files directory on Android, Application
    /// Support on iOS, the per-user data directory on desktop.
    pub data_dir: String,
    /// The most verbose log level forwarded as [`crate::SdkEvent::Log`].
    pub log_level: LogLevel,
    /// See [`DEFAULT_EVENT_BUFFER_CAPACITY`].
    pub event_buffer_capacity: u32,
}

/// Log verbosity, least verbose first: a level allows itself and everything before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, uniffi::Enum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub(crate) fn allows(self, level: LogLevel) -> bool {
        level <= self
    }
}

impl From<&tracing::Level> for LogLevel {
    fn from(level: &tracing::Level) -> Self {
        match *level {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::TRACE => LogLevel::Trace,
        }
    }
}

/// An [`SdkConfig`] that passed validation, in the types the engine works with.
#[derive(Debug, Clone)]
pub(crate) struct ValidatedConfig {
    pub base_url: Url,
    pub data_dir: PathBuf,
    pub log_level: LogLevel,
    pub event_buffer_capacity: usize,
}

impl SdkConfig {
    pub(crate) fn validate(&self) -> Result<ValidatedConfig, NomiError> {
        let base_url = Url::parse(self.base_url.trim())
            .map_err(|error| invalid(format!("base_url `{}`: {error}", self.base_url)))?;
        if !ALLOWED_SCHEMES.contains(&base_url.scheme()) {
            return Err(invalid(format!(
                "base_url must be http or https, got `{}`",
                base_url.scheme()
            )));
        }
        if base_url.host_str().is_none() {
            return Err(invalid(format!("base_url `{}` has no host", self.base_url)));
        }

        let data_dir = PathBuf::from(self.data_dir.trim());
        if !data_dir.is_absolute() {
            return Err(invalid(format!(
                "data_dir must be an absolute path, got `{}`",
                self.data_dir
            )));
        }

        let capacity_range = MIN_EVENT_BUFFER_CAPACITY..=MAX_EVENT_BUFFER_CAPACITY;
        if !capacity_range.contains(&self.event_buffer_capacity) {
            return Err(invalid(format!(
                "event_buffer_capacity must be within {MIN_EVENT_BUFFER_CAPACITY}..={MAX_EVENT_BUFFER_CAPACITY}, got {}",
                self.event_buffer_capacity
            )));
        }

        Ok(ValidatedConfig {
            base_url,
            data_dir,
            log_level: self.log_level,
            event_buffer_capacity: self.event_buffer_capacity as usize,
        })
    }
}

fn invalid(reason: String) -> NomiError {
    NomiError::InvalidConfig { reason }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default_sdk_config;

    const ABSOLUTE_DIR: &str = "/tmp/nomi";

    fn config(base_url: &str, data_dir: &str) -> SdkConfig {
        default_sdk_config(base_url.to_owned(), data_dir.to_owned())
    }

    #[test]
    fn accepts_a_default_config() {
        let validated = config("https://api-nomi.trian.space", ABSOLUTE_DIR)
            .validate()
            .unwrap();
        assert_eq!(validated.base_url.host_str(), Some("api-nomi.trian.space"));
        assert_eq!(
            validated.event_buffer_capacity,
            DEFAULT_EVENT_BUFFER_CAPACITY as usize
        );
    }

    #[test]
    fn accepts_plain_http_for_local_backends() {
        assert!(config("http://localhost:8080", ABSOLUTE_DIR)
            .validate()
            .is_ok());
    }

    #[test]
    fn rejects_an_unparseable_url() {
        assert!(matches!(
            config("not a url", ABSOLUTE_DIR).validate(),
            Err(NomiError::InvalidConfig { .. })
        ));
    }

    #[test]
    fn rejects_other_schemes() {
        assert!(matches!(
            config("ftp://example.com", ABSOLUTE_DIR).validate(),
            Err(NomiError::InvalidConfig { .. })
        ));
    }

    #[test]
    fn rejects_a_relative_data_dir() {
        assert!(matches!(
            config("https://example.com", "nomi").validate(),
            Err(NomiError::InvalidConfig { .. })
        ));
        assert!(matches!(
            config("https://example.com", "").validate(),
            Err(NomiError::InvalidConfig { .. })
        ));
    }

    #[test]
    fn rejects_an_event_buffer_out_of_range() {
        for capacity in [
            0,
            MIN_EVENT_BUFFER_CAPACITY - 1,
            MAX_EVENT_BUFFER_CAPACITY + 1,
        ] {
            let mut config = config("https://example.com", ABSOLUTE_DIR);
            config.event_buffer_capacity = capacity;
            assert!(
                matches!(config.validate(), Err(NomiError::InvalidConfig { .. })),
                "capacity {capacity}"
            );
        }
    }

    #[test]
    fn a_level_allows_itself_and_less_verbose_levels() {
        assert!(LogLevel::Info.allows(LogLevel::Error));
        assert!(LogLevel::Info.allows(LogLevel::Info));
        assert!(!LogLevel::Info.allows(LogLevel::Debug));
    }
}
