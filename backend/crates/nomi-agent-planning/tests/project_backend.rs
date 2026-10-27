use sqlx::PgPool;
use uuid::Uuid;

async fn seed_user_and_session(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    (user_id, session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_is_on_drizzle_with_no_relay_backend_until_told_otherwise(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let (backend, relay_project_id): (String, Option<Uuid>) = sqlx::query_as(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo') RETURNING backend, relay_project_id",
    )
    .bind(user_id)
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(backend, "drizzle");
    assert_eq!(relay_project_id, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_backend_is_drizzle_or_relay_only(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let insert = |backend: &'static str| {
        sqlx::query("INSERT INTO projects (user_id, session_id, name, backend) VALUES ($1, $2, 'Todo', $3)")
            .bind(user_id)
            .bind(session_id)
            .bind(backend)
    };
    insert("relay").execute(&pool).await.unwrap();
    assert!(insert("firebase").execute(&pool).await.is_err());
}

mod rena_chooses {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use nomi_agent_core::relay_ops::{NoRelayOps, ProjectFile, RelayEnv, RelayOps};
    use nomi_agent_core::SubAgent;
    use nomi_agent_planning::PlanningAgent;
    use serde_json::{json, Value};
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::seed_user_and_session;

    /// A Relay that's there (or not), remembers which projects it made backends for, and can be
    /// told to fail.
    #[derive(Default)]
    struct FakeRelay {
        available: bool,
        fails: bool,
        made: Mutex<Vec<Uuid>>,
    }

    #[async_trait]
    impl RelayOps for FakeRelay {
        async fn is_available(&self) -> bool {
            self.available
        }
        async fn ensure_backend(&self, _user_id: Uuid, project_id: Uuid) -> Result<Uuid, String> {
            if self.fails {
                return Err("Relay couldn't be reached. Try again in a moment.".into());
            }
            self.made.lock().unwrap().push(project_id);
            Ok(Uuid::new_v4())
        }
        async fn apply_migrations(&self, _: Uuid, _: Uuid, _: RelayEnv, _: &[ProjectFile]) -> Result<String, String> {
            unimplemented!()
        }
        async fn describe_backend(&self, _: Uuid, _: Uuid, _: RelayEnv) -> Result<String, String> {
            unimplemented!()
        }
        async fn deploy_functions(&self, _: Uuid, _: Uuid, _: RelayEnv, _: &[ProjectFile], _: &Value) -> Result<String, String> {
            unimplemented!()
        }
        async fn types(&self, _: Uuid, _: Uuid, _: RelayEnv) -> Result<String, String> {
            unimplemented!()
        }
    }

    async fn create(pool: &PgPool, relay: Arc<dyn RelayOps>, session_id: Uuid, user_id: Uuid, input: Value) -> Result<String, String> {
        let agent = PlanningAgent::new().with_relay_ops(relay);
        let mut conn = pool.acquire().await.unwrap();
        agent
            .execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "create_project", input)
            .await
            .map(|outcome| outcome.display_text)
    }

    async fn backend_of_session(pool: &PgPool, session_id: Uuid) -> String {
        sqlx::query_scalar("SELECT backend FROM projects WHERE session_id = $1").bind(session_id).fetch_one(pool).await.unwrap()
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn a_project_without_a_backend_named_stays_on_drizzle(pool: PgPool) {
        let (user_id, session_id) = seed_user_and_session(&pool).await;
        let relay = Arc::new(FakeRelay { available: true, ..Default::default() });
        create(&pool, relay.clone(), session_id, user_id, json!({"name": "Todo", "description": "Tasks"})).await.unwrap();
        assert_eq!(backend_of_session(&pool, session_id).await, "drizzle");
        assert!(relay.made.lock().unwrap().is_empty());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn relay_gets_the_project_a_backend_when_rena_names_it(pool: PgPool) {
        let (user_id, session_id) = seed_user_and_session(&pool).await;
        let relay = Arc::new(FakeRelay { available: true, ..Default::default() });
        let said = create(&pool, relay.clone(), session_id, user_id, json!({"name": "Todo", "description": "Tasks", "backend": "relay"})).await.unwrap();
        assert_eq!(backend_of_session(&pool, session_id).await, "relay");
        let project_id: Uuid = sqlx::query_scalar("SELECT id FROM projects WHERE session_id = $1").bind(session_id).fetch_one(&pool).await.unwrap();
        assert_eq!(*relay.made.lock().unwrap(), vec![project_id]);
        assert!(said.contains("Relay"), "{said}");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn without_relay_on_this_nomi_the_project_is_built_on_drizzle(pool: PgPool) {
        let (user_id, session_id) = seed_user_and_session(&pool).await;
        let said = create(&pool, Arc::new(NoRelayOps), session_id, user_id, json!({"name": "Todo", "description": "Tasks", "backend": "relay"})).await.unwrap();
        assert_eq!(backend_of_session(&pool, session_id).await, "drizzle");
        assert!(said.contains("Drizzle"), "{said}");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn when_relay_cant_make_the_backend_the_project_stays_on_drizzle(pool: PgPool) {
        let (user_id, session_id) = seed_user_and_session(&pool).await;
        let relay = Arc::new(FakeRelay { available: true, fails: true, ..Default::default() });
        let said = create(&pool, relay, session_id, user_id, json!({"name": "Todo", "description": "Tasks", "backend": "relay"})).await.unwrap();
        assert_eq!(backend_of_session(&pool, session_id).await, "drizzle");
        assert!(said.contains("couldn't be reached"), "{said}");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn a_project_with_files_keeps_its_backend(pool: PgPool) {
        let (user_id, session_id) = seed_user_and_session(&pool).await;
        let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo') RETURNING id")
            .bind(user_id)
            .bind(session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO project_files (project_id, path, size_bytes) VALUES ($1, 'src/db.ts', 10)")
            .bind(project_id)
            .execute(&pool)
            .await
            .unwrap();
        let relay = Arc::new(FakeRelay { available: true, ..Default::default() });
        let said = create(&pool, relay.clone(), session_id, user_id, json!({"name": "Todo", "description": "Tasks", "backend": "relay"})).await.unwrap();
        assert_eq!(backend_of_session(&pool, session_id).await, "drizzle");
        assert!(relay.made.lock().unwrap().is_empty());
        assert!(said.contains("keeps"), "{said}");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn an_unknown_backend_is_refused(pool: PgPool) {
        let (user_id, session_id) = seed_user_and_session(&pool).await;
        let result = create(&pool, Arc::new(NoRelayOps), session_id, user_id, json!({"name": "Todo", "backend": "firebase"})).await;
        assert!(result.unwrap_err().contains("backend"));
    }
}
