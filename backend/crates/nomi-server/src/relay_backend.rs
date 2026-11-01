//! Making a project's own Relay on Shipyard: a second app, `<app>-backend`, with one environment
//! per Relay (`dev` when the project is made, `production` at its first publish), each running
//! Relay, PostgREST and, with functions, the functions runtime next to its own Postgres.
//!
//! The job is a list of steps that each check before they change anything, so it can be run
//! again after any failure (the **Try again** button) and picks up where it stopped. Its state
//! is the row in `project_relays`; the setup step it's on, or failed at, is written there too.

use std::time::Duration;

use nomi_relay_client::{Relay, RelayConfig, RelayError};
use nomi_settings::config;
use nomi_shipyard::manifest::{self, Images, BACKEND_SUFFIX, DEFAULT_POSTGREST_IMAGE, DEFAULT_RELAY_VERSION, secret_name, SECRET_CONTROL_KEY, SECRET_ENCRYPTION_KEY, SECRET_FUNCTIONS_SECRET, SECRET_PGRST_PASSWORD};
use nomi_shipyard::{Shipyard, ShipyardConfig};
use nomi_storage::ProjectStore;
use rand::RngCore;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::project_relays::{self, CREATING, FAILED, READY, SLEEPING, WAKING};

use crate::project_ops::ShipyardAccess;
use nomi_agent_core::relay_ops::RelayPackages;
use crate::relay_ops::{SECRET_ANON_KEY, SECRET_SERVICE_KEY, SECRET_URL};

pub const DEV: &str = "dev";
pub const PRODUCTION: &str = "production";

/// Where previews run (WebContainers): the origins a dev backend answers from the browser.
pub const DEFAULT_PREVIEW_ORIGINS: [&str; 1] = [".webcontainer-api.io"];

/// How often a publish or a starting Relay is asked about, and how long each may take.
pub const POLL_EVERY: Duration = Duration::from_secs(3);
const PUBLISH_WAIT: Duration = Duration::from_secs(600);
const HEALTH_WAIT: Duration = Duration::from_secs(180);
const CONTROL_KEY_BYTES: usize = 32;
/// A development backend nobody has used for this long is shut down (its data stays).
pub const DEV_IDLE_AFTER: Duration = Duration::from_secs(60 * 60);
/// How often Nomi looks for idle development backends.
pub const IDLE_CHECK_EVERY: Duration = Duration::from_secs(5 * 60);
/// How long a caller waits for a sleeping backend to wake before saying it's still waking.
pub const BACKEND_READY_WAIT: Duration = Duration::from_secs(120);
/// How long Koda's reads of a backend (its tables, its types) wait for a new one to finish being
/// set up: writes don't wait at all, they're sent to it once it's ready.
pub const BACKEND_SETUP_WAIT: Duration = Duration::from_secs(300);
const STEP_WAKE: &str = "waking the backend";
/// How much of an unexpected answer goes into an error.
const MAX_DETAIL_CHARS: usize = 300;

const STEP_IMAGES: &str = "finding Relay's images";
const STEP_APP: &str = "preparing the backend app";
const STEP_SECRETS: &str = "setting its secrets";
const STEP_DEPLOY: &str = "starting its database and Relay";
const STEP_WAIT: &str = "waiting for it to start";
const STEP_SETUP: &str = "setting it up";

/// What the job needs: Nomi's database, Shipyard, and the key that seals Relay's control keys.
pub struct Deps<'a> {
    pub pool: &'a PgPool,
    pub http: &'a reqwest::Client,
    pub shipyard: &'a ShipyardConfig,
    pub settings_key: &'a [u8; 32],
}

/// A step that went wrong: which, and what to tell the person.
struct Failed {
    step: &'static str,
    message: String,
}

fn fail<E: std::fmt::Display>(step: &'static str) -> impl Fn(E) -> Failed {
    move |e| Failed { step, message: e.to_string() }
}

fn clip(text: &str) -> String {
    text.chars().take(MAX_DETAIL_CHARS).collect()
}

