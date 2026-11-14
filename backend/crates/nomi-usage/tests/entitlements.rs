use sqlx::PgPool;
use uuid::Uuid;

use nomi_usage::entitlements::{self, LimitHit, BYTES_PER_MB};
use nomi_usage::plans::{self, PlanInput, SubscriptionChange};

const THREE: i32 = 3;

async fn person(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn plan_input(slug: &str) -> PlanInput {
    serde_json::from_value(serde_json::json!({ "slug": slug, "name": slug, "monthly_tokens": 100 })).unwrap()
}

/// Puts `user` on a new plan made from `input`.
async fn on_plan(pool: &PgPool, user: Uuid, input: PlanInput) {
    let plan = plans::create_plan(pool, &input.validate().unwrap()).await.unwrap();
    let change = SubscriptionChange { plan_id: plan.id, quota_override: None, override_until: None, note: None };
    plans::change_subscription(pool, user, &change, user).await.unwrap();
}

async fn project(pool: &PgPool, owner: Uuid) -> Uuid {
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ('Home', true) RETURNING id").fetch_one(pool).await.unwrap();
    let session: Uuid =
        sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', $2) RETURNING id").bind(org).bind(Uuid::new_v4().to_string()).fetch_one(pool).await.unwrap();
    sqlx::query_scalar("INSERT INTO projects (user_id, session_id, org_id, name) VALUES ($1, $2, $3, 'P') RETURNING id")
        .bind(owner)
        .bind(session)
        .bind(org)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn file(pool: &PgPool, project: Uuid, path: &str, bytes: i64) {
    sqlx::query("INSERT INTO project_files (project_id, path, size_bytes) VALUES ($1, $2, $3)").bind(project).bind(path).bind(bytes as i32).execute(pool).await.unwrap();
}

async fn org_with_owner(pool: &PgPool, owner: Uuid, personal: bool) -> Uuid {
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ('Team', $1) RETURNING id").bind(personal).fetch_one(pool).await.unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')").bind(org).bind(owner).execute(pool).await.unwrap();
    org
}

#[sqlx::test(migrations = "../../migrations")]
async fn plans_are_unlimited_by_default(pool: PgPool) {
    let user = person(&pool).await;
    let e = entitlements::for_user(&pool, user).await.unwrap();
    assert_eq!((e.max_projects, e.max_orgs, e.max_org_members, e.max_custom_crew, e.max_storage_mb), (None, None, None, None, None));
    assert!(e.custom_domain && e.all_models && e.sandbox_size.is_none());
    for _ in 0..5 {
        project(&pool, user).await;
    }
    assert_eq!(entitlements::check_new_project(&pool, user).await.unwrap(), Ok(()));
    assert_eq!(entitlements::check_new_org(&pool, user).await.unwrap(), Ok(()));
    assert_eq!(entitlements::check_storage(&pool, user, i64::MAX / 2).await.unwrap(), Ok(()));
    assert_eq!(entitlements::check_custom_domain(&pool, user).await.unwrap(), Ok(()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn projects_are_refused_at_the_limit(pool: PgPool) {
    let user = person(&pool).await;
    on_plan(&pool, user, PlanInput { max_projects: Some(THREE), ..plan_input("capped") }).await;
    for _ in 0..THREE - 1 {
        project(&pool, user).await;
    }
    assert_eq!(entitlements::check_new_project(&pool, user).await.unwrap(), Ok(()));
    project(&pool, user).await;
    assert_eq!(entitlements::check_new_project(&pool, user).await.unwrap(), Err(LimitHit::Projects { limit: THREE }));
}

#[sqlx::test(migrations = "../../migrations")]
async fn orgs_count_only_owned_non_personal_ones(pool: PgPool) {
    let user = person(&pool).await;
    on_plan(&pool, user, PlanInput { max_orgs: Some(1), ..plan_input("capped") }).await;
    org_with_owner(&pool, user, true).await;
    assert_eq!(entitlements::check_new_org(&pool, user).await.unwrap(), Ok(()));
    org_with_owner(&pool, user, false).await;
    assert_eq!(entitlements::check_new_org(&pool, user).await.unwrap(), Err(LimitHit::Orgs { limit: 1 }));
}

#[sqlx::test(migrations = "../../migrations")]
async fn org_members_use_the_owners_plan_and_count_open_invites(pool: PgPool) {
    let owner = person(&pool).await;
    let guest = person(&pool).await;
    on_plan(&pool, owner, PlanInput { max_org_members: Some(THREE), ..plan_input("capped") }).await;
    let org = org_with_owner(&pool, owner, false).await;
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')").bind(org).bind(guest).execute(&pool).await.unwrap();
    assert_eq!(entitlements::check_new_org_member(&pool, org).await.unwrap(), Ok(()));
    sqlx::query("INSERT INTO org_invites (code, org_id, role, invited_by, expires_at) VALUES ('a', $1, 'member', $2, now() + interval '1 day')")
        .bind(org)
        .bind(owner)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(entitlements::check_new_org_member(&pool, org).await.unwrap(), Err(LimitHit::OrgMembers { limit: THREE }));
    // An expired invite no longer holds a seat.
    sqlx::query("UPDATE org_invites SET expires_at = now() - interval '1 day'").execute(&pool).await.unwrap();
    assert_eq!(entitlements::check_new_org_member(&pool, org).await.unwrap(), Ok(()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn storage_is_refused_when_it_would_overflow_but_shrinking_is_fine(pool: PgPool) {
    let user = person(&pool).await;
    on_plan(&pool, user, PlanInput { max_storage_mb: Some(1), ..plan_input("capped") }).await;
    let p = project(&pool, user).await;
    file(&pool, p, "a.txt", BYTES_PER_MB - 10).await;
    assert_eq!(entitlements::check_storage(&pool, user, 10).await.unwrap(), Ok(()));
    assert_eq!(entitlements::check_storage(&pool, user, 11).await.unwrap(), Err(LimitHit::Storage { limit_mb: 1 }));
    assert_eq!(entitlements::check_project_storage(&pool, p, 11).await.unwrap(), Err(LimitHit::Storage { limit_mb: 1 }));
    // Over the limit already (the plan was lowered): shrinking or keeping size is never refused.
    sqlx::query("UPDATE plans SET max_storage_mb = 0").execute(&pool).await.unwrap();
    assert_eq!(entitlements::check_storage(&pool, user, 0).await.unwrap(), Ok(()));
    assert_eq!(entitlements::check_storage(&pool, user, -5).await.unwrap(), Ok(()));
    assert_eq!(entitlements::check_project_storage(&pool, p, -5).await.unwrap(), Ok(()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn custom_domain_can_be_switched_off(pool: PgPool) {
    let user = person(&pool).await;
    on_plan(&pool, user, PlanInput { custom_domain: false, ..plan_input("capped") }).await;
    assert_eq!(entitlements::check_custom_domain(&pool, user).await.unwrap(), Err(LimitHit::CustomDomain));
}

#[sqlx::test(migrations = "../../migrations")]
async fn models_follow_the_plan_and_plan_models_round_trip(pool: PgPool) {
    let user = person(&pool).await;
    let model = |label: &'static str| {
        let pool = pool.clone();
        async move {
            sqlx::query_scalar::<_, Uuid>(
                "INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, is_default, updated_by) VALUES ($1, 'anthropic', 'x', '\\x00', false, $2) RETURNING id",
            )
            .bind(label)
            .bind(user)
            .fetch_one(&pool)
            .await
            .unwrap()
        }
    };
    let (included, other) = (model("A").await, model("B").await);
    let created = plans::create_plan(&pool, &PlanInput { all_models: false, model_ids: vec![included], ..plan_input("narrow") }.validate().unwrap()).await.unwrap();
    assert_eq!(created.model_ids, vec![included]);
    assert!(!created.all_models);
    let change = SubscriptionChange { plan_id: created.id, quota_override: None, override_until: None, note: None };
    plans::change_subscription(&pool, user, &change, user).await.unwrap();
    let e = entitlements::for_user(&pool, user).await.unwrap();
    assert!(entitlements::model_allowed(&e, included));
    assert!(!entitlements::model_allowed(&e, other));

    let swapped = PlanInput { all_models: false, model_ids: vec![other], sandbox_size: Some("large".into()), max_projects: Some(2), ..plan_input("narrow") };
    let updated = plans::update_plan(&pool, created.id, &swapped.validate().unwrap()).await.unwrap().unwrap();
    assert_eq!(updated.model_ids, vec![other]);
    assert_eq!((updated.sandbox_size.as_deref(), updated.max_projects), (Some("large"), Some(2)));
    assert_eq!(plans::get_plan(&pool, created.id).await.unwrap().unwrap(), updated);
    let everything = PlanInput { all_models: true, ..plan_input("narrow") };
    let cleared = plans::update_plan(&pool, created.id, &everything.validate().unwrap()).await.unwrap().unwrap();
    assert!(cleared.model_ids.is_empty() && cleared.all_models && cleared.sandbox_size.is_none());
}
