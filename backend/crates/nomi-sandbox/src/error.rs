//! What can go wrong talking to a sandbox provider, sorted by what the caller should do next.

use std::time::Duration;

use thiserror::Error;

/// A failed sandbox call. Only the first three kinds are worth retrying.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SandboxError {
    /// The provider is throttling us; try again after the wait.
    #[error("rate limited, retry in {retry_after:?}")]
    RateLimited { retry_after: Duration },
    /// The provider has no room right now; try again after the wait.
    #[error("out of capacity, retry in {retry_after:?}")]
    OutOfCapacity { retry_after: Duration },
    /// The provider account needs billing set up before it will run anything.
    #[error("billing required")]
    BillingRequired,
    /// The snapshot we wanted to start from has expired.
    #[error("snapshot expired")]
    SnapshotExpired,
    /// The sandbox (or process) does not exist.
    #[error("not found")]
    NotFound,
    /// The provider understood the request and said no.
    #[error("refused: {0}")]
    Refused(String),
    /// The provider could not be reached or failed on its side.
    #[error("unavailable: {0}")]
    Unavailable(String),
    /// Anything else.
    #[error("{0}")]
    Other(String),
}

impl SandboxError {
    /// Whether the same call may succeed if tried again later.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. } | Self::OutOfCapacity { .. } | Self::Unavailable(_)
        )
    }

    /// How long the provider asked us to wait, when it said.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::RateLimited { retry_after } | Self::OutOfCapacity { retry_after } => {
                Some(*retry_after)
            }
            _ => None,
        }
    }
}