/// A new control key: what Nomi calls this Relay with.
fn new_control_key() -> String {
    let mut bytes = [0u8; CONTROL_KEY_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Where Relay's images are, from admin's Configuration → Relay (or the environment). Relay's
/// and its functions runtime's repositories have no default: they're in the registry only the
/// person who runs Nomi knows.
pub async fn images(pool: &PgPool, settings_key: &[u8; 32]) -> Result<Images, String> {
    let section = config::load(pool, Some(settings_key), &config::RELAY).await.map_err(|e| e.to_string())?.with_env(|name| std::env::var(name).ok());
    let need = |field: &str| {
        section.get(field).ok_or_else(|| "Relay's images aren't set: add where they are under Admin → Configuration → Relay.".to_string())
    };
    Ok(Images {
        relay: need("relay_image")?.trim_end_matches('/').to_string(),
        functions: need("functions_image")?.trim_end_matches('/').to_string(),
        postgrest: section.get("postgrest_image").unwrap_or_else(|| DEFAULT_POSTGREST_IMAGE.to_string()),
        version: section.get("version").unwrap_or_else(|| DEFAULT_RELAY_VERSION.to_string()),
    })
}

/// The npm packages new projects use for Relay, as Admin → Configuration → Relay names them.
pub async fn packages(pool: &PgPool, settings_key: &[u8; 32]) -> RelayPackages {
    let defaults = RelayPackages::default();
    let Ok(section) = config::load(pool, Some(settings_key), &config::RELAY).await else { return defaults };
    let section = section.with_env(|name| std::env::var(name).ok());
    let pick = |field: &str, default: String| section.get(field).unwrap_or(default);
    RelayPackages {
        sdk: pick("sdk_package", defaults.sdk),
        sdk_version: pick("sdk_version", defaults.sdk_version),
        cli: pick("cli_package", defaults.cli),
        cli_version: pick("cli_version", defaults.cli_version),
    }
}

/// Whether Relay's images are set, so a backend can be made.
pub async fn images_set(pool: &PgPool, settings_key: &[u8; 32]) -> bool {
    images(pool, settings_key).await.is_ok()
}

/// The Shipyard app a project's backend lives in.
pub fn backend_app(app: &str) -> String {
    format!("{app}{BACKEND_SUFFIX}")
}

/// How long `save_settings` / `ensure_backend` retries while PostgREST or the functions runtime
/// finishes starting next to Relay.
const SETUP_RETRY_WAIT: Duration = Duration::from_secs(60);

fn active_setups() -> &'static std::sync::Mutex<std::collections::HashSet<(Uuid, String)>> {
    static ACTIVE: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<(Uuid, String)>>> = std::sync::OnceLock::new();
    ACTIVE.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()))
}

fn publish_notify() -> &'static tokio::sync::Notify {
    static NOTIFY: std::sync::OnceLock<tokio::sync::Notify> = std::sync::OnceLock::new();
    NOTIFY.get_or_init(tokio::sync::Notify::new)
}

struct ActiveGuard(Uuid, String);

impl ActiveGuard {
    fn enter(project_id: Uuid, env: &str) -> Self {
        let key = (project_id, env.to_string());
        if let Ok(mut set) = active_setups().lock() {
            set.insert(key.clone());
        }
        Self(key.0, key.1)
    }
}

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = active_setups().lock() {
            set.remove(&(self.0, self.1.clone()));
        }
    }
}

fn is_active_setup(project_id: Uuid, env: &str) -> bool {
    active_setups()
        .lock()
        .map(|set| set.contains(&(project_id, env.to_string())))
        .unwrap_or(false)
}

/// Sets up the project's `env` Relay, recording where it is and how it went. On a failure the
/// row says which step and why, and the project's admins are told. Safe to run again. Answers
/// whether it's ready.
pub async fn run(deps: &Deps<'_>, project_id: Uuid, env: &str, has_functions: bool) -> bool {
    let _guard = ActiveGuard::enter(project_id, env);
    finish_run(deps, project_id, env, set_up(deps, project_id, env, has_functions).await).await
}

async fn finish_run(deps: &Deps<'_>, project_id: Uuid, env: &str, outcome: Result<(), Failed>) -> bool {
    match outcome {
        Ok(()) => {
            if let Err(e) = project_relays::set_state(deps.pool, project_id, env, READY, None, None).await {
                tracing::error!(error = %e, %project_id, env, "couldn't mark a backend ready");
                return false;
            }
            tracing::info!(%project_id, env, "backend ready");
            true
        }
        Err(Failed { step, message }) => {
            tracing::warn!(%project_id, env, step, %message, "a backend couldn't be set up");
            if let Err(e) = project_relays::set_state(deps.pool, project_id, env, FAILED, Some(step), Some(&message)).await {
                tracing::error!(error = %e, %project_id, env, "couldn't record a backend failure");
            }
            crate::shipyard::notify_admins(deps.pool, project_id, "backend_setup_failed", &format!("It stopped while {step}: {message}.")).await;
            false
        }
    }
}

async fn at(deps: &Deps<'_>, project_id: Uuid, env: &str, step: &'static str) -> Result<(), Failed> {
    project_relays::set_state(deps.pool, project_id, env, CREATING, Some(step), None).await.map_err(fail(step))
}

