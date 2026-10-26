//! `/auth/*` on a project's host: sign-in with email codes, sessions, the signed-in user, and
//! user management with the service key. Paths, bodies and answers are what `nomi-relay`'s `auth`
//! sends and reads; refusals are `{ message, code }` like the rest of the gateway.

use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use relay_auth::codes::{self, Limits};
use relay_auth::flows;
use relay_auth::oauth::{Provider, Start};
use relay_auth::config::{self, EMAIL_CODE};
use relay_auth::sessions;
use relay_auth::users::{self, Attributes};
use relay_auth::{AuthError, Scope};
use relay_core::names::public_url;
use relay_core::settings::Settings;
use relay_core::Env;
use relay_gateway::auth::Kind;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::gateway::{refuse, Gateway, Target, MAX_BODY_BYTES};

pub const AUTH_PREFIX: &str = "/auth/";
const AUTHORIZE_PREFIX: &str = "/auth/authorize/";
const CALLBACK_PREFIX: &str = "/auth/callback/";
const DEFAULT_PAGE: i64 = 1;
const DEFAULT_PER_PAGE: i64 = 50;

fn answer(error: AuthError) -> Response {
    if let AuthError::Internal(detail) = &error {
        tracing::error!(%detail, "sign-in request failed");
    }
    let status = StatusCode::from_u16(error.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    refuse(status, error.code(), &error.message())
}

fn ok<T: serde::Serialize>(value: T) -> Response {
    Json(value).into_response()
}

async fn body<T: DeserializeOwned>(request: Request) -> Result<T, AuthError> {
    let bytes = to_bytes(request.into_body(), MAX_BODY_BYTES).await.map_err(|_| AuthError::bad("BODY_TOO_LARGE", "The request body is too large."))?;
    serde_json::from_slice(&bytes).map_err(|e| AuthError::bad("BAD_BODY", format!("The request body isn't what this takes: {e}")))
}

fn query(request: &Request, name: &str) -> Option<String> {
    request.uri().query()?.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then(|| value.to_string())
    })
}

#[derive(Deserialize)]
struct CodeRequest {
    email: String,
    #[serde(default = "yes")]
    create_user: bool,
}

fn yes() -> bool {
    true
}

