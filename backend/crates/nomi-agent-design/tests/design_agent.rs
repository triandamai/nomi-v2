//! Sora through the engine's eyes: a tool she doesn't have is refused, and without a provider
//! nothing is connected.

use nomi_agent_core::SubAgent;
use nomi_agent_design::DesignAgent;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn a_tool_sora_does_not_have_is_refused_by_name(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let refused = DesignAgent::new().execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "publish_preview", json!({})).await;
    assert_eq!(refused.err().as_deref(), Some("unknown tool: publish_preview"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_a_provider_listing_says_nothing_is_connected(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let listed = DesignAgent::new().execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "list_connected_tools", json!({})).await.unwrap();
    assert_eq!(listed.display_text, nomi_agent_core::mcp_ops::NOTHING_CONNECTED);
    let failed = DesignAgent::new().execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "use_connected_tool", json!({"server": "Refs", "tool": "search"})).await;
    assert!(failed.unwrap_err().contains("aren't available"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn sora_starts_from_the_chats_newest_brief_when_there_is_one(pool: PgPool) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(DesignAgent::new().domain_context(&mut conn, None, session_id, user_id, &[]).await, None, "no brief yet");

    for audience in ["Neighbours", "Office workers nearby"] {
        let brief = nomi_agent_core::design_brief::DesignBrief::parse(&json!({
            "company": [{"value": "Roti Pagi", "source": "confirmed"}],
            "audience": [{"value": audience, "source": "assumed"}],
            "objectives": [{"value": "Pre-orders", "source": "confirmed"}]
        }))
        .unwrap();
        nomi_agent_core::design_brief::save(&mut conn, session_id, Uuid::new_v4(), user_id, &brief).await.unwrap();
    }
    let context = DesignAgent::new().domain_context(&mut conn, None, session_id, user_id, &[]).await.expect("a brief");
    assert!(context.contains("# Design brief (version 2)") && context.contains("Office workers nearby") && !context.contains("Neighbours"));
    assert!(!context.contains("# Building the preview"), "no directions yet, so no preview rules");
}

fn two_directions() -> serde_json::Value {
    let direction = |id: &str, name: &str, accent: &str, font: &str| {
        json!({
            "id": id, "name": name,
            "summary": "What it looks like.", "rationale": "Why it fits.", "trade_offs": ["What it costs"],
            "palette": [{"name": "Paper", "hex": "#F6EFE4"}, {"name": "Accent", "hex": accent}],
            "typography": {"heading": font, "body": "Inter"},
            "layout": "One long page."
        })
    };
    json!({"directions": [direction("editorial", "Editorial", "#B5532A", "Fraunces"), direction("swiss", "Swiss utility", "#1F4FFF", "Helvetica")]})
}

#[sqlx::test(migrations = "../../migrations")]
async fn proposing_saves_a_version_shows_a_choice_and_refuses_a_set_that_is_not_valid(pool: PgPool) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let sora = DesignAgent::new();

    let brief = nomi_agent_core::design_brief::DesignBrief::parse(&json!({
        "company": [{"value": "Roti Pagi", "source": "confirmed"}],
        "audience": [{"value": "Neighbours", "source": "assumed"}],
        "objectives": [{"value": "Pre-orders", "source": "confirmed"}]
    }))
    .unwrap();
    nomi_agent_core::design_brief::save(&mut conn, session_id, Uuid::new_v4(), user_id, &brief).await.unwrap();

    let proposed = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", two_directions()).await.unwrap();
    assert!(proposed.display_text.starts_with("Directions saved as version 1, from design brief version 1:"));
    let Some(nomi_agent_core::ContentBlock::View(nomi_agent_core::View::Decision(choice))) = proposed.block else { panic!("a choice for the person") };
    assert_eq!(choice.options.len(), 2);

    let one = json!({"directions": [two_directions()["directions"][0].clone()]});
    let refused = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", one).await.unwrap_err();
    assert!(refused.starts_with("propose 2 to 4 directions"));
    let (version, brief_version, set) = nomi_agent_core::design_directions::latest(&mut conn, session_id).await.unwrap().unwrap();
    assert_eq!((version, brief_version, set.directions.len()), (1, Some(1), 2), "a refused set saves nothing");

    let context = sora.domain_context(&mut conn, None, session_id, user_id, &[]).await.unwrap();
    assert!(context.contains("# Design brief (version 1)") && context.contains("# Design directions (version 1)"));
    assert!(context.starts_with(nomi_agent_core::prompts::DESIGN_PREVIEW_RULES), "{context}");
    assert!(!context.contains("# Changing the preview") && !context.contains("# Handing the design to Koda"));
}

/// A publisher that answers as a test wants and remembers what it was given.
struct FakePublisher {
    answer: Result<String, String>,
    published: std::sync::Mutex<Vec<(i32, Vec<String>)>>,
}

impl FakePublisher {
    fn answering(answer: Result<&str, &str>) -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self { answer: answer.map(str::to_string).map_err(str::to_string), published: Default::default() })
    }
}

