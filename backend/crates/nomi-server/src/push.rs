//! Pushes to phones, for when no socket can stay open: inbox items and chat replies as alerts
//! (FCM HTTP v1 on Android, APNs on iOS), and iOS Live Activity updates and starts (APNs
//! `liveactivity` pushes). Desktops never get pushes; they rely on the live socket.
//!
//! Every push is fail-open: a provider that is down or misconfigured is logged and skipped, and
//! only a token the provider says is gone for good is forgotten.

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::notifications::Notice;

const FCM_SEND_URL_BASE: &str = "https://fcm.googleapis.com/v1/projects";
const FCM_SCOPE: &str = "https://www.googleapis.com/auth/firebase.messaging";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_JWT_BEARER_GRANT: &str = "urn:ietf:params:oauth:grant-type:jwt-bearer";
/// How long the signed assertion traded for an access token is valid (Google's maximum).
const GOOGLE_ASSERTION_LIFETIME_SECS: i64 = 3600;
/// Mint a fresh FCM access token this long before Google says the current one expires.
const FCM_TOKEN_EARLY_REFRESH: Duration = Duration::from_secs(60);
/// FCM's error code for a token whose app was uninstalled or whose registration expired.
const FCM_UNREGISTERED: &str = "UNREGISTERED";
const FCM_ERROR_DETAIL_TYPE: &str = "type.googleapis.com/google.firebase.fcm.v1.FcmError";

const APNS_PRODUCTION_HOST: &str = "https://api.push.apple.com";
const APNS_SANDBOX_HOST: &str = "https://api.sandbox.push.apple.com";
const APNS_DEFAULT_TOPIC: &str = "ai.nomi.app";
const APNS_LIVE_ACTIVITY_TOPIC_SUFFIX: &str = ".push-type.liveactivity";
const APNS_PUSH_TYPE_ALERT: &str = "alert";
const APNS_PUSH_TYPE_LIVE_ACTIVITY: &str = "liveactivity";
/// Deliver now (10) rather than at a power-friendly time (5).
const APNS_PRIORITY_IMMEDIATE: &str = "10";
/// Apple rejects provider tokens older than an hour and throttles ones refreshed more often than
/// every 20 minutes, so one is reused for 50.
const APNS_JWT_REUSE: Duration = Duration::from_secs(50 * 60);
/// APNs: the token is gone for good (410) or was never valid for this app.
const APNS_STATUS_GONE: u16 = 410;
const APNS_BAD_DEVICE_TOKEN: &str = "BadDeviceToken";
const APNS_EXPIRED_PROVIDER_TOKEN: &str = "ExpiredProviderToken";
/// The ActivityKit attributes type the iOS app's Live Activity is declared with.
const LIVE_ACTIVITY_ATTRIBUTES_TYPE: &str = "NomiActivityAttributes";

/// A single push call (either provider) gives up after this long.
const PUSH_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// The title of a chat-reply push when the chat has no title yet.
pub const CHAT_REPLY_FALLBACK_TITLE: &str = "Nomi";

/// The platforms that get pushes; desktops register without a token.
const PUSH_PLATFORMS: [&str; 2] = [PLATFORM_ANDROID, PLATFORM_IOS];
pub const PLATFORM_ANDROID: &str = "android";
pub const PLATFORM_IOS: &str = "ios";

/// One device a push goes to, with the token that fits the message (its push token for alerts,
/// its Live Activity token on iOS for activity pushes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceTarget {
    pub id: Uuid,
    pub platform: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PushKind {
    /// A notification the person sees: title, body and sound.
    Alert,
    /// Updates the running Live Activity (iOS only).
    LiveActivityUpdate { content_state: Value },
    /// Starts a Live Activity with push-to-start (iOS 17.2+).
    LiveActivityStart { attributes: Value, content_state: Value },
}

#[derive(Debug, Clone, PartialEq)]
pub struct PushMessage {
    pub title: String,
    pub body: String,
    /// Delivered with the push for the app to act on (deep links, ids); values are strings for FCM.
    pub data: BTreeMap<String, String>,
    pub kind: PushKind,
}

