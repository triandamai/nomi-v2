use nomi_orchestrator::bootstrap::resolve_llm_model_config;
use nomi_orchestrator::llm::ProviderKind;
use nomi_orchestrator::settings::crypto;
use nomi_orchestrator::settings::llm_models::{
    create_admin_llm_model, delete_admin_llm_model, set_user_llm_selection_admin, set_user_llm_selection_custom,
    CustomLlmSelection, NewAdminLlmModel,
};
use sqlx::PgPool;
use uuid::Uuid;

const SETTINGS_KEY: [u8; 32] = [7u8; 32];
const OTHER_KEY: [u8; 32] = [9u8; 32];

// `cargo test` runs the tests in this binary concurrently on separate threads, but
// `std::env::set_var`/`remove_var` mutate whole-process state. Two tests below exercise the
// LLM_PROVIDER env-var fallback path; without serializing them they can race (one test's
// `remove_var` firing between another's `set_var` and its `resolve_llm_model_config` call),
// causing an intermittent "LLM_PROVIDER must be set" panic unrelated to the behavior under test.
static ENV_VAR_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn insert_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

#[sqlx::test]
async fn resolves_to_the_users_admin_selection_when_set(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Picked",
            provider: "anthropic",
            model_id: "claude-haiku-4-5",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-picked"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    set_user_llm_selection_admin(&pool, user_id, model.id).await.unwrap();

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::Anthropic);
    assert_eq!(config.model_id, "claude-haiku-4-5");
    assert_eq!(config.api_key, "sk-picked");
}

#[sqlx::test]
async fn resolves_to_the_users_custom_selection_when_set(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    set_user_llm_selection_custom(
        &pool,
        user_id,
        CustomLlmSelection {
            label: "Mine",
            provider: "openai",
            model_id: "gpt-4o",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-mine"),
            base_url: Some("https://api.openai.com/v1"),
        },
    )
    .await
    .unwrap();

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::OpenAi);
    assert_eq!(config.model_id, "gpt-4o");
    assert_eq!(config.api_key, "sk-mine");
    assert_eq!(config.base_url, Some("https://api.openai.com/v1".to_string()));
}

#[sqlx::test]
async fn falls_back_to_the_admin_default_when_the_user_has_no_selection(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Default",
            provider: "gemini",
            model_id: "gemini-2.5-flash",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-default"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::Gemini);
    assert_eq!(config.api_key, "sk-default");
}

#[sqlx::test]
async fn falls_back_to_the_admin_default_when_the_referenced_model_was_deleted(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Default",
            provider: "gemini",
            model_id: "gemini-2.5-flash",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-default"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    let picked = create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Picked",
            provider: "anthropic",
            model_id: "claude-haiku-4-5",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-picked"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    set_user_llm_selection_admin(&pool, user_id, picked.id).await.unwrap();
    delete_admin_llm_model(&pool, picked.id).await.unwrap();

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::Gemini);
    assert_eq!(config.api_key, "sk-default");
}

#[sqlx::test]
async fn falls_back_to_the_env_var_config_when_no_admin_models_exist_at_all(pool: PgPool) {
    let _guard = ENV_VAR_LOCK.lock().await;
    let user_id = insert_user(&pool).await;
    std::env::set_var("LLM_PROVIDER", "fake");

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::Fake);

    std::env::remove_var("LLM_PROVIDER");
}

#[sqlx::test]
async fn falls_back_to_the_admin_default_when_the_users_admin_selection_key_cannot_be_decrypted(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Default",
            provider: "gemini",
            model_id: "gemini-2.5-flash",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-default"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    let picked = create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Picked (undecryptable)",
            provider: "anthropic",
            model_id: "claude-haiku-4-5",
            api_key_encrypted: crypto::encrypt(&OTHER_KEY, "sk-picked"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    set_user_llm_selection_admin(&pool, user_id, picked.id).await.unwrap();

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::Gemini);
    assert_eq!(config.api_key, "sk-default");
}

#[sqlx::test]
async fn falls_back_to_the_admin_default_when_the_users_custom_key_cannot_be_decrypted(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Default",
            provider: "gemini",
            model_id: "gemini-2.5-flash",
            api_key_encrypted: crypto::encrypt(&SETTINGS_KEY, "sk-default"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    set_user_llm_selection_custom(
        &pool,
        user_id,
        CustomLlmSelection {
            label: "Mine (undecryptable)",
            provider: "openai",
            model_id: "gpt-4o",
            api_key_encrypted: crypto::encrypt(&OTHER_KEY, "sk-mine"),
            base_url: None,
        },
    )
    .await
    .unwrap();

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::Gemini);
    assert_eq!(config.api_key, "sk-default");
}

#[sqlx::test]
async fn falls_back_to_the_env_config_when_the_default_admin_models_key_cannot_be_decrypted(pool: PgPool) {
    let _guard = ENV_VAR_LOCK.lock().await;
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(
        &pool,
        NewAdminLlmModel {
            label: "Default (undecryptable)",
            provider: "gemini",
            model_id: "gemini-2.5-flash",
            api_key_encrypted: crypto::encrypt(&OTHER_KEY, "sk-default"),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    std::env::set_var("LLM_PROVIDER", "fake");

    let config = resolve_llm_model_config(&pool, user_id, &SETTINGS_KEY).await;
    assert_eq!(config.provider, ProviderKind::Fake);

    std::env::remove_var("LLM_PROVIDER");
}
