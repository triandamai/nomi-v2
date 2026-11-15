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

async fn published(pool: &PgPool, project_id: Uuid, status: &str, failure: Option<serde_json::Value>, seconds_later: i32) {
    sqlx::query(
        "INSERT INTO project_publishes (project_id, environment, status, manifest, failure, created_at) \
         VALUES ($1, 'production', $2, '{}'::jsonb, $3, now() + make_interval(secs => $4))",
    )
    .bind(project_id)
    .bind(status)
    .bind(failure)
    .bind(seconds_later)
    .execute(pool)
    .await
    .unwrap();
}

async fn brief_of(pool: &PgPool, seeded: &Seeded) -> String {
    let messages = vec![said(&format!("Project {}: fix it", seeded.project_id))];
    let mut conn = pool.acquire().await.unwrap();
    agent().domain_context(&mut conn, None, seeded.session_id, seeded.user_id, &messages).await.expect("a brief")
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_brief_names_the_last_failed_publish(pool: PgPool) {
    let seeded = seed(&pool).await;
    published(&pool, seeded.project_id, "succeeded", None, 0).await;
    let failure = serde_json::json!({ "service": "web", "deployment_id": "d1", "step": "build_image", "phase": "build", "message": "npm run build exited with code 1", "tail": [], "runtime_logs": [] });
    published(&pool, seeded.project_id, "failed", Some(failure), 1).await;

    let brief = brief_of(&pool, &seeded).await;

    assert!(brief.contains("The last publish to production failed at build › build_image: npm run build exited with code 1"), "{brief}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_later_successful_publish_clears_the_failure(pool: PgPool) {
    let seeded = seed(&pool).await;
    let failure = serde_json::json!({ "service": "web", "deployment_id": "d1", "step": "build_image", "phase": "build", "message": "boom", "tail": [], "runtime_logs": [] });
    published(&pool, seeded.project_id, "failed", Some(failure), 0).await;
    published(&pool, seeded.project_id, "succeeded", None, 1).await;

    assert!(!brief_of(&pool, &seeded).await.contains("The last publish to"));
}

/// The `design` setting is read through the process-wide settings cache and pool, so the tests
/// that set it take turns.
static GLOBALS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn design(pool: &PgPool, on: bool) {
    let key: String = nomi_test_support::TEST_SETTINGS_KEY.iter().map(|b| format!("{b:02x}")).collect();
    std::env::set_var("SETTINGS_ENCRYPTION_KEY", key);
    nomi_settings::config::install(pool.clone());
    let admin: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let pairs: &[(&str, &str)] = if on { &[("enabled", "on"), ("rollout", "all")] } else { &[("enabled", "off")] };
    let values = pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    nomi_settings::config::save(pool, &nomi_test_support::TEST_SETTINGS_KEY, &nomi_settings::config::DESIGN, &values, admin).await.unwrap();
    nomi_settings::config::forget();
}

async fn handed_over(pool: &PgPool, seeded: &Seeded) {
    use nomi_agent_core::design_artifact::{self, DesignArtifact};
    use nomi_agent_core::design_handoff::{self, Package, Spec};
    use serde_json::json;
    let html = "<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>Home</title></head><body><h1 data-design-id=\"hero\">Fresh at six</h1></body></html>";
    let artifact = DesignArtifact::parse(&json!({"direction_id": "editorial", "title": "Approved pass", "files": [{"path": "index.html", "content": html}, {"path": "styles.css", "content": "h1 { color: #B5532A }"}]})).unwrap();
    let spec = json!({
        "design_system": {"colors": [{"name": "Accent", "value": "#B5532A"}], "typography": [{"role": "Heading", "font": "Fraunces"}]},
        "sitemap": [{"path": "index.html", "title": "Home", "purpose": "Landing"}],
        "user_flows": [{"name": "Pre-order", "steps": ["Open Home", "Tap Order"]}],
        "pages": [{"path": "index.html", "purpose": "Landing", "sections": ["hero"]}],
        "components": [{"id": "hero", "name": "Hero", "description": "Headline over a photo"}],
        "responsive": ["One column under 720px"],
        "accessibility": ["Contrast at least 4.5:1"],
        "decisions": [{"decision": "Serif headings", "reason": "Warmth"}]
    });
    let mut conn = pool.acquire().await.unwrap();
    let version = design_artifact::save(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, None, &artifact).await.unwrap();
    let package = Package {
        spec: Spec::parse(&spec, &artifact.manifest).unwrap(),
        artifact_version: version,
        direction_id: "editorial".into(),
        studio_link: format!("/chat/{}/design?version={version}", seeded.session_id),
        published_url: None,
        brief_version: None,
        brief: None,
        feedback: vec![],
    };
    assert_eq!(design_handoff::save(&mut conn, seeded.session_id, Uuid::new_v4(), seeded.user_id, &package).await.unwrap(), 1);
    assert_eq!(design_handoff::latest(&mut conn, seeded.session_id).await.unwrap(), Some((1, package)), "it is kept as it was written");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_approved_design_is_in_the_brief_where_design_is_rolled_out(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let seeded = seed(&pool).await;
    plan(&pool, &seeded, seeded.rena, "Bakery site", 1, BAKERY_PLAN).await;
    let messages = vec![said(&format!("Project {}: build the approved design", seeded.project_id))];
    let mut conn = pool.acquire().await.unwrap();

    design(&pool, true).await;
    let before = agent().domain_context(&mut conn, None, seeded.session_id, seeded.user_id, &messages).await.expect("a brief");
    assert!(!before.contains("The approved design"), "nothing was handed over yet");

    handed_over(&pool, &seeded).await;
    let brief = agent().domain_context(&mut conn, None, seeded.session_id, seeded.user_id, &messages).await.expect("a brief");
    assert!(brief.contains("## The approved design\n") && brief.contains("# Design handoff (version 1)\nApproved preview: version 1 of direction editorial"), "{brief}");
    assert!(brief.contains("- Accent: #B5532A") && brief.contains("## Responsive rules\n- One column under 720px"));
    assert!(brief.contains("#### index.html\n") && brief.contains("Fresh at six") && brief.contains("#### styles.css\nh1 { color: #B5532A }"), "the approved preview's own files");
    assert!(brief.contains("Bakery site") && brief.contains("Done when"), "Rena's plan is still there");
    assert!(brief.find("Bakery site").unwrap() < brief.find("## The approved design").unwrap(), "after the plan");

    design(&pool, false).await;
    let off = agent().domain_context(&mut conn, None, seeded.session_id, seeded.user_id, &messages).await.expect("a brief");
    assert_eq!(off, before, "with Design off the brief is what it always was");
}