impl PushMessage {
    /// An inbox item as an alert; `data` carries its category and where it leads.
    pub fn alert(notice: &Notice) -> Self {
        let mut data = BTreeMap::from([("kind".to_string(), "notification".to_string()), ("category".to_string(), notice.kind.as_str().to_string())]);
        if let Some(link) = &notice.link {
            data.insert("link".into(), link.clone());
        }
        PushMessage { title: notice.title.clone(), body: notice.body.clone(), data, kind: PushKind::Alert }
    }

    /// A chat reply as an alert, titled after the chat.
    pub fn chat_reply(chat_title: Option<String>, session_id: Uuid, message_id: Uuid, preview: &str) -> Self {
        let title = chat_title.filter(|t| !t.trim().is_empty()).unwrap_or_else(|| CHAT_REPLY_FALLBACK_TITLE.to_string());
        let data = BTreeMap::from([
            ("session_id".to_string(), session_id.to_string()),
            ("message_id".to_string(), message_id.to_string()),
            ("kind".to_string(), "chat_reply".to_string()),
        ]);
        PushMessage { title, body: preview.to_string(), data, kind: PushKind::Alert }
    }

    /// Moves the iPhone's Live Activity to `content_state`.
    pub fn live_activity_update(content_state: Value) -> Self {
        PushMessage { title: String::new(), body: String::new(), data: BTreeMap::new(), kind: PushKind::LiveActivityUpdate { content_state } }
    }

    fn is_live_activity(&self) -> bool {
        !matches!(self.kind, PushKind::Alert)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PushError {
    /// The provider says this token is gone for good; the device's row should be forgotten.
    #[error("the device's push token is no longer valid")]
    Invalid,
    /// No provider is set up for this platform or kind of push.
    #[error("no push provider for {0}")]
    Unsupported(String),
    #[error("push failed: {0}")]
    Failed(String),
}

#[async_trait::async_trait]
pub trait PushSender: Send + Sync {
    async fn send(&self, device: &DeviceTarget, message: &PushMessage) -> Result<(), PushError>;

    /// False when nothing would ever be sent, so callers can skip looking devices up at all.
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Sends nothing: no provider is configured (and in tests).
pub struct NoopPush;

#[async_trait::async_trait]
impl PushSender for NoopPush {
    async fn send(&self, _device: &DeviceTarget, _message: &PushMessage) -> Result<(), PushError> {
        Ok(())
    }

    fn is_enabled(&self) -> bool {
        false
    }
}

static SENDER: OnceLock<Arc<dyn PushSender>> = OnceLock::new();

/// The sender inbox items and chat replies go out through, set once at startup.
pub fn init(sender: Arc<dyn PushSender>) {
    let _ = SENDER.set(sender);
}

/// The sender set at startup, or one that sends nothing.
pub fn sender() -> Arc<dyn PushSender> {
    SENDER.get_or_init(|| Arc::new(NoopPush)).clone()
}

/// Sends `message` to each of `user_id`'s phones, forgetting tokens the provider rejects.
pub async fn dispatch(pool: PgPool, sender: Arc<dyn PushSender>, user_id: Uuid, message: PushMessage) {
    let rows: Vec<(Uuid, String, Option<String>, Option<String>)> = match sqlx::query_as(
        "SELECT id, platform, push_token, live_activity_start_token FROM devices WHERE user_id = $1 AND platform = ANY($2)",
    )
    .bind(user_id)
    .bind(&PUSH_PLATFORMS[..])
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, %user_id, "couldn't load devices to push to");
            return;
        }
    };
    let live_activity = message.is_live_activity();
    for (id, platform, push_token, activity_token) in rows {
        let token = match (live_activity, platform.as_str()) {
            (false, _) => push_token,
            (true, PLATFORM_IOS) => activity_token,
            (true, _) => None,
        };
        let Some(token) = token else { continue };
        let target = DeviceTarget { id, platform, token };
        match sender.send(&target, &message).await {
            Ok(()) => {}
            Err(PushError::Invalid) => forget(&pool, id, live_activity).await,
            Err(e) => tracing::warn!(error = %e, device_id = %id, "couldn't push to a device"),
        }
    }
}

