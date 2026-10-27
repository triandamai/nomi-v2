//! Nomi's side of Shipyard: making a project's organization and app exist there, the webhook
//! endpoint Nomi registers for itself, and what Shipyard's events change in Nomi (a publish's
//! status, a note to the project's admins when something goes wrong, and, in the project's
//! chat, where the app is live or why its publish failed, so Koda can fix it).

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_shipyard::{Shipyard, ShipyardConfig, ShipyardError};

use crate::notifications::{self, Kind, Notice};

/// The path Shipyard posts its events to on Nomi's web app (forwarded to `/api/hooks/shipyard`).
pub const WEBHOOK_PATH: &str = "/hooks/shipyard";

/// What a project is on Shipyard.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProjectTarget {
    pub id: Uuid,
    pub name: String,
    pub stack: String,
    pub files_version: i64,
    pub org_id: Uuid,
    pub org_name: String,
    pub shipyard_app: Option<String>,
    pub user_id: Uuid,
}

impl ProjectTarget {
    /// The organization's id on Shipyard (Nomi's own id).
    pub fn org_key(&self) -> String {
        self.org_id.to_string()
    }
}

pub async fn load_target(pool: &PgPool, project_id: Uuid) -> Result<Option<ProjectTarget>, sqlx::Error> {
    sqlx::query_as(
        "SELECT p.id, p.name, p.stack, p.files_version, p.org_id, o.name AS org_name, p.shipyard_app, p.user_id \
         FROM projects p JOIN organizations o ON o.id = p.org_id WHERE p.id = $1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
}

/// Makes sure the project's organization and app exist on Shipyard and returns the app's key.
/// Both calls are idempotent, so this runs before every publish.
pub async fn ensure_app(pool: &PgPool, client: &Shipyard<'_>, target: &ProjectTarget) -> Result<String, EnsureError> {
    client.put_org(&target.org_key(), &target.org_name).await?;
    sqlx::query("UPDATE organizations SET shipyard_synced_at = now() WHERE id = $1")
        .bind(target.org_id)
        .execute(pool)
        .await?;
    let app = match &target.shipyard_app {
        Some(app) => app.clone(),
        None => nomi_shipyard::app_slug(&target.name, &target.id.simple().to_string()),
    };
    client.put_app(&target.org_key(), &app, &target.name, false).await?;
    if target.shipyard_app.is_none() {
        sqlx::query("UPDATE projects SET shipyard_app = $2 WHERE id = $1 AND shipyard_app IS NULL")
            .bind(target.id)
            .bind(&app)
            .execute(pool)
            .await?;
    }
    Ok(app)
}

#[derive(Debug)]
pub enum EnsureError {
    Shipyard(ShipyardError),
    Db(sqlx::Error),
}

impl From<ShipyardError> for EnsureError {
    fn from(e: ShipyardError) -> Self {
        EnsureError::Shipyard(e)
    }
}

impl From<sqlx::Error> for EnsureError {
    fn from(e: sqlx::Error) -> Self {
        EnsureError::Db(e)
    }
}

/// The secrets a Shipyard webhook may be signed with: SHIPYARD_WEBHOOK_SECRET, and the one Nomi
/// stored when it registered (or rotated) its own endpoint. Both count, so an endpoint Nomi
/// registers itself works even when the environment also names a secret.
pub async fn webhook_secrets(pool: &PgPool, config: &ShipyardConfig, settings_key: &[u8; 32]) -> Vec<String> {
    let stored: Option<Vec<u8>> = sqlx::query_scalar("SELECT secret_encrypted FROM shipyard_webhook WHERE singleton")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    config
        .webhook_secret
        .clone()
        .into_iter()
        .chain(stored.and_then(|bytes| nomi_settings::crypto::decrypt(settings_key, &bytes).ok()))
        .collect()
}

/// Nomi's own endpoint among Shipyard's (`GET /webhooks`): the one Nomi registered, else any
/// pointing at a `/hooks/shipyard` path (one set up by hand, e.g. with SHIPYARD_WEBHOOK_SECRET).
pub fn own_endpoint<'a>(endpoints: &'a serde_json::Value, registered_id: Option<&str>) -> Option<&'a serde_json::Value> {
    let list = endpoints.as_array()?;
    let str_of = |e: &serde_json::Value, key: &str| e.get(key).and_then(serde_json::Value::as_str).map(str::to_string);
    registered_id
        .and_then(|id| list.iter().find(|e| str_of(e, "id").as_deref() == Some(id)))
        .or_else(|| {
            list.iter().find(|e| {
                str_of(e, "url")
                    .and_then(|url| reqwest::Url::parse(&url).ok())
                    .is_some_and(|url| url.path().trim_end_matches('/').ends_with(WEBHOOK_PATH))
            })
        })
}

