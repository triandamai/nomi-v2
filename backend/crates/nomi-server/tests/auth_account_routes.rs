//! Signing in (or creating an account) with an emailed code alone, setting a password while
//! signed in, and switching the active workspace.

mod support;

use std::sync::Arc;

use axum::http::StatusCode;
use nomi_auth::claims::Claims;
use nomi_mail::MemoryMailer;
use nomi_server::sign_in_codes::EmailCodes;
use serde_json::{json, Value};
use sqlx::PgPool;
use support::{App, SECRET};
use uuid::Uuid;

fn with_codes(pool: PgPool) -> (App, Arc<MemoryMailer>) {
    let mailer = Arc::new(MemoryMailer::default());
    let mut state = support::state(pool.clone());
    state.email_codes = EmailCodes::Required(mailer.clone());
    (App::with_state(pool, state), mailer)
}

fn code_sent_to(mailer: &MemoryMailer, to: &str) -> String {
    mailer.last_to(to).expect("an email was sent").subject.rsplit(' ').next().unwrap().to_string()
}

async fn start(app: &App, body: Value) -> (StatusCode, Value) {
    app.call("POST", "/api/auth/start", None, Some(body)).await
}

async fn verify(app: &App, challenge: &Value, code: &str) -> (StatusCode, Value) {
    app.call("POST", "/api/auth/verify", None, Some(json!({ "challenge_id": challenge, "code": code }))).await
}

