//! `/functions/<name>[/…]` on a project's host: the caller is checked (a signed-in user or the
//! service key, or anyone for a function `backend.json` marks `public`), then the call goes to the
//! functions runtime with the caller's claims, which only the runtime's secret makes it trust.
//! When the runtime has lost the build (it restarted), the gateway sends it again and retries; a
//! runtime that doesn't come back within the wake hold answers `503` with `Retry-After`.

use std::time::{Duration, Instant};

use axum::body::{to_bytes, Body, Bytes};
use axum::extract::Request;
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use relay_control::functions::{self, CurrentBuild, RUNTIME_SECRET_HEADER};
use relay_core::backend::Access;
use relay_core::settings::FunctionsRuntime;
use relay_core::Env;
use relay_gateway::auth::{request_claims, Caller, Kind};
use uuid::Uuid;

use crate::gateway::{refuse, Gateway};

pub const FUNCTIONS_PREFIX: &str = "/functions/";
/// How long a call waits for the runtime to come back (or take the build again).
const WAKE_HOLD: Duration = Duration::from_secs(10);
const WAKE_RETRY: Duration = Duration::from_millis(500);
const RETRY_AFTER_SECONDS: &str = "2";
/// The runtime answers a call over its timeout itself; this is only for a runtime that hangs.
const TIMEOUT_MARGIN: Duration = Duration::from_secs(5);
/// A function's request body stops here (the runtime's own limit).
pub const MAX_FUNCTION_BODY: usize = 6 * 1024 * 1024;
const CLAIMS_HEADER: &str = "x-nomi-claims";
const TIMEOUT_HEADER: &str = "x-relay-timeout";
/// Never passed to the runtime from the caller's request: the gateway sets these itself, or
/// they belong to the connection.
const DROPPED_REQUEST_HEADERS: [&str; 13] = [
    "host", "connection", "keep-alive", "transfer-encoding", "upgrade", "te", "trailer", "proxy-authorization", "proxy-connection", "content-length",
    CLAIMS_HEADER, RUNTIME_SECRET_HEADER, TIMEOUT_HEADER,
];
const DROPPED_RESPONSE_HEADERS: [&str; 6] = ["connection", "keep-alive", "transfer-encoding", "upgrade", "te", "trailer"];

fn header_text<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok()).filter(|v| !v.is_empty())
}

/// The caller's claims, as the runtime hands them to `ctx.claims`.
fn encoded_claims(project: Uuid, caller: &Caller) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(request_claims(project, caller).to_string())
}

fn unavailable() -> Response {
    let mut response = refuse(StatusCode::SERVICE_UNAVAILABLE, "RELAY_FUNCTIONS_WAKING", "The functions are starting: try again in a moment.");
    response.headers_mut().insert(header::RETRY_AFTER, HeaderValue::from_static(RETRY_AFTER_SECONDS));
    response
}

/// What a call to a function carries.
pub(crate) struct Call {
    pub name: String,
    /// After the function's name: `/items` in `/functions/place-order/items`.
    pub rest: String,
    pub query: Option<String>,
    pub method: Method,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub caller: Caller,
}

pub async fn handle(gateway: &Gateway, slug: &str, env: Env, request: Request) -> Response {
    let Some(runtime) = runtime(gateway) else {
        return refuse(StatusCode::SERVICE_UNAVAILABLE, "RELAY_NO_FUNCTIONS", "Hosted functions aren't set up on this Relay.");
    };
    let (project, _) = match gateway.locate(slug, env).await {
        Ok(found) => found,
        Err(response) => return response,
    };
    let build = match functions::current(&gateway.relay, project, env).await {
        Ok(Some(build)) => build,
        Ok(None) => return refuse(StatusCode::NOT_FOUND, "RELAY_NO_FUNCTION", "No functions are deployed here yet."),
        Err(e) => return gateway.failed(e.to_string()),
    };
    let path = request.uri().path().strip_prefix(FUNCTIONS_PREFIX).unwrap_or_default().to_string();
    let (name, rest) = path.split_once('/').map(|(n, r)| (n.to_string(), format!("/{r}"))).unwrap_or((path.clone(), String::new()));
    let Some(settings) = build.functions.get(&name) else {
        return refuse(StatusCode::NOT_FOUND, "RELAY_NO_FUNCTION", &format!("There's no function called {name}."));
    };
    let keyless = settings.auth == Access::Public && header_text(request.headers(), "apikey").is_none();
    let caller = if keyless {
        Caller { kind: Kind::Anon, sub: None, email: None, sid: None }
    } else {
        match gateway.target(slug, env, request.headers()).await {
            Ok(target) => target.caller,
            Err(response) => return response,
        }
    };
    if settings.auth == Access::Required && caller.kind == Kind::Anon {
        return refuse(StatusCode::UNAUTHORIZED, "RELAY_SIGN_IN", "Sign in first: this function needs a signed-in user.");
    }
    let query = request.uri().query().map(str::to_string);
    let method = request.method().clone();
    let mut headers = request.headers().clone();
    let Ok(body) = to_bytes(request.into_body(), MAX_FUNCTION_BODY).await else {
        return refuse(StatusCode::PAYLOAD_TOO_LARGE, "RELAY_TOO_LARGE", "A function's request body can be up to 6 MB.");
    };
    // The key stands in for the token when there's no user, as the SDK sends it.
    if headers.get(header::AUTHORIZATION).is_none() {
        if let Some(key) = header_text(&headers, "apikey").and_then(|k| HeaderValue::from_str(&format!("Bearer {k}")).ok()) {
            headers.insert(header::AUTHORIZATION, key);
        }
    }
    invoke(gateway, &runtime, project, env, &build, Call { name, rest, query, method, headers, body, caller }).await
}

