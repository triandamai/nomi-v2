use std::sync::Arc;

use futures_util::StreamExt;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmProvider, LlmRequest, LlmResponse, StopReason};
use nomi_test_support::FakeLlmProvider;
use nomi_usage::{MeteredProvider, ModelTag};

async fn person(pool: &PgPool) -> Uuid {
    let id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    sqlx::query("INSERT INTO user_preferences (user_id, timezone) VALUES ($1, 'UTC')").bind(id).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO user_subscriptions (user_id, plan_id, quota_override) SELECT $1, id, 100000000 FROM plans ORDER BY sort_order LIMIT 1")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn chat(pool: &PgPool, org: Uuid) -> Uuid {
    sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', $2) RETURNING id").bind(org).bind(Uuid::new_v4().to_string()).fetch_one(pool).await.unwrap()
}

/// A project and its own chat.
async fn project(pool: &PgPool, owner: Uuid, org: Uuid) -> (Uuid, Uuid) {
    let session = chat(pool, org).await;
    let id = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, org_id, name) VALUES ($1, $2, $3, 'Toko Roti Budi') RETURNING id")
        .bind(owner)
        .bind(session)
        .bind(org)
        .fetch_one(pool)
        .await
        .unwrap();
    (id, session)
}

/// An admin model at $3 / $15 per million input / output tokens.
async fn priced_model(pool: &PgPool, admin: Uuid) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, is_default, updated_by, input_usd_per_mtok, output_usd_per_mtok) \
         VALUES ('Nomi Standard', 'anthropic', 'claude-x', '\\x00', true, $1, 3, 15) RETURNING id",
    )
    .bind(admin)
    .fetch_one(pool)
    .await
    .unwrap()
}

fn tag(admin_model_id: Option<Uuid>, own_key: bool) -> ModelTag {
    ModelTag { admin_model_id, own_key, label: "Nomi Standard".into(), provider: "anthropic".into(), model_id: "claude-x".into(), connection_id: None }
}

/// One call of `input` / `output` tokens by `user` in chat `session`.
async fn call(pool: &PgPool, user: Uuid, session: Option<Uuid>, tag: ModelTag, input: u32, output: u32) {
    let reply = LlmResponse { content: vec![ContentBlock::Text { text: "ok".into() }], stop_reason: StopReason::EndTurn, input_tokens: input, output_tokens: output };
    let provider = MeteredProvider::new(Arc::new(FakeLlmProvider::sequence(vec![reply])), pool.clone(), user, tag).in_session(session);
    let request = LlmRequest { system: None, messages: vec![], tools: vec![], max_tokens: 100, enable_reasoning: false, reasoning_effort: Default::default() };
    let mut events = provider.complete_stream(request).await.unwrap();
    while events.next().await.is_some() {}
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_projects_usage_is_its_own_chat_and_the_chats_filed_into_it(pool: PgPool) {
    let budi = person(&pool).await;
    let sari = person(&pool).await;
    let model = priced_model(&pool, budi).await;
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Toko Roti') RETURNING id").fetch_one(&pool).await.unwrap();
    let (project_id, own_chat) = project(&pool, budi, org).await;
    let filed = chat(&pool, org).await;
    sqlx::query("UPDATE sessions SET in_project_id = $1 WHERE id = $2").bind(project_id).bind(filed).execute(&pool).await.unwrap();
    let elsewhere = chat(&pool, org).await;
    // The chat the crew works an issue in counts too.
    let issue_chat = chat(&pool, org).await;
    sqlx::query("INSERT INTO project_issues (project_id, number, title, kind, severity, session_id) VALUES ($1, 1, 'Checkout button broken', 'bug', 'high', $2)")
        .bind(project_id)
        .bind(issue_chat)
        .execute(&pool)
        .await
        .unwrap();

    call(&pool, budi, Some(own_chat), tag(Some(model), false), 1_000_000, 100_000).await;
    call(&pool, sari, Some(filed), tag(Some(model), false), 200_000, 0).await;
    call(&pool, sari, Some(filed), tag(None, true), 50_000, 0).await;
    call(&pool, budi, Some(issue_chat), tag(Some(model), false), 100_000, 0).await;
    // Neither counts toward the project: another chat, and a call made in no chat.
    call(&pool, budi, Some(elsewhere), tag(Some(model), false), 9_000_000, 0).await;
    call(&pool, budi, None, tag(Some(model), false), 9_000_000, 0).await;

    let usage = nomi_usage::project::project_month(&pool, project_id, budi, None, true).await.unwrap();
    assert_eq!(usage.calls, 4);
    assert_eq!(usage.tokens, 1_450_000);
    assert_eq!(usage.own_key_tokens, 50_000);
    // 1.3M in at $3 + 0.1M out at $15; the own-key call is never billed.
    assert!((usage.spend_usd - 5.4).abs() < 1e-9, "{}", usage.spend_usd);
    assert!(usage.counted_since.is_some());

    assert_eq!(usage.by_person.len(), 2);
    assert_eq!((usage.by_person[0].user_id, usage.by_person[0].tokens), (budi, 1_200_000));
    assert_eq!((usage.by_person[1].user_id, usage.by_person[1].tokens), (sari, 250_000));
    assert_eq!(usage.by_model.iter().map(|m| m.calls).sum::<i64>(), 4);

    // Someone who may not see who spent what gets the totals without anyone's part.
    let without = nomi_usage::project::project_month(&pool, project_id, sari, None, false).await.unwrap();
    assert!(without.by_person.is_empty());
    assert_eq!(without.calls, 4);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_deleted_chats_calls_stay_on_the_persons_month(pool: PgPool) {
    let budi = person(&pool).await;
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Toko Roti') RETURNING id").fetch_one(&pool).await.unwrap();
    let gone = chat(&pool, org).await;
    call(&pool, budi, Some(gone), tag(None, false), 1_000, 0).await;
    sqlx::query("DELETE FROM sessions WHERE id = $1").bind(gone).execute(&pool).await.unwrap();
    // A call recorded after its chat was deleted is still kept, with no chat.
    call(&pool, budi, Some(gone), tag(None, false), 2_000, 0).await;

    let month = nomi_usage::month(&pool, budi, None).await.unwrap();
    assert_eq!(month.calls, 2);
    let chats: i64 = sqlx::query_scalar("SELECT count(*) FROM llm_usage WHERE session_id IS NOT NULL").fetch_one(&pool).await.unwrap();
    assert_eq!(chats, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_with_no_calls_reads_as_zero(pool: PgPool) {
    let budi = person(&pool).await;
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Toko Roti') RETURNING id").fetch_one(&pool).await.unwrap();
    let (project_id, _) = project(&pool, budi, org).await;
    let usage = nomi_usage::project::project_month(&pool, project_id, budi, Some("2026-11"), true).await.unwrap();
    assert_eq!((usage.month.as_str(), usage.calls, usage.tokens, usage.spend_usd), ("2026-11", 0, 0, 0.0));
    assert!(usage.by_model.is_empty() && usage.by_person.is_empty());
    assert_eq!(usage.counted_since, None);
}