/// Keeps `secret` as the one for Nomi's own endpoint `endpoint_id` (registered, rotated or adopted).
pub async fn keep_webhook(pool: &PgPool, settings_key: &[u8; 32], endpoint_id: &str, url: &str, secret: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO shipyard_webhook (singleton, endpoint_id, url, secret_encrypted) VALUES (true, $1, $2, $3) \
         ON CONFLICT (singleton) DO UPDATE SET endpoint_id = $1, url = $2, secret_encrypted = $3, created_at = now()",
    )
    .bind(endpoint_id)
    .bind(url)
    .bind(nomi_settings::crypto::encrypt(settings_key, secret))
    .execute(pool)
    .await?;
    Ok(())
}

/// The registered endpoint, if Nomi registered one itself.
pub async fn registered_webhook(pool: &PgPool) -> Result<Option<(String, String, chrono::DateTime<chrono::Utc>)>, sqlx::Error> {
    sqlx::query_as("SELECT endpoint_id, url, created_at FROM shipyard_webhook WHERE singleton").fetch_optional(pool).await
}

/// Registers `<APP_URL>/hooks/shipyard` on Shipyard (replacing an endpoint Nomi registered
/// before) and keeps its secret, encrypted.
pub async fn register_webhook(pool: &PgPool, client: &Shipyard<'_>, settings_key: &[u8; 32], app_url: &str) -> Result<String, EnsureError> {
    let url = format!("{}{WEBHOOK_PATH}", app_url.trim_end_matches('/'));
    if let Some((old_id, _, _)) = registered_webhook(pool).await? {
        // Gone already is fine.
        let _ = client.delete_webhook(&old_id).await;
    }
    let (endpoint_id, secret) = client.create_webhook(&url).await?;
    keep_webhook(pool, settings_key, &endpoint_id, &url, &secret).await?;
    Ok(url)
}

/// At startup: when Shipyard has no endpoint for Nomi (never registered, or deleted there),
/// register one at `APP_URL`, so events arrive without anyone pressing "Catch up".
pub async fn register_webhook_on_start(pool: PgPool, http: reqwest::Client, config: ShipyardConfig, settings_key: [u8; 32]) {
    let client = Shipyard::new(&http, &config);
    let registered = match registered_webhook(&pool).await {
        Ok(registered) => registered.map(|(id, _, _)| id),
        Err(e) => {
            tracing::warn!(error = %e, "couldn't check the Shipyard webhook");
            return;
        }
    };
    match client.list_webhooks().await {
        Ok(endpoints) if own_endpoint(&endpoints, registered.as_deref()).is_some() => return,
        Ok(_) => {}
        Err(e) => {
            tracing::warn!(error = ?e, "couldn't list Shipyard's webhooks");
            return;
        }
    }
    let Some(app_url) = crate::public_url::app_url() else {
        tracing::warn!("Shipyard has no webhook for Nomi and APP_URL isn't set: set it so Nomi can register one");
        return;
    };
    match register_webhook(&pool, &client, &settings_key, &app_url).await {
        Ok(url) => tracing::info!(%url, "registered Nomi's Shipyard webhook"),
        Err(e) => tracing::warn!(error = ?e, "couldn't register Nomi's Shipyard webhook"),
    }
}

/// One webhook delivery, as Shipyard sends it.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Event {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub app: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(default)]
    pub service: Option<String>,
    #[serde(default)]
    pub data: Value,
}