async fn set_up(deps: &Deps<'_>, project_id: Uuid, env: &str, has_functions: bool) -> Result<(), Failed> {
    let client = Shipyard::new(deps.http, deps.shipyard);

    let images = match images(deps.pool, deps.settings_key).await {
        Ok(images) => images,
        Err(message) => {
            project_relays::begin(deps.pool, deps.settings_key, project_id, env, &new_control_key(), DEFAULT_RELAY_VERSION).await.map_err(fail(STEP_IMAGES))?;
            return Err(Failed { step: STEP_IMAGES, message });
        }
    };
    project_relays::begin(deps.pool, deps.settings_key, project_id, env, &new_control_key(), &images.version).await.map_err(fail(STEP_APP))?;
    at(deps, project_id, env, STEP_APP).await?;
    let target = crate::shipyard::load_target(deps.pool, project_id)
        .await
        .map_err(fail(STEP_APP))?
        .ok_or_else(|| Failed { step: STEP_APP, message: "the project no longer exists".to_string() })?;
    let app = crate::shipyard::ensure_app(deps.pool, &client, &target).await.map_err(|e| Failed { step: STEP_APP, message: format!("{e:?}") })?;
    let org = target.org_key();

    at(deps, project_id, env, STEP_SECRETS).await?;
    if !project_relays::secrets_made(deps.pool, project_id, env).await.map_err(fail(STEP_SECRETS))? {
        let control_key = project_relays::control_key(deps.pool, deps.settings_key, project_id, env)
            .await
            .map_err(fail(STEP_SECRETS))?
            .ok_or_else(|| Failed { step: STEP_SECRETS, message: "the backend's key is missing".to_string() })?;
        client.set_secret(&org, &app, &secret_name(SECRET_CONTROL_KEY, env), Some(&control_key), None).await.map_err(fail(STEP_SECRETS))?;
        // Made by Shipyard, once: Nomi never reads these back.
        for name in [SECRET_ENCRYPTION_KEY, SECRET_PGRST_PASSWORD, SECRET_FUNCTIONS_SECRET] {
            client.generate_secret(&org, &app, &secret_name(name, env), "hex32", None).await.map_err(fail(STEP_SECRETS))?;
        }
        project_relays::mark_secrets_made(deps.pool, project_id, env).await.map_err(fail(STEP_SECRETS))?;
    }

    at(deps, project_id, env, STEP_DEPLOY).await?;
    let base_domain = crate::configuration::apps_base_domain().await;
    let manifest = manifest::relay_backend_for_app(&app, &project_id.to_string(), env, has_functions, &images.version, base_domain.as_deref());
    let source = manifest::backend_source(&images, has_functions).map_err(fail(STEP_DEPLOY))?;
    let accepted = client.publish(&org, &app, env, &manifest, Some(source)).await.map_err(fail(STEP_DEPLOY))?;

    at(deps, project_id, env, STEP_WAIT).await?;
    let url = wait_for_publish(deps.pool, &client, project_id, &org, &app, env, &accepted.publish_id, accepted.url.clone())
        .await
        .map_err(fail(STEP_WAIT))?;
    configure_deployed_relay(deps, &client, &target, &app, &org, project_id, env, &url).await
}

#[allow(clippy::too_many_arguments)]
async fn configure_deployed_relay(
    deps: &Deps<'_>,
    client: &Shipyard<'_>,
    target: &crate::shipyard::ProjectTarget,
    app: &str,
    org: &str,
    project_id: Uuid,
    env: &str,
    url: &str,
) -> Result<(), Failed> {
    project_relays::set_url(deps.pool, project_id, env, url).await.map_err(fail(STEP_WAIT))?;
    let control_key = project_relays::control_key(deps.pool, deps.settings_key, project_id, env)
        .await
        .map_err(fail(STEP_WAIT))?
        .ok_or_else(|| Failed { step: STEP_WAIT, message: "the backend's key is missing".to_string() })?;
    let config = RelayConfig::from_values(Some(url), Some(&control_key)).ok_or_else(|| Failed { step: STEP_WAIT, message: "the backend has no address".to_string() })?;
    wait_for_relay(deps.http, &config).await.map_err(fail(STEP_WAIT))?;

    at(deps, project_id, env, STEP_SETUP).await?;
    let relay = Relay::new(deps.http, &config);
    let settings = relay_settings(deps, url, env).await.map_err(fail(STEP_SETUP))?;
    save_settings_with_retry(&relay, settings).await.map_err(|e| Failed { step: STEP_SETUP, message: describe(&e) })?;
    let made = ensure_backend_with_retry(&relay, project_id, &target.name).await.map_err(|e| Failed { step: STEP_SETUP, message: describe(&e) })?;
    if env == PRODUCTION {
        // The published app finds its backend through these, bound by name on every publish.
        let production = made.environment(PRODUCTION).ok_or_else(|| Failed { step: STEP_SETUP, message: "Relay answered without a production environment".to_string() })?;
        let keys = [(SECRET_URL, &production.url), (SECRET_ANON_KEY, &production.anon_key), (SECRET_SERVICE_KEY, &production.service_key)];
        for (name, value) in keys {
            client.set_secret(org, app, name, Some(value), None).await.map_err(fail(STEP_SETUP))?;
            sqlx::query(
                "INSERT INTO project_secrets (project_id, name, updated_by) VALUES ($1, $2, $3) \
                 ON CONFLICT (project_id, name) DO UPDATE SET updated_by = $3, updated_at = now()",
            )
            .bind(project_id)
            .bind(name)
            .bind(target.user_id)
            .execute(deps.pool)
            .await
            .map_err(fail(STEP_SETUP))?;
        }
    }
    Ok(())
}

async fn save_settings_with_retry(relay: &Relay<'_>, mut settings: Value) -> Result<Value, RelayError> {
    let deadline = tokio::time::Instant::now() + SETUP_RETRY_WAIT;
    loop {
        match relay.save_settings(&settings).await {
            Ok(saved) => return Ok(saved),
            Err(RelayError::Rejected { status, code, message, details }) => {
                let mut stripped = false;
                if message.contains("shared sender") && settings.get("shared_smtp").is_some() {
                    if let Some(obj) = settings.as_object_mut() {
                        obj.remove("shared_smtp");
                        stripped = true;
                    }
                }
                if message.contains("file storage") && settings.get("storage").is_some() {
                    if let Some(obj) = settings.as_object_mut() {
                        obj.remove("storage");
                        stripped = true;
                    }
                }
                if stripped {
                    tracing::warn!(%message, "Relay backend couldn't reach optional shared SMTP/storage; saving core settings without them");
                    continue;
                }
                let transient = message.contains("PostgREST") || message.contains("functions runtime") || message.contains("database");
                if !transient || tokio::time::Instant::now() >= deadline {
                    return Err(RelayError::Rejected { status, code, message, details });
                }
            }
            Err(RelayError::Unavailable(ref message)) => {
                if tokio::time::Instant::now() >= deadline {
                    return Err(RelayError::Unavailable(message.clone()));
                }
            }
            Err(other) => return Err(other),
        }
        tokio::time::sleep(POLL_EVERY).await;
    }
}

