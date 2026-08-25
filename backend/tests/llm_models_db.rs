use nomi_orchestrator::settings::llm_models::{
    create_admin_llm_model, delete_admin_llm_model, get_admin_llm_model, get_default_admin_llm_model,
    get_user_llm_selection, list_admin_llm_models, set_default_admin_llm_model, set_user_llm_selection_admin,
    set_user_llm_selection_custom, update_admin_llm_model, CustomLlmSelection, DeleteAdminLlmModelError,
    NewAdminLlmModel, SetDefaultAdminLlmModelError, UpdateAdminLlmModel,
};
use sqlx::PgPool;
use uuid::Uuid;

async fn insert_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn new_model_input(label: &str, updated_by: Uuid) -> NewAdminLlmModel<'_> {
    NewAdminLlmModel {
        label,
        provider: "anthropic",
        model_id: "claude-haiku-4-5",
        api_key_encrypted: vec![1, 2, 3],
        base_url: None,
        updated_by,
    }
}

#[sqlx::test]
async fn the_first_model_created_becomes_the_default(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    assert!(model.is_default);
}

#[sqlx::test]
async fn a_second_model_created_is_not_the_default(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    let second = create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();
    assert!(!second.is_default);
}

#[sqlx::test]
async fn list_returns_all_models_in_creation_order(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();

    let models = list_admin_llm_models(&pool).await.unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(models[0].label, "First");
    assert_eq!(models[1].label, "Second");
}

#[sqlx::test]
async fn get_default_admin_llm_model_returns_none_when_no_models_exist(pool: PgPool) {
    assert!(get_default_admin_llm_model(&pool).await.unwrap().is_none());
}

#[sqlx::test]
async fn update_changes_fields_and_keeps_existing_key_when_none_given(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();

    let updated = update_admin_llm_model(
        &pool,
        model.id,
        UpdateAdminLlmModel {
            label: "Renamed",
            provider: "openai",
            model_id: "gpt-4o",
            api_key_encrypted: None,
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(updated.label, "Renamed");
    assert_eq!(updated.provider, "openai");
    assert_eq!(updated.model_id, "gpt-4o");
    assert_eq!(updated.api_key_encrypted, vec![1, 2, 3]);
}

#[sqlx::test]
async fn update_replaces_the_key_when_one_is_given(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();

    let updated = update_admin_llm_model(
        &pool,
        model.id,
        UpdateAdminLlmModel {
            label: "First",
            provider: "anthropic",
            model_id: "claude-haiku-4-5",
            api_key_encrypted: Some(vec![9, 9, 9]),
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap()
    .unwrap();

    assert_eq!(updated.api_key_encrypted, vec![9, 9, 9]);
}

#[sqlx::test]
async fn update_of_a_nonexistent_model_returns_none(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let result = update_admin_llm_model(
        &pool,
        Uuid::new_v4(),
        UpdateAdminLlmModel {
            label: "X",
            provider: "anthropic",
            model_id: "x",
            api_key_encrypted: None,
            base_url: None,
            updated_by: user_id,
        },
    )
    .await
    .unwrap();
    assert!(result.is_none());
}

#[sqlx::test]
async fn delete_rejects_the_only_model_since_it_is_always_the_default(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("Only", user_id)).await.unwrap();

    let result = delete_admin_llm_model(&pool, model.id).await;
    assert!(matches!(result, Err(DeleteAdminLlmModelError::IsDefault)));
}

#[sqlx::test]
async fn delete_rejects_the_current_default_even_with_others_present(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let first = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();

    let result = delete_admin_llm_model(&pool, first.id).await;
    assert!(matches!(result, Err(DeleteAdminLlmModelError::IsDefault)));
}

#[sqlx::test]
async fn delete_succeeds_for_a_non_default_model_when_others_remain(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    let second = create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();

    delete_admin_llm_model(&pool, second.id).await.unwrap();
    assert!(get_admin_llm_model(&pool, second.id).await.unwrap().is_none());
}

#[sqlx::test]
async fn set_default_swaps_the_default_flag_to_exactly_the_new_model(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let first = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();
    let second = create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();

    set_default_admin_llm_model(&pool, second.id).await.unwrap();

    let first = get_admin_llm_model(&pool, first.id).await.unwrap().unwrap();
    let second = get_admin_llm_model(&pool, second.id).await.unwrap().unwrap();
    assert!(!first.is_default);
    assert!(second.is_default);
}

#[sqlx::test]
async fn set_default_of_a_nonexistent_model_returns_not_found(pool: PgPool) {
    let result = set_default_admin_llm_model(&pool, Uuid::new_v4()).await;
    assert!(matches!(result, Err(SetDefaultAdminLlmModelError::NotFound)));
}

#[sqlx::test]
async fn a_user_with_no_selection_row_resolves_to_none(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    assert!(get_user_llm_selection(&pool, user_id).await.unwrap().is_none());
}

#[sqlx::test]
async fn setting_an_admin_selection_then_a_custom_one_clears_the_admin_reference(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();

    set_user_llm_selection_admin(&pool, user_id, model.id).await.unwrap();
    let row = get_user_llm_selection(&pool, user_id).await.unwrap().unwrap();
    assert_eq!(row.admin_model_id, Some(model.id));

    set_user_llm_selection_custom(
        &pool,
        user_id,
        CustomLlmSelection {
            label: "My key",
            provider: "openai",
            model_id: "gpt-4o",
            api_key_encrypted: vec![4, 5, 6],
            base_url: None,
        },
    )
    .await
    .unwrap();

    let row = get_user_llm_selection(&pool, user_id).await.unwrap().unwrap();
    assert_eq!(row.admin_model_id, None);
    assert_eq!(row.custom_provider, Some("openai".to_string()));
    assert_eq!(row.custom_model_id, Some("gpt-4o".to_string()));
}

#[sqlx::test]
async fn setting_a_custom_selection_then_an_admin_one_clears_the_custom_fields(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    let model = create_admin_llm_model(&pool, new_model_input("First", user_id)).await.unwrap();

    set_user_llm_selection_custom(
        &pool,
        user_id,
        CustomLlmSelection {
            label: "My key",
            provider: "openai",
            model_id: "gpt-4o",
            api_key_encrypted: vec![4, 5, 6],
            base_url: None,
        },
    )
    .await
    .unwrap();

    set_user_llm_selection_admin(&pool, user_id, model.id).await.unwrap();

    let row = get_user_llm_selection(&pool, user_id).await.unwrap().unwrap();
    assert_eq!(row.admin_model_id, Some(model.id));
    assert_eq!(row.custom_provider, None);
    assert_eq!(row.custom_model_id, None);
}

#[sqlx::test]
async fn deleting_a_referenced_admin_model_clears_the_users_reference(pool: PgPool) {
    let user_id = insert_user(&pool).await;
    create_admin_llm_model(&pool, new_model_input("Default", user_id)).await.unwrap();
    let second = create_admin_llm_model(&pool, new_model_input("Second", user_id)).await.unwrap();

    set_user_llm_selection_admin(&pool, user_id, second.id).await.unwrap();
    delete_admin_llm_model(&pool, second.id).await.unwrap();

    let row = get_user_llm_selection(&pool, user_id).await.unwrap().unwrap();
    assert_eq!(row.admin_model_id, None);
}
