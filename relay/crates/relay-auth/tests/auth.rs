//! Sign-in against a real database with the `nomi` schema, mail caught by a recording outbox.
use relay_auth::codes::{self, Limits};
use relay_auth::config::{self, AuthConfig};
use relay_auth::mail::{Email, Outbox, RecordingOutbox};
use relay_auth::sessions;
use relay_auth::users::{self, Attributes};
use relay_auth::{AuthError, Scope};
use relay_core::keys::{new_signing_secret, verify_at};
use relay_core::settings::Smtp;
use relay_core::Env;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

const NOMI_SCHEMA: &str = include_str!("../../relay-control/src/nomi_schema.sql");
const STATE: &str = "CREATE SCHEMA IF NOT EXISTS relay; CREATE TABLE IF NOT EXISTS relay.auth_configs (project_id UUID NOT NULL, env TEXT NOT NULL, sealed BYTEA NOT NULL, PRIMARY KEY (project_id, env));";

struct World {
    pool: PgPool,
    project: Uuid,
    secret: String,
    outbox: RecordingOutbox,
}

impl World {
    fn scope(&self) -> Scope<'_> {
        Scope { pool: &self.pool, project: self.project, env: Env::Dev, secret: &self.secret }
    }

    /// The code in the last email sent.
    fn last_code(&self) -> String {
        let sent = self.outbox.sent();
        let (_, email) = sent.last().expect("an email was sent");
        email.subject.rsplit(' ').next().unwrap().to_string()
    }
}

async fn world(pool: PgPool) -> World {
    sqlx::raw_sql(NOMI_SCHEMA).execute(&pool).await.unwrap();
    World { pool, project: Uuid::new_v4(), secret: new_signing_secret(), outbox: RecordingOutbox::default() }
}

fn shared() -> Smtp {
    Smtp { host: "smtp.shared.test".into(), port: 587, user: "nomi".into(), password: "pw".into(), from: "Nomi <codes@nomi.test>".into(), security: "starttls".into() }
}

const SHARED: Limits = Limits::SHARED;

#[sqlx::test]
async fn a_code_signs_a_new_person_in_and_makes_them_a_verified_user(pool: PgPool) {
    let w = world(pool).await;
    codes::send(&w.scope(), &w.outbox, &shared(), SHARED, "Bakery", " Ana@Example.test ", true).await.unwrap();
    let sent = w.outbox.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].0.host, "smtp.shared.test");
    assert_eq!(sent[0].1.to, "ana@example.test");
    assert!(sent[0].1.subject.starts_with("Your Bakery sign-in code: "));
    let code = w.last_code();
    assert_eq!(code.len(), 6);

    let user = codes::verify(&w.scope(), "ana@example.test", &code).await.unwrap();
    assert!(user.email_verified);
    assert_eq!(user.identities[0].provider, "email");
    assert!(user.last_sign_in_at.is_some());
    assert!(matches!(codes::verify(&w.scope(), "ana@example.test", &code).await, Err(AuthError::BadRequest { code: "INVALID_CODE", .. })), "a code works once");
}

#[sqlx::test]
async fn five_wrong_guesses_kill_the_code(pool: PgPool) {
    let w = world(pool).await;
    codes::send(&w.scope(), &w.outbox, &shared(), SHARED, "Bakery", "ana@example.test", true).await.unwrap();
    let code = w.last_code();
    let wrong = if code == "000000" { "111111" } else { "000000" };
    for _ in 0..codes::MAX_ATTEMPTS {
        assert!(codes::verify(&w.scope(), "ana@example.test", wrong).await.is_err());
    }
    assert!(matches!(codes::verify(&w.scope(), "ana@example.test", &code).await, Err(AuthError::BadRequest { code: "INVALID_CODE", .. })));
}

#[sqlx::test]
async fn an_unknown_email_without_sign_up_gets_nothing_and_no_answer_says_so(pool: PgPool) {
    let w = world(pool).await;
    codes::send(&w.scope(), &w.outbox, &shared(), SHARED, "Bakery", "nobody@example.test", false).await.unwrap();
    assert!(w.outbox.sent().is_empty());
}

#[sqlx::test]
async fn the_shared_sender_stops_after_five_codes_an_hour_for_one_email(pool: PgPool) {
    let w = world(pool).await;
    for _ in 0..SHARED.per_email {
        codes::send(&w.scope(), &w.outbox, &shared(), SHARED, "Bakery", "ana@example.test", true).await.unwrap();
    }
    assert!(matches!(codes::send(&w.scope(), &w.outbox, &shared(), SHARED, "Bakery", "ana@example.test", true).await, Err(AuthError::TooMany { .. })));
    assert!(matches!(codes::send(&w.scope(), &w.outbox, &shared(), SHARED, "Bakery", "not an email", true).await, Err(AuthError::BadRequest { .. })));
}

async fn signed_in(w: &World) -> sessions::Session {
    codes::send(&w.scope(), &w.outbox, &shared(), SHARED, "Bakery", "ana@example.test", true).await.unwrap();
    let user = codes::verify(&w.scope(), "ana@example.test", &w.last_code()).await.unwrap();
    sessions::start(&w.scope(), user, config::DEFAULT_SESSION_DAYS).await.unwrap()
}

