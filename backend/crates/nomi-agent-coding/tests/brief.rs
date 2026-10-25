//! Koda starts a build from a brief instead of exploring: the project, its files, the plan Rena
//! agreed with the person (its checklist as what "done" means) and the guides for its stack.

use nomi_agent_coding::CodingAgent;
use nomi_agent_core::SubAgent;
use nomi_llm::{ContentBlock, LlmMessage, LlmRole};
use sqlx::PgPool;
use uuid::Uuid;

fn agent() -> CodingAgent {
    CodingAgent::new(nomi_test_support::test_project_storage())
}

fn said(text: &str) -> LlmMessage {
    LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: text.to_string() }] }
}

struct Seeded {
    session_id: Uuid,
    project_id: Uuid,
    rena: Uuid,
    koda: Uuid,
    user_id: Uuid,
}

async fn seed(pool: &PgPool) -> Seeded {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let identity: Uuid = sqlx::query_scalar("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(user_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, stack) VALUES ($1, $2, 'Bakery', 'sveltekit') RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap();
    for (path, size) in [("package.json", 512), ("src/routes/+page.svelte", 1834)] {
        sqlx::query("INSERT INTO project_files (project_id, path, size_bytes) VALUES ($1, $2, $3)").bind(project_id).bind(path).bind(size).execute(pool).await.unwrap();
    }
    let agent_session = |agent_type: &'static str| {
        let pool = pool.clone();
        async move {
            sqlx::query_scalar::<_, Uuid>("INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, $3, 'completed') RETURNING id")
                .bind(session_id)
                .bind(identity)
                .bind(agent_type)
                .fetch_one(&pool)
                .await
                .unwrap()
        }
    };
    let rena = agent_session("planning").await;
    let koda = agent_session("coding").await;
    Seeded { session_id, project_id, rena, koda, user_id }
}

async fn plan(pool: &PgPool, seeded: &Seeded, by: Uuid, title: &str, version: i32, content: &str) {
    sqlx::query("INSERT INTO agent_plans (session_id, agent_session_id, user_id, title, content, version) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(seeded.session_id)
        .bind(by)
        .bind(seeded.user_id)
        .bind(title)
        .bind(content)
        .bind(version)
        .execute(pool)
        .await
        .unwrap();
}

const BAKERY_PLAN: &str = "## Pages\n- Home with today's bakes\n\n## Steps\n- [x] Pick the colours\n- [ ] Home shows the menu\n- [ ] Contact page with a map\n";

#[sqlx::test(migrations = "../../migrations")]
async fn a_build_starts_from_a_brief_with_the_project_the_plan_and_the_stack_guide(pool: PgPool) {
    let seeded = seed(&pool).await;
    plan(&pool, &seeded, seeded.rena, "Bakery site", 2, BAKERY_PLAN).await;
    // Koda's own notes are never mistaken for the plan the person agreed to.
    plan(&pool, &seeded, seeded.koda, "Koda's notes", 1, "- [ ] refactor later").await;
    let messages = vec![said("Build it"), said(&format!("(Handed to you to do now: Project {}: build plan version 2)", seeded.project_id))];

    let mut conn = pool.acquire().await.unwrap();
    let brief = agent().domain_context(&mut conn, None, seeded.session_id, seeded.user_id, &messages).await.expect("a brief");

    assert!(brief.contains("Bakery") && brief.contains("sveltekit"), "the project and its stack");
    assert!(brief.contains("src/routes/+page.svelte (1834 B)") && brief.contains("package.json (512 B)"), "its files");
    assert!(brief.contains("Bakery site") && brief.contains("version 2") && brief.contains("Home with today's bakes"), "Rena's plan");
    assert!(!brief.contains("Koda's notes"));
    let done_when = &brief[brief.find("Done when").expect("acceptance criteria")..];
    assert!(done_when.contains("Home shows the menu") && done_when.contains("Contact page with a map"));
    assert!(brief.contains("# SvelteKit 3 + Svelte 5") && brief.contains("# How Nomi runs your project"), "the stack and preview guides, preloaded");
    assert!(!brief.contains("# Design: make it look made by a person with taste"), "other guides stay on demand");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_brief_says_when_the_last_check_on_these_files_failed(pool: PgPool) {
    let seeded = seed(&pool).await;
    sqlx::query("UPDATE projects SET files_version = 7, checked_version = 7, check_ok = false, check_output = 'src/routes/+page.svelte:3 Type error: x is not defined' WHERE id = $1")
        .bind(seeded.project_id)
        .execute(&pool)
        .await
        .unwrap();
    let messages = vec![said(&format!("(Handed to you to do now: Project {}: fix the build)", seeded.project_id))];
    let mut conn = pool.acquire().await.unwrap();
    let brief = agent().domain_context(&mut conn, None, seeded.session_id, seeded.user_id, &messages).await.unwrap();
    assert!(brief.contains("Type error: x is not defined"));
    assert!(brief.contains("No plan"), "and that there's no plan to follow, rather than nothing");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_brief_names_the_projects_secrets(pool: PgPool) {
    let seeded = seed(&pool).await;
    let messages = vec![said(&format!("(Handed to you to do now: Project {}: add payments)", seeded.project_id))];
    let mut conn = pool.acquire().await.unwrap();
    let brief = agent().domain_context(&mut conn, None, seeded.session_id, seeded.user_id, &messages).await.unwrap();
    assert!(!brief.contains("## Secrets"), "nothing to say before any are stored");
    sqlx::query("INSERT INTO project_secrets (project_id, name) VALUES ($1, 'STRIPE_KEY')").bind(seeded.project_id).execute(&pool).await.unwrap();
    let brief = agent().domain_context(&mut conn, None, seeded.session_id, seeded.user_id, &messages).await.unwrap();
    assert!(brief.contains("## Secrets") && brief.contains("STRIPE_KEY"), "{brief}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn there_is_no_brief_without_a_project(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'x') RETURNING id").bind(org_id).fetch_one(&pool).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(agent().domain_context(&mut conn, None, session_id, user_id, &[said("hi")]).await, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn naming_someone_elses_project_shows_nothing_of_it(pool: PgPool) {
    let seeded = seed(&pool).await;
    let stranger: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let messages = vec![said(&format!("(Handed to you to do now: Project {}: build it)", seeded.project_id))];
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(agent().domain_context(&mut conn, None, seeded.session_id, stranger, &messages).await, None);
}

#[test]
fn kodas_prompt_starts_from_the_brief_and_still_knows_the_way_without_one() {
    let prompt = agent().system_prompt();
    assert!(prompt.contains("Start from your brief"));
    assert!(prompt.contains("Without a brief, call read_plan, then list_files"));
}
