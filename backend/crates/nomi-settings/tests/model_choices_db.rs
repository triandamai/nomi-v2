use nomi_settings::model_choices::{get_session_model, model_visibility, set_model_visibility, set_session_model, ModelKey};
use sqlx::PgPool;
use uuid::Uuid;

async fn insert_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

async fn insert_session(pool: &PgPool, chat_id: &str) -> Uuid {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', $2) RETURNING id").bind(org_id).bind(chat_id).fetch_one(pool).await.unwrap()
}

async fn admin_model(pool: &PgPool, by: Uuid) -> Uuid {
    sqlx::query_scalar("INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, updated_by) VALUES ('Haiku', 'anthropic', 'claude-haiku-4-5', '\\x01', $1) RETURNING id")
        .bind(by)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn connection(pool: &PgPool, user: Uuid) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO llm_provider_connections (user_id, provider_id, label, kind, auth_type, api_key_encrypted) VALUES ($1, 'openrouter', 'OpenRouter', 'openrouter', 'api_key', '\\x01') RETURNING id",
    )
    .bind(user)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[test]
fn model_keys_name_nomis_models_and_models_from_a_connection() {
    let id = Uuid::new_v4();
    assert_eq!(ModelKey::parse(&format!("admin:{id}")), Some(ModelKey::Admin(id)));
    // A connection's model id can carry its own colons and slashes.
    let key = format!("conn:{id}:openrouter/anthropic/claude:beta");
    let parsed = ModelKey::parse(&key).unwrap();
    assert_eq!(parsed, ModelKey::Connection { connection_id: id, model_id: "openrouter/anthropic/claude:beta".to_string() });
    assert_eq!(parsed.to_string(), key);
    for bad in ["", "admin:", "admin:not-a-uuid", &format!("conn:{id}"), &format!("conn:{id}:"), "nomi:x", &format!("conn:{id}:{}", "m".repeat(301))] {
        assert_eq!(ModelKey::parse(bad), None, "{bad}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_chat_remembers_the_model_picked_for_it(pool: PgPool) {
    let ana = insert_user(&pool).await;
    let chat = insert_session(&pool, "chat-1").await;
    let other_chat = insert_session(&pool, "chat-2").await;
    let haiku = admin_model(&pool, ana).await;
    let mine = connection(&pool, ana).await;

    assert!(get_session_model(&pool, chat).await.unwrap().is_none());

    let picked = ModelKey::Connection { connection_id: mine, model_id: "x/y".to_string() };
    set_session_model(&pool, chat, &picked, Some("Y"), ana).await.unwrap();
    let saved = get_session_model(&pool, chat).await.unwrap().unwrap();
    assert_eq!((saved.key, saved.label.as_deref(), saved.set_by), (picked, Some("Y"), Some(ana)));
    assert!(get_session_model(&pool, other_chat).await.unwrap().is_none());

    // Switching to one of Nomi's models replaces the pick entirely.
    set_session_model(&pool, chat, &ModelKey::Admin(haiku), None, ana).await.unwrap();
    let saved = get_session_model(&pool, chat).await.unwrap().unwrap();
    assert_eq!((saved.key, saved.label), (ModelKey::Admin(haiku), None));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_removed_model_or_provider_is_forgotten_by_the_chats_that_picked_it(pool: PgPool) {
    let ana = insert_user(&pool).await;
    let (a, b) = (insert_session(&pool, "a").await, insert_session(&pool, "b").await);
    let haiku = admin_model(&pool, ana).await;
    let mine = connection(&pool, ana).await;
    set_session_model(&pool, a, &ModelKey::Admin(haiku), None, ana).await.unwrap();
    set_session_model(&pool, b, &ModelKey::Connection { connection_id: mine, model_id: "m".to_string() }, None, ana).await.unwrap();

    sqlx::query("DELETE FROM admin_llm_models WHERE id = $1").bind(haiku).execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM llm_provider_connections WHERE id = $1").bind(mine).execute(&pool).await.unwrap();

    assert!(get_session_model(&pool, a).await.unwrap().is_none());
    assert!(get_session_model(&pool, b).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_person_chooses_which_models_their_selector_shows(pool: PgPool) {
    let ana = insert_user(&pool).await;
    let budi = insert_user(&pool).await;
    assert!(model_visibility(&pool, ana).await.unwrap().is_empty());

    set_model_visibility(&pool, ana, &[("admin:1".to_string(), false), ("conn:2:m".to_string(), true)]).await.unwrap();
    set_model_visibility(&pool, ana, &[("conn:2:m".to_string(), false)]).await.unwrap();

    let ana_sees = model_visibility(&pool, ana).await.unwrap();
    assert_eq!(ana_sees.get("admin:1"), Some(&false));
    assert_eq!(ana_sees.get("conn:2:m"), Some(&false));
    assert!(model_visibility(&pool, budi).await.unwrap().is_empty());
}