#[sqlx::test]
async fn a_session_refreshes_with_a_new_token_and_an_old_one_ends_it(pool: PgPool) {
    let w = world(pool).await;
    let first = signed_in(&w).await;
    assert_eq!(first.token_type, "bearer");
    let claims = verify_at(&w.secret, &first.access_token, relay_core::keys::issued_now()).unwrap();
    assert_eq!(claims.sub, Some(first.user.id));
    let sid = claims.sid.unwrap();
    assert!(sessions::live(&w.pool, w.project, sid).await.unwrap());

    let second = sessions::refresh(&w.scope(), &first.refresh_token).await.unwrap();
    assert_ne!(second.refresh_token, first.refresh_token);
    assert!(sessions::live(&w.pool, w.project, sid).await.unwrap(), "a rotated session is still the same sign-in");

    assert!(matches!(sessions::refresh(&w.scope(), &first.refresh_token).await, Err(AuthError::Unauthorized { code: "REFRESH_TOKEN_REUSED", .. })));
    assert!(sessions::refresh(&w.scope(), &second.refresh_token).await.is_err(), "reuse ends the whole family");
    assert!(!sessions::live(&w.pool, w.project, sid).await.unwrap());
}

#[sqlx::test]
async fn signing_out_ends_the_session_and_deleting_the_user_ends_theirs(pool: PgPool) {
    let w = world(pool).await;
    let session = signed_in(&w).await;
    let sid = verify_at(&w.secret, &session.access_token, relay_core::keys::issued_now()).unwrap().sid.unwrap();
    sessions::end(&w.pool, w.project, sid).await.unwrap();
    assert!(sessions::refresh(&w.scope(), &session.refresh_token).await.is_err());

    let again = signed_in(&w).await;
    users::delete(&w.pool, w.project, again.user.id).await.unwrap();
    assert!(sessions::refresh(&w.scope(), &again.refresh_token).await.is_err());
}

#[sqlx::test]
async fn a_token_from_another_project_finds_nothing_here(pool: PgPool) {
    let w = world(pool).await;
    let session = signed_in(&w).await;
    let other = Scope { pool: &w.pool, project: Uuid::new_v4(), env: Env::Dev, secret: &w.secret };
    assert!(sessions::refresh(&other, &session.refresh_token).await.is_err());
    assert!(users::get(&w.pool, other.project, session.user.id).await.unwrap().is_none());
}

#[sqlx::test]
async fn admins_list_create_update_and_delete_users(pool: PgPool) {
    let w = world(pool).await;
    let made = users::create(&w.pool, w.project, Attributes { email: Some("bo@example.test".into()), email_verified: Some(true), user_metadata: Some(json!({ "plan": "pro" })) }).await.unwrap();
    assert!(matches!(users::create(&w.pool, w.project, Attributes { email: Some("BO@example.test".into()), ..Attributes::default() }).await, Err(AuthError::BadRequest { code: "USER_EXISTS", .. })));
    let updated = users::update(&w.pool, w.project, made.id, Attributes { user_metadata: Some(json!({ "name": "Bo" })), ..Attributes::default() }).await.unwrap();
    assert_eq!(updated.user_metadata, json!({ "plan": "pro", "name": "Bo" }));
    let (listed, total) = users::list(&w.pool, w.project, 1, 50).await.unwrap();
    assert_eq!((listed.len(), total), (1, 1));
    users::delete(&w.pool, w.project, made.id).await.unwrap();
    assert!(users::get(&w.pool, w.project, made.id).await.unwrap().is_none());
    assert!(matches!(users::delete(&w.pool, w.project, made.id).await, Err(AuthError::NotFound(_))));
}

#[sqlx::test]
async fn a_projects_auth_config_is_kept_sealed_and_defaults_to_email_codes(pool: PgPool) {
    sqlx::raw_sql(STATE).execute(&pool).await.unwrap();
    let key = [5u8; 32];
    let project = Uuid::new_v4();
    let default = config::load(&pool, &key, project, Env::Dev).await.unwrap();
    assert_eq!(default.methods, vec!["email_code".to_string()]);
    assert_eq!(default.session_ttl_days, config::DEFAULT_SESSION_DAYS);
    let own = AuthConfig { smtp: Some(Smtp { password: "secret-pw".into(), ..shared() }), ..AuthConfig::default() };
    config::save(&pool, &key, project, Env::Dev, &own).await.unwrap();
    assert_eq!(config::load(&pool, &key, project, Env::Dev).await.unwrap().smtp.unwrap().password, "secret-pw");
    assert!(config::load(&pool, &key, project, Env::Production).await.unwrap().smtp.is_none());
    let sealed: Vec<u8> = sqlx::query_scalar("SELECT sealed FROM relay.auth_configs").fetch_one(&pool).await.unwrap();
    assert!(!String::from_utf8_lossy(&sealed).contains("secret-pw"));
}

#[tokio::test]
async fn the_recording_outbox_keeps_what_it_was_asked_to_send() {
    let outbox = RecordingOutbox::default();
    outbox.send(&shared(), Email { to: "a@b.test".into(), subject: "s".into(), text: "t".into() }).await.unwrap();
    assert_eq!(outbox.sent().len(), 1);
}