async fn ensure_backend_with_retry(
    relay: &Relay<'_>,
    project_id: Uuid,
    name: &str,
) -> Result<nomi_relay_client::Backend, RelayError> {
    let deadline = tokio::time::Instant::now() + SETUP_RETRY_WAIT;
    loop {
        match relay.ensure_backend(project_id, name).await {
            Ok(made) => return Ok(made),
            Err(RelayError::Unavailable(message)) if tokio::time::Instant::now() < deadline => {
                tracing::debug!(%project_id, %message, "waiting to ensure backend on Relay");
            }
            Err(other) => return Err(other),
        }
        tokio::time::sleep(POLL_EVERY).await;
    }
}

fn describe(e: &RelayError) -> String {
    match e {
        RelayError::Rejected { message, .. } | RelayError::NotFound(message) | RelayError::Unavailable(message) | RelayError::Unauthorized(message) => clip(message),
    }
}

/// Resolves an environment's URL on Shipyard when a publish response or event didn't carry `urls`.
async fn resolve_env_url(client: &Shipyard<'_>, org: &str, app: &str, env: &str) -> Option<String> {
    let env_data = client.get_env(org, app, env).await.ok();
    if let Some(relay_url) = env_data
        .as_ref()
        .and_then(|d| d.get("services"))
        .and_then(Value::as_array)
        .and_then(|services| {
            services
                .iter()
                .find(|s| s.get("name").and_then(Value::as_str) == Some("relay"))
                .and_then(|s| {
                    s.get("url").and_then(Value::as_str).or_else(|| {
                        s.get("urls")
                            .and_then(Value::as_array)
                            .and_then(|u| u.first())
                            .and_then(Value::as_str)
                    })
                })
        })
        .filter(|u| !u.is_empty())
    {
        return Some(relay_url.to_string());
    }
    if let Ok(domain) = client.get_domain(org, app, env).await {
        if let Some(url) = domain.get("url").and_then(Value::as_str).filter(|u| !u.is_empty()) {
            return Some(url.to_string());
        }
    }
    let env_data = env_data?;
    env_data
        .get("url")
        .and_then(Value::as_str)
        .or_else(|| {
            env_data
                .get("last_publish")
                .and_then(|p| p.get("urls"))
                .and_then(Value::as_array)
                .and_then(|u| u.first())
                .and_then(Value::as_str)
        })
        .or_else(|| {
            env_data
                .get("services")
                .and_then(Value::as_array)
                .and_then(|services| {
                    services.iter().find_map(|s| {
                        s.get("url").and_then(Value::as_str).or_else(|| {
                            s.get("urls")
                                .and_then(Value::as_array)
                                .and_then(|u| u.first())
                                .and_then(Value::as_str)
                        })
                    })
                })
        })
        .filter(|u| !u.is_empty())
        .map(str::to_string)
}

/// Follows a publish until Shipyard says it succeeded or failed (via webhook or polling); answers
/// the environment's address.
#[allow(clippy::too_many_arguments)]
async fn wait_for_publish(
    pool: &PgPool,
    client: &Shipyard<'_>,
    project_id: Uuid,
    org: &str,
    app: &str,
    env: &str,
    publish_id: &str,
    accepted_url: Option<String>,
) -> Result<String, String> {
    let deadline = tokio::time::Instant::now() + PUBLISH_WAIT;
    loop {
        if let Ok(Some(stack)) = project_relays::stack(pool, project_id, env).await {
            if let Some(url) = stack.url.filter(|u| !u.is_empty()) {
                return Ok(url);
            }
            if stack.state == FAILED {
                return Err(stack.error.unwrap_or_else(|| "Shipyard stopped the deploy".to_string()));
            }
        }
        match client.get_publish(org, app, env, publish_id).await {
            Ok(publish) => match publish.get("status").and_then(Value::as_str).unwrap_or("running") {
                "succeeded" => {
                    let mut url = publish
                        .get("urls")
                        .and_then(Value::as_array)
                        .and_then(|urls| urls.first())
                        .and_then(Value::as_str)
                        .map(str::to_string)
                        .or_else(|| accepted_url.clone());
                    if url.is_none() {
                        url = resolve_env_url(client, org, app, env).await;
                    }
                    return url.ok_or_else(|| "Shipyard gave the backend no address".to_string());
                }
                "failed" => {
                    let why = match publish.get("error") {
                        Some(Value::String(s)) => s.clone(),
                        Some(Value::Null) | None => "Shipyard didn't say why".to_string(),
                        Some(other) => other.to_string(),
                    };
                    return Err(format!("Shipyard stopped the deploy: {}", clip(&why)));
                }
                _ => {}
            },
            // On Shipyard, an environment doesn't exist until its first deploy creates it, so
            // asking for the publish while the environment is still being provisioned can return
            // 404 NotFound or a transient Unavailable; keep waiting for it or for the webhook.
            Err(nomi_shipyard::ShipyardError::NotFound(_) | nomi_shipyard::ShipyardError::Unavailable(_)) => {
                tracing::debug!(%project_id, app, env, publish_id, "publish not visible yet on Shipyard; waiting");
            }
            Err(e) => return Err(format!("{e:?}")),
        }
        if tokio::time::Instant::now() >= deadline {
            return Err("it took longer than ten minutes to start".to_string());
        }
        let notified = publish_notify().notified();
        tokio::select! {
            _ = tokio::time::sleep(POLL_EVERY) => {}
            _ = notified => {}
        }
    }
}