/// Drops a rejected token: the whole device for a push token, only the activity token otherwise
/// (the phone still takes alerts).
async fn forget(pool: &PgPool, device_id: Uuid, live_activity: bool) {
    let query = if live_activity {
        "UPDATE devices SET live_activity_start_token = NULL WHERE id = $1"
    } else {
        "DELETE FROM devices WHERE id = $1"
    };
    match sqlx::query(query).bind(device_id).execute(pool).await {
        Ok(_) => tracing::info!(%device_id, "forgot a push token the provider rejected"),
        Err(e) => tracing::warn!(error = %e, %device_id, "couldn't forget a rejected push token"),
    }
}

/// Pushes `message` to `user_id`'s phones in the background, when a provider is set up.
pub fn spawn_dispatch(pool: &PgPool, user_id: Uuid, message: PushMessage) {
    let sender = sender();
    if !sender.is_enabled() {
        return;
    }
    tokio::spawn(dispatch(pool.clone(), sender, user_id, message));
}

// ------------------------------------------------------------------------------------------------
// Token caching (FCM access tokens, APNs provider JWTs)
// ------------------------------------------------------------------------------------------------

/// A bearer token and when to stop using it.
#[derive(Debug, Clone)]
struct CachedToken {
    value: String,
    use_until: Instant,
}

impl CachedToken {
    fn current(&self, now: Instant) -> Option<&str> {
        (now < self.use_until).then_some(self.value.as_str())
    }
}

/// When to replace an FCM access token Google said lasts `expires_in` from `obtained_at`.
fn fcm_token_use_until(obtained_at: Instant, expires_in: Duration) -> Instant {
    obtained_at + expires_in.saturating_sub(FCM_TOKEN_EARLY_REFRESH)
}

/// When to replace an APNs provider JWT signed at `signed_at`.
fn apns_jwt_use_until(signed_at: Instant) -> Instant {
    signed_at + APNS_JWT_REUSE
}

/// A secret given inline (`{...}` JSON or a `-----BEGIN` PEM, with `\n` escapes allowed in the
/// PEM) or as a path to the file holding it.
fn read_secret(value: &str) -> std::io::Result<String> {
    let trimmed = value.trim();
    if trimmed.starts_with('{') {
        Ok(trimmed.to_string())
    } else if trimmed.starts_with("-----BEGIN") {
        Ok(trimmed.replace("\\n", "\n"))
    } else {
        std::fs::read_to_string(trimmed)
    }
}

fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

// ------------------------------------------------------------------------------------------------
// FCM (Android)
// ------------------------------------------------------------------------------------------------

/// The fields of a Google service-account key file FCM needs.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccount {
    pub client_email: String,
    pub private_key: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub token_uri: Option<String>,
}

pub struct FcmSender {
    http: reqwest::Client,
    project_id: String,
    account: ServiceAccount,
    key: EncodingKey,
    token: Mutex<Option<CachedToken>>,
}

#[derive(Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    expires_in: u64,
}

/// `POST` target for one FCM message.
pub fn fcm_send_url(project_id: &str) -> String {
    format!("{FCM_SEND_URL_BASE}/{project_id}/messages:send")
}

/// The FCM HTTP v1 body for `message` to `token`. Live Activity kinds are iOS-only and never
/// reach here.
pub fn fcm_message_body(token: &str, message: &PushMessage) -> Value {
    json!({
        "message": {
            "token": token,
            "notification": { "title": message.title, "body": message.body },
            "data": message.data,
            "android": { "priority": "HIGH" },
        }
    })
}

/// Whether an FCM error response says the token is gone for good.
pub fn fcm_rejects_token(body: &Value) -> bool {
    body["error"]["details"].as_array().is_some_and(|details| {
        details.iter().any(|d| d["@type"] == FCM_ERROR_DETAIL_TYPE && d["errorCode"] == FCM_UNREGISTERED)
    })
}

