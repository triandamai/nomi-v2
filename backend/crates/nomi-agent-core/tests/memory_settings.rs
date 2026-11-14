use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::memory::{self, RetrievedMemory};
use nomi_agent_core::memory_settings::{self, MemorySettings};
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider};

fn answer(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text {
            text: text.to_string(),
        }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 5,
        output_tokens: 3,
    }
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// A unit vector along `axis`, so memories on different axes aren't duplicates.
fn unit(axis: usize) -> Vec<f32> {
    let mut v = vec![0.0; 1536];
    v[axis] = 1.0;
    v
}

async fn seed_memory(pool: &PgPool, user: Uuid, content: &str, weight: f64, axis: usize) {
    let literal = format!(
        "[{}]",
        unit(axis)
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    sqlx::query("INSERT INTO memory_items (user_id, content, embedding, embedding_provider, embedding_model, weight) VALUES ($1, $2, $3::vector, 'fake', 'fake-model', $4)")
        .bind(user)
        .bind(content)
        .bind(literal)
        .bind(weight)
        .execute(pool)
        .await
        .unwrap();
}

#[test]
fn recall_stops_at_the_memory_budget() {
    let m = |content: &str| RetrievedMemory {
        id: Uuid::new_v4(),
        content: content.to_string(),
        kind: "fact".into(),
    };
    let found = vec![m("aaaa"), m("bbbbbbbb"), m("cccc")];
    let kept: Vec<String> = memory::within_budget(found, 3)
        .into_iter()
        .map(|m| m.content)
        .collect();
    assert_eq!(kept, vec!["aaaa", "bbbbbbbb"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_memory_past_the_budget_retires_the_weakest(pool: PgPool) {
    let user = seed_user(&pool).await;
    seed_memory(&pool, user, &"w".repeat(40), 0.5, 1).await;
    memory_settings::save(
        &pool,
        user,
        &MemorySettings {
            memory_budget: 12,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(answer(
        r#"{"action":"add","kind":"preference","text":"Vegetarian"}"#,
    ));
    memory::extract_and_store_memory(
        &mut conn,
        &llm,
        &FakeEmbeddingProvider::success(unit(0)),
        user,
        "I don't eat meat",
        "Noted!",
    )
    .await;

    let live: Vec<String> = sqlx::query_scalar(
        "SELECT content FROM memory_items WHERE user_id = $1 AND archived_at IS NULL",
    )
    .bind(user)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(live, vec!["Vegetarian".to_string()]);
    assert_eq!(
        memory_settings::memory_tokens(&pool, user).await.unwrap(),
        3
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_queued_exchange_is_dropped_once_learning_is_off(pool: PgPool) {
    let user = seed_user(&pool).await;
    sqlx::query("INSERT INTO memory_jobs (user_id, person_text, answer, agent_type) VALUES ($1, 'I run', 'Nice', 'health')").bind(user).execute(&pool).await.unwrap();
    memory_settings::save(
        &pool,
        user,
        &MemorySettings {
            learn_from_chats: false,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let job = memory::take_learning_jobs(&pool, 10)
        .await
        .unwrap()
        .remove(0);
    assert_eq!(job.agent_type.as_deref(), Some("health"));
    let llm =
        FakeLlmProvider::success(answer(r#"{"action":"add","kind":"routine","text":"Runs"}"#));
    let learned =
        memory::run_learning_job(&pool, &llm, &FakeEmbeddingProvider::success(unit(0)), &job)
            .await
            .unwrap();
    assert_eq!(learned, memory::Learned::Nothing);
    assert!(
        llm.received_requests.lock().unwrap().is_empty(),
        "no model call"
    );
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM memory_jobs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(left, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_profile_is_rewritten_from_memories_within_its_budget(pool: PgPool) {
    let user = seed_user(&pool).await;
    assert!(
        memory_settings::profiles_to_refresh(&pool, 10)
            .await
            .unwrap()
            .is_empty(),
        "no memories, nothing to write"
    );
    seed_memory(&pool, user, "Vegetarian", 2.0, 0).await;
    assert_eq!(
        memory_settings::profiles_to_refresh(&pool, 10)
            .await
            .unwrap(),
        vec![user]
    );

    memory_settings::save(
        &pool,
        user,
        &MemorySettings {
            profile_budget: 3,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let llm = FakeLlmProvider::success(answer("Vegetarian designer in Jakarta"));
    assert!(memory_settings::refresh_profile(&pool, &llm, user)
        .await
        .unwrap());
    let profile = memory_settings::profile(&pool, user).await.unwrap();
    assert_eq!(profile.text, "Vegetarian");
    let request = llm.received_requests.lock().unwrap()[0].clone();
    assert_eq!(
        request.system.as_deref(),
        Some(nomi_agent_core::prompts::MEMORY_PROFILE_SYSTEM_PROMPT)
    );
    let ContentBlock::Text { text } = &request.messages[0].content[0] else {
        panic!("text input")
    };
    assert!(text.contains("(fact) Vegetarian"), "{text}");
    // Just written: not due again.
    assert!(memory_settings::profiles_to_refresh(&pool, 10)
        .await
        .unwrap()
        .is_empty());

    // Off: not rewritten, and agents don't read it.
    let off = MemorySettings {
        keep_profile: false,
        ..Default::default()
    };
    memory_settings::save(&pool, user, &off).await.unwrap();
    assert!(!memory_settings::refresh_profile(&pool, &llm, user)
        .await
        .unwrap());
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(
        memory_settings::profile_for_agents(&mut conn, user, &off).await,
        None
    );
    assert_eq!(
        memory_settings::profile_for_agents(&mut conn, user, &MemorySettings::default())
            .await
            .as_deref(),
        Some("Vegetarian")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_profile_rewrite_keeps_the_old_one(pool: PgPool) {
    let user = seed_user(&pool).await;
    seed_memory(&pool, user, "Vegetarian", 2.0, 0).await;
    memory_settings::set_profile(&pool, user, "Mine")
        .await
        .unwrap();
    let llm = FakeLlmProvider::failure("down");
    assert!(!memory_settings::refresh_profile(&pool, &llm, user)
        .await
        .unwrap());
    assert_eq!(
        memory_settings::profile(&pool, user).await.unwrap().text,
        "Mine"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn chats_are_not_noted_for_people_who_turned_off_past_chats(pool: PgPool) {
    let user = seed_user(&pool).await;
    let org: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id, user_id) VALUES ($1, 'web', $2, $3) RETURNING id")
        .bind(org)
        .bind(user.to_string())
        .bind(user)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO messages (session_id, content, created_at) VALUES ($1, 'hello', now() - interval '10 minutes')").bind(session).execute(&pool).await.unwrap();
    assert_eq!(
        nomi_agent_core::recent_activity::chats_gone_quiet(&pool, 10)
            .await
            .unwrap(),
        vec![(session, user)]
    );
    memory_settings::save(
        &pool,
        user,
        &MemorySettings {
            search_past_chats: false,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(
        nomi_agent_core::recent_activity::chats_gone_quiet(&pool, 10)
            .await
            .unwrap()
            .is_empty()
    );
}
