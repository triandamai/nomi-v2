/// Every failure the SDK reports across the FFI boundary.
///
/// Fields are named `reason`, never `message`: Kotlin maps this to an exception, where `message`
/// is already taken by `Throwable`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, uniffi::Error)]
pub enum NomiError {
    /// The configuration handed to [`crate::NomiSdk::new`] can't be used.
    #[error("invalid configuration: {reason}")]
    InvalidConfig { reason: String },
    /// The SDK was shut down; a new instance is needed.
    #[error("the SDK has been shut down")]
    ShutDown,
    /// The local filesystem refused an operation.
    #[error("storage error: {reason}")]
    Storage { reason: String },
    /// A failure inside the SDK that the caller can't act on.
    #[error("internal error: {reason}")]
    Internal { reason: String },
}