#[derive(Deserialize)]
struct VerifyRequest {
    email: String,
    code: String,
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[derive(Deserialize)]
struct PkceRequest {
    code: String,
    code_verifier: String,
}

/// An OAuth sign-in or link, as the SDK starts it.
#[derive(Deserialize)]
struct Begin {
    provider: String,
    #[serde(default)]
    redirect_to: String,
    #[serde(default)]
    scopes: Option<String>,
    #[serde(default)]
    code_challenge: String,
    #[serde(default)]
    code_challenge_method: String,
}

fn percent_decode(text: &str) -> String {
    url::form_urlencoded::parse(format!("x={text}").as_bytes()).next().map(|(_, v)| v.into_owned()).unwrap_or_default()
}

/// The provider's way back: Relay's own address on this host, one per provider.
fn callback_url(gateway: &Gateway, slug: &str, env: Env, provider: &str) -> Result<String, AuthError> {
    let domain = gateway.relay.live().map_err(|e| AuthError::Internal(e.to_string()))?.settings.public_domain.clone();
    Ok(format!("{}{CALLBACK_PREFIX}{provider}", public_url(&domain, slug, env)))
}

/// Checks where the person will come back to, records the flow, and answers the provider's URL.
async fn start_flow(gateway: &Gateway, settings: &Settings, scope: &Scope<'_>, slug: &str, params: Begin, link_user: Option<Uuid>) -> Result<String, AuthError> {
    let config = config::load(&gateway.relay.state, &gateway.relay.encryption_key, scope.project, scope.env).await?;
    let secrets = config::load_secrets(&gateway.relay.state, &gateway.relay.encryption_key, scope.project).await?;
    let provider = Provider::resolve(&config, &secrets, &params.provider)?;
    if !flows::redirect_allowed(&params.redirect_to, &config.redirect_urls, scope.env, &settings.preview_origins) {
        return Err(AuthError::bad("REDIRECT_NOT_ALLOWED", "That redirect_to isn't one of this app's addresses: add it to redirect_urls in backend.json."));
    }
    let (state, nonce) = flows::begin(scope, &params.provider, &params.redirect_to, &params.code_challenge, &params.code_challenge_method, link_user).await?;
    let start = Start { state, nonce, redirect_uri: callback_url(gateway, slug, scope.env, &params.provider)?, scopes: params.scopes };
    provider.authorize_url(&gateway.http, &start).await
}

/// `redirect_to` with these query parameters added.
fn back_to(redirect_to: &str, pairs: &[(&str, &str)]) -> Response {
    match url::Url::parse(redirect_to) {
        Ok(mut url) => {
            url.query_pairs_mut().extend_pairs(pairs);
            Redirect::to(url.as_str()).into_response()
        }
        Err(_) => refuse(StatusCode::BAD_REQUEST, "REDIRECT_NOT_ALLOWED", "The sign-in's redirect address isn't usable."),
    }
}

/// The provider sends the person back (GET, or Apple's form POST): finish the flow and send them
/// on to the app with a one-time code, or with the error.
async fn callback(gateway: &Gateway, settings: &Settings, slug: &str, env: Env, provider_name: &str, request: Request) -> Result<Response, Response> {
    let _ = settings;
    let (project, environment) = gateway.locate(slug, env).await?;
    let placement = gateway.relay.placement(project, env).await.map_err(|e| gateway.failed(e.to_string()))?;
    let scope = Scope { pool: &placement.pool, project, env, secret: &environment.signing_secret };
    let params: std::collections::BTreeMap<String, String> = if request.method() == Method::POST {
        let bytes = to_bytes(request.into_body(), MAX_BODY_BYTES).await.map_err(|_| answer(AuthError::bad("BODY_TOO_LARGE", "The request body is too large.")))?;
        url::form_urlencoded::parse(&bytes).into_owned().collect()
    } else {
        url::form_urlencoded::parse(request.uri().query().unwrap_or_default().as_bytes()).into_owned().collect()
    };
    let state = params.get("state").cloned().unwrap_or_default();
    let flow = flows::claim(&scope, provider_name, &state).await.map_err(answer)?;
    let fail = |error: AuthError| back_to(&flow.redirect_to, &[("error", error.code()), ("error_description", &error.message())]);
    if let Some(error) = params.get("error") {
        let _ = flows::drop_flow(&scope, &flow).await;
        let described = params.get("error_description").cloned().unwrap_or_else(|| error.clone());
        return Ok(back_to(&flow.redirect_to, &[("error", "PROVIDER_REFUSED"), ("error_description", &described)]));
    }
    let result = async {
        let config = config::load(&gateway.relay.state, &gateway.relay.encryption_key, project, env).await?;
        let secrets = config::load_secrets(&gateway.relay.state, &gateway.relay.encryption_key, project).await?;
        let provider = Provider::resolve(&config, &secrets, provider_name)?;
        let code = params.get("code").ok_or_else(|| AuthError::bad("INVALID_CODE", "The provider didn't send a code."))?;
        let profile = provider.exchange(&gateway.http, code, &callback_url(gateway, slug, env, provider_name)?, &flow.nonce).await?;
        flows::finish(&scope, &flow, &profile).await
    }
    .await;
    match result {
        Ok(code) => Ok(back_to(&flow.redirect_to, &[("code", &code)])),
        Err(error) => {
            if let AuthError::Internal(detail) = &error {
                tracing::error!(%detail, "OAuth callback failed");
            }
            let _ = flows::drop_flow(&scope, &flow).await;
            Ok(fail(error))
        }
    }
}

#[derive(Deserialize)]
struct UserChanges {
    email: Option<String>,
    user_metadata: Option<Value>,
}

pub async fn handle(gateway: &Gateway, settings: &Settings, slug: &str, env: Env, request: Request) -> Response {
    let path = request.uri().path().to_string();
    // The provider sends the browser back here with no key: the flow's state is what counts.
    if let Some(provider) = path.strip_prefix(CALLBACK_PREFIX) {
        let provider = percent_decode(provider);
        return match callback(gateway, settings, slug, env, &provider, request).await {
            Ok(response) | Err(response) => response,
        };
    }
    // A browser navigation can't send headers, so the SDK puts the key in the query.
    let mut headers = request.headers().clone();
    if path.starts_with(AUTHORIZE_PREFIX) && !headers.contains_key("apikey") {
        if let Some(key) = query(&request, "apikey").and_then(|k| HeaderValue::from_str(&percent_decode(&k)).ok()) {
            headers.insert("apikey", key);
        }
    }
    let target = match gateway.target(slug, env, &headers).await {
        Ok(target) => target,
        Err(response) => return response,
    };
    let placement = match gateway.relay.placement(target.project, env).await {
        Ok(placement) => placement,
        Err(e) => return gateway.failed(e.to_string()),
    };
    let scope = Scope { pool: &placement.pool, project: target.project, env, secret: &target.environment.signing_secret };
    match route(gateway, settings, &scope, &target, slug, request).await {
        Ok(response) => response,
        Err(error) => answer(error),
    }
}

async fn route(gateway: &Gateway, settings: &Settings, scope: &Scope<'_>, target: &Target, slug: &str, request: Request) -> Result<Response, AuthError> {
    let path = request.uri().path().trim_end_matches('/').to_string();
    let rest = path.strip_prefix(AUTH_PREFIX).unwrap_or_default();
    let method = request.method().clone();
    match (method, rest) {
        (Method::POST, "otp") => send_code(gateway, settings, scope, request).await,
        (Method::POST, "verify") => verify_code(gateway, scope, request).await,
        (Method::GET, authorize) if authorize.starts_with("authorize/") => {
            let provider = percent_decode(authorize.trim_start_matches("authorize/"));
            let params = Begin {
                provider: provider.clone(),
                redirect_to: query(&request, "redirect_to").map(|r| percent_decode(&r)).unwrap_or_default(),
                scopes: query(&request, "scopes").map(|s| percent_decode(&s)),
                code_challenge: query(&request, "code_challenge").unwrap_or_default(),
                code_challenge_method: query(&request, "code_challenge_method").unwrap_or_default(),
            };
            let url = start_flow(gateway, settings, scope, slug, params, None).await?;
            Ok(Redirect::to(&url).into_response())
        }
        (Method::POST, "identities") => {
            let (user, _) = signed_in(scope, target).await?;
            let params: Begin = body(request).await?;
            let url = start_flow(gateway, settings, scope, slug, params, Some(user.id)).await?;
            Ok(ok(json!({ "url": url })))
        }
        (Method::DELETE, identity) if identity.starts_with("identities/") => {
            let (user, _) = signed_in(scope, target).await?;
            let id = identity.trim_start_matches("identities/").parse::<Uuid>().map_err(|_| AuthError::NotFound("No such sign-in method on this user.".into()))?;
            users::unlink(scope.pool, scope.project, user.id, id).await?;
            Ok(StatusCode::NO_CONTENT.into_response())
        }
        (Method::POST, "token") => match query(&request, "grant_type").as_deref() {
            Some("refresh_token") => {
                let RefreshRequest { refresh_token } = body(request).await?;
                Ok(ok(sessions::refresh(scope, &refresh_token).await?))
            }
            Some("pkce") => {
                let config = config::load(&gateway.relay.state, &gateway.relay.encryption_key, scope.project, scope.env).await?;
                let PkceRequest { code, code_verifier } = body(request).await?;
                let user = flows::trade(scope, &code, &code_verifier).await?;
                Ok(ok(sessions::start(scope, user, config.session_ttl_days).await?))
            }
            _ => Err(AuthError::bad("UNSUPPORTED_GRANT", "This grant_type isn't supported here yet.")),
        },
        (Method::GET, "user") => {
            let (user, _) = signed_in(scope, target).await?;
            Ok(ok(user))
        }
        (Method::PATCH, "user") => {
            let (user, _) = signed_in(scope, target).await?;
            let changes: UserChanges = body(request).await?;
            if changes.email.is_some() {
                return Err(AuthError::bad("EMAIL_CHANGE_UNSUPPORTED", "Changing the email isn't supported yet: sign in with the new address instead."));
            }
            let attributes = Attributes { user_metadata: changes.user_metadata, ..Attributes::default() };
            Ok(ok(users::update(scope.pool, scope.project, user.id, attributes).await?))
        }
        (Method::POST, "logout") => {
            let (_, session) = signed_in(scope, target).await?;
            sessions::end(scope.pool, scope.project, session).await?;
            Ok(StatusCode::NO_CONTENT.into_response())
        }
        (method, admin) if admin == "admin/users" || admin.starts_with("admin/users/") => {
            if target.caller.kind != Kind::Service {
                return Ok(refuse(StatusCode::FORBIDDEN, "SERVICE_KEY_REQUIRED", "Managing users needs the service key, from server code."));
            }
            let id = admin.strip_prefix("admin/users/").map(|id| id.parse::<Uuid>().map_err(|_| AuthError::NotFound("No such user.".into()))).transpose()?;
            admin_users(scope, method, id, request).await
        }
        _ => Ok(refuse(StatusCode::NOT_FOUND, "RELAY_NOT_FOUND", "There's no such sign-in route.")),
    }
}

/// The signed-in user and their session, from a live session's access token.
async fn signed_in(scope: &Scope<'_>, target: &Target) -> Result<(users::User, Uuid), AuthError> {
    let not_signed_in = || AuthError::unauthorized("NOT_SIGNED_IN", "Sign in first: this needs a signed-in user's access token.");
    let (Kind::Authenticated, Some(user), Some(session)) = (target.caller.kind, target.caller.sub, target.caller.sid) else { return Err(not_signed_in()) };
    if !sessions::live(scope.pool, scope.project, session).await? {
        return Err(AuthError::unauthorized("SESSION_ENDED", "This session has ended: sign in again."));
    }
    let user = users::get(scope.pool, scope.project, user).await?.ok_or_else(not_signed_in)?;
    Ok((user, session))
}

async fn send_code(gateway: &Gateway, settings: &Settings, scope: &Scope<'_>, request: Request) -> Result<Response, AuthError> {
    let config = config::load(&gateway.relay.state, &gateway.relay.encryption_key, scope.project, scope.env).await?;
    if !config.allows(EMAIL_CODE) {
        return Err(AuthError::bad("METHOD_OFF", "Signing in with an email code is switched off for this app."));
    }
    let secrets = config::load_secrets(&gateway.relay.state, &gateway.relay.encryption_key, scope.project).await?;
    let (smtp, limits) = match (&config.smtp_with(&secrets), &settings.shared_smtp) {
        (Some(own), _) => (own.clone(), Limits::OWN),
        (None, Some(shared)) => (shared.clone(), Limits::SHARED),
        (None, None) => return Err(AuthError::Unavailable { code: "RELAY_NO_SENDER", message: "Relay can't send email yet: its shared sender isn't set up.".into() }),
    };
    let app_name: String = sqlx::query_scalar("SELECT name FROM relay.projects WHERE id = $1").bind(scope.project).fetch_one(&gateway.relay.state).await?;
    let CodeRequest { email, create_user } = body(request).await?;
    codes::send(scope, gateway.relay.outbox.as_ref(), &smtp, limits, &app_name, &email, create_user).await?;
    Ok(ok(Value::Null))
}

async fn verify_code(gateway: &Gateway, scope: &Scope<'_>, request: Request) -> Result<Response, AuthError> {
    let config = config::load(&gateway.relay.state, &gateway.relay.encryption_key, scope.project, scope.env).await?;
    let VerifyRequest { email, code } = body(request).await?;
    let user = codes::verify(scope, &email, &code).await?;
    Ok(ok(sessions::start(scope, user, config.session_ttl_days).await?))
}

async fn admin_users(scope: &Scope<'_>, method: Method, id: Option<Uuid>, request: Request) -> Result<Response, AuthError> {
    match (method, id) {
        (Method::GET, None) => {
            let page = query(&request, "page").and_then(|p| p.parse().ok()).unwrap_or(DEFAULT_PAGE);
            let per_page = query(&request, "per_page").and_then(|p| p.parse().ok()).unwrap_or(DEFAULT_PER_PAGE);
            let (users, total) = users::list(scope.pool, scope.project, page, per_page).await?;
            Ok(ok(json!({ "users": users, "total": total })))
        }
        (Method::POST, None) => {
            let attributes: Attributes = body(request).await?;
            if attributes.email.is_none() {
                return Err(AuthError::bad("EMAIL_REQUIRED", "A new user needs an email."));
            }
            Ok(ok(users::create(scope.pool, scope.project, attributes).await?))
        }
        (Method::GET, Some(id)) => users::get(scope.pool, scope.project, id).await?.map(ok).ok_or_else(|| AuthError::NotFound("No such user.".into())),
        (Method::PATCH, Some(id)) => {
            let attributes: Attributes = body(request).await?;
            Ok(ok(users::update(scope.pool, scope.project, id, attributes).await?))
        }
        (Method::DELETE, Some(id)) => {
            users::delete(scope.pool, scope.project, id).await?;
            Ok(StatusCode::NO_CONTENT.into_response())
        }
        _ => Ok(refuse(StatusCode::METHOD_NOT_ALLOWED, "METHOD_NOT_ALLOWED", "That isn't something you can do to users.")),
    }
}
