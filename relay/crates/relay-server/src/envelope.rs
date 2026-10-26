//! Every answer is `{ "data": …, "error": null }` or `{ "data": null, "error": { "code",
//! "message", "details" } }`, the same envelope Shipyard uses, which `nomi-relay-client` reads.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use relay_control::RelayError;
use serde::Serialize;
use serde_json::{json, Value};

pub struct Answer(pub Value);

pub fn ok<T: Serialize>(data: T) -> Answer {
    Answer(serde_json::to_value(data).unwrap_or(Value::Null))
}

impl IntoResponse for Answer {
    fn into_response(self) -> Response {
        Json(json!({ "data": self.0, "error": null })).into_response()
    }
}

pub struct Failure {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub details: Value,
}

impl Failure {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self { status, code, message: message.into(), details: Value::Null }
    }
}

impl From<RelayError> for Failure {
    fn from(e: RelayError) -> Self {
        match e {
            RelayError::NotFound(message) => Failure::new(StatusCode::NOT_FOUND, "NOT_FOUND", message),
            RelayError::Rejected { code, message, details } => {
                let status = match code {
                    "CONFLICT" => StatusCode::CONFLICT,
                    "BAD_REQUEST" => StatusCode::BAD_REQUEST,
                    relay_control::NOT_CONFIGURED => StatusCode::SERVICE_UNAVAILABLE,
                    _ => StatusCode::UNPROCESSABLE_ENTITY,
                };
                Failure { status, code, message, details }
            }
            RelayError::Internal(message) => {
                tracing::error!(%message, "control-plane request failed");
                Failure::new(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", "Relay couldn't do that right now")
            }
        }
    }
}

impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        let body = json!({ "data": null, "error": { "code": self.code, "message": self.message, "details": self.details } });
        (self.status, Json(body)).into_response()
    }
}