/// The claims of the RS256 assertion traded for an access token.
fn google_assertion_claims(account: &ServiceAccount, now_unix: i64) -> Value {
    json!({
        "iss": account.client_email,
        "scope": FCM_SCOPE,
        "aud": account.token_uri.as_deref().unwrap_or(GOOGLE_TOKEN_URL),
        "iat": now_unix,
        "exp": now_unix + GOOGLE_ASSERTION_LIFETIME_SECS,
    })
}

impl FcmSender {
    pub fn new(http: reqwest::Client, project_id: String, account: ServiceAccount) -> Result<Self, String> {
        let key = EncodingKey::from_rsa_pem(account.private_key.as_bytes()).map_err(|e| format!("service account key: {e}"))?;
        Ok(FcmSender { http, project_id, account, key, token: Mutex::new(None) })
    }

    /// From `FCM_PROJECT_ID` (or the key file's project) and `FCM_SERVICE_ACCOUNT_JSON` (the
    /// key file's contents or its path); None when unset.
    pub fn from_env(http: reqwest::Client) -> Option<Result<Self, String>> {
        let raw = env_nonempty("FCM_SERVICE_ACCOUNT_JSON")?;
        Some((|| {
            let json = read_secret(&raw).map_err(|e| format!("FCM_SERVICE_ACCOUNT_JSON: {e}"))?;
            let account: ServiceAccount = serde_json::from_str(&json).map_err(|e| format!("FCM_SERVICE_ACCOUNT_JSON: {e}"))?;
            let project_id = env_nonempty("FCM_PROJECT_ID")
                .or_else(|| account.project_id.clone())
                .ok_or("FCM_PROJECT_ID is not set and the service account names no project")?;
            FcmSender::new(http, project_id, account)
        })())
    }

    async fn access_token(&self) -> Result<String, PushError> {
        let mut cached = self.token.lock().await;
        if let Some(token) = cached.as_ref().and_then(|t| t.current(Instant::now())) {
            return Ok(token.to_string());
        }
        let assertion = jsonwebtoken::encode(&Header::new(Algorithm::RS256), &google_assertion_claims(&self.account, chrono::Utc::now().timestamp()), &self.key)
            .map_err(|e| PushError::Failed(format!("signing the FCM assertion: {e}")))?;
        let obtained_at = Instant::now();
        let response = self
            .http
            .post(self.account.token_uri.as_deref().unwrap_or(GOOGLE_TOKEN_URL))
            .timeout(PUSH_REQUEST_TIMEOUT)
            .form(&[("grant_type", GOOGLE_JWT_BEARER_GRANT), ("assertion", assertion.as_str())])
            .send()
            .await
            .map_err(|e| PushError::Failed(format!("FCM token: {e}")))?;
        if !response.status().is_success() {
            return Err(PushError::Failed(format!("FCM token: {}", response.status())));
        }
        let token: GoogleTokenResponse = response.json().await.map_err(|e| PushError::Failed(format!("FCM token: {e}")))?;
        let use_until = fcm_token_use_until(obtained_at, Duration::from_secs(token.expires_in));
        *cached = Some(CachedToken { value: token.access_token.clone(), use_until });
        Ok(token.access_token)
    }
}

#[async_trait::async_trait]
impl PushSender for FcmSender {
    async fn send(&self, device: &DeviceTarget, message: &PushMessage) -> Result<(), PushError> {
        if message.is_live_activity() {
            return Err(PushError::Unsupported(format!("{} Live Activity", device.platform)));
        }
        let access_token = self.access_token().await?;
        let response = self
            .http
            .post(fcm_send_url(&self.project_id))
            .timeout(PUSH_REQUEST_TIMEOUT)
            .bearer_auth(access_token)
            .json(&fcm_message_body(&device.token, message))
            .send()
            .await
            .map_err(|e| PushError::Failed(format!("FCM: {e}")))?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let body: Value = response.json().await.unwrap_or(Value::Null);
        if fcm_rejects_token(&body) {
            Err(PushError::Invalid)
        } else {
            Err(PushError::Failed(format!("FCM {status}: {}", body["error"]["message"].as_str().unwrap_or(""))))
        }
    }
}