/// Stores the event once (deliveries repeat). Returns its project when it's new and belongs
/// to one, for [`apply_event`].
pub async fn store_event(pool: &PgPool, event: &Event) -> Result<Option<Uuid>, sqlx::Error> {
    let project_id: Option<Uuid> = match &event.app {
        Some(app) => {
            let base = app.strip_suffix(nomi_shipyard::manifest::BACKEND_SUFFIX);
            sqlx::query_scalar("SELECT id FROM projects WHERE shipyard_app = $1 OR ($2::text IS NOT NULL AND shipyard_app = $2)")
                .bind(app)
                .bind(base)
                .fetch_optional(pool)
                .await?
        }
        None => None,
    };
    let inserted = sqlx::query(
        "INSERT INTO shipyard_events (id, type, project_id, environment, service, data, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (id) DO NOTHING",
    )
    .bind(&event.id)
    .bind(&event.kind)
    .bind(project_id)
    .bind(&event.environment)
    .bind(&event.service)
    .bind(&event.data)
    .bind(event.created_at)
    .execute(pool)
    .await?
    .rows_affected()
        > 0;
    Ok(if inserted { project_id } else { None })
}

/// Whether an event came from the project's `<app>-backend` Relay app on Shipyard rather than
/// its web app.
pub async fn is_backend_event(pool: &PgPool, project_id: Uuid, event: &Event) -> Result<bool, sqlx::Error> {
    let Some(event_app) = event.app.as_deref() else {
        return Ok(false);
    };
    if !event_app.ends_with(nomi_shipyard::manifest::BACKEND_SUFFIX) {
        return Ok(false);
    }
    let shipyard_app: Option<String> = sqlx::query_scalar("SELECT shipyard_app FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(pool)
        .await?
        .flatten();
    Ok(shipyard_app.as_deref().is_some_and(|app| crate::relay_backend::backend_app(app) == event_app))
}

/// The error text of a failed publish or deployment.
pub(crate) fn error_text(data: &Value) -> Option<String> {
    match data.get("error") {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(Value::Object(o)) => o.get("message").and_then(Value::as_str).map(str::to_string),
        _ => None,
    }
}

/// Where a failed publish broke, from its failed deployment (see `nomi_shipyard::DeploymentFailure`),
/// with what the app printed when it failed while starting.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct PublishFailure {
    pub service: String,
    pub deployment_id: String,
    pub step: String,
    pub phase: String,
    pub message: String,
    pub tail: Vec<String>,
    pub runtime_logs: Vec<String>,
}

/// Lines of the app's own output kept when it failed while starting.
const FAILURE_RUNTIME_LOG_LINES: u32 = 60;
/// Log lines quoted in the chat.
const FAILURE_LOG_LINES_IN_CHAT: usize = 30;

