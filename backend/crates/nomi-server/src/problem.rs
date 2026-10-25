//! One error contract for every route, for clients that ask for it.
//!
//! Routes answer errors in their own way (most with a plain sentence, some with JSON). A client
//! that sends `Accept: application/problem+json` gets every error as RFC 9457 problem details
//! instead, whatever the route wrote:
//!
//! ```json
//! {"type": "about:blank", "title": "Not Found", "status": 404, "code": "not_found",
//!  "detail": "project not found", "request_id": "…"}
//! ```
//!
//! `code` is stable to branch on; `detail` is for people. Clients that don't ask (the web app and
//! the native apps today) keep getting what they parse now.

use axum::body::{to_bytes, Body};
use axum::extract::Request;
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use serde_json::{json, Value};

pub const PROBLEM_JSON: &str = "application/problem+json";
/// The header carrying each request's id, on every response (see `app::build_router`).
pub const REQUEST_ID_HEADER: &str = "x-request-id";
/// Error bodies are short sentences; anything longer is cut, not buffered whole.
const MAX_ERROR_BODY_BYTES: usize = 64 * 1024;

/// A stable, machine-readable name for a status.
pub fn code_for(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => "bad_request",
        StatusCode::UNAUTHORIZED => "unauthorized",
        StatusCode::PAYMENT_REQUIRED => "payment_required",
        StatusCode::FORBIDDEN => "forbidden",
        StatusCode::NOT_FOUND => "not_found",
        StatusCode::METHOD_NOT_ALLOWED => "method_not_allowed",
        StatusCode::CONFLICT => "conflict",
        StatusCode::GONE => "gone",
        StatusCode::PAYLOAD_TOO_LARGE => "payload_too_large",
        StatusCode::UNSUPPORTED_MEDIA_TYPE => "unsupported_media_type",
        StatusCode::UNPROCESSABLE_ENTITY => "unprocessable",
        StatusCode::TOO_MANY_REQUESTS => "rate_limited",
        StatusCode::BAD_GATEWAY => "upstream_failed",
        StatusCode::SERVICE_UNAVAILABLE => "unavailable",
        StatusCode::GATEWAY_TIMEOUT => "upstream_timeout",
        s if s.is_server_error() => "internal",
        _ => "error",
    }
}

/// What a route's own error body says, for `detail`: the sentence it wrote, or the `message`,
/// `error` or `detail` of a JSON body.
pub fn detail_from(body: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(body).ok()?.trim();
    if text.is_empty() {
        return None;
    }
    if let Ok(parsed) = serde_json::from_str::<Value>(text) {
        for key in ["detail", "message", "error"] {
            if let Some(found) = parsed.get(key).and_then(Value::as_str) {
                return Some(found.to_string());
            }
        }
        return None;
    }
    Some(text.to_string())
}

/// The problem details for an error response.
pub fn problem(status: StatusCode, detail: Option<String>, request_id: Option<&str>) -> Value {
    let mut body = json!({
        "type": "about:blank",
        "title": status.canonical_reason().unwrap_or("Error"),
        "status": status.as_u16(),
        "code": code_for(status),
    });
    if let Some(detail) = detail {
        body["detail"] = json!(detail);
    }
    if let Some(id) = request_id {
        body["request_id"] = json!(id);
    }
    body
}

fn wants_problem_json(request: &Request) -> bool {
    request
        .headers()
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|accept| accept.contains(PROBLEM_JSON))
}

/// Rewrites error responses as problem details for clients that ask for them.
pub async fn problem_details(request: Request, next: Next) -> Response {
    let wanted = wants_problem_json(&request);
    let response = next.run(request).await;
    let status = response.status();
    if !wanted || !(status.is_client_error() || status.is_server_error()) {
        return response;
    }
    let (mut parts, body) = response.into_parts();
    if parts.headers.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()) == Some(PROBLEM_JSON) {
        return Response::from_parts(parts, body);
    }
    let bytes = to_bytes(body, MAX_ERROR_BODY_BYTES).await.unwrap_or_default();
    let request_id = parts.headers.get(REQUEST_ID_HEADER).and_then(|v| v.to_str().ok()).map(str::to_string);
    let body = problem(status, detail_from(&bytes), request_id.as_deref()).to_string();
    parts.headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(PROBLEM_JSON));
    parts.headers.remove(header::CONTENT_LENGTH);
    Response::from_parts(parts, Body::from(body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn details_come_from_text_or_json_bodies() {
        assert_eq!(detail_from(b"project not found").as_deref(), Some("project not found"));
        assert_eq!(detail_from(br#"{"error": "a publish is already running"}"#).as_deref(), Some("a publish is already running"));
        assert_eq!(detail_from(br#"{"message": "nope"}"#).as_deref(), Some("nope"));
        assert_eq!(detail_from(b"  "), None);
        assert_eq!(detail_from(br#"{"items": []}"#), None);
    }

    #[test]
    fn problems_carry_a_stable_code() {
        let body = problem(StatusCode::CONFLICT, Some("busy".into()), Some("r1"));
        assert_eq!(body, json!({"type": "about:blank", "title": "Conflict", "status": 409, "code": "conflict", "detail": "busy", "request_id": "r1"}));
        assert_eq!(code_for(StatusCode::INTERNAL_SERVER_ERROR), "internal");
    }

    #[tokio::test]
    async fn only_clients_that_ask_get_problem_details() {
        use axum::routing::get;
        use tower::ServiceExt;
        let app = axum::Router::new()
            .route("/missing", get(|| async { (StatusCode::NOT_FOUND, "project not found") }))
            .layer(axum::middleware::from_fn(problem_details));

        let plain = app.clone().oneshot(Request::get("/missing").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(to_bytes(plain.into_body(), 1024).await.unwrap(), "project not found");

        let asked = app
            .oneshot(Request::get("/missing").header(header::ACCEPT, PROBLEM_JSON).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(asked.headers()[header::CONTENT_TYPE], PROBLEM_JSON);
        let body: Value = serde_json::from_slice(&to_bytes(asked.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(body["code"], "not_found");
        assert_eq!(body["detail"], "project not found");
    }
}
