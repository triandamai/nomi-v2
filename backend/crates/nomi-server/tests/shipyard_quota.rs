//! Integration tests for syncing Nomi organization project quotas to Shipyard on org creation,
//! plan upgrades, and admin plan limit updates.

use nomi_server::shipyard::{sync_org, sync_plan_orgs_quota, sync_user_orgs_quota};
use nomi_shipyard::{Shipyard, ShipyardConfig};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn ok() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": {}, "error": null }))
}

fn config(server: &MockServer) -> ShipyardConfig {
    ShipyardConfig { base_url: server.uri(), key: "manage".into(), reveal_key: None, webhook_secret: None }
}

#[sqlx::test(migrations = "../../migrations")]
async fn sync_org_and_user_and_plan_quotas_hit_shipyard_with_expected_limits(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let org_a: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, created_by) VALUES ('Acme', $1) RETURNING id")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let org_b: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, created_by) VALUES ('Beta', $1) RETURNING id")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    for org in [org_a, org_b] {
        sqlx::query("INSERT INTO memberships (org_id, user_id, role, status) VALUES ($1, $2, 'owner', 'active')")
            .bind(org)
            .bind(user_id)
            .execute(&pool)
            .await
            .unwrap();
    }

    // Put the user on a plan with max_projects = 5, max_org_members = 10, max_orgs = 3.
    let pro_plan_id: Uuid = sqlx::query_scalar(
        "INSERT INTO plans (slug, name, monthly_tokens, max_projects, max_org_members, max_orgs) \
         VALUES ('pro-quota-test', 'Pro', 1000000, 5, 10, 3) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO user_subscriptions (user_id, plan_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(pro_plan_id)
        .execute(&pool)
        .await
        .unwrap();

    let server = MockServer::start().await;
    let cfg = config(&server);
    let http = reqwest::Client::new();
    let client = Shipyard::new(&http, &cfg);

    // 1. sync_org provisions org_a with its name and owner's quota.
    Mock::given(method("PUT"))
        .and(path(format!("/orgs/{org_a}")))
        .and(body_json(json!({
            "name": "Acme",
            "quota": { "max_projects": 5, "max_members": 10, "max_orgs": 3 }
        })))
        .respond_with(ok())
        .expect(1)
        .mount(&server)
        .await;
    assert!(sync_org(&pool, &client, org_a).await.unwrap());
    assert!(!sync_org(&pool, &client, Uuid::new_v4()).await.unwrap());

    // 2. Upgrade user to an unlimited plan: already-synced org_a gets PUT /orgs/{org_a}/quota,
    //    and unsynced org_b gets PUT /orgs/{org_b} with its name and new quota.
    let max_plan_id: Uuid = sqlx::query_scalar(
        "INSERT INTO plans (slug, name, monthly_tokens, max_projects, max_org_members, max_orgs) \
         VALUES ('max-quota-test', 'Max', 5000000, NULL, NULL, NULL) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE user_subscriptions SET plan_id = $2 WHERE user_id = $1")
        .bind(user_id)
        .bind(max_plan_id)
        .execute(&pool)
        .await
        .unwrap();

    Mock::given(method("PUT"))
        .and(path(format!("/orgs/{org_a}/quota")))
        .and(body_json(json!({
            "quota": { "max_projects": -1, "max_members": -1, "max_orgs": -1 }
        })))
        .respond_with(ok())
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path(format!("/orgs/{org_b}")))
        .and(body_json(json!({
            "name": "Beta",
            "quota": { "max_projects": -1, "max_members": -1, "max_orgs": -1 }
        })))
        .respond_with(ok())
        .expect(1)
        .mount(&server)
        .await;
    sync_user_orgs_quota(&pool, &client, user_id).await;

    // 3. Admin updates max_plan_id to max_projects = 20: both synced orgs receive PUT /orgs/:org/quota.
    sqlx::query("UPDATE plans SET max_projects = 20 WHERE id = $1").bind(max_plan_id).execute(&pool).await.unwrap();
    for org in [org_a, org_b] {
        Mock::given(method("PUT"))
            .and(path(format!("/orgs/{org}/quota")))
            .and(body_json(json!({
                "quota": { "max_projects": 20, "max_members": -1, "max_orgs": -1 }
            })))
            .respond_with(ok())
            .expect(1)
            .mount(&server)
            .await;
    }
    sync_plan_orgs_quota(&pool, &client, max_plan_id).await;
}