/// The failed deployment of a failed publish: from the event's `deployments`, or by asking for
/// the publish when the event didn't list them.
async fn publish_failure(pool: &PgPool, client: &Shipyard<'_>, project_id: Uuid, event: &Event) -> Option<PublishFailure> {
    let target = load_target(pool, project_id).await.ok()??;
    let app = target.shipyard_app.as_deref()?;
    let env = event.environment.as_deref()?;
    let mut deployments = event.data.get("deployments").and_then(Value::as_array).cloned().unwrap_or_default();
    if deployments.is_empty() {
        let publish_id = event.data.get("publish_id").and_then(Value::as_str)?;
        let publish = client.get_publish(&target.org_key(), app, env, publish_id).await.ok()?;
        deployments = publish.get("deployments").and_then(Value::as_array).cloned().unwrap_or_default();
    }
    let failed = deployments.iter().find(|d| d.get("status").and_then(Value::as_str) == Some("failed"))?;
    let service = failed.get("service").and_then(Value::as_str).unwrap_or_default().to_string();
    let deployment_id = failed.get("deployment_id").and_then(Value::as_str)?.to_string();
    let found = client.deployment_failure(&target.org_key(), app, env, &deployment_id).await.ok()??;
    // The image built but the app didn't come up: what it printed says why.
    let runtime_logs = if matches!(found.phase.as_str(), "deploy" | "finalize") && !service.is_empty() {
        client
            .service_logs(&target.org_key(), app, env, &service, FAILURE_RUNTIME_LOG_LINES, None)
            .await
            .ok()
            .map(|logs| nomi_shipyard::log_lines(&logs))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Some(PublishFailure { service, deployment_id, step: found.step, phase: found.phase, message: found.message, tail: found.tail, runtime_logs })
}

/// What the project's chat says about a failed publish: where it broke and the end of the log,
/// in the project owner's language. It's an ordinary message, so the crew reads it.
pub fn failure_chat_text(locale: nomi_i18n::Locale, environment: &str, why: &str, failure: Option<&PublishFailure>) -> String {
    let Some(failure) = failure else {
        return locale.tf("deploy.chat_failed_plain", &[("env", environment), ("message", why)]);
    };
    let phase = locale.t(&format!("deploy.phase_{}", failure.phase));
    let message = if failure.message.is_empty() { why } else { &failure.message };
    let mut text = locale.tf("deploy.chat_failed", &[("env", environment), ("phase", &phase), ("step", &failure.step), ("message", message)]);
    let quote = |title: String, lines: &[String]| {
        let shown = &lines[lines.len().saturating_sub(FAILURE_LOG_LINES_IN_CHAT)..];
        format!("\n\n{title}\n\n```\n{}\n```", shown.join("\n"))
    };
    if !failure.tail.is_empty() {
        text.push_str(&quote(locale.t("deploy.chat_failed_log"), &failure.tail));
    }
    if !failure.runtime_logs.is_empty() {
        text.push_str(&quote(locale.t("deploy.chat_runtime_log"), &failure.runtime_logs));
    }
    text.push_str("\n\n");
    text.push_str(&locale.t("deploy.chat_failed_still_up"));
    text
}

/// Posts into the project's chat, live, as Nomi; with next-step buttons when given.
async fn post_to_project_chat(pool: &PgPool, project_id: Uuid, text: &str, suggestions: Vec<String>) {
    let Ok(Some(session_id)) = sqlx::query_scalar::<_, Uuid>("SELECT session_id FROM projects WHERE id = $1").bind(project_id).fetch_optional(pool).await else {
        return;
    };
    let Ok(mut conn) = pool.acquire().await else { return };
    let message_id: Option<Uuid> = sqlx::query_scalar("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2) RETURNING id")
        .bind(session_id)
        .bind(text)
        .fetch_one(&mut *conn)
        .await
        .ok();
    let mqtt = crate::project_events::publisher();
    if let (Some(message_id), Some(mqtt)) = (message_id, mqtt) {
        let _ = mqtt.publish(session_id, &nomi_realtime::StreamEnvelope::MessageCreated { message_id }).await;
    }
    nomi_agent_core::suggestions::post_suggestions(&mut conn, mqtt, session_id, "Nomi", suggestions).await;
}

/// The project owner's language, for what Nomi says in the project's chat.
async fn project_locale(pool: &PgPool, project_id: Uuid) -> nomi_i18n::Locale {
    match sqlx::query_scalar::<_, Uuid>("SELECT user_id FROM projects WHERE id = $1").bind(project_id).fetch_optional(pool).await {
        Ok(Some(owner)) => person_locale(pool, owner).await,
        _ => nomi_i18n::Locale::from_code_or_default(None),
    }
}

/// What an event changes: a publish's status, a note to the admins when something broke, and
/// the outcome in the project's chat. `client` looks up why a publish failed.
pub async fn apply_event(pool: &PgPool, client: Option<&Shipyard<'_>>, project_id: Uuid, event: &Event) -> Result<(), sqlx::Error> {
    let publish_id = event.data.get("publish_id").and_then(Value::as_str);
    match event.kind.as_str() {
        "publish.succeeded" | "publish.failed" => {
            if is_backend_event(pool, project_id, event).await? {
                crate::relay_backend::record_publish_event(pool, client, project_id, event).await;
                crate::project_events::changed(project_id, &event.kind);
                return Ok(());
            }
            let status = if event.kind == "publish.succeeded" { "succeeded" } else { "failed" };
            let url = event.data.get("urls").and_then(Value::as_array).and_then(|u| u.first()).and_then(Value::as_str);
            if let Some(publish_id) = publish_id {
                // Events can arrive out of order: a finished publish stays finished.
                sqlx::query(
                    "UPDATE project_publishes SET status = $2, url = COALESCE($3, url), error = $4, finished_at = $5 \
                     WHERE shipyard_publish_id = $1 AND status = 'running'",
                )
                .bind(publish_id)
                .bind(status)
                .bind(url)
                .bind(error_text(&event.data))
                .bind(event.created_at)
                .execute(pool)
                .await?;
            }
            // The open pages update now, before the admins' notifications go out.
            crate::project_events::changed(project_id, &event.kind);
            let environment = event.environment.as_deref().unwrap_or("production");
            let locale = project_locale(pool, project_id).await;
            if status == "failed" {
                let why = error_text(&event.data).unwrap_or_default();
                let failure = match client {
                    Some(client) => publish_failure(pool, client, project_id, event).await,
                    None => None,
                };
                if let (Some(publish_id), Some(failure)) = (publish_id, &failure) {
                    sqlx::query("UPDATE project_publishes SET failure = $2 WHERE shipyard_publish_id = $1")
                        .bind(publish_id)
                        .bind(serde_json::to_value(failure).unwrap_or_default())
                        .execute(pool)
                        .await?;
                    crate::project_events::changed(project_id, "publish.failure");
                }
                let text = failure_chat_text(locale, environment, &why, failure.as_ref());
                post_to_project_chat(pool, project_id, &text, vec![locale.t("deploy.suggest_fix")]).await;
                notify_admins(pool, project_id, "publish_failed", &why).await;
            } else if let Some(url) = url {
                post_to_project_chat(pool, project_id, &locale.tf("deploy.chat_published", &[("env", environment), ("url", url)]), Vec::new()).await;
                notify_admins(pool, project_id, "published", url).await;
            }
        }
        "service.crashed" => {
            let code = event.data.get("exit_code").map(|c| c.to_string()).unwrap_or_default();
            if !alerted_recently(pool, project_id, &event.kind, &event.id).await? {
                notify_admins(pool, project_id, "crashed", &code).await;
            }
        }
        "service.resource_alert" => {
            let metric = event.data.get("metric").and_then(Value::as_str).unwrap_or("cpu");
            let value = event.data.get("value").and_then(Value::as_f64).map(|v| format!("{v:.0}%")).unwrap_or_default();
            if !alerted_recently(pool, project_id, &event.kind, &event.id).await? {
                notify_admins(pool, project_id, "resource_alert", &format!("{metric} {value}")).await;
            }
        }
        "service.error_rate_alert" => {
            let count = |key: &str| event.data.get(key).and_then(Value::as_u64).unwrap_or_default();
            let rate = event.data.get("error_rate").and_then(Value::as_f64).map(|v| format!("{v:.0}%")).unwrap_or_default();
            if !alerted_recently(pool, project_id, &event.kind, &event.id).await? {
                notify_admins(pool, project_id, "error_rate_alert", &format!("{} of {} ({rate})", count("errors_5xx"), count("requests"))).await;
            }
        }
        _ => {}
    }
    if !event.kind.starts_with("publish.") {
        crate::project_events::changed(project_id, &event.kind);
    }
    Ok(())
}

/// Whether the admins already heard about this kind of event in the last half hour.
async fn alerted_recently(pool: &PgPool, project_id: Uuid, kind: &str, this_event: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM shipyard_events WHERE project_id = $1 AND type = $2 AND id <> $3 \
         AND received_at > now() - interval '30 minutes')",
    )
    .bind(project_id)
    .bind(kind)
    .bind(this_event)
    .fetch_one(pool)
    .await
}

