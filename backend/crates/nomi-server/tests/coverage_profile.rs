//! The profile beyond name and username: usernames being checked, a public page, the showcase,
//! and avatars uploaded through Nomi and served back.

mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use support::App;

const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13, b'I', b'H', b'D', b'R'];

fn upload(token: &str, content_type: &str, body: Vec<u8>) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/profile/avatar")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", content_type)
        .body(Body::from(body))
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn usernames_are_checked_and_a_public_profile_shows_what_was_chosen(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    let ben = app.person("ben@acme.test").await;
    let (project, _) = app.project(&ana, "Bakery").await;
    sqlx::query("UPDATE projects SET status = 'ready' WHERE id = $1").bind(project).execute(&pool).await.unwrap();

    for (name, available) in [("ana_k", true), ("", false), ("a b", false), (&"x".repeat(40)[..], false)] {
        let (status, body) = app.call("GET", &format!("/api/profile/check-username?username={}", urlencoding(name)), Some(&ben.token), None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["available"], available, "{name}: {body}");
    }

    let (status, saved) = app
        .send("PUT", "/api/profile", &ana, json!({"display_name": "Ana K", "username": "ana_k", "location": "Bandung", "show_stats_publicly": true, "show_crew_publicly": true, "showcase_project_ids": [project]}))
        .await;
    assert_eq!(status, StatusCode::OK, "{saved}");

    // Taken now, except for Ana herself.
    let (_, taken) = app.call("GET", "/api/profile/check-username?username=@ANA_K", Some(&ben.token), None).await;
    assert_eq!(taken["available"], false);
    let (_, mine) = app.call("GET", "/api/profile/check-username?username=ana_k", Some(&ana.token), None).await;
    assert_eq!(mine["available"], true);

    let (status, public) = app.call("GET", "/api/public/profile/@ana_k", None, None).await;
    assert_eq!(status, StatusCode::OK, "{public}");
    assert_eq!(public["display_name"], "Ana K");
    assert_eq!(public["location"], "Bandung");
    assert!(public["showcase_projects"].as_array().unwrap().iter().any(|p| p["name"] == "Bakery"), "{public}");
    assert_eq!(app.call("GET", "/api/public/profile/nobody", None, None).await.0, StatusCode::NOT_FOUND);

    let (status, profile) = app.get("/api/profile", &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(profile["show_crew_publicly"], true);
    assert!(profile.get("stats").is_some());

    // Hiding stats hides them publicly.
    app.send("PUT", "/api/profile", &ana, json!({"show_stats_publicly": false, "showcase_project_ids": []})).await;
    let (_, public) = app.call("GET", "/api/public/profile/ana_k", None, None).await;
    assert!(public["stats"].is_null(), "{public}");
}

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c.to_string() } else { format!("%{:02X}", c as u32) }).collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_avatar_is_checked_stored_and_served(pool: PgPool) {
    let dir = std::env::temp_dir().join(format!("nomi-avatars-{}", uuid::Uuid::new_v4()));
    nomi_storage::blob::update_attachment_store(None);
    std::env::set_var("ATTACHMENTS_DIR", &dir);
    nomi_storage::blob::update_attachment_store(None);
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;

    let (status, _, body) = app.raw(upload(&ana.token, "image/svg+xml", b"<svg/>".to_vec())).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{}", String::from_utf8_lossy(&body));
    assert_eq!(app.raw(upload(&ana.token, "image/png", vec![])).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(app.raw(upload(&ana.token, "image/png", b"GIF89a not a png".to_vec())).await.0, StatusCode::BAD_REQUEST);

    let (status, _, body) = app.raw(upload(&ana.token, "image/png", PNG.to_vec())).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let url = serde_json::from_slice::<serde_json::Value>(&body).unwrap()["avatar_url"].as_str().unwrap().to_string();
    assert!(url.starts_with(&format!("/avatars/{}/", ana.id)) && url.ends_with(".png"), "{url}");

    let served = Request::builder().uri(format!("/api{url}")).header("authorization", format!("Bearer {}", ana.token)).body(Body::empty()).unwrap();
    let (status, headers, bytes) = app.raw(served).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers.get("content-type").unwrap(), "image/png");
    assert_eq!(bytes, PNG);

    let public = Request::builder().uri(format!("/api/public{url}")).body(Body::empty()).unwrap();
    assert_eq!(app.raw(public).await.0, StatusCode::OK);
    // Only names Nomi made, and only pictures that exist.
    let bad = Request::builder().uri(format!("/api/public/avatars/{}/passwd", ana.id)).body(Body::empty()).unwrap();
    assert_eq!(app.raw(bad).await.0, StatusCode::NOT_FOUND);
    let missing = Request::builder().uri(format!("/api/public/avatars/{}/{}.png", ana.id, uuid::Uuid::new_v4())).body(Body::empty()).unwrap();
    assert_eq!(app.raw(missing).await.0, StatusCode::NOT_FOUND);
    // The private route needs someone signed in.
    let anonymous = Request::builder().uri(format!("/api{url}")).body(Body::empty()).unwrap();
    assert_eq!(app.raw(anonymous).await.0, StatusCode::UNAUTHORIZED);
    let _ = std::fs::remove_dir_all(dir);
}
