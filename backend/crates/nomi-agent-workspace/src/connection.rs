//! A user's own Google connection: the OAuth flow that creates it, and the encrypted tokens it
//! keeps. Everything is keyed by the Nomi user, so one person's account is never reachable from
//! another's request.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use uuid::Uuid;

/// The Google services a user can allow, in the order the app shows them.
pub const SERVICES: [&str; 5] = ["gmail", "sheets", "docs", "drive", "calendar"];

/// Asked for on every connection: who the account is.
const IDENTITY_SCOPES: [&str; 2] = ["openid", "email"];

/// The OAuth scope that lets the agent use a service.
pub fn service_scope(service: &str) -> Option<&'static str> {
    match service {
        // Read, search, draft and send. Sending still waits for the user's OK in chat.
        "gmail" => Some("https://www.googleapis.com/auth/gmail.modify"),
        "sheets" => Some("https://www.googleapis.com/auth/spreadsheets"),
        "docs" => Some("https://www.googleapis.com/auth/documents"),
        // Finding files only.
        "drive" => Some("https://www.googleapis.com/auth/drive.readonly"),
        "calendar" => Some("https://www.googleapis.com/auth/calendar.events"),
        _ => None,
    }
}

/// Where Google lives. Every URL is swappable so tests can point at a fake server.
#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
    /// The app's callback page, registered with the OAuth client
    /// (e.g. https://nomi.example.com/connections/google/callback).
    pub redirect_uri: String,
    /// The app's Google sign-in callback page (GOOGLE_SIGNIN_REDIRECT_URI, else
    /// /auth/google/callback on the same site as `redirect_uri`).
    pub signin_redirect_uri: String,
    pub auth_url: String,
    pub token_url: String,
    pub revoke_url: String,
    pub userinfo_url: String,
    pub gmail_base: String,
    pub sheets_base: String,
    pub docs_base: String,
    /// Drive and Calendar share www.googleapis.com.
    pub drive_base: String,
    pub calendar_base: String,
}

impl GoogleConfig {
    /// Google's real endpoints for an OAuth client.
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        let signin_redirect_uri = reqwest::Url::parse(&redirect_uri)
            .ok()
            .and_then(|u| u.join("/auth/google/callback").ok())
            .map(|u| u.to_string())
            .unwrap_or_default();
        Self {
            client_id,
            client_secret,
            redirect_uri,
            signin_redirect_uri,
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            revoke_url: "https://oauth2.googleapis.com/revoke".to_string(),
            userinfo_url: "https://openidconnect.googleapis.com/v1/userinfo".to_string(),
            gmail_base: "https://gmail.googleapis.com".to_string(),
            sheets_base: "https://sheets.googleapis.com".to_string(),
            docs_base: "https://docs.googleapis.com".to_string(),
            drive_base: "https://www.googleapis.com".to_string(),
            calendar_base: "https://www.googleapis.com".to_string(),
        }
    }

    /// Every endpoint on one server (tests).
    pub fn all_at(base: &str) -> Self {
        let mut config = Self::new("client-id".to_string(), "client-secret".to_string(), "http://app.test/connections/google/callback".to_string());
        config.auth_url = format!("{base}/o/oauth2/v2/auth");
        config.token_url = format!("{base}/token");
        config.revoke_url = format!("{base}/revoke");
        config.userinfo_url = format!("{base}/v1/userinfo");
        for url in [&mut config.gmail_base, &mut config.sheets_base, &mut config.docs_base, &mut config.drive_base, &mut config.calendar_base] {
            *url = base.to_string();
        }
        config
    }

    /// The server's OAuth client, from GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET and
    /// GOOGLE_REDIRECT_URI. `None` until whoever runs Nomi sets all three.
    pub fn from_env() -> Option<Self> {
        let get = |name: &str| std::env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let mut config = Self::new(get("GOOGLE_CLIENT_ID")?, get("GOOGLE_CLIENT_SECRET")?, get("GOOGLE_REDIRECT_URI")?);
        if let Some(signin) = get("GOOGLE_SIGNIN_REDIRECT_URI") {
            config.signin_redirect_uri = signin;
        }
        // Development only: every Google endpoint on one fake server.
        if let Some(base) = get("GOOGLE_FAKE_BASE_URL") {
            let fake = Self::all_at(&base);
            config = Self { client_id: config.client_id, client_secret: config.client_secret, redirect_uri: config.redirect_uri, signin_redirect_uri: config.signin_redirect_uri, ..fake };
        }
        Some(config)
    }
}

/// Only known services, each once, in the app's order.
pub fn normalize_services(requested: &[String]) -> Vec<String> {
    SERVICES.iter().filter(|s| requested.iter().any(|r| r == *s)).map(|s| s.to_string()).collect()
}

