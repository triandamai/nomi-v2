use nomi_orchestrator::settings::{get_settings, mask_api_key, upsert_settings, UpsertInput};
use sqlx::PgPool;
use uuid::Uuid;

async fn insert_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test]
async fn get_settings_returns_none_when_no_row_exists(pool: PgPool) {
    let row = get_settings(&pool, "llm").await.unwrap();
    assert!(row.is_none());
}

#[sqlx::test]
async fn upsert_then_get_roundtrips_the_row(pool: PgPool) {
    let user_id = insert_user(&pool).await;

    upsert_settings(
        &pool,
        UpsertInput {
            setting_type: "llm",
            provider: "anthropic",
            model_id: "claude-haiku-4-5",
            api_key_encrypted: vec![1, 2, 3],
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();

    let row = get_settings(&pool, "llm").await.unwrap().unwrap();
    assert_eq!(row.provider, "anthropic");
    assert_eq!(row.model_id, "claude-haiku-4-5");
    assert_eq!(row.api_key_encrypted, vec![1, 2, 3]);
    assert_eq!(row.base_url, None);
}

#[sqlx::test]
async fn upsert_twice_updates_the_same_row_instead_of_inserting_a_second_one(pool: PgPool) {
    let user_id = insert_user(&pool).await;

    for model_id in ["claude-haiku-4-5", "claude-sonnet-5"] {
        upsert_settings(
            &pool,
            UpsertInput {
                setting_type: "llm",
                provider: "anthropic",
                model_id,
                api_key_encrypted: vec![1, 2, 3],
                base_url: None,
                updated_by: user_id,
            },
        )
        .await
        .unwrap();
    }

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM provider_settings WHERE setting_type = 'llm'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);

    let row = get_settings(&pool, "llm").await.unwrap().unwrap();
    assert_eq!(row.model_id, "claude-sonnet-5");
}

#[sqlx::test]
async fn llm_and_embedding_rows_are_independent(pool: PgPool) {
    let user_id = insert_user(&pool).await;

    upsert_settings(
        &pool,
        UpsertInput {
            setting_type: "llm",
            provider: "anthropic",
            model_id: "claude-haiku-4-5",
            api_key_encrypted: vec![1],
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    upsert_settings(
        &pool,
        UpsertInput {
            setting_type: "embedding",
            provider: "openai",
            model_id: "text-embedding-3-small",
            api_key_encrypted: vec![2],
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();

    assert_eq!(get_settings(&pool, "llm").await.unwrap().unwrap().provider, "anthropic");
    assert_eq!(get_settings(&pool, "embedding").await.unwrap().unwrap().provider, "openai");
}

#[test]
fn mask_api_key_shows_only_the_last_four_characters() {
    assert_eq!(mask_api_key("sk-abcdefgh1234"), "...1234");
}

#[test]
fn mask_api_key_of_an_empty_key_is_empty() {
    assert_eq!(mask_api_key(""), "");
}
