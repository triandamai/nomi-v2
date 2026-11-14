//! A project's own domains: adding one gives the records to set, looking them up says which are
//! found, a verified one can be made primary, and it can be removed. DNS answers are the test's.

mod support;

use std::sync::{Arc, LazyLock};

use axum::http::StatusCode;
use nomi_server::dns::{RecordKind, StaticDns};
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

static DNS: LazyLock<Arc<StaticDns>> = LazyLock::new(|| {
    let dns = Arc::new(StaticDns::default());
    nomi_server::dns::install(dns.clone());
    dns
});

async fn publish_live(pool: &PgPool, project_id: Uuid, url: &str) {
    sqlx::query("UPDATE projects SET shipyard_app = $2 WHERE id = $1")
        .bind(project_id)
        .bind(format!("todo-{project_id}"))
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO project_publishes (project_id, environment, status, url, manifest) VALUES ($1, 'production', 'succeeded', $2, '{}')")
        .bind(project_id)
        .bind(url)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_domain_is_added_checked_made_primary_and_removed(pool: PgPool) {
    let dns = DNS.clone();
    let app = App::new(pool.clone());
    let owner = app.person("owner@domains.test").await;
    let other = app.person("other@domains.test").await;
    let (project, _) = app.project(&owner, "Todo").await;
    let base = format!("/api/projects/{project}/domains");

    // Not live yet: there's no address to point at.
    let (status, _) = app
        .send(
            "POST",
            &base,
            &owner,
            json!({"hostname": "shop.example.com"}),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);

    publish_live(&pool, project, "https://todo.apps.nomi.test").await;
    dns.set("todo.apps.nomi.test", RecordKind::A, &["203.0.113.7"]);

    assert_eq!(
        app.send("POST", &base, &owner, json!({"hostname": "not a domain"}))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        app.send(
            "POST",
            &base,
            &owner,
            json!({"hostname": "todo.apps.nomi.test"})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        app.send(
            "POST",
            &base,
            &other,
            json!({"hostname": "shop.example.com"})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );

    let (status, added) = app
        .send(
            "POST",
            &base,
            &owner,
            json!({"hostname": "https://Shop.Example.com/"}),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{added}");
    assert_eq!(added["hostname"], "shop.example.com");
    assert_eq!(added["https"], "pending");
    assert_eq!(added["primary"], false);
    assert!(
        added["checked_at"].is_string() && added["verified_at"].is_null(),
        "{added}"
    );
    let records = added["records"].as_array().unwrap();
    assert_eq!(
        records[0],
        json!({"type": "CNAME", "name": "shop.example.com", "value": "todo.apps.nomi.test", "status": "waiting"})
    );
    assert_eq!(records[1]["type"], "TXT");
    assert_eq!(records[1]["name"], "_nomi-verify.shop.example.com");
    let token = records[1]["value"].as_str().unwrap().to_string();
    assert!(token.starts_with("nomi-verify="));

    // One project per hostname.
    let (second, _) = app.project(&owner, "Other").await;
    publish_live(&pool, second, "https://other.apps.nomi.test").await;
    assert_eq!(
        app.send(
            "POST",
            &format!("/api/projects/{second}/domains"),
            &owner,
            json!({"hostname": "shop.example.com"})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );

    let one = format!("{base}/shop.example.com");
    assert_eq!(
        app.call("PUT", &format!("{one}/primary"), Some(&owner.token), None)
            .await
            .0,
        StatusCode::CONFLICT,
        "not verified yet"
    );

    // Half set: the CNAME is found, the TXT still waiting.
    dns.set(
        "shop.example.com",
        RecordKind::Cname,
        &["todo.apps.nomi.test"],
    );
    let (status, checked) = app
        .call("POST", &format!("{one}/check"), Some(&owner.token), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(checked["records"][0]["status"], "found");
    assert_eq!(checked["records"][1]["status"], "waiting");
    assert!(checked["verified_at"].is_null());

    dns.set("_nomi-verify.shop.example.com", RecordKind::Txt, &[&token]);
    let (_, checked) = app
        .call("POST", &format!("{one}/check"), Some(&owner.token), None)
        .await;
    assert!(checked["verified_at"].is_string(), "{checked}");

    let (status, primary) = app
        .call("PUT", &format!("{one}/primary"), Some(&owner.token), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(primary["primary"], true);

    // A bare domain points A records at the app's ingress.
    let (status, apex) = app
        .send("POST", &base, &owner, json!({"hostname": "example.com"}))
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        apex["records"][0],
        json!({"type": "A", "name": "example.com", "value": "203.0.113.7", "status": "waiting"})
    );
    dns.set("example.com", RecordKind::A, &["203.0.113.7"]);

    // The worker looks again at the ones due.
    sqlx::query("UPDATE project_domains SET checked_at = now() - interval '1 hour' WHERE hostname = 'example.com'").execute(&pool).await.unwrap();
    assert!(nomi_server::routes::custom_domains::recheck_due(&pool).await >= 1);
    let (status, list) = app.get(&base, &owner).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list[0]["hostname"], "shop.example.com", "the primary first");
    assert_eq!(list[1]["records"][0]["status"], "found");
    assert_eq!(app.get(&base, &other).await.0, StatusCode::NOT_FOUND);

    assert_eq!(
        app.call(
            "POST",
            &format!("{base}/nope.example.com/check"),
            Some(&owner.token),
            None
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.call("DELETE", &one, Some(&owner.token), None).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        app.call("DELETE", &one, Some(&owner.token), None).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.call("PUT", &format!("{one}/primary"), Some(&owner.token), None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}
