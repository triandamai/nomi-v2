//! OAuth sign-ins in progress. `begin` records the app's PKCE challenge and where to send the
//! person back; the provider's callback `claim`s the flow (once, within ten minutes) and `finish`es
//! it with a one-time code; the app `trade`s that code, with the PKCE verifier, for a session.

use base64::Engine;
use chrono::{Duration, Utc};
use rand::RngCore;
use relay_core::Env;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::oauth::Profile;
use crate::users::{self, User};
use crate::{digest, AuthError, Scope};

/// How long someone has to come back from the provider.
pub const FLOW_MINUTES: i64 = 10;
/// How long the app has to trade the code it was sent back with.
pub const CODE_MINUTES: i64 = 5;
const TOKEN_BYTES: usize = 32;
/// A base64url SHA-256 is 43 characters.
const S256_CHALLENGE_LEN: usize = 43;
const LOCAL_HOSTS: [&str; 2] = ["localhost", "127.0.0.1"];

pub struct Claimed {
    pub provider: String,
    pub redirect_to: String,
    pub nonce: String,
    pub link_user: Option<Uuid>,
    id: Uuid,
}

fn random_token() -> String {
    let mut bytes = [0u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn s256(verifier: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

/// Whether `redirect_to` is one of the app's addresses (a `redirect_urls` entry or below it);
/// on `dev`, Nomi's previews and the builder's own machine too.
pub fn redirect_allowed(redirect_to: &str, allowed: &[String], env: Env, preview_origins: &[String]) -> bool {
    let Ok(target) = url::Url::parse(redirect_to) else { return false };
    if !matches!(target.scheme(), "http" | "https") {
        return false;
    }
    let listed = allowed.iter().filter_map(|a| url::Url::parse(a).ok()).any(|a| {
        a.scheme() == target.scheme() && a.host_str() == target.host_str() && a.port_or_known_default() == target.port_or_known_default() && target.path().starts_with(a.path().trim_end_matches('/'))
    });
    let preview = env == Env::Dev
        && target.host_str().is_some_and(|host| LOCAL_HOSTS.contains(&host) || preview_origins.iter().any(|suffix| host.ends_with(suffix.as_str())));
    listed || preview
}

/// Starts a sign-in (or a link, for `link_user`); answers the state and nonce to send the
/// provider.
pub async fn begin(scope: &Scope<'_>, provider: &str, redirect_to: &str, challenge: &str, challenge_method: &str, link_user: Option<Uuid>) -> Result<(String, String), AuthError> {
    let shaped = challenge.len() == S256_CHALLENGE_LEN && challenge.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if challenge_method != "S256" || !shaped {
        return Err(AuthError::bad("PKCE_REQUIRED", "Send a code_challenge made with S256."));
    }
    let (state, nonce) = (random_token(), random_token());
    sqlx::query(
        "INSERT INTO nomi.oauth_flows (project_id, state_hash, provider, code_challenge, redirect_to, nonce, link_user_id, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(scope.project)
    .bind(digest(&["state", &state]))
    .bind(provider)
    .bind(challenge)
    .bind(redirect_to)
    .bind(&nonce)
    .bind(link_user)
    .bind(Utc::now() + Duration::minutes(FLOW_MINUTES))
    .execute(scope.pool)
    .await?;
    Ok((state, nonce))
}

/// The flow `state` names, claimed so it can't be used again.
pub async fn claim(scope: &Scope<'_>, provider: &str, state: &str) -> Result<Claimed, AuthError> {
    let row: Option<(Uuid, String, String, Option<Uuid>)> = sqlx::query_as(
        "UPDATE nomi.oauth_flows SET claimed_at = now() WHERE project_id = $1 AND state_hash = $2 AND provider = $3 \
         AND claimed_at IS NULL AND expires_at > now() RETURNING id, redirect_to, nonce, link_user_id",
    )
    .bind(scope.project)
    .bind(digest(&["state", state]))
    .bind(provider)
    .fetch_optional(scope.pool)
    .await?;
    let (id, redirect_to, nonce, link_user) = row.ok_or_else(|| AuthError::bad("INVALID_STATE", "This sign-in has expired or was already finished: start again."))?;
    Ok(Claimed { provider: provider.to_string(), redirect_to, nonce, link_user, id })
}

/// Records who signed in and answers the one-time code the app trades for a session.
pub async fn finish(scope: &Scope<'_>, flow: &Claimed, profile: &Profile) -> Result<String, AuthError> {
    let user = users::signed_in_by_identity(scope.pool, scope.project, &flow.provider, profile, flow.link_user).await?;
    let code = random_token();
    sqlx::query("UPDATE nomi.oauth_flows SET user_id = $2, code_hash = $3, expires_at = $4 WHERE id = $1")
        .bind(flow.id)
        .bind(user)
        .bind(digest(&["code", &code]))
        .bind(Utc::now() + Duration::minutes(CODE_MINUTES))
        .execute(scope.pool)
        .await?;
    Ok(code)
}

/// Trades the one-time code for its user, once, if `verifier` matches the challenge the flow
/// began with. A wrong verifier spends the code.
pub async fn trade(scope: &Scope<'_>, code: &str, verifier: &str) -> Result<User, AuthError> {
    let invalid = || AuthError::bad("INVALID_CODE", "That sign-in code isn't valid, has expired or was already used: sign in again.");
    let row: Option<(String, Uuid)> = sqlx::query_as(
        "DELETE FROM nomi.oauth_flows WHERE project_id = $1 AND code_hash = $2 AND expires_at > now() AND user_id IS NOT NULL RETURNING code_challenge, user_id",
    )
    .bind(scope.project)
    .bind(digest(&["code", code]))
    .fetch_optional(scope.pool)
    .await?;
    let (challenge, user) = row.ok_or_else(invalid)?;
    if s256(verifier) != challenge {
        return Err(invalid());
    }
    users::get(scope.pool, scope.project, user).await?.ok_or_else(invalid)
}

/// Throws a claimed flow away (its sign-in failed at the provider).
pub async fn drop_flow(scope: &Scope<'_>, flow: &Claimed) -> Result<(), AuthError> {
    sqlx::query("DELETE FROM nomi.oauth_flows WHERE id = $1").bind(flow.id).execute(scope.pool).await?;
    Ok(())
}