#[async_trait::async_trait]
impl nomi_agent_core::static_publish::StaticPublisher for FakePublisher {
    async fn publish(&self, request: &nomi_agent_core::static_publish::PublishRequest<'_>) -> Result<nomi_agent_core::static_publish::Published, String> {
        self.published.lock().unwrap().push((request.version, request.files.iter().map(|f| f.path.clone()).collect()));
        self.answer.clone().map(|url| nomi_agent_core::static_publish::Published { provider: "fake".into(), url })
    }
}

async fn chat(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    (session_id, user_id)
}

/// The person answers the directions in the chat (their pick), which is what lets Sora build
/// the first preview.
async fn picked(pool: &PgPool, session_id: Uuid, user_id: Uuid) {
    let identity: Uuid = sqlx::query_scalar("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(user_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, 'Editorial', clock_timestamp())")
        .bind(session_id)
        .bind(identity)
        .execute(pool)
        .await
        .unwrap();
}

fn directions() -> serde_json::Value {
    let direction = |id: &str, name: &str, accent: &str, font: &str| {
        json!({
            "id": id, "name": name, "summary": "What it looks like.", "rationale": "Why it fits.", "trade_offs": ["What it costs"],
            "palette": [{"name": "Paper", "hex": "#F6EFE4"}, {"name": "Accent", "hex": accent}],
            "typography": {"heading": font, "body": "Inter"}, "layout": "One long page."
        })
    };
    json!({"directions": [direction("editorial", "Editorial", "#B5532A", "Fraunces"), direction("swiss", "Swiss utility", "#1F4FFF", "Helvetica")]})
}

fn preview(direction: &str, body: &str) -> serde_json::Value {
    let html = format!("<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>Home</title></head><body>{body}</body></html>");
    json!({"direction_id": direction, "title": "A pass", "files": [{"path": "index.html", "content": html}, {"path": "styles.css", "content": "h1 { margin: 0 }"}]})
}

const HERO_AND_MENU: &str = "<h1 data-design-id=\"hero\">Hi</h1><ul data-design-id=\"menu-list\"></ul>";
const HERO_ONLY: &str = "<h1 data-design-id=\"hero\">Hello</h1>";

#[sqlx::test(migrations = "../../migrations")]
async fn a_preview_is_of_a_direction_sora_proposed(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let sora = DesignAgent::new();
    let early = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", HERO_ONLY)).await;
    assert!(early.unwrap_err().starts_with("propose directions with propose_directions first"));

    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", directions()).await.unwrap();
    let waiting = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", HERO_ONLY)).await;
    assert_eq!(waiting.unwrap_err(), nomi_agent_core::prompts::FLOW_DIRECTION_NOT_PICKED, "the first preview waits for the person's pick");
    picked(&pool, session_id, user_id).await;
    let unknown = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("brutalist", HERO_ONLY)).await;
    assert_eq!(unknown.unwrap_err(), "there is no direction \"brutalist\": the directions are editorial, swiss");
    let broken = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", "<h1>No ids</h1>")).await;
    assert!(broken.unwrap_err().starts_with("index.html has no data-design-id attributes"));
    assert_eq!(nomi_agent_core::design_artifact::latest(&mut conn, session_id).await.unwrap(), None, "nothing refused was saved");
}

