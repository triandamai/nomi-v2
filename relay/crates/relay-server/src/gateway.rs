//! Apps calling their backend: `https://<slug>[-dev].<domain>/rest/…`. The gateway works out the
//! project and who's calling, then hands the request to that environment's PostgREST as the
//! project's own database role, in the project's schema, whatever schema the client asked for.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{to_bytes, Body};
use axum::extract::Request;
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use relay_control::{state, Relay};
use relay_core::names::schema_for;
use relay_core::settings::Settings;
use relay_core::{Env, Tier};
use relay_gateway::auth::{internal_token, resolve, Refusal};
use relay_gateway::cors;
use serde_json::json;

/// The largest request body the gateway forwards.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;
/// How long a browser may cache a preflight answer.
const PREFLIGHT_MAX_AGE_SECONDS: u32 = 600;
const REST_PREFIX: &str = "/rest";
/// What the client may tell PostgREST about its request; everything else is dropped.
const FORWARDED_REQUEST_HEADERS: [&str; 5] = ["accept", "content-type", "prefer", "range", "range-unit"];
/// What PostgREST may tell the client.
const FORWARDED_RESPONSE_HEADERS: [&str; 4] = ["content-type", "content-range", "location", "preference-applied"];
const ALLOWED_METHODS: &str = "GET, HEAD, POST, PATCH, PUT, DELETE, OPTIONS";
const ALLOWED_HEADERS: &str = "authorization, apikey, content-type, prefer, range, range-unit, accept, x-client-info";
const EXPOSED_HEADERS: &str = "content-range, location, preference-applied";

/// Reads Relay's current settings on every request, so saving them in Nomi's admin takes effect
/// at once.
pub struct Gateway {
    pub relay: Arc<Relay>,
    pub http: reqwest::Client,
}

/// A refusal in PostgREST's own error shape, so the SDK reads it the same way.
fn refuse(status: StatusCode, code: &str, message: &str) -> Response {
    (status, Json(json!({ "message": message, "code": code, "details": null, "hint": null }))).into_response()
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
}

fn header_text<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

fn cors_headers(env: Env, origin: Option<&str>, preview_origins: &[String]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::VARY, HeaderValue::from_static("Origin"));
    let Some(origin) = origin.filter(|o| cors::allowed(env, o, preview_origins)) else { return headers };
    if let Ok(value) = HeaderValue::from_str(origin) {
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
        headers.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, HeaderValue::from_static(EXPOSED_HEADERS));
    }
    headers
}

fn refusal_response(refusal: Refusal) -> Response {
    let code = match refusal {
        Refusal::MissingKey => "RELAY_NO_KEY",
        Refusal::BadKey => "RELAY_BAD_KEY",
        Refusal::BadToken => "RELAY_BAD_TOKEN",
    };
    refuse(StatusCode::UNAUTHORIZED, code, refusal.message())
}

