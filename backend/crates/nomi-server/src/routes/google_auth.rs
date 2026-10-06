//! Sign in with Google (see `crate::google_sign_in`).

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use nomi_auth::google::{self as google_sign_in, Finished, GoogleSignInConfig, Purpose, SignInError, SignInMethods};
use nomi_agent_workspace::connection::GoogleConfig;
use nomi_auth::extractor::AuthClaims;

fn not_configured() -> (StatusCode, String) {
    (StatusCode::SERVICE_UNAVAILABLE, "Google sign-in isn't set up on this server yet.".to_string())
}

/// Signing in uses the server's Google OAuth client, the same one the Workspace agent connects with.
fn sign_in_config() -> Option<GoogleSignInConfig> {
    GoogleConfig::from_env().map(|c| GoogleSignInConfig {
        client_id: c.client_id,
        client_secret: c.client_secret,
        redirect_uri: c.signin_redirect_uri,
        auth_url: c.auth_url,
        token_url: c.token_url,
        userinfo_url: c.userinfo_url,
    })
}

fn sign_in_error(e: SignInError) -> (StatusCode, String) {
    let status = match e {
        SignInError::Other(ref detail) => {
            tracing::error!(error = %detail, "google sign-in failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Google sign-in failed. Try again.".to_string());
        }
        SignInError::LinkedElsewhere => StatusCode::CONFLICT,
        _ => StatusCode::BAD_REQUEST,
    };
    (status, e.to_string())
}

#[derive(Deserialize, Default)]
pub struct StartRequest {
    pub invite_code: Option<String>,
}

#[derive(Serialize)]
pub struct StartResponse {
    pub url: String,
}

/// Public: the Google page for signing in or signing up.
pub async fn start(State(state): State<AppState>, body: Option<Json<StartRequest>>) -> Result<Json<StartResponse>, (StatusCode, String)> {
    let config = sign_in_config().ok_or_else(not_configured)?;
    let invite_code = body.and_then(|Json(b)| b.invite_code);
    let url = google_sign_in::start(&state.pool, &config, Purpose::SignIn { invite_code }).await.map_err(sign_in_error)?;
    Ok(Json(StartResponse { url }))
}

/// Signed in: the Google page for adding Google to this account.
pub async fn start_link(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<StartResponse>, (StatusCode, String)> {
    let config = sign_in_config().ok_or_else(not_configured)?;
    let url = google_sign_in::start(&state.pool, &config, Purpose::Link { user_id: claims.sub }).await.map_err(sign_in_error)?;
    Ok(Json(StartResponse { url }))
}

#[derive(Deserialize)]
pub struct CallbackRequest {
    pub code: String,
    pub state: String,
}

#[derive(Serialize)]
pub struct CallbackResponse {
    pub email: String,
    /// The account was created by this sign-in.
    pub is_new: bool,
    /// Google was added to an account that was already signed in (no new tokens).
    pub linked: bool,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

pub async fn callback(State(state): State<AppState>, Json(req): Json<CallbackRequest>) -> Result<Json<CallbackResponse>, (StatusCode, String)> {
    let config = sign_in_config().ok_or_else(not_configured)?;
    let finished = google_sign_in::finish(&state.pool, &state.http_client, &config, &req.code, &req.state).await.map_err(sign_in_error)?;
    match finished {
        Finished::Linked { email, .. } => Ok(Json(CallbackResponse { email, is_new: false, linked: true, access_token: None, refresh_token: None })),
        Finished::SignedIn { user_id, email, is_new } => {
            let failed = |_| (StatusCode::INTERNAL_SERVER_ERROR, "Signed in, but couldn't start your session. Try again.".to_string());
            let access_token = nomi_auth::login::issue_access_token(&state.pool, user_id, &state.jwt_secret).await.map_err(failed)?;
            let refresh_token = nomi_auth::refresh_token::issue_refresh_token(&state.pool, user_id)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Signed in, but couldn't start your session. Try again.".to_string()))?;
            Ok(Json(CallbackResponse { email, is_new, linked: false, access_token: Some(access_token), refresh_token: Some(refresh_token) }))
        }
    }
}

#[derive(Serialize)]
pub struct MethodsResponse {
    pub configured: bool,
    #[serde(flatten)]
    pub methods: SignInMethods,
}

pub async fn get_methods(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<MethodsResponse>, (StatusCode, String)> {
    let methods = google_sign_in::methods(&state.pool, claims.sub)
        .await
        .map_err(|e| sign_in_error(SignInError::Other(e.to_string())))?;
    Ok(Json(MethodsResponse { configured: GoogleConfig::from_env().is_some(), methods }))
}

pub async fn unlink(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<StatusCode, (StatusCode, String)> {
    if google_sign_in::unlink(&state.pool, claims.sub).await.map_err(sign_in_error)? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::CONFLICT, "Google is the only way you sign in. Set a password first.".to_string()))
    }
}

#[derive(Serialize)]
pub struct AvailableResponse {
    pub available: bool,
    /// The exact callback addresses sent to Google. Each must be listed, character for
    /// character, under the OAuth client's "Authorized redirect URIs" (else Google answers
    /// redirect_uri_mismatch). Not secret: Google's sign-in page shows them anyway.
    pub sign_in_redirect_uri: Option<String>,
    pub workspace_redirect_uri: Option<String>,
}

/// Public: whether the sign-in page should offer "Continue with Google", and which callback
/// addresses this server sends to Google.
pub async fn available() -> Json<AvailableResponse> {
    let config = GoogleConfig::from_env();
    Json(AvailableResponse {
        available: config.is_some(),
        sign_in_redirect_uri: config.as_ref().map(|c| c.signin_redirect_uri.clone()),
        workspace_redirect_uri: config.map(|c| c.redirect_uri),
    })
}