#[sqlx::test(migrations = "../../migrations")]
async fn previews_are_saved_as_versions_even_when_nothing_publishes_them(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let sora = DesignAgent::new();
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", directions()).await.unwrap();
    picked(&pool, session_id, user_id).await;

    let first = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", HERO_AND_MENU)).await.unwrap();
    assert!(first.display_text.starts_with("Preview saved as version 1 of direction editorial: 1 page, 2 element ids."), "{}", first.display_text);
    assert!(first.display_text.contains("It isn't published outside Nomi"));
    assert!(first.display_text.contains(&format!("(/chat/{session_id}/design?version=1)")), "{}", first.display_text);

    let second = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", HERO_ONLY)).await.unwrap();
    assert!(second.display_text.starts_with("Preview saved as version 2"));
    assert!(second.display_text.contains("gone: menu-list."), "{}", second.display_text);

    let other = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("swiss", HERO_ONLY)).await.unwrap();
    assert!(!other.display_text.contains("gone:"), "another direction's ids aren't compared");

    let (version, manifest) = nomi_agent_core::design_artifact::latest(&mut conn, session_id).await.unwrap().expect("a preview");
    assert_eq!((version, manifest.direction_id.as_str()), (3, "swiss"));
    let directions_version: Option<i32> = sqlx::query_scalar("SELECT directions_version FROM design_artifacts WHERE session_id = $1 AND version = 1").bind(session_id).fetch_one(&mut *conn).await.unwrap();
    assert_eq!(directions_version, Some(1));
    assert_eq!(nomi_agent_core::static_publish::latest(&mut conn, session_id, 1).await.unwrap(), None);

    let context = sora.domain_context(&mut conn, None, session_id, user_id, &[]).await.expect("context");
    assert!(context.contains("# Design preview (version 3, direction: swiss)") && context.contains("Not published") && context.contains(&format!("/chat/{session_id}/design?version=3")));
    assert!(context.contains("# Building the preview") && context.contains("# Changing the preview"));
    assert!(!context.contains("# Handing the design to Koda"), "nothing is approved yet");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_publisher_gets_the_files_and_its_address_is_kept(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let publisher = FakePublisher::answering(Ok("https://previews.example/roti"));
    let sora = DesignAgent::new().with_static_publisher(publisher.clone());
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", directions()).await.unwrap();
    picked(&pool, session_id, user_id).await;

    let saved = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", HERO_ONLY)).await.unwrap();
    assert!(saved.display_text.contains("It is also published at https://previews.example/roti"), "{}", saved.display_text);
    assert_eq!(*publisher.published.lock().unwrap(), [(1, vec!["index.html".to_string(), "styles.css".to_string()])]);
    let kept = nomi_agent_core::static_publish::latest(&mut conn, session_id, 1).await.unwrap().expect("its address");
    assert_eq!((kept.provider.as_str(), kept.url.as_str()), ("fake", "https://previews.example/roti"));
    let context = sora.domain_context(&mut conn, None, session_id, user_id, &[]).await.expect("context");
    assert!(context.contains("Published at: https://previews.example/roti"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_publisher_that_fails_or_answers_badly_doesnt_lose_the_preview(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    for (answer, said) in [(Err("the service is down"), "(the service is down)"), (Ok("http://previews.example/roti"), "(fake gave an address that can't be used)")] {
        let sora = DesignAgent::new().with_static_publisher(FakePublisher::answering(answer));
        sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", directions()).await.unwrap();
        picked(&pool, session_id, user_id).await;
    picked(&pool, session_id, user_id).await;
        let saved = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", HERO_ONLY)).await.unwrap();
        assert!(saved.display_text.contains("It isn't published outside Nomi") && saved.display_text.contains(said), "{}", saved.display_text);
        assert!(!saved.display_text.contains("previews.example"), "a refused address is never passed on");
    }
    assert_eq!(nomi_agent_core::design_artifact::latest(&mut conn, session_id).await.unwrap().map(|(version, _)| version), Some(2));
    assert_eq!(nomi_agent_core::static_publish::latest(&mut conn, session_id, 2).await.unwrap(), None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn sora_reads_her_preview_back_and_a_new_version_marks_the_feedback_it_dealt_with(pool: PgPool) {
    use nomi_agent_core::design_feedback::{self, NewFeedback, Status};
    let (session_id, user_id) = chat(&pool).await;
    let sora = DesignAgent::new();
    let mut conn = pool.acquire().await.unwrap();
    let nothing = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "read_preview", json!({})).await.unwrap_err();
    assert_eq!(nothing, "there is no preview in this chat yet: build one with save_preview");

    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", directions()).await.unwrap();
    picked(&pool, session_id, user_id).await;
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", HERO_ONLY)).await.unwrap();
    let read = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "read_preview", json!({})).await.unwrap().display_text;
    assert!(read.starts_with("# Design preview (version 1, direction: editorial)") && read.contains("## index.html\n"), "{read}");
    let missing = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "read_preview", json!({"version": 7})).await.unwrap_err();
    assert_eq!(missing, "this chat has no preview version 7");

    let manifest = nomi_agent_core::design_artifact::version(&mut conn, session_id, 1).await.unwrap().unwrap();
    for body in ["Make it shorter", "Warmer overall"] {
        let new = NewFeedback::parse(1, &manifest, &json!({"level": "global", "body": body})).unwrap();
        design_feedback::add(&mut conn, session_id, user_id, &new).await.unwrap();
    }
    let context = sora.domain_context(&mut conn, None, session_id, user_id, &[]).await.unwrap();
    assert!(context.contains("- #1 (version 1, the whole design): Make it shorter") && context.contains("- #2 "), "{context}");

    let mut stray = preview("editorial", HERO_ONLY);
    stray["resolves"] = json!([{"feedback": 9, "note": "done"}]);
    let refused = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", stray).await.unwrap_err();
    assert_eq!(refused, "feedback #9 isn't waiting: the feedback still open is #1, #2");
    assert_eq!(nomi_agent_core::design_artifact::latest(&mut conn, session_id).await.unwrap().unwrap().0, 1, "nothing was saved");

    let mut refined = preview("editorial", HERO_ONLY);
    refined["resolves"] = json!([{"feedback": 1, "note": "Hero is half the height"}]);
    let saved = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", refined).await.unwrap().display_text;
    assert!(saved.contains("Feedback #1 is marked as dealt with in version 2; 1 piece of feedback is still waiting."), "{saved}");

    let kept = design_feedback::list(&mut conn, session_id).await.unwrap();
    assert_eq!((kept[0].status, kept[0].resolved_in_version, kept[0].resolution.as_deref()), (Status::Resolved, Some(2), Some("Hero is half the height")));
    assert_eq!(kept[0].body, "Make it shorter", "what was said is never changed");
    assert!(kept[0].resolved_at.is_some());
    assert_eq!((kept[1].status, kept[1].resolved_in_version), (Status::Open, None));
    assert_eq!(design_feedback::open_count(&mut conn, session_id).await.unwrap(), 1);
    let context = sora.domain_context(&mut conn, None, session_id, user_id, &[]).await.unwrap();
    assert!(!context.contains("Make it shorter") && context.contains("Warmer overall"), "{context}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_handoff_is_written_only_for_a_version_the_person_approved(pool: PgPool) {
    use nomi_agent_core::{design_approval, design_feedback, design_handoff};
    let (session_id, user_id) = chat(&pool).await;
    let sora = DesignAgent::new();
    let mut conn = pool.acquire().await.unwrap();
    let spec = json!({
        "design_system": {"colors": [{"name": "Accent", "value": "#B5532A"}], "typography": [{"role": "Heading", "font": "Fraunces"}]},
        "sitemap": [{"path": "index.html", "title": "Home", "purpose": "Landing"}],
        "user_flows": [{"name": "Read", "steps": ["Open Home"]}],
        "pages": [{"path": "index.html", "purpose": "Landing", "sections": ["hero"]}],
        "components": [{"id": "hero", "name": "Hero", "description": "Headline"}],
        "responsive": ["One column under 720px"],
        "accessibility": ["Contrast at least 4.5:1"],
        "decisions": [{"decision": "Serif headings", "reason": "Warmth"}]
    });
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", directions()).await.unwrap();
    picked(&pool, session_id, user_id).await;
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", HERO_AND_MENU)).await.unwrap();
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", HERO_ONLY)).await.unwrap();

    let early = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "create_handoff", spec.clone()).await.unwrap_err();
    assert_eq!(early, nomi_agent_design::tools::NOT_APPROVED_YET, "her newest version isn't approved by being newest");
    assert_eq!(design_handoff::latest(&mut conn, session_id).await.unwrap(), None);

    let manifest = nomi_agent_core::design_artifact::version(&mut conn, session_id, 2).await.unwrap().unwrap();
    let said = design_feedback::NewFeedback::parse(2, &manifest, &json!({"level": "global", "body": "Warmer"})).unwrap();
    design_feedback::add(&mut conn, session_id, user_id, &said).await.unwrap();
    design_approval::approve(&mut conn, session_id, user_id, 2, None).await.unwrap();
    let context = sora.domain_context(&mut conn, None, session_id, user_id, &[]).await.unwrap();
    assert!(context.contains("The person approved version 2") && context.contains("It is the newest version."), "{context}");
    assert!(context.contains(nomi_agent_core::prompts::DESIGN_HANDOFF_RULES), "approved, so the handoff rules join");

    let mut wrong = spec.clone();
    wrong["components"][0]["id"] = json!("menu-list");
    let refused = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "create_handoff", wrong).await.unwrap_err();
    assert_eq!(refused, "components names \"menu-list\", which isn't a data-design-id of the approved version", "version 1 had it; the approved one doesn't");

    let saved = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "create_handoff", spec).await.unwrap().display_text;
    assert!(saved.starts_with("Handoff saved as version 1, for approved preview version 2.") && saved.contains("There is no project in this chat yet"), "{saved}");
    let (version, package) = design_handoff::latest(&mut conn, session_id).await.unwrap().unwrap();
    assert_eq!((version, package.artifact_version, package.direction_id.as_str()), (1, 2, "editorial"));
    assert_eq!(package.studio_link, format!("/chat/{session_id}/design?version=2"));
    assert_eq!(package.feedback.iter().map(|f| (f.number, f.body.as_str(), f.resolved_in_version)).collect::<Vec<_>>(), [(1, "Warmer", None)]);
    assert_eq!((package.brief, package.published_url), (None, None));
}

