//! The signed-in session: credentials, sign-in, refresh, sign-out, and authorized requests.
//!
//! Tokens live in the platform's secure store, reached through the [`CredentialStore`] the host
//! implements (Keychain, Keystore, the desktop keyring). The SDK caches them in memory and never
//! logs them.

use std::sync::{Arc, Mutex};

use base64::Engine as _;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use url::Url;

use super::http::{self, Http};
use crate::error::{EmailCodeError, NomiError};
use crate::event::{EventBus, SdkEvent};

const LOGIN_PATH: &str = "/api/auth/login";
const VERIFY_PATH: &str = "/api/auth/verify";
const RESEND_PATH: &str = "/api/auth/resend";
const REFRESH_PATH: &str = "/api/auth/refresh";
const LOGOUT_PATH: &str = "/api/auth/logout";
const PROFILE_PATH: &str = "/api/profile";
const API_ROOT: &str = "/api";
const JWT_SEGMENTS: usize = 3;

/// The session's tokens, as the host stores them.
#[derive(Clone, PartialEq, Eq, uniffi::Record)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: String,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Credentials { .. }")
    }
}

/// Secure, platform-specific persistence of the session, implemented by the host. Calls come from
/// SDK threads, so implementations must be thread-safe, and should be quick.
#[uniffi::export(callback_interface)]
pub trait CredentialStore: Send + Sync {
    fn load(&self) -> Option<Credentials>;
    fn save(&self, credentials: Credentials);
    fn clear(&self);
}

/// Who is signed in. `user_id` is also the id to open the account's local store with.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct SignedInUser {
    pub user_id: String,
}

/// Where an emailed sign-in code went (`VerificationInfo` in the backend's `routes/auth.rs`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct VerificationInfo {
    pub challenge_id: String,
    pub purpose: String,
    pub email: String,
    pub expires_in: i64,
    pub resend_in: i64,
    pub sends_left: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum SignInResult {
    SignedIn {
        user: SignedInUser,
    },
    /// The backend emailed a code; finish with `verify_email_code`.
    EmailCodeRequired {
        verification: VerificationInfo,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum SignOutReason {
    /// The host called `sign_out`.
    Requested,
    /// The backend refused the refresh token.
    SessionExpired,
    /// The backend still refused the session right after a successful refresh.
    AccessRevoked,
    /// The person signed this device out from another of their devices.
    SignedOutElsewhere,
}

/// `ProfileResponse` in the backend's `routes/profile.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct Profile {
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub email: String,
    pub avatar_url: Option<String>,
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct VerifyRequest<'a> {
    challenge_id: &'a str,
    code: &'a str,
}

#[derive(Serialize)]
struct ChallengeRequest<'a> {
    challenge_id: &'a str,
}

#[derive(Serialize)]
struct RefreshRequest<'a> {
    refresh_token: &'a str,
}

#[derive(Deserialize)]
struct TokenPair {
    access_token: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct AccessToken {
    access_token: String,
}

/// The untagged `SignInResponse`: tokens, or `{ "verification": { .. } }`.
#[derive(Deserialize)]
#[serde(untagged)]
enum SignInResponse {
    Verification { verification: VerificationInfo },
    Tokens(TokenPair),
}

#[derive(Deserialize)]
struct CodeErrorBody {
    error: String,
    attempts_left: Option<u32>,
    seconds: Option<u32>,
}

#[derive(Deserialize)]
struct JwtClaims {
    sub: String,
}

/// What a refresh attempt came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Refresh {
    /// There is a fresh access token (this call's, or a concurrent one's).
    Refreshed,
    /// The backend refused the refresh token; the session has ended.
    Rejected,
    /// There is no session to refresh.
    NotSignedIn,
}

pub(crate) struct Auth {
    http: Http,
    store: Box<dyn CredentialStore>,
    cache: Mutex<Option<Credentials>>,
    /// One refresh at a time; callers that waited reuse its result.
    refreshing: tokio::sync::Mutex<()>,
    session: watch::Sender<Option<SignedInUser>>,
    bus: Arc<EventBus>,
}

impl Auth {
    pub fn new(http: Http, store: Box<dyn CredentialStore>, bus: Arc<EventBus>) -> Self {
        let stored = store.load();
        let user = stored
            .as_ref()
            .and_then(|credentials| user_from_token(&credentials.access_token));
        if stored.is_some() && user.is_none() {
            tracing::warn!(
                "the stored session's access token is unreadable; treating it as signed out"
            );
        }
        let cache = if user.is_some() { stored } else { None };
        Self {
            http,
            store,
            cache: Mutex::new(cache),
            refreshing: tokio::sync::Mutex::new(()),
            session: watch::Sender::new(user),
            bus,
        }
    }