/// Records the URL or failure from a Shipyard `publish.succeeded` / `publish.failed` event for
/// `<app>-backend` and wakes any running [`wait_for_publish`].
pub async fn record_publish_event(
    pool: &PgPool,
    client: Option<&Shipyard<'_>>,
    project_id: Uuid,
    event: &crate::shipyard::Event,
) {
    let env = event.environment.as_deref().unwrap_or(DEV);
    match event.kind.as_str() {
        "publish.succeeded" => {
            let mut url = event
                .data
                .get("urls")
                .and_then(Value::as_array)
                .and_then(|u| u.first())
                .and_then(Value::as_str)
                .filter(|u| !u.is_empty())
                .map(str::to_string);
            if url.is_none() {
                if let (Some(client), Some(event_app)) = (client, event.app.as_deref()) {
                    if let Ok(Some(target)) = crate::shipyard::load_target(pool, project_id).await {
                        url = resolve_env_url(client, &target.org_key(), event_app, env).await;
                    }
                }
            }
            if let Some(url) = url {
                let _ = project_relays::set_url(pool, project_id, env, &url).await;
            }
            publish_notify().notify_waiters();
        }
        "publish.failed" => {
            let why = crate::shipyard::error_text(&event.data).unwrap_or_else(|| "Shipyard didn't say why".to_string());
            if matches!(project_relays::stack(pool, project_id, env).await, Ok(Some(stack)) if stack.state != READY) {
                let message = format!("Shipyard stopped the deploy: {}", clip(&why));
                let _ = project_relays::set_state(pool, project_id, env, FAILED, Some(STEP_WAIT), Some(&message)).await;
            }
            publish_notify().notify_waiters();
        }
        _ => {}
    }
}

/// When Shipyard delivers `publish.succeeded` for `<app>-backend` and no in-process [`run`] is
/// actively waiting on it (for instance, polling failed earlier or Nomi restarted while Shipyard
/// was building), finishes configuring the Relay backend and syncing `nomi/` files.
pub async fn finish_on_event(
    pool: &PgPool,
    http: &reqwest::Client,
    shipyard: &ShipyardConfig,
    settings_key: &[u8; 32],
    project_storage: Option<ProjectStore>,
    project_id: Uuid,
    event: &crate::shipyard::Event,
) {
    if event.kind != "publish.succeeded" {
        return;
    }
    if !crate::shipyard::is_backend_event(pool, project_id, event).await.unwrap_or(false) {
        return;
    }
    let env = event.environment.as_deref().unwrap_or(DEV);
    if is_active_setup(project_id, env) {
        return;
    }
    let Ok(Some(stack)) = project_relays::stack(pool, project_id, env).await else {
        return;
    };
    if stack.state == READY {
        return;
    }
    let _guard = ActiveGuard::enter(project_id, env);
    let deps = Deps { pool, http, shipyard, settings_key };
    let outcome = async {
        at(&deps, project_id, env, STEP_WAIT).await?;
        let client = Shipyard::new(http, shipyard);
        let target = crate::shipyard::load_target(pool, project_id)
            .await
            .map_err(fail(STEP_WAIT))?
            .ok_or_else(|| Failed { step: STEP_WAIT, message: "the project no longer exists".to_string() })?;
        let app = target
            .shipyard_app
            .clone()
            .ok_or_else(|| Failed { step: STEP_WAIT, message: "the project has no app on Shipyard".to_string() })?;
        let org = target.org_key();
        let event_app = event.app.as_deref().unwrap_or(&app);
        let url = stack
            .url
            .filter(|u| !u.is_empty())
            .or_else(|| {
                event
                    .data
                    .get("urls")
                    .and_then(Value::as_array)
                    .and_then(|u| u.first())
                    .and_then(Value::as_str)
                    .filter(|u| !u.is_empty())
                    .map(str::to_string)
            });
        let url = match url {
            Some(url) => url,
            None => resolve_env_url(&client, &org, event_app, env)
                .await
                .ok_or_else(|| Failed { step: STEP_WAIT, message: "Shipyard gave the backend no address".to_string() })?,
        };
        configure_deployed_relay(&deps, &client, &target, &app, &org, project_id, env, &url).await
    }
    .await;
    if finish_run(&deps, project_id, env, outcome).await && env == DEV {
        if let Some(storage) = project_storage {
            send_project_files(pool.clone(), http.clone(), shipyard.clone(), *settings_key, storage, project_id).await;
        }
    }
}