/// A built project as a test wants it.
struct FakeSource(Result<Vec<nomi_agent_core::design_review::SourceFile>, String>);

#[async_trait::async_trait]
impl nomi_agent_core::design_review::ProjectSource for FakeSource {
    async fn files(&self, _user_id: Uuid, _project_id: Uuid) -> Result<Vec<nomi_agent_core::design_review::SourceFile>, String> {
        self.0.clone()
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_build_is_checked_against_the_handoff_it_was_built_from(pool: PgPool) {
    use nomi_agent_core::design_review::{self, SourceFile};
    use nomi_agent_core::{design_approval, design_handoff};
    let (session_id, user_id) = chat(&pool).await;
    let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, stack) VALUES ($1, $2, 'Bakery', 'sveltekit') RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let built = vec![SourceFile { path: "src/app.css".into(), content: "h1 { color: #b5532a }".into() }, SourceFile { path: "src/routes/+page.svelte".into(), content: "<h1 data-design-id=\"hero\">Hi</h1>".into() }];
    let sora = DesignAgent::new().with_project_source(std::sync::Arc::new(FakeSource(Ok(built))));
    let mut conn = pool.acquire().await.unwrap();

    let early = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "review_build", json!({})).await.unwrap_err();
    assert!(early.starts_with("there is no handoff in this chat yet"), "{early}");

    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", directions()).await.unwrap();
    picked(&pool, session_id, user_id).await;
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", HERO_ONLY)).await.unwrap();
    design_approval::approve(&mut conn, session_id, user_id, 1, None).await.unwrap();
    let spec = json!({
        "design_system": {"colors": [{"name": "Accent", "value": "#B5532A"}], "typography": [{"role": "Heading", "font": "Fraunces"}]},
        "sitemap": [{"path": "index.html", "title": "Home", "purpose": "Landing"}],
        "user_flows": [{"name": "Read", "steps": ["Open Home"]}],
        "pages": [{"path": "index.html", "purpose": "Landing", "sections": ["hero"]}],
        "components": [{"id": "hero", "name": "Hero", "description": "Headline"}],
        "responsive": ["One column under 720px"],
        "accessibility": ["Contrast at least 4.5:1"],
        "decisions": [{"decision": "Serif headings", "reason": "Warmth"}]
    });
    let handed = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "create_handoff", spec).await.unwrap().display_text;
    assert!(handed.contains(&format!("\"Project {project_id}: ")), "the chat's project is named for the delegation: {handed}");
    assert_eq!(design_handoff::latest(&mut conn, session_id).await.unwrap().unwrap().0, 1);

    let said = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "review_build", json!({})).await.unwrap().display_text;
    assert!(said.starts_with("# Design check (version 1, against handoff version 1)\n2 of 3 checks match."), "{said}");
    assert!(said.contains("- Font Fraunces (Heading): not named in the project's files"), "{said}");
    assert!(said.contains(&format!("\"Project {project_id}: ")), "what differs goes back to Koda");
    let (version, handoff_version, review) = design_review::latest(&mut conn, session_id).await.unwrap().unwrap();
    assert_eq!((version, handoff_version, review.deviations().len()), (1, 1, 1));

    let unreadable = DesignAgent::new().with_project_source(std::sync::Arc::new(FakeSource(Err("no connection".into()))));
    let failed = unreadable.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "review_build", json!({})).await.unwrap_err();
    assert_eq!(failed, "the build couldn't be checked: no connection");
    assert_eq!(design_review::latest(&mut conn, session_id).await.unwrap().unwrap().0, 1, "a check that couldn't read the build isn't kept");
}