    pub fn credentials(&self) -> Option<Credentials> {
        self.cache.lock().expect("credentials poisoned").clone()
    }

    pub fn user(&self) -> Option<SignedInUser> {
        self.session.borrow().clone()
    }

    pub fn watch_session(&self) -> watch::Receiver<Option<SignedInUser>> {
        self.session.subscribe()
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<SignInResult, NomiError> {
        let response = self
            .post_public(LOGIN_PATH, &LoginRequest { email, password })
            .await?;
        if response.status() == StatusCode::UNAUTHORIZED {
            return Err(NomiError::Unauthorized);
        }
        match http::json::<SignInResponse>(response).await? {
            SignInResponse::Verification { verification } => {
                Ok(SignInResult::EmailCodeRequired { verification })
            }
            SignInResponse::Tokens(pair) => Ok(SignInResult::SignedIn {
                user: self.begin_session(pair)?,
            }),
        }
    }

    pub async fn verify_email_code(
        &self,
        challenge_id: &str,
        code: &str,
    ) -> Result<SignedInUser, NomiError> {
        let response = self
            .post_public(VERIFY_PATH, &VerifyRequest { challenge_id, code })
            .await?;
        let pair = http::json::<TokenPair>(code_checked(response).await?).await?;
        self.begin_session(pair)
    }

    pub async fn resend_email_code(
        &self,
        challenge_id: &str,
    ) -> Result<VerificationInfo, NomiError> {
        let response = self
            .post_public(RESEND_PATH, &ChallengeRequest { challenge_id })
            .await?;
        http::json(code_checked(response).await?).await
    }

    /// Exchanges the refresh token for a new access token. With `stale_access`, a refresh that
    /// already replaced that token meanwhile counts, and no second request is made.
    pub async fn refresh(&self, stale_access: Option<&str>) -> Result<Refresh, NomiError> {
        let _one_at_a_time = self.refreshing.lock().await;
        let Some(current) = self.credentials() else {
            return Ok(Refresh::NotSignedIn);
        };
        if stale_access.is_some_and(|stale| stale != current.access_token) {
            return Ok(Refresh::Refreshed);
        }
        let response = self
            .post_public(
                REFRESH_PATH,
                &RefreshRequest {
                    refresh_token: &current.refresh_token,
                },
            )
            .await?;
        // A sign-in or sign-out replaced the session while this request was out: its verdict is
        // about tokens nobody holds any more, so it must neither overwrite nor end anything.
        let latest = self.credentials();
        if latest.as_ref().map(|c| &c.refresh_token) != Some(&current.refresh_token) {
            return Ok(if latest.is_some() {
                Refresh::Refreshed
            } else {
                Refresh::NotSignedIn
            });
        }
        if response.status() == StatusCode::UNAUTHORIZED {
            self.end_session(SignOutReason::SessionExpired);
            return Ok(Refresh::Rejected);
        }
        let access = http::json::<AccessToken>(response).await?.access_token;
        self.save(Credentials {
            access_token: access,
            refresh_token: current.refresh_token,
        });
        Ok(Refresh::Refreshed)
    }

    /// Forgets the session locally, then revokes its refresh token on the backend (best effort).
    pub async fn sign_out(&self) {
        let Some(previous) = self.credentials() else {
            return;
        };
        self.end_session(SignOutReason::Requested);
        let revoke = self
            .post_public(
                LOGOUT_PATH,
                &RefreshRequest {
                    refresh_token: &previous.refresh_token,
                },
            )
            .await;
        if let Err(error) = revoke {
            tracing::info!(%error, "revoking the session on the backend failed; it is gone locally");
        }
    }

    /// Ends the session for `reason`: clears the store and tells subscribers. A no-op without one.
    pub fn end_session(&self, reason: SignOutReason) {
        let had_session = self
            .cache
            .lock()
            .expect("credentials poisoned")
            .take()
            .is_some();
        if !had_session {
            return;
        }
        self.store.clear();
        self.session.send_replace(None);
        tracing::info!(?reason, "signed out");
        self.bus.emit(SdkEvent::SignedOut { reason });
    }

    /// Sends the request `build` makes with the session's bearer token. A 401 refreshes the token
    /// once and retries; a refused refresh ends the session.
    pub async fn send(
        &self,
        build: impl Fn(&Client) -> RequestBuilder,
    ) -> Result<Response, NomiError> {
        let credentials = self.credentials().ok_or(NomiError::NotSignedIn)?;
        let response = self.attempt(&build, &credentials.access_token).await?;
        if response.status() != StatusCode::UNAUTHORIZED {
            return Ok(response);
        }
        match self.refresh(Some(&credentials.access_token)).await? {
            Refresh::Refreshed => {
                let fresh = self.credentials().ok_or(NomiError::NotSignedIn)?;
                let retried = self.attempt(&build, &fresh.access_token).await?;
                if retried.status() == StatusCode::UNAUTHORIZED {
                    return Err(NomiError::Unauthorized);
                }
                Ok(retried)
            }
            Refresh::Rejected => Err(NomiError::Unauthorized),
            Refresh::NotSignedIn => Err(NomiError::NotSignedIn),
        }
    }

    /// `/api/<segments...>` under the base URL, each segment percent-encoded.
    pub fn api_url(&self, segments: &[&str]) -> Url {
        let mut url = self.http.url(API_ROOT);
        url.path_segments_mut()
            .expect("an http(s) base URL has a path")
            .extend(segments);
        url
    }

    pub async fn profile(&self) -> Result<Profile, NomiError> {
        let url = self.http.url(PROFILE_PATH);
        http::json(self.send(|client| client.get(url.clone())).await?).await
    }

    async fn attempt(
        &self,
        build: &impl Fn(&Client) -> RequestBuilder,
        access: &str,
    ) -> Result<Response, NomiError> {
        build(self.http.client())
            .bearer_auth(access)
            .send()
            .await
            .map_err(http::transport_error)
    }

    async fn post_public(&self, path: &str, body: &impl Serialize) -> Result<Response, NomiError> {
        self.http
            .client()
            .post(self.http.url(path))
            .json(body)
            .send()
            .await
            .map_err(http::transport_error)
    }

    fn begin_session(&self, pair: TokenPair) -> Result<SignedInUser, NomiError> {
        let user =
            user_from_token(&pair.access_token).ok_or_else(|| NomiError::InvalidResponse {
                reason: "the access token names no user".into(),
            })?;
        self.save(Credentials {
            access_token: pair.access_token,
            refresh_token: pair.refresh_token,
        });
        let changed = self.session.send_if_modified(|current| {
            let changed = current.as_ref() != Some(&user);
            *current = Some(user.clone());
            changed
        });
        if changed {
            tracing::info!(user_id = %user.user_id, "signed in");
            self.bus.emit(SdkEvent::SignedIn {
                user_id: user.user_id.clone(),
            });
        }
        Ok(user)
    }

    fn save(&self, credentials: Credentials) {
        self.store.save(credentials.clone());
        *self.cache.lock().expect("credentials poisoned") = Some(credentials);
    }
}

/// The user a backend access token was issued to (its `sub`). The SDK only reads the claim; the
/// backend is what verifies the token.
pub(crate) fn user_from_token(token: &str) -> Option<SignedInUser> {
    let segments: Vec<&str> = token.split('.').collect();
    if segments.len() != JWT_SEGMENTS {
        return None;
    }
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(segments[1].trim_end_matches('='))
        .ok()?;
    let claims: JwtClaims = serde_json::from_slice(&payload).ok()?;
    (!claims.sub.is_empty()).then_some(SignedInUser {
        user_id: claims.sub,
    })
}

/// The response when it is 2xx, else its `code_error` as `NomiError::EmailCode` when it has one.
async fn code_checked(response: Response) -> Result<Response, NomiError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let body = response.text().await.unwrap_or_default();
    let code_error = serde_json::from_str::<CodeErrorBody>(&body)
        .ok()
        .and_then(|body| match body.error.as_str() {
            "wrong_code" => body
                .attempts_left
                .map(|attempts_left| EmailCodeError::WrongCode { attempts_left }),
            "expired" => Some(EmailCodeError::Expired),
            "too_many_attempts" => Some(EmailCodeError::TooManyAttempts),
            "wait" => body.seconds.map(|seconds| EmailCodeError::Wait { seconds }),
            "too_many_codes" => Some(EmailCodeError::TooManyCodes),
            _ => None,
        });
    Err(match code_error {
        Some(error) => NomiError::EmailCode { error },
        None => NomiError::Http {
            status: status.as_u16(),
            reason: body,
        },
    })
}