/// The project's admins: whoever started it, and everyone made an admin on it.
pub async fn project_admins(pool: &PgPool, project_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT user_id FROM projects WHERE id = $1 \
         UNION SELECT user_id FROM project_members WHERE project_id = $1 AND role = 'admin'",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// Tells each of the project's admins, in their language, by notification, push and email.
pub(crate) async fn notify_admins(pool: &PgPool, project_id: Uuid, what: &str, detail: &str) {
    let name: String = sqlx::query_scalar("SELECT name FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(pool)
        .await
        .unwrap_or_default();
    let admins = project_admins(pool, project_id).await.unwrap_or_default();
    for user_id in admins {
        let locale = person_locale(pool, user_id).await;
        let (title_key, body_key) = match what {
            "published" => ("deploy.published_title", "deploy.published_body"),
            "publish_failed" => ("deploy.publish_failed_title", "deploy.publish_failed_body"),
            "backend_failed" => ("deploy.publish_failed_title", "deploy.backend_failed_body"),
            "backend_setup_failed" => ("deploy.backend_setup_failed_title", "deploy.backend_setup_failed_body"),
            "crashed" => ("deploy.crashed_title", "deploy.crashed_body"),
            "error_rate_alert" => ("deploy.error_rate_title", "deploy.error_rate_body"),
            _ => ("deploy.resource_title", "deploy.resource_body"),
        };
        let title = locale.tf(title_key, &[("name", &name)]);
        let body = locale.tf(body_key, &[("detail", detail)]);
        let notice = Notice { kind: Kind::Deploy, title, body, link: Some(format!("/projects/{project_id}/app?sheet=deploy")) };
        if let Err(e) = notifications::send(pool, user_id, notice).await {
            tracing::warn!(error = %e, "couldn't send a deploy notification");
        }
    }
}

async fn person_locale(pool: &PgPool, user_id: Uuid) -> nomi_i18n::Locale {
    let language: Option<String> = sqlx::query_scalar("SELECT language FROM user_preferences WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    nomi_i18n::Locale::from_code_or_default(language.as_deref())
}

/// What one catch-up pass found.
#[derive(Debug, Default, serde::Serialize)]
pub struct CatchUp {
    pub orgs: usize,
    /// Events Nomi hadn't seen yet.
    pub new_events: usize,
    /// Organizations Shipyard couldn't be asked about.
    pub errors: Vec<String>,
}

/// Pulls each synced organization's event feed from where Nomi last got to, and records and
/// applies the events no webhook delivered. Safe to run any time: events are deduplicated.
pub async fn catch_up(pool: &PgPool, client: &Shipyard<'_>) -> Result<CatchUp, sqlx::Error> {
    let orgs: Vec<(Uuid, Option<String>)> = sqlx::query_as(
        "SELECT o.id, c.after_id FROM organizations o LEFT JOIN shipyard_event_cursors c ON c.org_id = o.id \
         WHERE o.shipyard_synced_at IS NOT NULL",
    )
    .fetch_all(pool)
    .await?;
    let mut result = CatchUp { orgs: orgs.len(), ..Default::default() };
    for (org_id, mut after) in orgs {
        // A few pages per pass; the next pass continues from the cursor.
        for _ in 0..5 {
            let page = match client.org_events(&org_id.to_string(), after.as_deref(), 200).await {
                Ok(page) => page,
                Err(ShipyardError::NotFound(_)) => break,
                Err(e) => {
                    result.errors.push(format!("{org_id}: {e}"));
                    break;
                }
            };
            let events = page.get("events").and_then(Value::as_array).cloned().unwrap_or_default();
            for raw in &events {
                let Ok(event) = serde_json::from_value::<Event>(raw.clone()) else { continue };
                let existed: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM shipyard_events WHERE id = $1)")
                    .bind(&event.id)
                    .fetch_one(pool)
                    .await?;
                if existed {
                    continue;
                }
                result.new_events += 1;
                // Old events are only recorded: a first catch-up shouldn't send stale alerts.
                let recent = event.created_at > chrono::Utc::now() - chrono::Duration::hours(1);
                if let Some(project_id) = store_event(pool, &event).await?.filter(|_| recent) {
                    if let Err(e) = apply_event(pool, Some(client), project_id, &event).await {
                        tracing::warn!(error = %e, event = %event.id, "failed to apply a caught-up Shipyard event");
                    }
                }
            }
            let next = page.get("next_after").and_then(Value::as_str).map(str::to_string);
            if next.is_some() && next != after {
                sqlx::query(
                    "INSERT INTO shipyard_event_cursors (org_id, after_id) VALUES ($1, $2) \
                     ON CONFLICT (org_id) DO UPDATE SET after_id = $2, updated_at = now()",
                )
                .bind(org_id)
                .bind(&next)
                .execute(pool)
                .await?;
                after = next;
            }
            if events.len() < 200 {
                break;
            }
        }
    }
    Ok(result)
}

/// Every ten minutes, [`catch_up`] — in case a webhook delivery never arrived.
pub async fn catch_up_loop(pool: PgPool, http: reqwest::Client, config: ShipyardConfig) {
    let client = Shipyard::new(&http, &config);
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(600));
    loop {
        tick.tick().await;
        match catch_up(&pool, &client).await {
            Ok(done) if done.new_events > 0 => tracing::info!(events = done.new_events, "caught up on Shipyard events"),
            Ok(_) => {}
            Err(e) => tracing::warn!(error = %e, "Shipyard catch-up failed"),
        }
    }
}

/// Keeps the organization's name on Shipyard in step with Nomi, when it's there.
pub async fn sync_org_name(pool: &PgPool, client: &Shipyard<'_>, org_id: Uuid) -> Result<bool, EnsureError> {
    let row: Option<(String, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as("SELECT name, shipyard_synced_at FROM organizations WHERE id = $1").bind(org_id).fetch_optional(pool).await?;
    let Some((name, Some(_))) = row else { return Ok(false) };
    client.put_org(&org_id.to_string(), &name).await?;
    sqlx::query("UPDATE organizations SET shipyard_synced_at = now() WHERE id = $1").bind(org_id).execute(pool).await?;
    Ok(true)
}

#[cfg(test)]
mod tests {

    #[test]
    fn a_failed_publish_reads_as_where_it_broke_and_the_log() {
        let failure = PublishFailure {
            service: "web".into(),
            deployment_id: "d1".into(),
            step: "build_image".into(),
            phase: "build".into(),
            message: "npm run build exited with code 1".into(),
            tail: vec!["src/routes/+page.svelte:12:5 'x' is not defined".into()],
            runtime_logs: vec![],
        };
        let text = failure_chat_text(nomi_i18n::Locale::En, "production", "web failed", Some(&failure));
        assert!(text.starts_with("Publishing to production failed while building (`build_image`): npm run build exited with code 1"), "{text}");
        assert!(text.contains("```\nsrc/routes/+page.svelte:12:5 'x' is not defined\n```"), "{text}");
        assert!(text.ends_with("The version that was live keeps running."), "{text}");
        assert_eq!(
            failure_chat_text(nomi_i18n::Locale::En, "staging", "missing secret STRIPE_KEY", None),
            "Publishing to staging failed: missing secret STRIPE_KEY"
        );
    }
    use super::*;
    use serde_json::json;

    #[test]
    fn nomis_endpoint_is_the_registered_one_else_one_at_its_hook_path() {
        let endpoints = json!([
            { "id": "a", "url": "https://ops.example/events" },
            { "id": "b", "url": "https://nomi.example/hooks/shipyard" },
            { "id": "c", "url": "https://nomi.example/api/hooks/shipyard/" },
        ]);
        assert_eq!(own_endpoint(&endpoints, Some("a")).unwrap()["id"], "a");
        assert_eq!(own_endpoint(&endpoints, Some("gone")).unwrap()["id"], "b");
        assert_eq!(own_endpoint(&endpoints, None).unwrap()["id"], "b");
        assert!(own_endpoint(&json!([{ "id": "a", "url": "https://ops.example/hooks/shipyard-x" }]), None).is_none());
        assert!(own_endpoint(&json!([]), None).is_none());
    }
}
