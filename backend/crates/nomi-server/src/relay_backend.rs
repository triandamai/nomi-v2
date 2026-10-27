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
use nomi_shipyard::manifest::{self, Images, BACKEND_SUFFIX, DEFAULT_POSTGREST_IMAGE, DEFAULT_RELAY_VERSION, SECRET_CONTROL_KEY, SECRET_ENCRYPTION_KEY, SECRET_FUNCTIONS_SECRET, SECRET_PGRST_PASSWORD};
use nomi_shipyard::{Shipyard, ShipyardConfig};
use rand::RngCore;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::project_relays::{self, CREATING, FAILED, READY, SLEEPING, WAKING};

use crate::project_ops::ShipyardAccess;
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

/// Whether Relay's images are set, so a backend can be made.
pub async fn images_set(pool: &PgPool, settings_key: &[u8; 32]) -> bool {
    images(pool, settings_key).await.is_ok()
}

/// The Shipyard app a project's backend lives in.
pub fn backend_app(app: &str) -> String {
    format!("{app}{BACKEND_SUFFIX}")
}

/// Sets up the project's `env` Relay, recording where it is and how it went. On a failure the
/// row says which step and why, and the project's admins are told. Safe to run again.
pub async fn run(deps: &Deps<'_>, project_id: Uuid, env: &str, has_functions: bool) {
    match set_up(deps, project_id, env, has_functions).await {
        Ok(()) => {
            if let Err(e) = project_relays::set_state(deps.pool, project_id, env, READY, None, None).await {
                tracing::error!(error = %e, %project_id, env, "couldn't mark a backend ready");
            }
            tracing::info!(%project_id, env, "backend ready");
        }
        Err(Failed { step, message }) => {
            tracing::warn!(%project_id, env, step, %message, "a backend couldn't be set up");
            if let Err(e) = project_relays::set_state(deps.pool, project_id, env, FAILED, Some(step), Some(&message)).await {
                tracing::error!(error = %e, %project_id, env, "couldn't record a backend failure");
            }
            crate::shipyard::notify_admins(deps.pool, project_id, "backend_setup_failed", &format!("It stopped while {step}: {message}.")).await;
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
    let backend = backend_app(&app);
    client.put_app(&org, &backend, &format!("{} backend", target.name), false).await.map_err(fail(STEP_APP))?;

    at(deps, project_id, env, STEP_SECRETS).await?;
    if !project_relays::secrets_made(deps.pool, project_id, env).await.map_err(fail(STEP_SECRETS))? {
        let control_key = project_relays::control_key(deps.pool, deps.settings_key, project_id, env)
            .await
            .map_err(fail(STEP_SECRETS))?
            .ok_or_else(|| Failed { step: STEP_SECRETS, message: "the backend's key is missing".to_string() })?;
        client.set_secret(&org, &backend, SECRET_CONTROL_KEY, Some(&control_key), Some(env)).await.map_err(fail(STEP_SECRETS))?;
        // Made by Shipyard, once: Nomi never reads these back.
        for name in [SECRET_ENCRYPTION_KEY, SECRET_PGRST_PASSWORD, SECRET_FUNCTIONS_SECRET] {
            client.generate_secret(&org, &backend, name, "hex32", env).await.map_err(fail(STEP_SECRETS))?;
        }
        project_relays::mark_secrets_made(deps.pool, project_id, env).await.map_err(fail(STEP_SECRETS))?;
    }

    at(deps, project_id, env, STEP_DEPLOY).await?;
    let manifest = manifest::relay_backend(&project_id.to_string(), env, has_functions, &images.version);
    let source = manifest::backend_source(&images, has_functions).map_err(fail(STEP_DEPLOY))?;
    let accepted = client.publish(&org, &backend, env, &manifest, Some(source)).await.map_err(fail(STEP_DEPLOY))?;

    at(deps, project_id, env, STEP_WAIT).await?;
    let url = wait_for_publish(&client, &org, &backend, env, &accepted.publish_id, accepted.url.clone()).await.map_err(fail(STEP_WAIT))?;
    project_relays::set_url(deps.pool, project_id, env, &url).await.map_err(fail(STEP_WAIT))?;
    let control_key = project_relays::control_key(deps.pool, deps.settings_key, project_id, env)
        .await
        .map_err(fail(STEP_WAIT))?
        .ok_or_else(|| Failed { step: STEP_WAIT, message: "the backend's key is missing".to_string() })?;
    let config = RelayConfig::from_values(Some(&url), Some(&control_key)).ok_or_else(|| Failed { step: STEP_WAIT, message: "the backend has no address".to_string() })?;
    wait_for_relay(deps.http, &config).await.map_err(fail(STEP_WAIT))?;

    at(deps, project_id, env, STEP_SETUP).await?;
    let relay = Relay::new(deps.http, &config);
    let settings = relay_settings(deps, &url, env).await.map_err(fail(STEP_SETUP))?;
    relay.save_settings(&settings).await.map_err(|e| Failed { step: STEP_SETUP, message: describe(&e) })?;
    let made = relay.ensure_backend(project_id, &target.name).await.map_err(|e| Failed { step: STEP_SETUP, message: describe(&e) })?;
    if env == PRODUCTION {
        // The published app finds its backend through these, bound by name on every publish.
        let production = made.environment(PRODUCTION).ok_or_else(|| Failed { step: STEP_SETUP, message: "Relay answered without a production environment".to_string() })?;
        let keys = [(SECRET_URL, &production.url), (SECRET_ANON_KEY, &production.anon_key), (SECRET_SERVICE_KEY, &production.service_key)];
        for (name, value) in keys {
            client.set_secret(&org, &app, name, Some(value), None).await.map_err(fail(STEP_SETUP))?;
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

fn describe(e: &RelayError) -> String {
    match e {
        RelayError::Rejected { message, .. } | RelayError::NotFound(message) | RelayError::Unavailable(message) | RelayError::Unauthorized(message) => clip(message),
    }
}

/// Follows a publish until Shipyard says it succeeded or failed; answers the environment's address.
async fn wait_for_publish(client: &Shipyard<'_>, org: &str, app: &str, env: &str, publish_id: &str, accepted_url: Option<String>) -> Result<String, String> {
    let deadline = tokio::time::Instant::now() + PUBLISH_WAIT;
    loop {
        let publish = client.get_publish(org, app, env, publish_id).await.map_err(|e| format!("{e:?}"))?;
        match publish.get("status").and_then(Value::as_str).unwrap_or("running") {
            "succeeded" => {
                let url = publish.get("urls").and_then(Value::as_array).and_then(|urls| urls.first()).and_then(Value::as_str).map(str::to_string).or(accepted_url);
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
        }
        if tokio::time::Instant::now() >= deadline {
            return Err("it took longer than ten minutes to start".to_string());
        }
        tokio::time::sleep(POLL_EVERY).await;
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
/// so a call a moment later hears "still being set up" instead of reaching the shared Relay.
pub async fn start(pool: PgPool, http: reqwest::Client, shipyard: ShipyardConfig, settings_key: [u8; 32], project_id: Uuid, env: &'static str, has_functions: bool) -> Result<(), String> {
    project_relays::begin(&pool, &settings_key, project_id, env, &new_control_key(), DEFAULT_RELAY_VERSION).await?;
    tokio::spawn(async move {
        run(&Deps { pool: &pool, http: &http, shipyard: &shipyard, settings_key: &settings_key }, project_id, env, has_functions).await;
    });
    Ok(())
}

/// The org and backend app a project's Relay lives in on Shipyard.
async fn backend_target(pool: &PgPool, project_id: Uuid) -> Result<(String, String), String> {
    let target = crate::shipyard::load_target(pool, project_id).await.map_err(|e| e.to_string())?.ok_or("the project no longer exists")?;
    let app = target.shipyard_app.clone().ok_or("the project has no app on Shipyard")?;
    Ok((target.org_key(), backend_app(&app)))
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
/// until it's ready: for callers that can't show "waking" (Koda's tools). `Ok` when it's ready
/// or has no sleeping backend; the reason in words otherwise.
pub async fn ensure_awake(deps: &Deps<'_>, project_id: Uuid) -> Result<(), String> {
    if project_relays::claim_wake(deps.pool, project_id, DEV).await? {
        wake(deps, project_id).await;
    }
    let deadline = tokio::time::Instant::now() + BACKEND_READY_WAIT;
    loop {
        match project_relays::stack(deps.pool, project_id, DEV).await? {
            Some(stack) if stack.state == WAKING => {}
            Some(stack) if stack.state == FAILED => return Err(stack.error.unwrap_or_else(|| "the backend couldn't be woken".to_string())),
            _ => return Ok(()),
        }
        if tokio::time::Instant::now() >= deadline {
            return Err("This project's backend is still waking up. Try again in a moment.".to_string());
        }
        tokio::time::sleep(POLL_EVERY).await;
    }
}

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