pub(crate) fn runtime(gateway: &Gateway) -> Option<FunctionsRuntime> {
    gateway.relay.live().ok().and_then(|live| live.settings.functions.clone()).filter(|r| !r.url.is_empty())
}

/// Sends one call to the runtime, sending the build again if the runtime lost it.
pub(crate) async fn invoke(gateway: &Gateway, runtime: &FunctionsRuntime, project: Uuid, env: Env, build: &CurrentBuild, call: Call) -> Response {
    let timeout = build.functions.get(&call.name).map(|s| s.timeout).unwrap_or(relay_core::backend::DEFAULT_FUNCTION_TIMEOUT);
    let url = format!("{}/run/{project}/{}/{}/{}{}{}", runtime.url.trim_end_matches('/'), env.as_str(), build.version, call.name, call.rest, call.query.map(|q| format!("?{q}")).unwrap_or_default());
    let mut headers = reqwest::header::HeaderMap::new();
    for (name, value) in &call.headers {
        if !DROPPED_REQUEST_HEADERS.contains(&name.as_str()) {
            headers.append(name.clone(), value.clone());
        }
    }
    let (Ok(secret), Ok(claims)) = (HeaderValue::from_str(&runtime.secret), HeaderValue::from_str(&encoded_claims(project, &call.caller))) else {
        return gateway.failed("unsendable runtime header".into());
    };
    headers.insert(HeaderName::from_static(RUNTIME_SECRET_HEADER), secret);
    headers.insert(HeaderName::from_static(CLAIMS_HEADER), claims);
    headers.insert(HeaderName::from_static(TIMEOUT_HEADER), HeaderValue::from(timeout));

    let deadline = Instant::now() + WAKE_HOLD;
    let mut reloaded = false;
    loop {
        let sent = gateway
            .http
            .request(call.method.clone(), &url)
            .headers(headers.clone())
            .body(call.body.clone())
            .timeout(Duration::from_secs(u64::from(timeout)) + TIMEOUT_MARGIN)
            .send()
            .await;
        match sent {
            Ok(answer) if answer.status() == StatusCode::PRECONDITION_FAILED && !reloaded => {
                reloaded = true;
                let left = deadline.saturating_duration_since(Instant::now());
                match tokio::time::timeout(left, functions::reload(&gateway.relay, project, env, build.version)).await {
                    Ok(Ok(())) => continue,
                    Ok(Err(e)) => return gateway.failed(format!("reloading functions: {e}")),
                    Err(_) => return unavailable(),
                }
            }
            Ok(answer) => return passed_back(answer).await,
            Err(e) if (e.is_connect() || e.is_request()) && Instant::now() + WAKE_RETRY < deadline => {
                tokio::time::sleep(WAKE_RETRY).await;
            }
            Err(e) if e.is_timeout() => return refuse(StatusCode::GATEWAY_TIMEOUT, "FUNCTION_TIMEOUT", &format!("{} didn't answer in time.", call.name)),
            Err(e) => {
                tracing::warn!(error = %e, %project, "functions runtime unreachable");
                return unavailable();
            }
        }
    }
}

async fn passed_back(answer: reqwest::Response) -> Response {
    let status = answer.status();
    let mut headers = HeaderMap::new();
    for (name, value) in answer.headers() {
        if !DROPPED_RESPONSE_HEADERS.contains(&name.as_str()) {
            headers.append(name.clone(), value.clone());
        }
    }
    let mut response = Response::new(Body::from_stream(answer.bytes_stream()));
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response.into_response()
}
