//! Sora is in everyone's crew unless an admin switches her off in Admin → Agents; a member that
//! is off is in nobody's crew, and the rest of the crew is as it was.

mod support;

use axum::http::StatusCode;
use nomi_agent_core::crew::DESIGN_AGENT_TYPE;
use nomi_agent_core::personal_crew;
use serde_json::{json, Value};
use sqlx::PgPool;
use support::{App, Person};

/// The crew switches are read through the process-wide settings cache and pool, so the tests
/// that set them take turns.
static GLOBALS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn settings(pool: &PgPool) {
    nomi_settings::config::install(pool.clone());
    nomi_settings::config::forget();
}

async fn crew_types(app: &App, who: &Person) -> Vec<String> {
    let (status, body) = app.get("/api/agents", who).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.get("agents").unwrap_or(&body).as_array().cloned().unwrap_or_default();
    list.iter().filter_map(|a: &Value| a["agent_type"].as_str().map(str::to_string)).collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn sora_is_in_the_crew_with_nothing_saved(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    settings(&pool);
    let app = App::new(pool.clone());
    let person = app.person("ada@acme.test").await;

    let crew = crew_types(&app, &person).await;
    assert!(crew.iter().any(|t| t == DESIGN_AGENT_TYPE), "{crew:?}");
    let mut conn = pool.acquire().await.unwrap();
    assert!(personal_crew::is_enabled(&mut conn, person.id, DESIGN_AGENT_TYPE).await);
    assert!(!personal_crew::disabled_agent_types(&mut conn, person.id).await.contains(DESIGN_AGENT_TYPE));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_member_switched_off_for_everyone_is_in_nobodys_crew(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    settings(&pool);
    let app = App::new(pool.clone());
    let person = app.person("ada@acme.test").await;
    let other = app.person("bob@acme.test").await;

    nomi_settings::config::set_agent_on(&pool, DESIGN_AGENT_TYPE, false, person.id).await.unwrap();
    for who in [&person, &other] {
        let crew = crew_types(&app, who).await;
        assert!(!crew.iter().any(|t| t == DESIGN_AGENT_TYPE), "{crew:?}");
        assert!(crew.iter().any(|t| t == "planning") && crew.iter().any(|t| t == "coding"), "the rest of the crew is as it was");
    }
    let mut conn = pool.acquire().await.unwrap();
    assert!(!personal_crew::is_enabled(&mut conn, person.id, DESIGN_AGENT_TYPE).await);
    assert!(personal_crew::disabled_agent_types(&mut conn, other.id).await.contains(DESIGN_AGENT_TYPE));
    assert!(personal_crew::is_enabled(&mut conn, person.id, "planning").await);
    drop(conn);

    // Switched back on: everyone has her again, and nothing is left saved.
    nomi_settings::config::set_agent_on(&pool, DESIGN_AGENT_TYPE, true, person.id).await.unwrap();
    assert!(crew_types(&app, &other).await.iter().any(|t| t == DESIGN_AGENT_TYPE));
    let saved: i64 = sqlx::query_scalar("SELECT count(*) FROM app_config WHERE section = 'crew'").fetch_one(&pool).await.unwrap();
    assert_eq!(saved, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_an_admin_switches_members_and_nomi_stays_on(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    settings(&pool);
    let app = App::new(pool.clone());
    let person = app.person("ada@acme.test").await;

    let (status, _) = app.get("/api/admin/crew", &person).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "not an admin");
    let (status, _) = app.send("PUT", "/api/admin/crew/design", &person, json!({"enabled": false})).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(nomi_settings::config::agent_on(DESIGN_AGENT_TYPE).await, "nothing changed");

    let admin = app.admin("admin@acme.test").await;
    let (status, body) = app.get("/api/admin/crew", &admin).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let listed = body["agents"].as_array().unwrap();
    assert!(listed[0]["locked"].as_bool().unwrap() && listed.iter().all(|a| a["enabled"].as_bool().unwrap()), "{body}");
    assert!(listed.iter().any(|a| a["agent_type"] == DESIGN_AGENT_TYPE && a["name"] == "Sora"));

    let (status, body) = app.send("PUT", "/api/admin/crew/design", &admin, json!({"enabled": false})).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["agents"].as_array().unwrap().iter().any(|a| a["agent_type"] == DESIGN_AGENT_TYPE && a["enabled"] == false));
    assert!(!crew_types(&app, &person).await.iter().any(|t| t == DESIGN_AGENT_TYPE));

    let nomi = listed[0]["agent_type"].as_str().unwrap();
    assert_eq!(app.send("PUT", &format!("/api/admin/crew/{nomi}"), &admin, json!({"enabled": false})).await.0, StatusCode::CONFLICT);
    assert_eq!(app.send("PUT", "/api/admin/crew/made_up", &admin, json!({"enabled": false})).await.0, StatusCode::NOT_FOUND);
    nomi_settings::config::set_agent_on(&pool, DESIGN_AGENT_TYPE, true, admin.id).await.unwrap();
}