// ------------------------------------------------------------------------------------------------
// APNs (iOS)
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ApnsConfig {
    pub key_id: String,
    pub team_id: String,
    /// The app's bundle id, e.g. `ai.nomi.app`.
    pub topic: String,
    /// The production host, or the sandbox one for development builds.
    pub production: bool,
}

/// One APNs request, built without touching the network.
#[derive(Debug, Clone, PartialEq)]
pub struct ApnsRequest {
    pub url: String,
    /// `apns-push-type`.
    pub push_type: &'static str,
    /// `apns-topic`.
    pub topic: String,
    pub payload: Value,
}

/// The request for `message` to the device `token` at `now_unix` (Live Activity pushes carry
/// their own timestamp so iOS drops stale ones).
pub fn apns_request(config: &ApnsConfig, token: &str, message: &PushMessage, now_unix: i64) -> ApnsRequest {
    let host = if config.production { APNS_PRODUCTION_HOST } else { APNS_SANDBOX_HOST };
    let url = format!("{host}/3/device/{token}");
    let live_activity_topic = format!("{}{APNS_LIVE_ACTIVITY_TOPIC_SUFFIX}", config.topic);
    let (push_type, topic, aps) = match &message.kind {
        PushKind::Alert => (
            APNS_PUSH_TYPE_ALERT,
            config.topic.clone(),
            json!({ "alert": { "title": message.title, "body": message.body }, "sound": "default" }),
        ),
        PushKind::LiveActivityUpdate { content_state } => (
            APNS_PUSH_TYPE_LIVE_ACTIVITY,
            live_activity_topic,
            json!({ "timestamp": now_unix, "event": "update", "content-state": content_state }),
        ),
        PushKind::LiveActivityStart { attributes, content_state } => (
            APNS_PUSH_TYPE_LIVE_ACTIVITY,
            live_activity_topic,
            json!({
                "timestamp": now_unix,
                "event": "start",
                "content-state": content_state,
                "attributes-type": LIVE_ACTIVITY_ATTRIBUTES_TYPE,
                "attributes": attributes,
                "alert": { "title": message.title, "body": message.body },
            }),
        ),
    };
    let mut payload = serde_json::Map::new();
    for (key, value) in &message.data {
        payload.insert(key.clone(), Value::String(value.clone()));
    }
    payload.insert("aps".into(), aps);
    ApnsRequest { url, push_type, topic, payload: Value::Object(payload) }
}

/// Whether an APNs response says the token is gone for good.
pub fn apns_rejects_token(status: u16, reason: Option<&str>) -> bool {
    status == APNS_STATUS_GONE || reason == Some(APNS_BAD_DEVICE_TOKEN)
}

#[derive(Serialize)]
struct ApnsJwtClaims<'a> {
    iss: &'a str,
    iat: i64,
}


pub struct ApnsSender {
    http: reqwest::Client,
    config: ApnsConfig,
    key: EncodingKey,
    jwt: Mutex<Option<CachedToken>>,
}

impl ApnsSender {
    /// `private_key_p8` is the `.p8` key file's PEM contents.
    pub fn new(config: ApnsConfig, private_key_p8: &str) -> Result<Self, String> {
        let key = EncodingKey::from_ec_pem(private_key_p8.as_bytes()).map_err(|e| format!("APNs key: {e}"))?;
        // APNs only speaks HTTP/2.
        let http = reqwest::Client::builder()
            .http2_prior_knowledge()
            .timeout(PUSH_REQUEST_TIMEOUT)
            .build()
            .map_err(|e| format!("APNs client: {e}"))?;
        Ok(ApnsSender { http, config, key, jwt: Mutex::new(None) })
    }

    /// From `APNS_KEY_ID`, `APNS_TEAM_ID`, `APNS_PRIVATE_KEY_P8` (contents or path), `APNS_TOPIC`
    /// and `APNS_PRODUCTION`; None when unset.
    pub fn from_env() -> Option<Result<Self, String>> {
        let key_id = env_nonempty("APNS_KEY_ID")?;
        let team_id = env_nonempty("APNS_TEAM_ID")?;
        let raw_key = env_nonempty("APNS_PRIVATE_KEY_P8")?;
        let topic = env_nonempty("APNS_TOPIC").unwrap_or_else(|| APNS_DEFAULT_TOPIC.to_string());
        let production = env_nonempty("APNS_PRODUCTION").is_some_and(|v| v == "true" || v == "1");
        Some(
            read_secret(&raw_key)
                .map_err(|e| format!("APNS_PRIVATE_KEY_P8: {e}"))
                .and_then(|pem| ApnsSender::new(ApnsConfig { key_id, team_id, topic, production }, &pem)),
        )
    }