/// Waits until the new Relay answers its health check.
async fn wait_for_relay(http: &reqwest::Client, config: &RelayConfig) -> Result<(), String> {
    let deadline = tokio::time::Instant::now() + HEALTH_WAIT;
    let url = format!("{}/health", config.control_url);
    loop {
        if http.get(&url).timeout(POLL_EVERY).send().await.map(|r| r.status().is_success()).unwrap_or(false) {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err("Relay didn't answer after it started".to_string());
        }
        tokio::time::sleep(POLL_EVERY).await;
    }
}

/// What Relay is told about itself: its own address, who may call it from a browser, and the
/// sign-in code sender and file storage from Nomi's Configuration, when those are set.
async fn relay_settings(deps: &Deps<'_>, url: &str, env: &str) -> Result<Value, String> {
    let host = reqwest::Url::parse(url).ok().and_then(|u| u.host_str().map(str::to_string)).ok_or_else(|| format!("{url} isn't an address"))?;
    let mut settings = json!({
        "public_domain": host,
        "postgrest_dev_url": "",
        "postgrest_production_url": "",
        "preview_origins": if env == DEV { json!(DEFAULT_PREVIEW_ORIGINS) } else { json!([]) },
    });
    let env_var = |name: &str| std::env::var(name).ok();
    let smtp = config::load(deps.pool, Some(deps.settings_key), &config::SMTP).await.map_err(|e| e.to_string())?.with_env(env_var);
    if let (Some(host), Some(from)) = (smtp.get("host"), smtp.get("from")) {
        settings["shared_smtp"] = json!({
            "host": host,
            "port": smtp.number("port").unwrap_or(587),
            "user": smtp.get("username").unwrap_or_default(),
            "password": smtp.get("password").unwrap_or_default(),
            "from": from,
            "security": smtp.get("security").unwrap_or_default(),
        });
    }
    let storage = config::load(deps.pool, Some(deps.settings_key), &config::STORAGE).await.map_err(|e| e.to_string())?.with_env(env_var);
    if let (Some(bucket), Some(key), Some(secret)) = (storage.get("bucket"), storage.get("access_key_id"), storage.get("secret_access_key")) {
        settings["storage"] = json!({
            "endpoint": storage.get("endpoint_url").unwrap_or_default(),
            "region": storage.get("region").unwrap_or_default(),
            "bucket": bucket,
            "access_key_id": key,
            "secret_access_key": secret,
            // Self-hosted S3 stores (an endpoint is set) serve `endpoint/bucket/key`.
            "path_style": storage.get("endpoint_url").is_some(),
        });
    }
    Ok(settings)
}

/// Starts the job for the project's `env` Relay in the background: records it as being made first,
/// so a call a moment later hears "still being set up" instead of reaching the shared Relay. A
/// development backend then gets what Koda wrote under `nomi/` while it was being made.
#[allow(clippy::too_many_arguments)]
pub async fn start(
    pool: PgPool,
    http: reqwest::Client,
    shipyard: ShipyardConfig,
    settings_key: [u8; 32],
    project_storage: ProjectStore,
    project_id: Uuid,
    env: &'static str,
    has_functions: bool,
) -> Result<(), String> {
    project_relays::begin(&pool, &settings_key, project_id, env, &new_control_key(), DEFAULT_RELAY_VERSION).await?;
    tokio::spawn(async move {
        let ready = run(&Deps { pool: &pool, http: &http, shipyard: &shipyard, settings_key: &settings_key }, project_id, env, has_functions).await;
        if ready && env == DEV {
            send_project_files(pool, http, shipyard, settings_key, project_storage, project_id).await;
        }
    });
    Ok(())
}

/// Sends the project's `nomi/` (migrations, `backend.json`, functions) to its development backend,
/// as its owner. What fails stays for Koda: his next write there applies the migrations again
/// and says what's wrong.
async fn send_project_files(pool: PgPool, http: reqwest::Client, shipyard: ShipyardConfig, settings_key: [u8; 32], project_storage: ProjectStore, project_id: Uuid) {
    let owner: Option<Uuid> = match sqlx::query_scalar("SELECT user_id FROM projects WHERE id = $1").bind(project_id).fetch_optional(&pool).await {
        Ok(owner) => owner,
        Err(e) => {
            tracing::warn!(error = %e, %project_id, "couldn't read a project to send its files to its backend");
            return;
        }
    };
    let Some(owner) = owner else { return };
    let ops = crate::relay_ops::ServerRelayOps::stored(pool.clone(), http, Some(settings_key), RelayConfig::from_env()).with_shipyard(project_storage.clone(), shipyard);
    match nomi_agent_coding::relay::sync_to_dev(&pool, &project_storage, &ops, owner, project_id).await {
        Ok(said) => tracing::info!(%project_id, said = %said.join(" | "), "sent the project's nomi/ files to its new backend"),
        Err(e) => tracing::warn!(error = %e, %project_id, "couldn't send the project's nomi/ files to its new backend"),
    }
}

/// The org and app a project's Relay lives in on Shipyard.
async fn backend_target(pool: &PgPool, project_id: Uuid) -> Result<(String, String), String> {
    let target = crate::shipyard::load_target(pool, project_id).await.map_err(|e| e.to_string())?.ok_or("the project no longer exists")?;
    let app = target.shipyard_app.clone().ok_or("the project has no app on Shipyard")?;
    Ok((target.org_key(), app))
}

