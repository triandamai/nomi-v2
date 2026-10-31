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
    /// An argument the caller passed can't be used, e.g. an account id with path characters.
    #[error("invalid argument: {reason}")]
    InvalidArgument { reason: String },
    /// The account store was closed (sign-out, account switch, shutdown); open the account again.
    #[error("the account store is closed")]
    AccountClosed,
    /// The backend couldn't be reached, or didn't answer in time. Worth retrying.
    #[error("network error: {reason}")]
    Network { reason: String },
    /// The backend answered with a status the SDK doesn't expect for this call.
    #[error("the server answered {status}: {reason}")]
    Http { status: u16, reason: String },
    /// The backend's answer couldn't be read.
    #[error("unexpected response: {reason}")]
    InvalidResponse { reason: String },
    /// The credentials were rejected, or the session ended (its refresh token was refused).
    #[error("unauthorized")]
    Unauthorized,
    /// The call needs a signed-in session and there is none.
    #[error("not signed in")]
    NotSignedIn,
    /// An emailed sign-in code was not accepted or resent.
    #[error("email code: {error:?}")]
    EmailCode { error: EmailCodeError },
    /// The local filesystem or database refused an operation.
    #[error("storage error: {reason}")]
    Storage { reason: String },
    /// A failure inside the SDK that the caller can't act on.
    #[error("internal error: {reason}")]
    Internal { reason: String },
}

/// Why an emailed sign-in code was not accepted or resent (`code_error` in the backend's
/// `routes/auth.rs`).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum EmailCodeError {
    /// The code is wrong; `attempts_left` more tries before the sign-in locks.
    WrongCode { attempts_left: u32 },
    /// The sign-in expired: start over.
    Expired,
    /// Too many wrong codes: start over.
    TooManyAttempts,
    /// A new code can be sent in `seconds`.
    Wait { seconds: u32 },
    /// No more codes for this sign-in.
    TooManyCodes,
}

impl From<rusqlite::Error> for NomiError {
    fn from(error: rusqlite::Error) -> Self {
        NomiError::Storage {
            reason: error.to_string(),
        }
    }
}

impl From<std::io::Error> for NomiError {
    fn from(error: std::io::Error) -> Self {
        NomiError::Storage {
            reason: error.to_string(),
        }
    }
}