    async fn provider_token(&self) -> Result<String, PushError> {
        let mut cached = self.jwt.lock().await;
        let now = Instant::now();
        if let Some(token) = cached.as_ref().and_then(|t| t.current(now)) {
            return Ok(token.to_string());
        }
        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.config.key_id.clone());
        let claims = ApnsJwtClaims { iss: &self.config.team_id, iat: chrono::Utc::now().timestamp() };
        let jwt = jsonwebtoken::encode(&header, &claims, &self.key).map_err(|e| PushError::Failed(format!("signing the APNs token: {e}")))?;
        *cached = Some(CachedToken { value: jwt.clone(), use_until: apns_jwt_use_until(now) });
        Ok(jwt)
    }
}

#[derive(Deserialize)]
struct ApnsErrorBody {
    reason: Option<String>,
}

#[async_trait::async_trait]
impl PushSender for ApnsSender {
    async fn send(&self, device: &DeviceTarget, message: &PushMessage) -> Result<(), PushError> {
        let request = apns_request(&self.config, &device.token, message, chrono::Utc::now().timestamp());
        let jwt = self.provider_token().await?;
        let response = self
            .http
            .post(&request.url)
            .header("authorization", format!("bearer {jwt}"))
            .header("apns-push-type", request.push_type)
            .header("apns-topic", &request.topic)
            .header("apns-priority", APNS_PRIORITY_IMMEDIATE)
            .json(&request.payload)
            .send()
            .await
            .map_err(|e| PushError::Failed(format!("APNs: {e}")))?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let reason = response.json::<ApnsErrorBody>().await.ok().and_then(|b| b.reason);
        if reason.as_deref() == Some(APNS_EXPIRED_PROVIDER_TOKEN) {
            // Sign a fresh one next time.
            *self.jwt.lock().await = None;
        }
        if apns_rejects_token(status.as_u16(), reason.as_deref()) {
            Err(PushError::Invalid)
        } else {
            Err(PushError::Failed(format!("APNs {status}: {}", reason.unwrap_or_default())))
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Both providers
// ------------------------------------------------------------------------------------------------

/// Routes each push to FCM (Android) or APNs (iOS), whichever is set up.
pub struct ProviderPush {
    pub fcm: Option<FcmSender>,
    pub apns: Option<ApnsSender>,
}

#[async_trait::async_trait]
impl PushSender for ProviderPush {
    async fn send(&self, device: &DeviceTarget, message: &PushMessage) -> Result<(), PushError> {
        match (device.platform.as_str(), &self.fcm, &self.apns) {
            (PLATFORM_ANDROID, Some(fcm), _) => fcm.send(device, message).await,
            (PLATFORM_IOS, _, Some(apns)) => apns.send(device, message).await,
            (platform, _, _) => Err(PushError::Unsupported(platform.to_string())),
        }
    }
}

/// The push sender the environment sets up: FCM and/or APNs, or `NoopPush` when neither is
/// configured (or both fail to load, which is logged).
pub fn from_env(http: reqwest::Client) -> Arc<dyn PushSender> {
    let fcm = match FcmSender::from_env(http) {
        Some(Ok(fcm)) => Some(fcm),
        Some(Err(e)) => {
            tracing::warn!(error = %e, "FCM push not set up");
            None
        }
        None => None,
    };
    let apns = match ApnsSender::from_env() {
        Some(Ok(apns)) => Some(apns),
        Some(Err(e)) => {
            tracing::warn!(error = %e, "APNs push not set up");
            None
        }
        None => None,
    };
    tracing::info!(fcm = fcm.is_some(), apns = apns.is_some(), "push providers");
    if fcm.is_none() && apns.is_none() {
        return Arc::new(NoopPush);
    }
    Arc::new(ProviderPush { fcm, apns })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notifications::Kind;

    fn chat_reply() -> PushMessage {
        let session_id = Uuid::nil();
        let message_id = Uuid::max();
        PushMessage::chat_reply(Some("Trip to Bali".into()), session_id, message_id, "Here's the plan")
    }

    #[test]
    fn the_fcm_body_carries_the_notification_data_and_high_priority() {
        let body = fcm_message_body("fcm-token", &chat_reply());
        assert_eq!(
            body,
            json!({
                "message": {
                    "token": "fcm-token",
                    "notification": { "title": "Trip to Bali", "body": "Here's the plan" },
                    "data": {
                        "kind": "chat_reply",
                        "message_id": Uuid::max().to_string(),
                        "session_id": Uuid::nil().to_string(),
                    },
                    "android": { "priority": "HIGH" },
                }
            })
        );
        assert_eq!(fcm_send_url("nomi-prod"), "https://fcm.googleapis.com/v1/projects/nomi-prod/messages:send");
    }

    #[test]
    fn fcm_reports_only_unregistered_tokens_as_gone() {
        let unregistered = json!({ "error": { "code": 404, "status": "NOT_FOUND", "details": [
            { "@type": "type.googleapis.com/google.firebase.fcm.v1.FcmError", "errorCode": "UNREGISTERED" }
        ] } });
        let quota = json!({ "error": { "code": 429, "details": [
            { "@type": "type.googleapis.com/google.firebase.fcm.v1.FcmError", "errorCode": "QUOTA_EXCEEDED" }
        ] } });
        assert!(fcm_rejects_token(&unregistered));
        assert!(!fcm_rejects_token(&quota));
        assert!(!fcm_rejects_token(&Value::Null));
    }

    fn apns_config(production: bool) -> ApnsConfig {
        ApnsConfig { key_id: "KEY123".into(), team_id: "TEAM456".into(), topic: "ai.nomi.app".into(), production }
    }

    #[test]
    fn an_apns_alert_has_title_body_and_sound_with_the_data_beside_it() {
        let request = apns_request(&apns_config(true), "abc", &chat_reply(), 1_700_000_000);
        assert_eq!(request.url, "https://api.push.apple.com/3/device/abc");
        assert_eq!(request.push_type, "alert");
        assert_eq!(request.topic, "ai.nomi.app");
        assert_eq!(request.payload["aps"], json!({ "alert": { "title": "Trip to Bali", "body": "Here's the plan" }, "sound": "default" }));
        assert_eq!(request.payload["kind"], "chat_reply");
        assert_eq!(request.payload["session_id"], Uuid::nil().to_string());
    }

    #[test]
    fn an_apns_live_activity_update_goes_to_the_liveactivity_topic_with_a_timestamp() {
        let message = PushMessage::live_activity_update(json!({ "phase": "replied", "preview": "Done" }));
        let request = apns_request(&apns_config(false), "activity-token", &message, 1_700_000_000);
        assert_eq!(request.url, "https://api.sandbox.push.apple.com/3/device/activity-token");
        assert_eq!(request.push_type, "liveactivity");
        assert_eq!(request.topic, "ai.nomi.app.push-type.liveactivity");
        assert_eq!(
            request.payload,
            json!({ "aps": { "timestamp": 1_700_000_000, "event": "update", "content-state": { "phase": "replied", "preview": "Done" } } })
        );
    }

    #[test]
    fn an_apns_live_activity_start_names_the_attributes_type_and_alerts() {
        let message = PushMessage {
            title: "Nomi".into(),
            body: "Working on it".into(),
            data: BTreeMap::new(),
            kind: PushKind::LiveActivityStart { attributes: json!({ "sessionId": "s1" }), content_state: json!({ "phase": "thinking" }) },
        };
        let request = apns_request(&apns_config(true), "start-token", &message, 42);
        assert_eq!(request.push_type, "liveactivity");
        assert_eq!(
            request.payload["aps"],
            json!({
                "timestamp": 42,
                "event": "start",
                "content-state": { "phase": "thinking" },
                "attributes-type": "NomiActivityAttributes",
                "attributes": { "sessionId": "s1" },
                "alert": { "title": "Nomi", "body": "Working on it" },
            })
        );
    }

    #[test]
    fn apns_reports_gone_and_bad_tokens_only() {
        assert!(apns_rejects_token(410, Some("Unregistered")));
        assert!(apns_rejects_token(400, Some("BadDeviceToken")));
        assert!(!apns_rejects_token(400, Some("DeviceTokenNotForTopic")));
        assert!(!apns_rejects_token(403, Some("ExpiredProviderToken")));
        assert!(!apns_rejects_token(500, None));
    }

    #[test]
    fn an_fcm_access_token_is_replaced_a_minute_before_it_expires() {
        let obtained = Instant::now();
        let token = CachedToken { value: "ya29".into(), use_until: fcm_token_use_until(obtained, Duration::from_secs(3599)) };
        assert_eq!(token.current(obtained + Duration::from_secs(3538)), Some("ya29"));
        assert_eq!(token.current(obtained + Duration::from_secs(3539)), None);
        // A token Google says is shorter-lived than the margin is never reused.
        let short = CachedToken { value: "x".into(), use_until: fcm_token_use_until(obtained, Duration::from_secs(30)) };
        assert_eq!(short.current(obtained), None);
    }

    #[test]
    fn an_apns_provider_token_is_reused_for_fifty_minutes() {
        let signed = Instant::now();
        let token = CachedToken { value: "jwt".into(), use_until: apns_jwt_use_until(signed) };
        assert_eq!(token.current(signed + Duration::from_secs(49 * 60)), Some("jwt"));
        assert_eq!(token.current(signed + Duration::from_secs(50 * 60)), None);
    }

    #[test]
    fn a_google_assertion_asks_for_the_fcm_scope_for_an_hour() {
        let account = ServiceAccount { client_email: "push@nomi.iam.gserviceaccount.com".into(), private_key: String::new(), project_id: None, token_uri: None };
        assert_eq!(
            google_assertion_claims(&account, 1000),
            json!({
                "iss": "push@nomi.iam.gserviceaccount.com",
                "scope": "https://www.googleapis.com/auth/firebase.messaging",
                "aud": "https://oauth2.googleapis.com/token",
                "iat": 1000,
                "exp": 4600,
            })
        );
    }

    #[test]
    fn secrets_come_inline_or_from_a_file() {
        assert_eq!(read_secret(" {\"a\":\"b\\n\"} ").unwrap(), "{\"a\":\"b\\n\"}", "JSON escapes are left for the parser");
        assert_eq!(read_secret("-----BEGIN PRIVATE KEY-----\\nabc\\n-----END PRIVATE KEY-----").unwrap(), "-----BEGIN PRIVATE KEY-----\nabc\n-----END PRIVATE KEY-----");
        let path = std::env::temp_dir().join(format!("nomi-push-secret-{}", Uuid::new_v4()));
        std::fs::write(&path, "from a file").unwrap();
        assert_eq!(read_secret(path.to_str().unwrap()).unwrap(), "from a file");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn an_inbox_item_alerts_with_its_category_and_link_and_a_chat_falls_back_to_nomi() {
        let notice = Notice { kind: Kind::Quota, title: "Almost out".into(), body: "90% used".into(), link: Some("/billing".into()) };
        let message = PushMessage::alert(&notice);
        assert_eq!((message.title.as_str(), message.body.as_str(), &message.kind), ("Almost out", "90% used", &PushKind::Alert));
        assert_eq!(message.data.get("category").map(String::as_str), Some("quota"));
        assert_eq!(message.data.get("link").map(String::as_str), Some("/billing"));
        assert_eq!(message.data.get("kind").map(String::as_str), Some("notification"));

        assert_eq!(PushMessage::chat_reply(None, Uuid::nil(), Uuid::nil(), "hi").title, CHAT_REPLY_FALLBACK_TITLE);
        assert_eq!(PushMessage::chat_reply(Some("  ".into()), Uuid::nil(), Uuid::nil(), "hi").title, CHAT_REPLY_FALLBACK_TITLE);
    }
}
