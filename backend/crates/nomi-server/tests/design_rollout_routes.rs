//! Sora is in a person's crew only where Admin → Configuration → Design rolls the design
//! workflow out to them; everyone else's crew is as it was.

mod support;

use std::collections::BTreeMap;

use axum::http::StatusCode;
use nomi_agent_core::crew::DESIGN_AGENT_TYPE;
use nomi_agent_core::personal_crew;
use serde_json::Value;
use sqlx::PgPool;
use support::{App, Person};
use uuid::Uuid;

/// The `design` setting is read through the process-wide settings cache and pool, so the tests
/// that set it take turns.
static GLOBALS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Saves the design section as the admin page would.
async fn save_design(pool: &PgPool, admin: Uuid, values: &[(&str, &str)]) {
    std::env::set_var("SETTINGS_ENCRYPTION_KEY", hex::encode(nomi_test_support::TEST_SETTINGS_KEY));
    nomi_settings::config::install(pool.clone());
    let values: BTreeMap<String, String> = values.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    nomi_settings::config::save(pool, &nomi_test_support::TEST_SETTINGS_KEY, &nomi_settings::config::DESIGN, &values, admin).await.unwrap();
    nomi_settings::config::forget();
}

async fn crew_types(app: &App, who: &Person) -> Vec<String> {
    let (status, body) = app.get("/api/agents", who).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.get("agents").unwrap_or(&body).as_array().cloned().unwrap_or_default();
    list.iter().filter_map(|a: &Value| a["agent_type"].as_str().map(str::to_string)).collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn sora_is_not_in_the_crew_until_design_is_switched_on(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let person = app.person("ada@acme.test").await;
    save_design(&pool, person.id, &[("enabled", "off")]).await;

    let crew = crew_types(&app, &person).await;
    assert!(!crew.iter().any(|t| t == DESIGN_AGENT_TYPE), "{crew:?}");
    assert!(crew.iter().any(|t| t == "planning") && crew.iter().any(|t| t == "coding"), "the rest of the crew is as it was");
    let mut conn = pool.acquire().await.unwrap();
    assert!(!personal_crew::is_enabled(&mut conn, person.id, DESIGN_AGENT_TYPE).await);
    assert!(personal_crew::disabled_agent_types(&mut conn, person.id).await.contains(DESIGN_AGENT_TYPE));
    assert!(personal_crew::is_enabled(&mut conn, person.id, "planning").await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn sora_joins_the_crew_of_the_people_design_is_rolled_out_to(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let chosen = app.person("ada@acme.test").await;
    let other = app.person("bob@acme.test").await;
    save_design(&pool, chosen.id, &[("enabled", "on"), ("rollout", "allowlist"), ("rollout_allowlist", &chosen.id.to_string())]).await;

    let crew = crew_types(&app, &chosen).await;
    let sora = crew.iter().position(|t| t == DESIGN_AGENT_TYPE);
    assert!(sora.is_some(), "{crew:?}");
    assert!(!crew_types(&app, &other).await.iter().any(|t| t == DESIGN_AGENT_TYPE));
    let mut conn = pool.acquire().await.unwrap();
    assert!(personal_crew::is_enabled(&mut conn, chosen.id, DESIGN_AGENT_TYPE).await);
    assert!(!personal_crew::disabled_agent_types(&mut conn, chosen.id).await.contains(DESIGN_AGENT_TYPE));
    assert!(!personal_crew::is_enabled(&mut conn, other.id, DESIGN_AGENT_TYPE).await);

    // Switched back off: nobody has her.
    save_design(&pool, chosen.id, &[("enabled", "off")]).await;
    assert!(!crew_types(&app, &chosen).await.iter().any(|t| t == DESIGN_AGENT_TYPE));
}