/// Shuts down the development backends nobody has used for `idle_after`: every service scaled to
/// nothing, the database's data kept. Answers how many it stopped. A Shipyard that can't scale
/// yet (`NotFound`) leaves them all running.
pub async fn sleep_idle(deps: &Deps<'_>, idle_after: Duration) -> Result<usize, String> {
    let client = Shipyard::new(deps.http, deps.shipyard);
    let mut stopped = 0;
    for project_id in project_relays::idle_dev(deps.pool, idle_after.as_secs() as i64).await? {
        let (org, backend) = backend_target(deps.pool, project_id).await?;
        match client.scale_env(&org, &backend, DEV, 0).await {
            Ok(_) => {
                project_relays::set_state(deps.pool, project_id, DEV, SLEEPING, None, None).await?;
                stopped += 1;
                tracing::info!(%project_id, "an idle development backend was shut down");
            }
            Err(nomi_shipyard::ShipyardError::NotFound(_)) => {
                return Err("Shipyard can't scale an environment yet, so idle backends keep running".to_string());
            }
            Err(e) => tracing::warn!(error = ?e, %project_id, "couldn't shut down an idle development backend"),
        }
    }
    Ok(stopped)
}

/// Checks for idle development backends every `IDLE_CHECK_EVERY`, for as long as Nomi runs.
pub async fn sleep_loop(pool: PgPool, http: reqwest::Client, shipyard: ShipyardConfig, settings_key: [u8; 32]) {
    let mut said_so = false;
    loop {
        tokio::time::sleep(IDLE_CHECK_EVERY).await;
        let deps = Deps { pool: &pool, http: &http, shipyard: &shipyard, settings_key: &settings_key };
        match sleep_idle(&deps, DEV_IDLE_AFTER).await {
            Ok(_) => said_so = false,
            // Said once, not every five minutes, until it works.
            Err(e) if !said_so => {
                tracing::warn!(error = %e, "idle development backends weren't shut down");
                said_so = true;
            }
            Err(_) => {}
        }
    }
}

/// Starts a sleeping development backend and waits for Relay to answer again. Only the caller
/// that wins the claim does the work; the rest just see `waking`.
async fn wake(deps: &Deps<'_>, project_id: Uuid) {
    let started = async {
        let (org, backend) = backend_target(deps.pool, project_id).await?;
        let client = Shipyard::new(deps.http, deps.shipyard);
        client.scale_env(&org, &backend, DEV, 1).await.map_err(|e| format!("{e:?}"))?;
        let stack = project_relays::stack(deps.pool, project_id, DEV).await?.ok_or("the backend is gone")?;
        let control_key = project_relays::control_key(deps.pool, deps.settings_key, project_id, DEV).await?.ok_or("the backend's key is missing")?;
        let config = RelayConfig::from_values(stack.url.as_deref(), Some(&control_key)).ok_or("the backend has no address")?;
        wait_for_relay(deps.http, &config).await
    }
    .await;
    let outcome = match started {
        Ok(()) => project_relays::set_state(deps.pool, project_id, DEV, READY, None, None).await,
        Err(message) => project_relays::set_state(deps.pool, project_id, DEV, FAILED, Some(STEP_WAKE), Some(&message)).await,
    };
    if let Err(e) = outcome {
        tracing::error!(error = %e, %project_id, "couldn't record a backend waking");
    }
}

/// Wakes the project's development backend if it's asleep, without waiting: the Relay sheet and
/// the preview show it waking.
pub fn wake_in_background(pool: PgPool, http: reqwest::Client, shipyard: ShipyardConfig, settings_key: [u8; 32], project_id: Uuid) {
    tokio::spawn(async move {
        if matches!(project_relays::claim_wake(&pool, project_id, DEV).await, Ok(true)) {
            wake(&Deps { pool: &pool, http: &http, shipyard: &shipyard, settings_key: &settings_key }, project_id).await;
        }
    });
}

/// Wakes the project's development backend if it's asleep and waits, up to `BACKEND_READY_WAIT`,
/// until it's ready (also while it's still being set up): for callers that can't show "waking"
/// (Koda's tools, the preview). `Ok` when it's ready
/// or has no sleeping backend; the reason in words otherwise.
pub async fn ensure_awake(deps: &Deps<'_>, project_id: Uuid) -> Result<(), String> {
    wait_until_ready(deps, project_id, BACKEND_READY_WAIT).await
}

/// `ensure_awake`, waiting up to `wait`.
pub async fn wait_until_ready(deps: &Deps<'_>, project_id: Uuid, wait: Duration) -> Result<(), String> {
    if project_relays::claim_wake(deps.pool, project_id, DEV).await? {
        wake(deps, project_id).await;
    }
    let deadline = tokio::time::Instant::now() + wait;
    loop {
        match project_relays::stack(deps.pool, project_id, DEV).await? {
            Some(stack) if stack.state == WAKING || stack.state == CREATING => {}
            Some(stack) if stack.state == FAILED => return Err(project_relays::unavailable(&stack)),
            _ => return Ok(()),
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(STILL_STARTING.to_string());
        }
        tokio::time::sleep(POLL_EVERY).await;
    }
}

