//! Renaming an organization: owners only, and the new name follows on Shipyard.

mod support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[sqlx::test(migrations = "../../migrations")]
async fn a_rename_reaches_shipyard(pool: PgPool) {
    let shipyard = MockServer::start().await;
    Mock::given(method("PUT")).and(path_regex(r"^/orgs/[^/]+$")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}, "error": null}))).mount(&shipyard).await;
    let mut state = support::state(pool.clone());
    state.shipyard = Some(nomi_shipyard::ShipyardConfig { base_url: shipyard.uri(), key: "manage".into(), reveal_key: None, webhook_secret: None });
    let app = App::with_state(pool.clone(), state);
    let owner = app.person("owner@acme.test").await;
    let other = app.person("other@acme.test").await;
    let org: uuid::Uuid = sqlx::query_scalar("SELECT org_id FROM memberships WHERE user_id = $1").bind(owner.id).fetch_one(&pool).await.unwrap();
    sqlx::query("UPDATE organizations SET shipyard_synced_at = now() WHERE id = $1").bind(org).execute(&pool).await.unwrap();

    assert!(app.send("PATCH", &format!("/api/orgs/{org}"), &other, json!({"name": "Mine"})).await.0.is_client_error());
    assert_eq!(app.send("PATCH", &format!("/api/orgs/{org}"), &owner, json!({"name": "  "})).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(app.send("PATCH", &format!("/api/orgs/{org}"), &owner, json!({"name": "Acme Bakery"})).await.0, StatusCode::NO_CONTENT);
    let name: String = sqlx::query_scalar("SELECT name FROM organizations WHERE id = $1").bind(org).fetch_one(&pool).await.unwrap();
    assert_eq!(name, "Acme Bakery");
    let mut synced = false;
    for _ in 0..50 {
        if !shipyard.received_requests().await.unwrap().is_empty() {
            synced = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
    }
    assert!(synced, "Shipyard gets the new name");
}