/// Starts and finishes an email-code sign-in; the verify answer.
async fn sign_in_with_code(app: &App, mailer: &MemoryMailer, email: &str) -> Value {
    let (status, body) = start(app, json!({ "email": email })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = verify(app, &body["verification"]["challenge_id"], &code_sent_to(mailer, email)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

fn active_org(token: &str) -> Uuid {
    Claims::decode(token, SECRET).unwrap().active_org_id
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_address_gets_an_account_with_a_personal_workspace_from_the_code(pool: PgPool) {
    let (app, mailer) = with_codes(pool.clone());
    let (status, body) = start(&app, json!({ "email": "  Nia@Example.com ", "language": "id" })).await;
    assert_eq!(status, StatusCode::OK);
    let v = &body["verification"];
    assert_eq!(v["purpose"], "register");
    assert_eq!(v["email"], "n**@example.com");
    assert!(mailer.last_to("nia@example.com").unwrap().subject.starts_with("Konfirmasi"), "in the page's language");
    let accounts: i64 = sqlx::query_scalar("SELECT count(*) FROM web_credentials").fetch_one(&pool).await.unwrap();
    assert_eq!(accounts, 0, "no account until the code is entered");

    // A fresh code for a new address goes out in the same language.
    sqlx::query("UPDATE sign_in_challenges SET last_sent_at = now() - interval '2 minutes'").execute(&pool).await.unwrap();
    let (status, _) = app.call("POST", "/api/auth/resend", None, Some(json!({ "challenge_id": v["challenge_id"] }))).await;
    assert_eq!(status, StatusCode::OK);
    let (status, info) = app.call("GET", &format!("/api/auth/challenge/{}", v["challenge_id"].as_str().unwrap()), None, None).await;
    assert_eq!((status, info["email"].as_str()), (StatusCode::OK, Some("n**@example.com")));

    let (status, tokens) = verify(&app, &v["challenge_id"], &code_sent_to(&mailer, "nia@example.com")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tokens["is_new"], true);
    assert!(tokens["refresh_token"].is_string());
    let token = tokens["access_token"].as_str().unwrap();

    let (personal, name, role): (bool, String, String) = sqlx::query_as(
        "SELECT o.is_personal, o.name, m.role FROM organizations o JOIN memberships m ON m.org_id = o.id WHERE o.id = $1",
    )
    .bind(active_org(token))
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(personal);
    assert_eq!((name.as_str(), role.as_str()), ("nia's space", "owner"));
    let (password_login, verified): (bool, bool) =
        sqlx::query_as("SELECT password_login, email_verified_at IS NOT NULL FROM web_credentials WHERE email = 'nia@example.com'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!password_login && verified);

    // The same address again signs in to that account.
    let again = sign_in_with_code(&app, &mailer, "nia@example.com").await;
    assert_eq!(again["is_new"], false);
    assert_eq!(active_org(again["access_token"].as_str().unwrap()), active_org(token));
    let accounts: i64 = sqlx::query_scalar("SELECT count(*) FROM web_credentials").fetch_one(&pool).await.unwrap();
    assert_eq!(accounts, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_address_with_an_invite_joins_that_workspace(pool: PgPool) {
    let (app, mailer) = with_codes(pool.clone());
    let owner = sign_in_with_code(&app, &mailer, "owner@example.com").await;
    let owner_id = Claims::decode(owner["access_token"].as_str().unwrap(), SECRET).unwrap().sub;
    let org = active_org(owner["access_token"].as_str().unwrap());
    sqlx::query("INSERT INTO org_invites (code, org_id, role, invited_by, expires_at) VALUES ('JOINME', $1, 'member', $2, now() + interval '1 day')")
        .bind(org)
        .bind(owner_id)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = start(&app, json!({ "email": "guest@example.com", "invite_code": "NOPE" })).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("invalid_invite")));
    let (status, body) = start(&app, json!({ "email": "not-an-email" })).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("invalid_email")));

    let (_, body) = start(&app, json!({ "email": "guest@example.com", "invite_code": "JOINME" })).await;
    let (status, tokens) = verify(&app, &body["verification"]["challenge_id"], &code_sent_to(&mailer, "guest@example.com")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tokens["is_new"], true);
    assert_eq!(active_org(tokens["access_token"].as_str().unwrap()), org);
    let used: bool = sqlx::query_scalar("SELECT used_at IS NOT NULL FROM org_invites WHERE code = 'JOINME'").fetch_one(&pool).await.unwrap();
    assert!(used);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_existing_account_gets_a_sign_in_code_and_password_sign_in_still_works(pool: PgPool) {
    let (app, mailer) = with_codes(pool);
    let (_, body) = app
        .call("POST", "/api/auth/register", None, Some(json!({ "email": "ana@example.com", "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })))
        .await;
    verify(&app, &body["verification"]["challenge_id"], &code_sent_to(&mailer, "ana@example.com")).await;

    let (status, body) = start(&app, json!({ "email": "ANA@example.com" })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["verification"]["purpose"], "login");
    let (_, tokens) = verify(&app, &body["verification"]["challenge_id"], &code_sent_to(&mailer, "ana@example.com")).await;
    assert_eq!(tokens["is_new"], false);

    let (status, body) =
        app.call("POST", "/api/auth/login", None, Some(json!({ "email": "ana@example.com", "password": "correct-password" }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["verification"]["purpose"], "login");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_password_is_set_then_changed_while_signed_in(pool: PgPool) {
    let (app, mailer) = with_codes(pool);
    let tokens = sign_in_with_code(&app, &mailer, "nia@example.com").await;
    let token = tokens["access_token"].as_str().unwrap();
    let put = |body: Value| app.call("PUT", "/api/auth/password", Some(token), Some(body));

    let (status, body) = put(json!({ "new_password": "short" })).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("too_short")));
    let (status, _) = put(json!({ "new_password": "first-password" })).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "no password yet: none to confirm");

    let login = |password: &'static str| {
        app.call("POST", "/api/auth/login", None, Some(json!({ "email": "nia@example.com", "password": password })))
    };
    assert_eq!(login("first-password").await.0, StatusCode::OK);

    let (status, body) = put(json!({ "new_password": "second-password" })).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::FORBIDDEN, Some("wrong_password")));
    let (status, _) = put(json!({ "current_password": "nope-nope", "new_password": "second-password" })).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = put(json!({ "current_password": "first-password", "new_password": "second-password" })).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(login("first-password").await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(login("second-password").await.0, StatusCode::OK);

    let (status, _) = app
        .call("POST", "/api/auth/refresh", None, Some(json!({ "refresh_token": tokens["refresh_token"] })))
        .await;
    assert_eq!(status, StatusCode::OK, "other sign-ins stay");
    assert_eq!(app.call("PUT", "/api/auth/password", None, Some(json!({ "new_password": "whatever-1" }))).await.0, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn switching_the_active_workspace_issues_a_token_that_sign_ins_keep(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@example.com").await;
    let first = active_org(&ana.token);
    let (status, org) = app.send("POST", "/api/orgs", &ana, json!({ "name": "Second" })).await;
    assert!(status.is_success(), "{status} {org}");
    let second: Uuid = org["id"].as_str().unwrap().parse().unwrap();

    let (status, body) = app.send("PUT", "/api/auth/active-org", &ana, json!({ "org_id": second })).await;
    assert_eq!(status, StatusCode::OK);
    let switched = body["access_token"].as_str().unwrap();
    assert_eq!(active_org(switched), second);

    let again = app.login("ana@example.com").await;
    assert_eq!(active_org(&again.token), second, "a later sign-in keeps it");
    let (_, login) = app.call("POST", "/api/auth/login", None, Some(json!({ "email": "ana@example.com", "password": "correct-password" }))).await;
    let (_, refreshed) = app.call("POST", "/api/auth/refresh", None, Some(json!({ "refresh_token": login["refresh_token"] }))).await;
    assert_eq!(active_org(refreshed["access_token"].as_str().unwrap()), second, "and a refresh");

    let bo = app.person("bo@example.com").await;
    let (status, body) = app.send("PUT", "/api/auth/active-org", &bo, json!({ "org_id": second })).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::NOT_FOUND, Some("not_a_member")));

    // Leaving the workspace falls back to the first one.
    sqlx::query("DELETE FROM memberships WHERE user_id = $1 AND org_id = $2").bind(ana.id).bind(second).execute(&pool).await.unwrap();
    assert_eq!(active_org(&app.login("ana@example.com").await.token), first);
}