/// What a backend call answers when the project's development backend is still being set up or
/// woken after `BACKEND_READY_WAIT`.
pub const STILL_STARTING: &str = "This project's backend is still starting (it's set up in the background and takes a few minutes).";

/// A backend still marked `creating` after this long was left by a Nomi that stopped mid-job.
const STALE_AFTER_SECS: i64 = 20 * 60;

/// Before a project's first publish to production: when the project has its own development
/// backend and no production one yet, sets the production backend up first, then publishes by
/// itself when it's ready. Answers what to tell the person when it did; `None` when the publish
/// can go ahead now (no Relay of its own, or its production backend is ready).
pub(crate) async fn prepare_for_publish(
    state: &impl ShipyardAccess,
    project_id: Uuid,
    user_id: Uuid,
    environment: &str,
    agent: Option<&str>,
) -> Result<Option<String>, String> {
    if environment != PRODUCTION {
        return Ok(None);
    }
    let pool = state.pool();
    let Some(settings_key) = state.settings_key() else { return Ok(None) };
    if project_relays::stack(pool, project_id, DEV).await?.is_none() {
        return Ok(None);
    }
    let fresh_creating = |stack: &project_relays::Stack| stack.state == CREATING;
    if let Some(stack) = project_relays::stack(pool, project_id, PRODUCTION).await? {
        if stack.state == READY {
            return Ok(None);
        }
        let age: Option<i64> = sqlx::query_scalar("SELECT extract(epoch FROM now() - updated_at)::bigint FROM project_relays WHERE project_id = $1 AND env = $2")
            .bind(project_id)
            .bind(PRODUCTION)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
        if fresh_creating(&stack) && age.unwrap_or(i64::MAX) < STALE_AFTER_SECS {
            return Ok(Some(WAITING.to_string()));
        }
    }
    let handle = crate::relay_ops::ShipyardHandle::of(state).ok_or_else(|| "publishing isn't set up on this Nomi".to_string())?;
    let files = crate::routes::deploy::project_files(state, project_id).await.map_err(|(_, message)| message)?;
    let has_functions = files.iter().any(|(path, _)| path.starts_with("nomi/functions/"));
    project_relays::begin(pool, &settings_key, project_id, PRODUCTION, &new_control_key(), DEFAULT_RELAY_VERSION).await?;
    let agent = agent.map(str::to_string);
    tokio::spawn(async move {
        let deps = Deps { pool: handle.pool(), http: handle.http_client(), shipyard: &handle.config, settings_key: &settings_key };
        run(&deps, project_id, PRODUCTION, has_functions).await;
        let ready = matches!(project_relays::stack(deps.pool, project_id, PRODUCTION).await, Ok(Some(stack)) if stack.state == READY);
        if !ready {
            return; // `run` already said why.
        }
        if let Err((_, message)) = crate::routes::deploy::start_publish(&handle, project_id, user_id, PRODUCTION, agent.as_deref()).await {
            tracing::warn!(%project_id, %message, "the publish after the backend was set up didn't start");
            crate::shipyard::notify_admins(deps.pool, project_id, "backend_failed", &message).await;
        }
    });
    Ok(Some(PREPARING.to_string()))
}

const PREPARING: &str = "Setting up this app's own database and backend first, which takes a few minutes. Publishing starts by itself when it's ready.";
const WAITING: &str = "This app's backend is still being set up. Publishing starts by itself when it's ready.";

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn save_settings_strips_unreachable_optional_storage_and_smtp_without_failing() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/control/settings"))
            .respond_with(ResponseTemplate::new(422).set_body_json(json!({
                "data": null,
                "error": { "code": "SETTINGS_INVALID", "message": "These didn't work: shared sender, file storage" }
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("PUT"))
            .and(path("/control/settings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": { "configured": true }, "error": null })))
            .mount(&server)
            .await;

        let http = reqwest::Client::new();
        let config = RelayConfig::from_values(Some(&server.uri()), Some("key")).unwrap();
        let relay = Relay::new(&http, &config);
        let settings = json!({
            "public_domain": "backend.apps.test",
            "postgrest_dev_url": "",
            "postgrest_production_url": "",
            "preview_origins": [".webcontainer-api.io"],
            "shared_smtp": { "host": "localhost", "from": "noreply@test" },
            "storage": { "bucket": "local", "endpoint": "http://localhost:9000" }
        });

        let saved = save_settings_with_retry(&relay, settings).await.unwrap();
        assert_eq!(saved["configured"], true);
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
        let second: Value = serde_json::from_slice(&requests[1].body).unwrap();
        assert!(second.get("shared_smtp").is_none());
        assert!(second.get("storage").is_none());
    }

    #[tokio::test]
    async fn resolve_env_url_falls_back_to_domain_or_env_when_publish_omits_urls() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/orgs/org-1/apps/todo-backend/envs/dev/domain"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": { "url": "https://todo-backend-dev.apps.test" },
                "error": null
            })))
            .mount(&server)
            .await;

        let http = reqwest::Client::new();
        let cfg = ShipyardConfig { base_url: server.uri(), key: "k".into(), reveal_key: None, webhook_secret: None };
        let client = Shipyard::new(&http, &cfg);
        let resolved = resolve_env_url(&client, "org-1", "todo-backend", "dev").await;
        assert_eq!(resolved.as_deref(), Some("https://todo-backend-dev.apps.test"));
    }
}
