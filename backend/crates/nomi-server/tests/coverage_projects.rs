//! Projects past listing: starting one, its files and preview, importing code (an archive or a
//! batch), exporting it, and moving it to another organization.

mod support;

use std::io::Write;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use support::App;

fn zip_of(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut out);
        for (name, body) in entries {
            writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
            writer.write_all(body.as_bytes()).unwrap();
        }
        writer.finish().unwrap();
    }
    out.into_inner()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_is_started_filled_previewed_exported_and_moved(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    let ben = app.person("ben@acme.test").await;

    // No organization picked for projects yet.
    assert_eq!(app.call("POST", "/api/projects", Some(&ana.token), None).await.0, StatusCode::CONFLICT);
    let org: uuid::Uuid = sqlx::query_scalar("SELECT org_id FROM memberships WHERE user_id = $1").bind(ana.id).fetch_one(&pool).await.unwrap();
    let (status, created) = app.send("POST", "/api/projects", &ana, json!({"org_id": org})).await;
    assert!(status.is_success(), "{status} {created}");
    let session = created["session_id"].as_str().unwrap().to_string();
    let (_, mine) = app.get("/api/projects", &ana).await;
    let project = mine["projects"].as_array().and_then(|p| p.first()).map(|p| p["id"].as_str().unwrap().to_string());
    let project = match project {
        Some(id) => id,
        None => sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM projects WHERE session_id = $1").bind(session.parse::<uuid::Uuid>().unwrap()).fetch_one(&pool).await.unwrap().to_string(),
    };
    assert_eq!(app.get(&format!("/api/projects/by-session/{session}"), &ana).await.0, StatusCode::OK);
    assert_eq!(app.get(&format!("/api/projects/by-session/{session}"), &ben).await.0, StatusCode::NOT_FOUND);
    // An organization that isn't yours can't hold it.
    let (status, _) = app.send("POST", "/api/projects", &ana, json!({"org_id": uuid::Uuid::new_v4()})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Import an archive, then a batch of files that finishes the import.
    let archive = zip_of(&[("site/package.json", r#"{"dependencies":{"svelte":"5","@sveltejs/kit":"2"}}"#), ("site/src/routes/+page.svelte", "<h1>Hi</h1>"), ("site/node_modules/x/index.js", "skip me")]);
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/projects/{project}/import"))
        .header("authorization", format!("Bearer {}", ana.token))
        .header("content-type", "application/zip")
        .body(Body::from(archive))
        .unwrap();
    let (status, _, body) = app.raw(request).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let imported: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(imported["imported"].as_u64().unwrap() >= 2, "{imported}");

    let wrong_type = Request::builder()
        .method("POST")
        .uri(format!("/api/projects/{project}/import"))
        .header("authorization", format!("Bearer {}", ana.token))
        .header("content-type", "text/plain")
        .body(Body::from("nope"))
        .unwrap();
    assert_eq!(app.raw(wrong_type).await.0, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let not_zip = Request::builder()
        .method("POST")
        .uri(format!("/api/projects/{project}/import"))
        .header("authorization", format!("Bearer {}", ana.token))
        .header("content-type", "application/zip")
        .body(Body::from("PK not really"))
        .unwrap();
    assert_eq!(app.raw(not_zip).await.0, StatusCode::UNPROCESSABLE_ENTITY);

    let batch = json!({"files": [{"path": "src/lib/util.ts", "content": "export const x = 1;"}, {"path": "../escape.txt", "content": "no"}], "finish": true});
    let (status, result) = app.send("POST", &format!("/api/projects/{project}/import/files"), &ana, batch).await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(app.send("POST", &format!("/api/projects/{project}/import/files"), &ben, json!({"files": []})).await.0, StatusCode::NOT_FOUND);
    let too_many: Vec<_> = (0..501).map(|i| json!({"path": format!("f{i}.txt"), "content": ""})).collect();
    assert_eq!(app.send("POST", &format!("/api/projects/{project}/import/files"), &ana, json!({"files": too_many})).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(app.send("POST", &format!("/api/projects/{project}/import/github"), &ana, json!({"repo": "not a repo"})).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(app.send("POST", &format!("/api/projects/{project}/import/github"), &ana, json!({"repo": "nomi/app", "ref": "bad ref!"})).await.0, StatusCode::BAD_REQUEST);

    // Files: read, write, delete; the preview serves them.
    let file = format!("/api/projects/{project}/files/src/lib/util.ts");
    assert_eq!(app.get(&file, &ana).await.0, StatusCode::OK);
    assert!(app.call("DELETE", &file, Some(&ana.token), None).await.0.is_success());
    assert_eq!(app.get(&file, &ana).await.0, StatusCode::NOT_FOUND);
    app.send("PUT", &format!("/api/projects/{project}/files/index.html"), &ana, json!({"content": "<h1>Bakery</h1>"})).await;
    let preview = Request::builder().uri(format!("/api/projects/{project}/preview")).header("authorization", format!("Bearer {}", ana.token)).body(Body::empty()).unwrap();
    let (status, _, _) = app.raw(preview).await;
    assert!(status.is_success() || status == StatusCode::NOT_FOUND, "{status}");
    let page = Request::builder().uri(format!("/api/projects/{project}/preview/index.html")).header("authorization", format!("Bearer {}", ana.token)).body(Body::empty()).unwrap();
    let (status, _, body) = app.raw(page).await;
    assert!(status.is_success(), "{status} {}", String::from_utf8_lossy(&body));

    // Export: a summary, then the zip.
    let (status, summary) = app.get(&format!("/api/projects/{project}/export"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{summary}");
    let download = Request::builder().uri(format!("/api/projects/{project}/export/zip")).header("authorization", format!("Bearer {}", ana.token)).body(Body::empty()).unwrap();
    let (status, headers, zip_bytes) = app.raw(download).await;
    assert_eq!(status, StatusCode::OK);
    assert!(headers.get("content-disposition").is_some());
    assert!(zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).unwrap().len() >= 2);
    assert_eq!(app.get(&format!("/api/projects/{project}/export"), &ben).await.0, StatusCode::NOT_FOUND);

    // Move it: only to another organization, with its name typed to confirm.
    let (status, team) = app.send("POST", "/api/orgs", &ana, json!({"name": "Team"})).await;
    assert!(status.is_success(), "{status} {team}");
    let team_id = team["id"].as_str().map(str::to_string).unwrap_or_else(|| team["org"]["id"].as_str().unwrap_or_default().to_string());
    let (status, orgs) = app.get(&format!("/api/projects/{project}/transfer/orgs?q=te"), &ana).await;
    assert_eq!(status, StatusCode::OK, "{orgs}");
    let name: String = sqlx::query_scalar("SELECT name FROM projects WHERE id = $1").bind(project.parse::<uuid::Uuid>().unwrap()).fetch_one(&pool).await.unwrap();
    let transfer = format!("/api/projects/{project}/transfer");
    assert_eq!(app.send("POST", &transfer, &ana, json!({"target_org_id": team_id, "confirm_name": "wrong"})).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(app.send("POST", &transfer, &ben, json!({"target_org_id": team_id, "confirm_name": name})).await.0, StatusCode::NOT_FOUND);
    assert_eq!(app.send("POST", &transfer, &ana, json!({"target_org_id": uuid::Uuid::new_v4(), "confirm_name": name})).await.0, StatusCode::NOT_FOUND);
    let (status, body) = app.send("POST", &transfer, &ana, json!({"target_org_id": team_id, "confirm_name": name})).await;
    assert!(status.is_success(), "{status} {body}");
    assert_eq!(app.send("POST", &transfer, &ana, json!({"target_org_id": team_id, "confirm_name": name})).await.0, StatusCode::BAD_REQUEST);
}
