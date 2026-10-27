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
use nomi_shipyard::manifest::{self, BACKEND_SUFFIX, RELAY_STACK_VERSION, SECRET_CONTROL_KEY, SECRET_ENCRYPTION_KEY, SECRET_FUNCTIONS_SECRET, SECRET_PGRST_PASSWORD};
use nomi_shipyard::{Shipyard, ShipyardConfig};
use rand::RngCore;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::project_relays::{self, CREATING, FAILED, READY};

pub const DEV: &str = "dev";
pub const PRODUCTION: &str = "production";

/// Where previews run (WebContainers): the origins a dev backend answers from the browser.
pub const DEFAULT_PREVIEW_ORIGINS: [&str; 1] = [".webcontainer-api.io"];

/// How often a publish or a starting Relay is asked about, and how long each may take.
pub const POLL_EVERY: Duration = Duration::from_secs(3);
const PUBLISH_WAIT: Duration = Duration::from_secs(600);
const HEALTH_WAIT: Duration = Duration::from_secs(180);
const CONTROL_KEY_BYTES: usize = 32;
/// How much of an unexpected answer goes into an error.
const MAX_DETAIL_CHARS: usize = 300;

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

    project_relays::begin(deps.pool, deps.settings_key, project_id, env, &new_control_key(), RELAY_STACK_VERSION).await.map_err(fail(STEP_APP))?;
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
    let manifest = manifest::relay_backend(&project_id.to_string(), env, has_functions);
    let source = manifest::backend_source(RELAY_STACK_VERSION, has_functions).map_err(fail(STEP_DEPLOY))?;
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
    relay.ensure_backend(project_id, &target.name).await.map_err(|e| Failed { step: STEP_SETUP, message: describe(&e) })?;
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
    project_relays::begin(&pool, &settings_key, project_id, env, &new_control_key(), RELAY_STACK_VERSION).await?;
    tokio::spawn(async move {
        run(&Deps { pool: &pool, http: &http, shipyard: &shipyard, settings_key: &settings_key }, project_id, env, has_functions).await;
    });
    Ok(())
}
