use nomi_settings::provider_connections::{
    create_connection, delete_connection, get_connection, list_connections, models_using, update_connection, ConnectionUpdate, CreateConnectionError,
    DeleteConnectionError, NewConnection, Scope,
};
use sqlx::PgPool;
use uuid::Uuid;

async fn insert_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn api_key_connection<'a>(provider_id: &'a str, kind: &'a str, updated_by: Uuid) -> NewConnection<'a> {
    NewConnection {
        provider_id,
        label: provider_id,
        kind,
        auth_type: "api_key",
        api_key_encrypted: Some(vec![1, 2, 3]),
        base_url: None,
        headers_encrypted: None,
        updated_by,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn connections_are_kept_apart_per_person_and_for_the_instance(pool: PgPool) {
    let ana = insert_user(&pool).await;
    let budi = insert_user(&pool).await;

    let shared = create_connection(&pool, Scope::Admin, api_key_connection("openai", "openai", ana)).await.unwrap();
    let mine = create_connection(&pool, Scope::User(ana), api_key_connection("openai", "openai", ana)).await.unwrap();

    assert_eq!(list_connections(&pool, Scope::Admin).await.unwrap().iter().map(|c| c.id).collect::<Vec<_>>(), [shared.id]);
    assert_eq!(list_connections(&pool, Scope::User(ana)).await.unwrap().iter().map(|c| c.id).collect::<Vec<_>>(), [mine.id]);
    assert!(list_connections(&pool, Scope::User(budi)).await.unwrap().is_empty());

    // Someone else's connection (or the instance's) can't be read through a person's scope.
    assert!(get_connection(&pool, Scope::User(budi), mine.id).await.unwrap().is_none());
    assert!(get_connection(&pool, Scope::User(ana), shared.id).await.unwrap().is_none());
    assert!(get_connection(&pool, Scope::Admin, mine.id).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_provider_is_connected_once_per_scope_but_custom_endpoints_can_repeat(pool: PgPool) {
    let ana = insert_user(&pool).await;
    create_connection(&pool, Scope::User(ana), api_key_connection("anthropic", "anthropic", ana)).await.unwrap();
    let again = create_connection(&pool, Scope::User(ana), api_key_connection("anthropic", "anthropic", ana)).await;
    assert!(matches!(again, Err(CreateConnectionError::AlreadyConnected)));

    let custom = |label| NewConnection {
        provider_id: "custom",
        label,
        kind: "openai_compatible",
        auth_type: "custom_endpoint",
        api_key_encrypted: None,
        base_url: Some("https://llm.example.com/v1"),
        headers_encrypted: Some(vec![9, 9]),
        updated_by: ana,
    };
    create_connection(&pool, Scope::User(ana), custom("Team gateway")).await.unwrap();
    create_connection(&pool, Scope::User(ana), custom("Lab")).await.unwrap();
    let same_name = create_connection(&pool, Scope::User(ana), custom("team gateway")).await;
    assert!(matches!(same_name, Err(CreateConnectionError::AlreadyConnected)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn updating_keeps_what_was_not_given(pool: PgPool) {
    let ana = insert_user(&pool).await;
    let connection = create_connection(&pool, Scope::User(ana), api_key_connection("openai", "openai", ana)).await.unwrap();

    let updated = update_connection(&pool, Scope::User(ana), connection.id, ConnectionUpdate { api_key_encrypted: Some(vec![7, 7]), updated_by: ana, ..Default::default() })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.api_key_encrypted, Some(vec![7, 7]));
    assert_eq!(updated.label, "openai");

    let other = insert_user(&pool).await;
    let not_theirs = update_connection(&pool, Scope::User(other), connection.id, ConnectionUpdate { label: Some("x"), updated_by: other, ..Default::default() }).await.unwrap();
    assert!(not_theirs.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_instance_connection_in_use_by_models_cannot_be_removed(pool: PgPool) {
    let admin = insert_user(&pool).await;
    let connection = create_connection(&pool, Scope::Admin, api_key_connection("openai", "openai", admin)).await.unwrap();
    sqlx::query(
        "INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, connection_id, updated_by) VALUES ('GPT', 'openai', 'gpt-5', NULL, $1, $2)",
    )
    .bind(connection.id)
    .bind(admin)
    .execute(&pool)
    .await
    .unwrap();

    assert_eq!(models_using(&pool, connection.id).await.unwrap(), ["GPT"]);
    let result = delete_connection(&pool, Scope::Admin, connection.id).await;
    assert!(matches!(result, Err(DeleteConnectionError::InUse(models)) if models == ["GPT"]));

    sqlx::query("DELETE FROM admin_llm_models").execute(&pool).await.unwrap();
    delete_connection(&pool, Scope::Admin, connection.id).await.unwrap();
    assert!(list_connections(&pool, Scope::Admin).await.unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn disconnecting_a_personal_provider_forgets_the_saved_own_key(pool: PgPool) {
    let ana = insert_user(&pool).await;
    let connection = create_connection(&pool, Scope::User(ana), api_key_connection("openai", "openai", ana)).await.unwrap();
    sqlx::query(
        "INSERT INTO user_llm_selections (user_id, custom_label, custom_provider, custom_model_id, custom_api_key_encrypted, custom_connection_id) \
         VALUES ($1, 'Mine', 'openai', 'gpt-5', '\\x010203', $2)",
    )
    .bind(ana)
    .bind(connection.id)
    .execute(&pool)
    .await
    .unwrap();

    delete_connection(&pool, Scope::User(ana), connection.id).await.unwrap();

    let (provider, key, link): (Option<String>, Option<Vec<u8>>, Option<Uuid>) =
        sqlx::query_as("SELECT custom_provider, custom_api_key_encrypted, custom_connection_id FROM user_llm_selections WHERE user_id = $1")
            .bind(ana)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((provider, key, link), (None, None, None));
    assert!(matches!(delete_connection(&pool, Scope::User(ana), connection.id).await, Err(DeleteConnectionError::NotFound)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn existing_keys_become_connections_without_changing_which_key_a_model_uses(pool: PgPool) {
    let admin = insert_user(&pool).await;
    let ana = insert_user(&pool).await;
    // Rows as the previous version wrote them: every model and the own-key slot carry a key.
    for (label, provider, key) in [("Haiku", "anthropic", vec![1u8]), ("Sonnet", "anthropic", vec![2u8]), ("Flash", "gemini", vec![3u8]), ("Dev", "fake", vec![4u8])] {
        sqlx::query("INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, updated_by) VALUES ($1, $2, 'm', $3, $4)")
            .bind(label)
            .bind(provider)
            .bind(key)
            .bind(admin)
            .execute(&pool)
            .await
            .unwrap();
    }
    sqlx::query(
        "INSERT INTO user_llm_selections (user_id, custom_label, custom_provider, custom_model_id, custom_api_key_encrypted, custom_base_url) \
         VALUES ($1, 'Mine', 'openrouter', 'x/y', '\\x0909', 'https://openrouter.ai/api/v1')",
    )
    .bind(ana)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("SELECT backfill_provider_connections()").execute(&pool).await.unwrap();
    // Running it again changes nothing.
    sqlx::query("SELECT backfill_provider_connections()").execute(&pool).await.unwrap();

    let shared = list_connections(&pool, Scope::Admin).await.unwrap();
    let mut ids: Vec<&str> = shared.iter().map(|c| c.provider_id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, ["anthropic", "google"]);
    let google = shared.iter().find(|c| c.provider_id == "google").unwrap();
    assert_eq!((google.kind.as_str(), google.label.as_str(), google.api_key_encrypted.clone()), ("gemini", "Google", Some(vec![3u8])));

    let models: Vec<(String, Option<Vec<u8>>, Option<Uuid>)> =
        sqlx::query_as("SELECT label, api_key_encrypted, connection_id FROM admin_llm_models ORDER BY label").fetch_all(&pool).await.unwrap();
    for (label, key, connection_id) in &models {
        // Each model keeps its own key (used first), so nothing changes for anyone.
        assert!(key.is_some(), "{label}");
        assert_eq!(connection_id.is_some(), label != "Dev", "{label}");
    }

    let mine = list_connections(&pool, Scope::User(ana)).await.unwrap();
    assert_eq!(mine.len(), 1);
    assert_eq!((mine[0].provider_id.as_str(), mine[0].base_url.as_deref()), ("openrouter", Some("https://openrouter.ai/api/v1")));
    let link: Option<Uuid> = sqlx::query_scalar("SELECT custom_connection_id FROM user_llm_selections WHERE user_id = $1").bind(ana).fetch_one(&pool).await.unwrap();
    assert_eq!(link, Some(mine[0].id));
}
