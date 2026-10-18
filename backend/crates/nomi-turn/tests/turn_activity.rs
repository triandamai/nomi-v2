use sqlx::PgPool;
use uuid::Uuid;

use nomi_realtime::UserEnvelope;
use nomi_turn::activity::{chat_activity, Ended};

/// A chat with its owner's channel identity: (session, identity, user).
async fn seed_chat(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ('t', true) RETURNING id").fetch_one(pool).await.unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let identity: Uuid = sqlx::query_scalar("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'web', $2) RETURNING id")
        .bind(user_id)
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_type, chat_id, user_id, title) VALUES ($1, 'web', 'dm', 'c', $2, 'Trip to Bandung') RETURNING id")
        .bind(org_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap();
    (session_id, identity, user_id)
}

async fn say(pool: &PgPool, session_id: Uuid, from: Option<Uuid>, agent: Option<&str>, blocks: Option<serde_json::Value>) {
    sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name, content_blocks, created_at) VALUES ($1, $2, 'x', $3, $4, clock_timestamp())")
        .bind(session_id)
        .bind(from)
        .bind(agent)
        .bind(blocks)
        .execute(pool)
        .await
        .unwrap();
}

fn outcome(envelope: Option<UserEnvelope>) -> Option<(String, Option<String>, Option<String>)> {
    envelope.map(|UserEnvelope::ChatActivity { outcome, waiting, agent, .. }| (outcome, waiting, agent))
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_finished_chat_says_done_with_who_answered(pool: PgPool) {
    let (session_id, me, _) = seed_chat(&pool).await;
    say(&pool, session_id, Some(me), None, None).await;
    say(&pool, session_id, None, Some("Finley"), None).await;

    let envelope = chat_activity(&pool, session_id, Ended::Completed).await.unwrap();
    let Some(UserEnvelope::ChatActivity { title, project_id, .. }) = envelope.clone() else { panic!("expected an activity") };
    assert_eq!((title.as_deref(), project_id), (Some("Trip to Bandung"), None));
    assert_eq!(outcome(envelope), Some(("done".into(), None, Some("Finley".into()))));
    assert_eq!(outcome(chat_activity(&pool, session_id, Ended::Failed).await.unwrap()).unwrap().0, "failed");
}

#[sqlx::test(migrations = "../../migrations")]
async fn nothing_is_said_while_a_crew_member_still_has_the_work(pool: PgPool) {
    let (session_id, me, user_id) = seed_chat(&pool).await;
    say(&pool, session_id, Some(me), None, None).await;
    say(&pool, session_id, None, Some("Nomi"), None).await;
    sqlx::query("INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status) VALUES ($1, $2, 'chitchat', 'money', 't', 'processing')")
        .bind(session_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(chat_activity(&pool, session_id, Ended::Completed).await.unwrap(), None);

    sqlx::query("UPDATE agent_delegations SET status = 'completed'").execute(&pool).await.unwrap();
    assert_eq!(outcome(chat_activity(&pool, session_id, Ended::Completed).await.unwrap()).unwrap().0, "done");
}

#[sqlx::test(migrations = "../../migrations")]
async fn something_waiting_on_the_person_is_said_right_away(pool: PgPool) {
    let (session_id, me, user_id) = seed_chat(&pool).await;
    say(&pool, session_id, Some(me), None, None).await;
    // A delegated agent paused for approval still counts as running; the ask goes out anyway.
    sqlx::query("INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status) VALUES ($1, $2, 'chitchat', 'workspace', 't', 'processing')")
        .bind(session_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let approval = serde_json::json!([{"kind": "approval_request", "id": "a", "tool_name": "gmail_send", "description": "", "input": {}, "status": "pending", "decided_at": null}]);
    say(&pool, session_id, None, Some("Tara"), Some(approval)).await;
    assert_eq!(outcome(chat_activity(&pool, session_id, Ended::Completed).await.unwrap()), Some(("waiting".into(), Some("approval".into()), Some("Tara".into()))));
}

#[sqlx::test(migrations = "../../migrations")]
async fn questions_decisions_and_forms_wait_for_an_answer_but_older_ones_do_not(pool: PgPool) {
    let (session_id, me, _) = seed_chat(&pool).await;
    say(&pool, session_id, Some(me), None, None).await;
    say(&pool, session_id, None, Some("Rena"), Some(serde_json::json!([{"kind": "view", "view": "decision", "question": "Which?", "options": []}]))).await;
    assert_eq!(outcome(chat_activity(&pool, session_id, Ended::Completed).await.unwrap()).unwrap().1.as_deref(), Some("question"));

    // The person answered; the crew's reply to that isn't waiting on anything.
    say(&pool, session_id, Some(me), None, None).await;
    say(&pool, session_id, None, Some("Rena"), Some(serde_json::json!([{"kind": "view", "view": "answer", "value": 1}]))).await;
    assert_eq!(outcome(chat_activity(&pool, session_id, Ended::Completed).await.unwrap()), Some(("done".into(), None, Some("Rena".into()))));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_chat_carries_the_project_and_its_name(pool: PgPool) {
    let (session_id, me, user_id) = seed_chat(&pool).await;
    let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Landing page') RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    say(&pool, session_id, Some(me), None, None).await;
    say(&pool, session_id, None, Some("Koda"), None).await;
    let Some(UserEnvelope::ChatActivity { title, project_id: got, .. }) = chat_activity(&pool, session_id, Ended::Completed).await.unwrap() else { panic!() };
    assert_eq!((title.as_deref(), got), (Some("Landing page"), Some(project_id)));
}