fn page(body: &str) -> String {
    format!("<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>Home</title></head><body>{body}</body></html>")
}

fn piece(direction: &str, files: &[(&str, &str)], extra: serde_json::Value) -> serde_json::Value {
    let files: Vec<_> = files.iter().map(|(path, content)| json!({"path": path, "content": content})).collect();
    let mut input = json!({"direction_id": direction, "title": "A pass", "files": files});
    input.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    input
}

async fn ready_to_preview(pool: &PgPool) -> (Uuid, Uuid, DesignAgent) {
    let (session_id, user_id) = chat(pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let sora = DesignAgent::new();
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", directions()).await.unwrap();
    picked(pool, session_id, user_id).await;
    (session_id, user_id, sora)
}

fn paths(files: &[nomi_agent_core::design_artifact::ArtifactFile]) -> Vec<&str> {
    let mut paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
    paths.sort();
    paths
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_preview_can_be_built_in_pieces(pool: PgPool) {
    let (session_id, user_id, sora) = ready_to_preview(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let staged = sora
        .execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", piece("editorial", &[("styles.css", "h1{}")], json!({"finished": false})))
        .await
        .unwrap();
    assert!(staged.display_text.contains("styles.css"), "{}", staged.display_text);
    assert_eq!(nomi_agent_core::design_artifact::latest(&mut conn, session_id).await.unwrap(), None, "nothing is shown yet");

    let finished = sora
        .execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", piece("editorial", &[("index.html", &page(HERO_ONLY))], json!({"finished": true})))
        .await
        .unwrap();
    assert!(finished.display_text.starts_with("Preview saved as version 1"), "{}", finished.display_text);
    assert!(finished.display_text.contains("from the draft: styles.css"), "{}", finished.display_text);
    let files = nomi_agent_core::design_artifact::files(&mut conn, session_id, 1).await.unwrap();
    assert_eq!(paths(&files), ["index.html", "styles.css"]);
    assert_eq!(nomi_agent_core::design_draft::draft(&mut conn, session_id, "editorial").await.unwrap(), vec![], "the draft became the version");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_change_sends_only_the_changed_files(pool: PgPool) {
    let (session_id, user_id, sora) = ready_to_preview(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", preview("editorial", HERO_ONLY)).await.unwrap();

    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", piece("editorial", &[("index.html", &page(HERO_AND_MENU))], json!({}))).await.unwrap();
    let second = nomi_agent_core::design_artifact::files(&mut conn, session_id, 2).await.unwrap();
    assert_eq!(paths(&second), ["index.html", "styles.css"]);
    assert_eq!(second.iter().find(|f| f.path == "styles.css").unwrap().content, "h1 { margin: 0 }", "kept from version 1");
    assert!(second.iter().find(|f| f.path == "index.html").unwrap().content.contains("menu-list"));

    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", piece("editorial", &[("index.html", &page(HERO_ONLY))], json!({"remove": ["styles.css"]})))
        .await
        .unwrap();
    assert_eq!(paths(&nomi_agent_core::design_artifact::files(&mut conn, session_id, 3).await.unwrap()), ["index.html"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_finish_keeps_the_draft(pool: PgPool) {
    let (session_id, user_id, sora) = ready_to_preview(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let no_viewport = "<html><body><h1 data-design-id=\"about\">About</h1></body></html>";
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", piece("editorial", &[("about.html", no_viewport)], json!({"finished": false})))
        .await
        .unwrap();
    let failed = sora
        .execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", piece("editorial", &[("index.html", &page(HERO_ONLY))], json!({})))
        .await
        .unwrap_err();
    assert!(failed.contains("about.html") && failed.contains("The draft is kept"), "{failed}");
    assert_eq!(paths(&nomi_agent_core::design_draft::draft(&mut conn, session_id, "editorial").await.unwrap()), ["about.html"]);
    assert_eq!(nomi_agent_core::design_artifact::latest(&mut conn, session_id).await.unwrap(), None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_draft_past_the_file_limit_is_refused(pool: PgPool) {
    use nomi_agent_core::design_artifact::MAX_FILES;
    let (session_id, user_id, sora) = ready_to_preview(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let names: Vec<String> = (0..MAX_FILES).map(|n| format!("s{n}.css")).collect();
    let full: Vec<(&str, &str)> = names.iter().map(|n| (n.as_str(), "")).collect();
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", piece("editorial", &full, json!({"finished": false}))).await.unwrap();
    let refused = sora
        .execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", piece("editorial", &[("one-more.css", "")], json!({"finished": false})))
        .await
        .unwrap_err();
    assert_eq!(refused, format!("a preview has at most {MAX_FILES} files"));
    assert_eq!(nomi_agent_core::design_draft::draft(&mut conn, session_id, "editorial").await.unwrap().len(), MAX_FILES);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_draft_stays_with_its_direction(pool: PgPool) {
    let (session_id, user_id, sora) = ready_to_preview(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", piece("editorial", &[("editorial.css", "h1{}")], json!({"finished": false})))
        .await
        .unwrap();
    sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "save_preview", piece("swiss", &[("index.html", &page(HERO_ONLY))], json!({}))).await.unwrap();
    assert_eq!(paths(&nomi_agent_core::design_artifact::files(&mut conn, session_id, 1).await.unwrap()), ["index.html"]);
    assert_eq!(paths(&nomi_agent_core::design_draft::draft(&mut conn, session_id, "editorial").await.unwrap()), ["editorial.css"]);
}

#[test]
fn sora_is_told_to_continue_in_pieces() {
    use nomi_agent_core::prompts::{DESIGN_CUT_OFF_RETRY, DESIGN_PREVIEW_RULES};
    assert_eq!(DesignAgent::new().cut_off_hint(), DESIGN_CUT_OFF_RETRY);
    assert!(DESIGN_CUT_OFF_RETRY.contains("finished"));
    assert!(DESIGN_PREVIEW_RULES.contains("finished"));
    assert!(!DESIGN_PREVIEW_RULES.contains("send every file again"));
}
