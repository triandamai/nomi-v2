//! Settings: reminder pushes and quiet hours, the profile's active hours and insights, General's
//! preferences, changing the email and closing the account.

/// The server tests' shared helpers.
#[path = "../../nomi-server/tests/support/mod.rs"]
mod support;

use std::sync::{Arc, Mutex};

use axum::http::StatusCode;
use nomi_mail::MemoryMailer;
use nomi_server::push::{DeviceTarget, PushError, PushMessage, PushSender};
use nomi_server::sign_in_codes::EmailCodes;
use serde_json::{json, Value};
use sqlx::PgPool;
use support::{App, Person};
use uuid::Uuid;

/// The same app, with emailed codes going to `mailer` (people sign in through `App` first).
fn with_mailer(pool: &PgPool) -> (App, Arc<MemoryMailer>) {
    let mailer = Arc::new(MemoryMailer::default());
    let mut state = support::state(pool.clone());
    state.email_codes = EmailCodes::Required(mailer.clone());
    (App::with_state(pool.clone(), state), mailer)
}

/// The code in the last email sent to `to`.
fn code_sent_to(mailer: &MemoryMailer, to: &str) -> String {
    let email = mailer.last_to(to).expect("an email was sent");
    email.subject.rsplit(' ').next().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn reminder_pushes_and_quiet_hours_are_kept_with_the_email_settings(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;

    // The email settings keep their two-field shape; the reminder ones have their own route.
    let (status, prefs) = app.get("/api/notifications/preferences", &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        prefs,
        json!({ "email_account": true, "email_promos": true })
    );
    let (status, prefs) = app
        .get("/api/notifications/reminder-preferences", &ana)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        prefs,
        json!({ "push_reminders": true, "quiet_hours": null })
    );

    let (status, prefs) = app
        .send(
            "PUT",
            "/api/notifications/reminder-preferences",
            &ana,
            json!({ "push_reminders": false, "quiet_hours": { "start": "22:00", "end": "06:00" } }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{prefs}");
    assert_eq!(prefs["push_reminders"], false);
    assert_eq!(
        prefs["quiet_hours"],
        json!({ "start": "22:00", "end": "06:00" })
    );

    // The email settings alone, as the older screens send them, leave the rest as it is; one
    // form may send them all to the email route.
    let (_, prefs) = app
        .send(
            "PUT",
            "/api/notifications/preferences",
            &ana,
            json!({ "email_account": true, "email_promos": false }),
        )
        .await;
    assert_eq!(
        prefs,
        json!({ "email_account": true, "email_promos": false })
    );
    let (_, prefs) = app
        .get("/api/notifications/reminder-preferences", &ana)
        .await;
    assert_eq!(prefs["push_reminders"], false);
    assert_eq!(prefs["quiet_hours"]["start"], "22:00");
    let (_, prefs) = app
        .send(
            "PUT",
            "/api/notifications/preferences",
            &ana,
            json!({ "push_reminders": true }),
        )
        .await;
    assert_eq!(
        prefs,
        json!({ "email_account": true, "email_promos": false })
    );
    let (_, prefs) = app
        .get("/api/notifications/reminder-preferences", &ana)
        .await;
    assert_eq!(prefs["push_reminders"], true);

    // A held reminder goes back to its time when the hours change; null turns them off.
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) SELECT org_id, 'web', 'c' FROM memberships WHERE user_id = $1 RETURNING id")
        .bind(ana.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO reminders (user_id, session_id, title, due_at, held_until) VALUES ($1, $2, 'Stretch', now(), now() + interval '5 hours')")
        .bind(ana.id)
        .bind(session)
        .execute(&pool)
        .await
        .unwrap();
    let (_, prefs) = app
        .send(
            "PUT",
            "/api/notifications/reminder-preferences",
            &ana,
            json!({ "quiet_hours": null }),
        )
        .await;
    assert_eq!(prefs["quiet_hours"], Value::Null);
    let held: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT held_until FROM reminders")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(held.is_none());

    for bad in [
        json!({ "start": "25:00", "end": "06:00" }),
        json!({ "start": "07:00", "end": "07:00" }),
    ] {
        let (status, _) = app
            .send(
                "PUT",
                "/api/notifications/reminder-preferences",
                &ana,
                json!({ "quiet_hours": bad }),
            )
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

struct RecordingPush {
    sent: Mutex<Vec<(String, PushMessage)>>,
}

#[async_trait::async_trait]
impl PushSender for RecordingPush {
    async fn send(&self, device: &DeviceTarget, message: &PushMessage) -> Result<(), PushError> {
        self.sent
            .lock()
            .unwrap()
            .push((device.token.clone(), message.clone()));
        Ok(())
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_reminder_is_pushed_unless_reminder_pushes_are_off(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    let (status, _) = app
        .send(
            "POST",
            "/api/devices",
            &ana,
            json!({ "platform": "android", "push_token": "fcm-ana", "app_version": "1" }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let fired = nomi_agent_reminders::Fired {
        reminder_id: Uuid::new_v4(),
        user_id: ana.id,
        session_id: Uuid::new_v4(),
        message_id: Uuid::new_v4(),
        title: "Call Mum".into(),
        notes: Some("before dinner".into()),
    };

    let sender = Arc::new(RecordingPush {
        sent: Mutex::new(Vec::new()),
    });
    nomi_worker::reminders_worker::push_reminder(&pool, sender.clone(), &fired).await;
    {
        let sent = sender.sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        let (token, message) = &sent[0];
        assert_eq!(
            (
                token.as_str(),
                message.title.as_str(),
                message.body.as_str()
            ),
            ("fcm-ana", "Call Mum", "before dinner")
        );
        assert_eq!(message.data["kind"], "reminder");
        assert_eq!(message.data["session_id"], fired.session_id.to_string());
    }

    app.send(
        "PUT",
        "/api/notifications/preferences",
        &ana,
        json!({ "push_reminders": false }),
    )
    .await;
    nomi_worker::reminders_worker::push_reminder(&pool, sender.clone(), &fired).await;
    assert_eq!(
        sender.sent.lock().unwrap().len(),
        1,
        "no push once they're off"
    );
}

async fn usage(
    pool: &PgPool,
    who: &Person,
    model: Option<Uuid>,
    hours_ago: i64,
    input: i32,
    output: i32,
    cost: f64,
) {
    sqlx::query(
        "INSERT INTO llm_usage (user_id, admin_model_id, source, model_label, provider, model_id, input_tokens, output_tokens, cost_usd, created_at) \
         VALUES ($1, $2, 'nomi', 'big-model', 'fake', 'big', $3, $4, $5, now() - make_interval(hours => $6))",
    )
    .bind(who.id)
    .bind(model)
    .bind(input)
    .bind(output)
    .bind(cost)
    .bind(hours_ago as i32)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_profile_names_the_active_hours_and_its_insights(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;

    let (status, profile) = app.get("/api/profile", &ana).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(profile["active_hours"], Value::Null, "nothing to say yet");
    assert_eq!(profile["insights"], json!([]));

    // A cheap model, and quick questions all answered by an expensive one at one hour of the day.
    let cheap: Uuid = sqlx::query_scalar(
        "INSERT INTO admin_llm_models (label, provider, model_id, updated_by, input_usd_per_mtok, output_usd_per_mtok) \
         VALUES ('gemini-flash', 'fake', 'flash', $1, 0.1, 0.4) RETURNING id",
    )
    .bind(ana.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let busy_hour: i64 =
        sqlx::query_scalar("SELECT EXTRACT(HOUR FROM now() AT TIME ZONE 'UTC')::bigint")
            .fetch_one(&pool)
            .await
            .unwrap();
    for _ in 0..10 {
        usage(&pool, &ana, None, 0, 2_000, 200, 0.05).await;
    }

    let (_, profile) = app.get("/api/profile", &ana).await;
    // The two hours ending with this one (the earlier of two equal windows).
    let from = format!("{:02}:00", if busy_hour == 0 { 0 } else { busy_hour - 1 });
    assert_eq!(profile["active_hours"]["from"], from, "{profile}");
    let insights = profile["insights"].as_array().unwrap();
    let cheaper = insights
        .iter()
        .find(|i| i["kind"] == "cheaper_model")
        .expect("a cheaper model insight");
    assert!(
        cheaper["title"].as_str().unwrap().starts_with("Save ~"),
        "{cheaper}"
    );
    assert!(
        cheaper["body"].as_str().unwrap().contains("gemini-flash")
            && cheaper["body"]
                .as_str()
                .unwrap()
                .contains("10 of your last 10")
    );
    assert_eq!(cheaper["action"]["href"], "/models");
    // Quick questions that already went to the cheap model save nothing.
    sqlx::query("UPDATE llm_usage SET admin_model_id = $1")
        .bind(cheap)
        .execute(&pool)
        .await
        .unwrap();
    let (_, profile) = app.get("/api/profile", &ana).await;
    assert!(profile["insights"]
        .as_array()
        .unwrap()
        .iter()
        .all(|i| i["kind"] != "cheaper_model"));

    // Late at night, most of the tokens: "You work late".
    sqlx::query("DELETE FROM llm_usage")
        .execute(&pool)
        .await
        .unwrap();
    let to_22h: i64 = (22 - busy_hour).rem_euclid(24);
    for _ in 0..3 {
        usage(
            &pool,
            &ana,
            Some(cheap),
            (24 - to_22h) % 24,
            1_000,
            500,
            0.0,
        )
        .await;
    }
    let (_, profile) = app.get("/api/profile", &ana).await;
    assert_eq!(
        profile["active_hours"],
        json!({ "from": "21:00", "to": "23:00" }),
        "{profile}"
    );
    let late = profile["insights"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["kind"] == "late_hours")
        .cloned()
        .expect("late hours");
    assert_eq!(late["body"], "100% of your tokens land after 21:00.");

    // The public profile shows none of it.
    sqlx::query("INSERT INTO user_profiles (user_id, username) VALUES ($1, 'ana')")
        .bind(ana.id)
        .execute(&pool)
        .await
        .unwrap();
    let (_, public) = app.call("GET", "/api/public/profile/ana", None, None).await;
    assert!(public.get("insights").is_none() && public.get("active_hours").is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn general_keeps_formats_who_chats_start_with_and_how_the_composer_sends(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;

    let (_, prefs) = app.get("/api/preferences", &ana).await;
    assert_eq!(
        (
            prefs["formats"].clone(),
            prefs["default_agent"].clone(),
            prefs["enter_sends"].clone(),
            prefs["show_reply_tokens"].clone()
        ),
        (Value::Null, Value::Null, json!(true), json!(false))
    );

    let (status, prefs) = app
        .send("PUT", "/api/preferences", &ana, json!({ "formats": "en-GB", "default_agent": "money", "enter_sends": false, "show_reply_tokens": true }))
        .await;
    assert_eq!(status, StatusCode::OK, "{prefs}");
    assert_eq!(
        (prefs["formats"].clone(), prefs["default_agent"].clone()),
        (json!("en-GB"), json!("money"))
    );
    assert_eq!(
        (
            prefs["enter_sends"].clone(),
            prefs["show_reply_tokens"].clone()
        ),
        (json!(false), json!(true))
    );

    // Another field alone leaves them; an empty string goes back to the default.
    let (_, prefs) = app
        .send("PUT", "/api/preferences", &ana, json!({ "theme": "dark" }))
        .await;
    assert_eq!(prefs["formats"], "en-GB");
    let (_, prefs) = app
        .send(
            "PUT",
            "/api/preferences",
            &ana,
            json!({ "formats": "", "default_agent": "" }),
        )
        .await;
    assert_eq!(
        (
            prefs["formats"].clone(),
            prefs["default_agent"].clone(),
            prefs["enter_sends"].clone()
        ),
        (Value::Null, Value::Null, json!(false))
    );

    for bad in [
        json!({ "formats": "english" }),
        json!({ "formats": "en-gb" }),
        json!({ "default_agent": "Nomi!" }),
    ] {
        let (status, _) = app.send("PUT", "/api/preferences", &ana, bad).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_email_moves_to_a_new_address_once_its_code_is_entered(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    app.person("budi@acme.test").await;
    let (codes, mailer) = with_mailer(&pool);

    let (status, body) = codes
        .send(
            "POST",
            "/api/account/email",
            &ana,
            json!({ "email": "budi@acme.test" }),
        )
        .await;
    assert_eq!(
        (status, body["error"].clone()),
        (StatusCode::CONFLICT, json!("email_taken"))
    );
    let (status, _) = codes
        .send(
            "POST",
            "/api/account/email",
            &ana,
            json!({ "email": "ANA@acme.test" }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "the same address");

    let (status, body) = codes
        .send(
            "POST",
            "/api/account/email",
            &ana,
            json!({ "email": " Ana.New@Example.com " }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["purpose"], "change_email");
    assert_eq!(body["email"], "a******@example.com");
    let challenge = body["challenge_id"].clone();
    let code = code_sent_to(&mailer, "ana.new@example.com");

    // The code never signs anyone in, and only its own account can use it.
    let (status, _) = codes
        .call(
            "POST",
            "/api/auth/verify",
            None,
            Some(json!({ "challenge_id": challenge, "code": code })),
        )
        .await;
    assert_eq!(status, StatusCode::GONE);
    let budi = app.login("budi@acme.test").await;
    let (status, _) = codes
        .send(
            "POST",
            "/api/account/email/verify",
            &budi,
            json!({ "challenge_id": challenge, "code": code }),
        )
        .await;
    assert_eq!(status, StatusCode::GONE);
    let (status, body) = codes
        .send(
            "POST",
            "/api/account/email/verify",
            &ana,
            json!({ "challenge_id": challenge, "code": "000000" }),
        )
        .await;
    assert_eq!(
        (status, body["error"].clone()),
        (StatusCode::BAD_REQUEST, json!("wrong_code"))
    );

    let (status, body) = codes
        .send(
            "POST",
            "/api/account/email/verify",
            &ana,
            json!({ "challenge_id": challenge, "code": code }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["email"], "ana.new@example.com");
    let (status, _) = codes
        .send(
            "POST",
            "/api/account/email/verify",
            &ana,
            json!({ "challenge_id": challenge, "code": code }),
        )
        .await;
    assert_eq!(status, StatusCode::GONE, "used up");

    let (status, _) = app
        .call(
            "POST",
            "/api/auth/login",
            None,
            Some(json!({ "email": "ana@acme.test", "password": "correct-password" })),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    app.login("ana.new@example.com").await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn closing_an_account_needs_the_password_or_a_code_and_signs_out_everywhere(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    let (_, login) = app
        .call(
            "POST",
            "/api/auth/login",
            None,
            Some(json!({ "email": "ana@acme.test", "password": "correct-password" })),
        )
        .await;
    let refresh = login["refresh_token"].clone();
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM memberships WHERE user_id = $1")
        .bind(ana.id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let (status, body) = app.send("DELETE", "/api/account", &ana, json!({})).await;
    assert_eq!(
        (status, body["error"].clone()),
        (StatusCode::BAD_REQUEST, json!("password_or_code_required"))
    );
    let (status, body) = app
        .send(
            "DELETE",
            "/api/account",
            &ana,
            json!({ "password": "nope" }),
        )
        .await;
    assert_eq!(
        (status, body["error"].clone()),
        (StatusCode::FORBIDDEN, json!("wrong_password"))
    );

    let (status, _) = app
        .send(
            "DELETE",
            "/api/account",
            &ana,
            json!({ "password": "correct-password" }),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = app
        .call(
            "POST",
            "/api/auth/refresh",
            None,
            Some(json!({ "refresh_token": refresh })),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "signed out");
    let (status, _) = app
        .call(
            "POST",
            "/api/auth/login",
            None,
            Some(json!({ "email": "ana@acme.test", "password": "correct-password" })),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (deleted, org_closed, members): (bool, bool, i64) = sqlx::query_as(
        "SELECT (SELECT deleted_at IS NOT NULL FROM users WHERE id = $1), (SELECT deleted_at IS NOT NULL FROM organizations WHERE id = $2), \
                (SELECT count(*) FROM memberships WHERE user_id = $1 AND status = 'active')",
    )
    .bind(ana.id)
    .bind(org)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (deleted, org_closed, members),
        (true, true, 0),
        "the workspace only they were in closes with it"
    );
    // The address is free for a new account.
    app.person("ana@acme.test").await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_only_owner_of_a_shared_workspace_hands_it_over_first_and_a_code_also_closes_it(
    pool: PgPool,
) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    let budi = app.person("budi@acme.test").await;
    let (org, name): (Uuid, String) =
        sqlx::query_as("SELECT o.id, o.name FROM memberships m JOIN organizations o ON o.id = m.org_id WHERE m.user_id = $1").bind(ana.id).fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(org)
        .bind(budi.id)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = app
        .send(
            "DELETE",
            "/api/account",
            &ana,
            json!({ "password": "correct-password" }),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "sole_owner");
    assert_eq!(body["orgs"], json!([{ "id": org, "name": name }]));

    // Once Budi owns it too, Ana can go, with a code; the workspace stays for Budi.
    sqlx::query("UPDATE memberships SET role = 'owner' WHERE user_id = $1")
        .bind(budi.id)
        .execute(&pool)
        .await
        .unwrap();
    let (codes, mailer) = with_mailer(&pool);
    let (status, body) = codes
        .call("POST", "/api/account/delete/code", Some(&ana.token), None)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["purpose"], "delete_account");
    let code = code_sent_to(&mailer, "ana@acme.test");
    let (status, _) = codes
        .send(
            "DELETE",
            "/api/account",
            &ana,
            json!({ "challenge_id": body["challenge_id"], "code": "000000" }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = codes
        .send(
            "DELETE",
            "/api/account",
            &ana,
            json!({ "challenge_id": body["challenge_id"], "code": code }),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (org_open, budi_active): (bool, bool) = sqlx::query_as(
        "SELECT (SELECT deleted_at IS NULL FROM organizations WHERE id = $1), \
                EXISTS (SELECT 1 FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active')",
    )
    .bind(org)
    .bind(budi.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(org_open && budi_active);
}