/// Starts connecting: remembers who asked (and which chat to carry on in), and returns the
/// Google page to send them to.
pub async fn start_authorization(
    conn: &mut PgConnection,
    config: &GoogleConfig,
    user_id: Uuid,
    services: &[String],
    resume_session_id: Option<Uuid>,
    login_hint: Option<&str>,
) -> Result<String, String> {
    let services = normalize_services(services);
    if services.is_empty() {
        return Err("pick at least one service".to_string());
    }
    let state = {
        use rand::RngCore;
        let mut bytes = [0u8; 24];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    };
    sqlx::query("DELETE FROM workspace_oauth_states WHERE created_at < now() - interval '10 minutes'")
        .execute(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("INSERT INTO workspace_oauth_states (state, user_id, services, resume_session_id) VALUES ($1, $2, $3, $4)")
        .bind(&state)
        .bind(user_id)
        .bind(&services)
        .bind(resume_session_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;

    let scopes: Vec<&str> = IDENTITY_SCOPES.iter().copied().chain(services.iter().filter_map(|s| service_scope(s))).collect();
    let mut url = reqwest::Url::parse(&config.auth_url).map_err(|e| e.to_string())?;
    url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("redirect_uri", &config.redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", &scopes.join(" "))
        // A refresh token, so the agent keeps working after the first hour.
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent")
        .append_pair("include_granted_scopes", "true")
        .append_pair("state", &state);
    // Pre-selects the Google account the user signs in to Nomi with.
    if let Some(hint) = login_hint {
        url.query_pairs_mut().append_pair("login_hint", hint);
    }
    Ok(url.to_string())
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_token: Option<String>,
    #[serde(default)]
    scope: String,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    email: String,
}

/// The outcome of a finished authorization.
#[derive(Debug, Clone, PartialEq)]
pub struct Connected {
    pub email: String,
    pub services: Vec<String>,
    pub resume_session_id: Option<Uuid>,
}

/// Finishes connecting: checks the `state` belongs to this user, trades the code for tokens and
/// stores them encrypted. Services the user unticked on Google's screen are left out.
pub async fn complete_authorization(
    conn: &mut PgConnection,
    http: &reqwest::Client,
    config: &GoogleConfig,
    key: &[u8; 32],
    user_id: Uuid,
    code: &str,
    state: &str,
) -> Result<Connected, String> {
    let pending: Option<(Vec<String>, Option<Uuid>)> = sqlx::query_as(
        "DELETE FROM workspace_oauth_states WHERE state = $1 AND user_id = $2 AND created_at > now() - interval '10 minutes' \
         RETURNING services, resume_session_id",
    )
    .bind(state)
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| e.to_string())?;
    let Some((_, resume_session_id)) = pending else {
        return Err("This sign-in link has expired. Start connecting again.".to_string());
    };

    let response = http
        .post(&config.token_url)
        .form(&[
            ("code", code),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
            ("redirect_uri", &config.redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        tracing::warn!(%body, "workspace: token exchange failed");
        return Err("Google didn't accept the sign-in. Start connecting again.".to_string());
    }
    let tokens: TokenResponse = response.json().await.map_err(|e| e.to_string())?;

    let info: UserInfo = http
        .get(&config.userinfo_url)
        .bearer_auth(&tokens.access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    // What Google actually granted (the user can untick services on its screen).
    let granted: Vec<String> = SERVICES
        .iter()
        .filter(|s| service_scope(s).is_some_and(|scope| tokens.scope.split_whitespace().any(|g| g == scope)))
        .map(|s| s.to_string())
        .collect();

    let expires_at = Utc::now() + Duration::seconds(tokens.expires_in);
    let access = nomi_settings::crypto::encrypt(key, &tokens.access_token);
    let refresh = tokens.refresh_token.as_deref().map(|t| nomi_settings::crypto::encrypt(key, t));
    sqlx::query(
        "INSERT INTO workspace_connections (user_id, google_email, services, access_token_encrypted, refresh_token_encrypted, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (user_id) DO UPDATE SET google_email = EXCLUDED.google_email, services = EXCLUDED.services, \
           access_token_encrypted = EXCLUDED.access_token_encrypted, \
           refresh_token_encrypted = COALESCE(EXCLUDED.refresh_token_encrypted, workspace_connections.refresh_token_encrypted), \
           expires_at = EXCLUDED.expires_at, updated_at = now()",
    )
    .bind(user_id)
    .bind(&info.email)
    .bind(&granted)
    .bind(access)
    .bind(refresh)
    .bind(expires_at)
    .execute(&mut *conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Connected { email: info.email, services: granted, resume_session_id })
}

/// What the Connections page shows.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ConnectionStatus {
    pub email: String,
    pub services: Vec<String>,
    pub connected_at: DateTime<Utc>,
}

pub async fn status(conn: &mut PgConnection, user_id: Uuid) -> Result<Option<ConnectionStatus>, sqlx::Error> {
    let row: Option<(String, Vec<String>, DateTime<Utc>)> =
        sqlx::query_as("SELECT google_email, services, created_at FROM workspace_connections WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&mut *conn)
            .await?;
    Ok(row.map(|(email, services, connected_at)| ConnectionStatus { email, services, connected_at }))
}

/// Disconnects: asks Google to revoke the token (best effort) and forgets it.
pub async fn disconnect(conn: &mut PgConnection, http: &reqwest::Client, config: Option<&GoogleConfig>, key: &[u8; 32], user_id: Uuid) -> Result<bool, sqlx::Error> {
    let row: Option<(Vec<u8>, Option<Vec<u8>>)> = sqlx::query_as(
        "DELETE FROM workspace_connections WHERE user_id = $1 RETURNING access_token_encrypted, refresh_token_encrypted",
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((access, refresh)) = row else { return Ok(false) };
    if let Some(config) = config {
        let token = refresh.as_deref().or(Some(access.as_slice())).and_then(|t| nomi_settings::crypto::decrypt(key, t).ok());
        if let Some(token) = token {
            let _ = http.post(&config.revoke_url).form(&[("token", token.as_str())]).send().await;
        }
    }
    Ok(true)
}

/// Why the agent can't use a user's Google right now.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessError {
    /// Never connected, or Google revoked it: the user has to connect (again).
    NotConnected,
    /// Connected, but this service wasn't allowed.
    ServiceNotAllowed(String),
    Other(String),
}

/// services, access token, refresh token, expiry (both tokens encrypted).
type TokenRow = (Vec<String>, Vec<u8>, Option<Vec<u8>>, DateTime<Utc>);

/// A live access token for this user and service, refreshed when it's about to expire.
pub async fn access_token(
    conn: &mut PgConnection,
    http: &reqwest::Client,
    config: &GoogleConfig,
    key: &[u8; 32],
    user_id: Uuid,
    service: &str,
) -> Result<String, AccessError> {
    let row: Option<TokenRow> = sqlx::query_as(
        "SELECT services, access_token_encrypted, refresh_token_encrypted, expires_at FROM workspace_connections WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| AccessError::Other(e.to_string()))?;
    let Some((services, access, refresh, expires_at)) = row else { return Err(AccessError::NotConnected) };
    if !services.iter().any(|s| s == service) {
        return Err(AccessError::ServiceNotAllowed(service.to_string()));
    }
    if expires_at > Utc::now() + Duration::seconds(60) {
        return nomi_settings::crypto::decrypt(key, &access).map_err(|_| AccessError::NotConnected);
    }

    let Some(refresh) = refresh.and_then(|r| nomi_settings::crypto::decrypt(key, &r).ok()) else {
        return Err(AccessError::NotConnected);
    };
    let response = http
        .post(&config.token_url)
        .form(&[
            ("client_id", config.client_id.as_str()),
            ("client_secret", config.client_secret.as_str()),
            ("refresh_token", refresh.as_str()),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|e| AccessError::Other(e.to_string()))?;
    if response.status().as_u16() == 400 || response.status().as_u16() == 401 {
        // invalid_grant: the user revoked access in their Google account, or it lapsed.
        let _ = sqlx::query("DELETE FROM workspace_connections WHERE user_id = $1").bind(user_id).execute(&mut *conn).await;
        return Err(AccessError::NotConnected);
    }
    let tokens: TokenResponse = response
        .error_for_status()
        .map_err(|e| AccessError::Other(e.to_string()))?
        .json()
        .await
        .map_err(|e| AccessError::Other(e.to_string()))?;
    sqlx::query("UPDATE workspace_connections SET access_token_encrypted = $2, expires_at = $3, updated_at = now() WHERE user_id = $1")
        .bind(user_id)
        .bind(nomi_settings::crypto::encrypt(key, &tokens.access_token))
        .bind(Utc::now() + Duration::seconds(tokens.expires_in))
        .execute(&mut *conn)
        .await
        .map_err(|e| AccessError::Other(e.to_string()))?;
    Ok(tokens.access_token)
}

/// One change the agent made in the user's account.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Activity {
    pub service: String,
    pub summary: String,
    pub link: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn record_activity(conn: &mut PgConnection, user_id: Uuid, service: &str, summary: &str, link: Option<&str>) {
    let _ = sqlx::query("INSERT INTO workspace_activity (user_id, service, summary, link) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(service)
        .bind(summary)
        .bind(link)
        .execute(&mut *conn)
        .await;
}

pub async fn recent_activity(conn: &mut PgConnection, user_id: Uuid, limit: i64) -> Result<Vec<Activity>, sqlx::Error> {
    let rows: Vec<(String, String, Option<String>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT service, summary, link, created_at FROM workspace_activity WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows.into_iter().map(|(service, summary, link, created_at)| Activity { service, summary, link, created_at }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_and_repeated_services_are_dropped_and_order_is_the_apps() {
        let picked = normalize_services(&["drive".into(), "gmail".into(), "gmail".into(), "photos".into()]);
        assert_eq!(picked, vec!["gmail".to_string(), "drive".to_string()]);
    }
}