impl Gateway {
    pub async fn handle(&self, slug: String, env: Env, request: Request) -> Response {
        let live = match self.relay.live() {
            Ok(live) => live,
            Err(_) => return refuse(StatusCode::SERVICE_UNAVAILABLE, "RELAY_NOT_CONFIGURED", "Relay isn't set up yet."),
        };
        let origin = header_text(request.headers(), "origin").map(str::to_string);
        let mut cors = cors_headers(env, origin.as_deref(), &live.settings.preview_origins);
        let mut response = if request.method() == Method::OPTIONS {
            if cors.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN) {
                cors.insert(header::ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static(ALLOWED_METHODS));
                cors.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static(ALLOWED_HEADERS));
                cors.insert(header::ACCESS_CONTROL_MAX_AGE, HeaderValue::from(PREFLIGHT_MAX_AGE_SECONDS));
            }
            StatusCode::NO_CONTENT.into_response()
        } else {
            self.forward(&live.settings, &slug, env, request).await
        };
        response.headers_mut().extend(cors);
        response
    }

    async fn forward(&self, settings: &Settings, slug: &str, env: Env, request: Request) -> Response {
        let Some(rest) = request.uri().path().strip_prefix(REST_PREFIX).filter(|p| p.is_empty() || p.starts_with('/')) else {
            return refuse(StatusCode::NOT_FOUND, "RELAY_NOT_FOUND", "Nothing here: the data API is under /rest.");
        };
        let rest = if rest.is_empty() { "/" } else { rest };
        let project = match state::by_slug(&self.relay, slug).await {
            Ok(Some(project)) => project,
            Ok(None) => return refuse(StatusCode::NOT_FOUND, "RELAY_NO_PROJECT", "No project lives at this address."),
            Err(e) => return self.failed(e.to_string()),
        };
        let environment = match state::environment(&self.relay, project, env).await {
            Ok(Some(row)) => row,
            Ok(None) => return refuse(StatusCode::NOT_FOUND, "RELAY_NO_PROJECT", "No project lives at this address."),
            Err(e) => return self.failed(e.to_string()),
        };
        if environment.tier == Tier::Dedicated {
            return refuse(StatusCode::NOT_IMPLEMENTED, "RELAY_DEDICATED", "Dedicated backends aren't served by the gateway yet.");
        }
        let headers = request.headers();
        let bearer = header_text(headers, "authorization").and_then(|v| v.strip_prefix("Bearer "));
        let caller = match resolve(&environment.signing_secret, project, env, header_text(headers, "apikey"), bearer, now()) {
            Ok(caller) => caller,
            Err(refusal) => return refusal_response(refusal),
        };
        let too_big = header_text(headers, "content-length").and_then(|v| v.parse::<usize>().ok()).is_some_and(|n| n > MAX_BODY_BYTES);
        if too_big {
            return self.too_big();
        }
        let base = settings.postgrest_url(env);
        if base.is_empty() || settings.postgrest_jwt_secret.is_empty() {
            return self.failed(format!("no PostgREST set for {}", env.as_str()));
        }
        let query = request.uri().query().map(|q| format!("?{q}")).unwrap_or_default();
        let url = format!("{}{rest}{query}", base.trim_end_matches('/'));
        let method = request.method().clone();
        let profile = if matches!(method, Method::GET | Method::HEAD) { "accept-profile" } else { "content-profile" };

        let mut upstream = HeaderMap::new();
        for name in FORWARDED_REQUEST_HEADERS {
            for value in headers.get_all(name) {
                upstream.append(HeaderName::from_static(name), value.clone());
            }
        }
        let token = internal_token(&settings.postgrest_jwt_secret, project, &caller, now());
        let (Ok(auth), Ok(schema)) = (HeaderValue::from_str(&format!("Bearer {token}")), HeaderValue::from_str(&schema_for(project, environment.tier))) else {
            return self.failed("unsendable header".into());
        };
        upstream.insert(header::AUTHORIZATION, auth);
        upstream.insert(HeaderName::from_static(profile), schema);

        let body = match to_bytes(request.into_body(), MAX_BODY_BYTES).await {
            Ok(body) => body,
            Err(_) => return self.too_big(),
        };
        let answer = match self.http.request(method, url).headers(upstream).body(body).send().await {
            Ok(answer) => answer,
            Err(e) => {
                tracing::warn!(error = %e, %project, env = env.as_str(), "PostgREST unreachable");
                return refuse(StatusCode::BAD_GATEWAY, "RELAY_UNAVAILABLE", "The database API didn't answer: try again in a moment.");
            }
        };
        let status = answer.status();
        let mut passed = HeaderMap::new();
        for name in FORWARDED_RESPONSE_HEADERS {
            if let Some(value) = answer.headers().get(name) {
                passed.insert(HeaderName::from_static(name), value.clone());
            }
        }
        let bytes = match answer.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => return self.failed(e.to_string()),
        };
        let mut response = Response::new(Body::from(bytes));
        *response.status_mut() = status;
        *response.headers_mut() = passed;
        response
    }

    fn too_big(&self) -> Response {
        refuse(StatusCode::PAYLOAD_TOO_LARGE, "RELAY_TOO_LARGE", "The request body is over the gateway's 1 MB limit.")
    }

    fn failed(&self, message: String) -> Response {
        tracing::error!(%message, "gateway request failed");
        refuse(StatusCode::INTERNAL_SERVER_ERROR, "RELAY_INTERNAL", "Relay couldn't do that right now.")
    }
}
