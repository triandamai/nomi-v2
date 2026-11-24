//! Sora, the designer: turns the plan Rena agreed with the person into visual design.

pub mod library;
pub mod tools;

use nomi_agent_core::ToolContext;
use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::crew::DESIGN_AGENT_TYPE;
use nomi_agent_core::prompts::{DESIGN_CHANGE_RULES, DESIGN_HANDOFF_RULES, DESIGN_PREVIEW_RULES, DESIGN_SYSTEM_PROMPT};
use nomi_agent_core::design_approval::{self, Approval};
use nomi_agent_core::design_artifact::{self, Manifest};
use nomi_agent_core::design_brief::{self, DesignBrief};
use nomi_agent_core::design_directions::{self, DesignDirections};
use nomi_agent_core::design_feedback::{self, Feedback};
use nomi_agent_core::design_review::{NoProjectSource, SharedProjectSource};
use nomi_agent_core::mcp_ops::{NoMcpOps, SharedMcpOps};
use nomi_agent_core::static_publish::{self, NoStaticPublisher, Published, SharedStaticPublisher};
use nomi_agent_core::{find_tool, tool_definitions, SubAgent};
use nomi_llm::ToolDefinition;

/// What Sora is told before the brief.
pub use nomi_agent_core::prompts::DESIGN_BRIEF_INTRO as BRIEF_INTRO;

/// The chat's brief as Sora starts from it.
pub fn brief_context(version: i32, brief: &DesignBrief) -> String {
    format!("{BRIEF_INTRO}\n\n{}", brief.to_markdown(version))
}

/// What Sora is told before the directions she already proposed.
pub use nomi_agent_core::prompts::DESIGN_DIRECTIONS_INTRO as DIRECTIONS_INTRO;

/// The chat's newest directions as Sora carries on from them.
pub fn directions_context(version: i32, set: &DesignDirections) -> String {
    format!("{DIRECTIONS_INTRO}\n\n{}", set.to_markdown(version))
}

/// What Sora is told before the preview she built last.
pub use nomi_agent_core::prompts::DESIGN_PREVIEW_INTRO as PREVIEW_INTRO;

/// The chat's newest preview as Sora carries on from it: its pages, its ids and its address.
pub fn preview_context(session_id: Uuid, version: i32, manifest: &Manifest, published: Option<&Published>) -> String {
    let address = match published {
        Some(published) => format!("Published at: {}", published.url),
        None => "Not published outside Nomi.".to_string(),
    };
    format!("{PREVIEW_INTRO}\n\n{}\nDesign Studio: {}\n{address}", manifest.to_markdown(version), tools::studio_link(session_id, version))
}

/// What Sora is told before the feedback that is still waiting.
pub use nomi_agent_core::prompts::DESIGN_FEEDBACK_INTRO as FEEDBACK_INTRO;

/// The feedback still waiting on a new version; `None` when there is none.
pub fn feedback_context(feedback: &[Feedback]) -> Option<String> {
    design_feedback::open_markdown(feedback).map(|open| format!("{FEEDBACK_INTRO}\n\n{open}"))
}

/// What Sora is told while the person hasn't approved a version.
pub use nomi_agent_core::prompts::DESIGN_NOT_APPROVED as NOT_APPROVED;

/// The approval that stands, as Sora reads it; `newest` is the chat's newest preview version.
pub fn approval_context(approval: &Approval, newest: i32) -> String {
    approval.to_markdown(newest)
}

/// The rules for the steps this chat has reached, in order, added after Sora's core prompt:
/// building a preview once she has proposed directions (or already built one), changing it once a
/// preview exists, and the handoff and review once the person approved a version.
pub fn stage_rules(has_directions: bool, has_preview: bool, approved: bool) -> Vec<&'static str> {
    let mut rules = Vec::new();
    if has_directions || has_preview {
        rules.push(DESIGN_PREVIEW_RULES);
    }
    if has_preview {
        rules.push(DESIGN_CHANGE_RULES);
    }
    if approved {
        rules.push(DESIGN_HANDOFF_RULES);
    }
    rules
}

/// Sora. She is registered for everyone, and reachable unless an admin switched her off
/// (Admin → Agents; see `nomi_agent_core::personal_crew::with_admin_switches`).
pub struct DesignAgent {
    mcp: SharedMcpOps,
    publisher: SharedStaticPublisher,
    source: SharedProjectSource,
}

impl DesignAgent {
    pub fn new() -> Self {
        Self { mcp: std::sync::Arc::new(NoMcpOps), publisher: std::sync::Arc::new(NoStaticPublisher), source: std::sync::Arc::new(NoProjectSource) }
    }

    /// Where previews are published, when the server has somewhere.
    pub fn with_static_publisher(mut self, publisher: SharedStaticPublisher) -> Self {
        self.publisher = publisher;
        self
    }

    /// Where a build is read from, to check it against the design.
    pub fn with_project_source(mut self, source: SharedProjectSource) -> Self {
        self.source = source;
        self
    }

    /// The person's connected MCP servers, through the server's access to them.
    pub fn with_mcp_ops(mut self, mcp: SharedMcpOps) -> Self {
        self.mcp = mcp;
        self
    }
}

impl Default for DesignAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SubAgent for DesignAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(DESIGN_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(DESIGN_SYSTEM_PROMPT)
    }

    /// Proposing directions, building previews and the person's connected tools; the rest of what she uses (read_plan, ask_user, the web)
    /// is the engine's.
    fn tools(&self) -> Vec<ToolDefinition> {
        tool_definitions(&tools::own_tools(&self.mcp, &self.publisher, &self.source))
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        let own = tools::own_tools(&self.mcp, &self.publisher, &self.source);
        match find_tool(&own, name) {
            Some(tool) => tool.execute(conn, &ToolContext::new(session_id, agent_session_id, user_id, self.agent_type().as_ref(), self.display_name().as_ref()), input).await,
            None => Err(format!("unknown tool: {name}")),
        }
    }

    /// Proposing directions, building a preview and looking at what is connected need no OK;
    /// calling a connected tool does.
    fn tool_needs_approval(&self, tool_name: &str) -> bool {
        tool_name == tools::USE_CONNECTED_TOOL
    }

    fn describe_action(&self, tool_name: &str, input: &Value, _locale: nomi_agent_core::Locale) -> Option<String> {
        if tool_name != tools::USE_CONNECTED_TOOL {
            return None;
        }
        let text = |field: &str| input.get(field).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty());
        Some(format!("Use {} on {}", text("tool")?, text("server")?))
    }

    /// The rules for the steps this chat has reached (`stage_rules`), then the chat's newest design
    /// brief, when Rena saved one, the directions already proposed, the preview built last, the
    /// feedback still waiting on a new version and what the person approved.
    async fn domain_context(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _s3: Option<&nomi_storage::S3Config>,
        session_id: Uuid,
        _user_id: Uuid,
        _messages: &[nomi_llm::LlmMessage],
    ) -> Option<String> {
        let brief = design_brief::latest(&mut **conn, session_id).await.ok().flatten().map(|(version, brief)| brief_context(version, &brief));
        let directions =
            design_directions::latest(&mut **conn, session_id).await.ok().flatten().map(|(version, _, set)| directions_context(version, &set));
        let newest = design_artifact::latest(&mut **conn, session_id).await.ok().flatten();
        let approved = match &newest {
            Some(_) => design_approval::current(&mut **conn, session_id).await.ok().flatten(),
            None => None,
        };
        let rules = stage_rules(directions.is_some(), newest.is_some(), approved.is_some());
        let rules = (!rules.is_empty()).then(|| rules.join("\n\n"));
        let approval = newest.as_ref().map(|(version, _)| match &approved {
            Some(approval) => approval_context(approval, *version),
            None => NOT_APPROVED.to_string(),
        });
        let preview = match newest {
            Some((version, manifest)) => {
                let published = static_publish::latest(&mut **conn, session_id, version).await.ok().flatten();
                Some(preview_context(session_id, version, &manifest, published.as_ref()))
            }
            None => None,
        };
        let feedback = design_feedback::list(&mut **conn, session_id).await.ok().and_then(|all| feedback_context(&all));
        let flow = nomi_agent_core::build_flow::context_for(&mut **conn, session_id).await;
        let parts: Vec<String> = [rules, brief, directions, preview, feedback, approval, flow].into_iter().flatten().collect();
        (!parts.is_empty()).then(|| parts.join("\n\n"))
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(DESIGN_AGENT_TYPE)
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed(
            "the user wants design work on a site or app: its look and feel, visual directions to choose from, layout, colours, fonts, or feedback on how a design looks",
        )
    }

    /// Remembers durable facts the user mentions here (their brand, what they like).
    fn uses_memory(&self) -> bool {
        true
    }

    /// Her previews go through save_preview, so a cut-off one continues in pieces of it.
    fn cut_off_hint(&self) -> &'static str {
        nomi_agent_core::prompts::DESIGN_CUT_OFF_RETRY
    }

    fn uses_personality(&self) -> bool {
        true
    }

    /// An approved design goes to Koda.
    fn can_delegate(&self) -> bool {
        true
    }

    fn supports_todos(&self) -> bool {
        true
    }

    /// She designs what Rena and the person agreed.
    fn reads_plans(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nomi_agent_core::crew::DESIGN_AGENT_TYPE;
    use nomi_agent_core::prompts::DESIGN_SYSTEM_PROMPT;
    use nomi_agent_core::SubAgent;

    #[test]
    fn sora_is_the_design_agent_in_the_crew() {
        let sora = DesignAgent::new();
        assert_eq!(sora.agent_type(), DESIGN_AGENT_TYPE);
        assert_eq!(sora.agent_type(), "design");
        assert_eq!(sora.display_name(), "Sora");
        assert_eq!(sora.intent_label(), "design");
        assert_eq!(sora.system_prompt(), DESIGN_SYSTEM_PROMPT);
        assert!(sora.system_prompt().starts_with("You are Sora, the designer in Nomi's crew."));
        assert!(!sora.is_default());
    }

    #[test]
    fn sora_reads_the_plan_asks_and_can_hand_work_on() {
        let sora = DesignAgent::new();
        assert!(sora.reads_plans(), "she designs what was agreed");
        assert!(sora.asks_questions());
        assert!(sora.can_delegate(), "an approved design goes to Koda");
        assert!(sora.is_delegation_target(), "Rena hands design work to her");
        assert!(sora.uses_memory() && sora.uses_personality() && sora.supports_todos());
        assert!(!sora.supports_plans(), "the plan stays Rena's");
    }

    #[test]
    fn soras_own_tools_are_directions_previews_and_the_persons_connected_tools() {
        let names: Vec<String> = DesignAgent::new().tools().into_iter().map(|t| t.name).collect();
        assert_eq!(names, [tools::READ_DESIGN_SKILL, tools::PROPOSE_DIRECTIONS, tools::SAVE_PREVIEW, tools::READ_PREVIEW, tools::APPROVE_VERSION, tools::CREATE_HANDOFF, tools::REVIEW_BUILD, tools::LIST_CONNECTED_TOOLS, tools::USE_CONNECTED_TOOL]);
    }

    #[test]
    fn sora_reads_her_design_library_without_asking() {
        let sora = DesignAgent::new();
        assert!(!sora.tool_needs_approval(tools::READ_DESIGN_SKILL), "reading her own library changes nothing");
        let tool = sora.tools().into_iter().find(|t| t.name == tools::READ_DESIGN_SKILL).unwrap();
        assert!(tool.description.contains(&library::index()), "the tool lists every skill she can read");
    }

    #[test]
    fn sora_studies_taste_before_directions_and_the_canvas_format_before_drawing() {
        let core = DesignAgent::new().system_prompt();
        assert!(core.contains("read_design_skill") && core.contains("taste-core") && core.contains("styles-index"), "{core}");
        for said in ["read_design_skill", "pen-format", "components-kit", "taste-check"] {
            assert!(DESIGN_PREVIEW_RULES.contains(said), "her preview rules say {said:?}");
        }
        assert!(DESIGN_PREVIEW_RULES.contains("Taste wins"), "where her craft notes and Taste differ, Taste decides");
    }

    #[test]
    fn the_old_bans_are_gone_so_taste_decides() {
        for ban in ["no gradient text", "no glowing dots", "no emoji as icons", "no coloured stripe", "no uppercase eyebrow", "no three identical feature cards"] {
            assert!(!DESIGN_PREVIEW_RULES.contains(ban), "{ban:?} is Taste's call now");
            assert!(!DESIGN_SYSTEM_PROMPT.contains(ban), "{ban:?} is Taste's call now");
        }
    }

    #[test]
    fn looking_needs_no_ok_but_using_a_connected_tool_does() {
        let sora = DesignAgent::new();
        assert!(!sora.tool_needs_approval(tools::LIST_CONNECTED_TOOLS));
        assert!(sora.tool_needs_approval(tools::USE_CONNECTED_TOOL));
        assert!(!sora.tool_needs_approval(tools::PROPOSE_DIRECTIONS), "proposing only shows the person a choice");
        assert!(!sora.tool_needs_approval(tools::SAVE_PREVIEW), "a preview is the design work they asked for");
        let action = sora.describe_action(tools::USE_CONNECTED_TOOL, &serde_json::json!({"server": "Refs", "tool": "search_references"}), nomi_agent_core::Locale::En);
        assert_eq!(action.as_deref(), Some("Use search_references on Refs"));
        assert_eq!(sora.describe_action(tools::LIST_CONNECTED_TOOLS, &serde_json::json!({}), nomi_agent_core::Locale::En), None);
    }

    #[test]
    fn the_brief_is_put_in_front_of_sora_with_its_version_and_how_to_read_it() {
        let brief = nomi_agent_core::design_brief::DesignBrief::parse(&serde_json::json!({
            "company": [{"value": "Roti Pagi", "source": "confirmed"}],
            "audience": [{"value": "Neighbours", "source": "assumed"}],
            "objectives": [{"value": "Pre-orders", "source": "confirmed"}]
        }))
        .unwrap();
        let context = brief_context(4, &brief);
        assert!(context.contains("# Design brief (version 4)"));
        assert!(context.contains("- Neighbours (assumed, not confirmed)"));
        assert!(context.starts_with(BRIEF_INTRO));
        assert!(BRIEF_INTRO.contains("assumed") && BRIEF_INTRO.contains("confirmed"));
    }

    #[test]
    fn the_directions_already_proposed_are_put_in_front_of_sora_too() {
        let set = nomi_agent_core::design_directions::DesignDirections::parse(&tools::tests::two_directions()).unwrap();
        let context = directions_context(2, &set);
        assert!(context.starts_with(DIRECTIONS_INTRO));
        assert!(context.contains("# Design directions (version 2)") && context.contains("(id: editorial)"));
        assert!(DIRECTIONS_INTRO.contains("same id"));
    }

    #[test]
    fn the_previews_already_built_are_put_in_front_of_sora_with_their_ids() {
        let html = "<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>Home</title></head><body><h1 data-design-id=\"hero\">Hi</h1></body></html>";
        let input = serde_json::json!({"direction_id": "editorial", "title": "First pass", "files": [{"path": "index.html", "content": html}]});
        let manifest = nomi_agent_core::design_artifact::DesignArtifact::parse(&input).unwrap().manifest;
        let published = nomi_agent_core::static_publish::Published { provider: "fake".into(), url: "https://previews.example/a".into() };
        let context = preview_context(Uuid::nil(), 2, &manifest, Some(&published));
        assert!(context.starts_with(PREVIEW_INTRO));
        assert!(context.contains("# Design preview (version 2, direction: editorial)") && context.contains("- hero (element"));
        assert!(context.ends_with("Published at: https://previews.example/a"));
        assert!(context.contains("Design Studio: /chat/00000000-0000-0000-0000-000000000000/design?version=2"));
        assert!(preview_context(Uuid::nil(), 2, &manifest, None).ends_with("Not published outside Nomi."));
        assert!(PREVIEW_INTRO.contains("same data-design-id"));
    }

    #[test]
    fn feedback_still_waiting_is_put_in_front_of_sora_by_number() {
        let html = "<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>Home</title></head><body><h1 data-design-id=\"hero\">Hi</h1></body></html>";
        let input = serde_json::json!({"direction_id": "editorial", "title": "First pass", "files": [{"path": "index.html", "content": html}]});
        let manifest = nomi_agent_core::design_artifact::DesignArtifact::parse(&input).unwrap().manifest;
        let new = nomi_agent_core::design_feedback::NewFeedback::parse(1, &manifest, &serde_json::json!({"level": "element", "page": "index.html", "element_id": "hero", "body": "Shorter"})).unwrap();
        let feedback = nomi_agent_core::design_feedback::Feedback {
            id: Uuid::nil(),
            number: 4,
            version: new.version,
            level: new.level,
            page: new.page,
            element_id: new.element_id,
            body: new.body,
            pin: None,
            status: nomi_agent_core::design_feedback::Status::Open,
            resolved_in_version: None,
            resolution: None,
            user_id: Uuid::nil(),
            created_at: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
            resolved_at: None,
        };
        let context = feedback_context(std::slice::from_ref(&feedback)).unwrap();
        assert!(context.starts_with(FEEDBACK_INTRO));
        assert!(context.ends_with("- #4 (version 1, element hero on index.html): Shorter"), "{context}");
        assert!(FEEDBACK_INTRO.contains("only what it asks"));
        let dealt_with = nomi_agent_core::design_feedback::Feedback { status: nomi_agent_core::design_feedback::Status::Resolved, ..feedback };
        assert_eq!(feedback_context(&[dealt_with]), None, "feedback a version dealt with isn't asked for again");
    }

    use nomi_agent_core::prompts::{DESIGN_CHANGE_RULES, DESIGN_HANDOFF_RULES, DESIGN_PREVIEW_RULES};

    #[test]
    fn each_stages_rules_join_only_once_the_chat_reaches_that_stage() {
        let none: [&str; 0] = [];
        assert_eq!(stage_rules(false, false, false), none, "before any directions, the core prompt is enough");
        assert_eq!(stage_rules(true, false, false), [DESIGN_PREVIEW_RULES]);
        assert_eq!(stage_rules(true, true, false), [DESIGN_PREVIEW_RULES, DESIGN_CHANGE_RULES]);
        assert_eq!(stage_rules(false, true, false), [DESIGN_PREVIEW_RULES, DESIGN_CHANGE_RULES], "a preview means she's building");
        assert_eq!(stage_rules(true, true, true), [DESIGN_PREVIEW_RULES, DESIGN_CHANGE_RULES, DESIGN_HANDOFF_RULES]);
    }

    #[test]
    fn the_core_prompt_is_short_and_keeps_the_rules_that_must_always_hold() {
        let core = DesignAgent::new().system_prompt();
        for said in ["Only the person approves", "never say a design is approved", "never invent a link", "propose_directions", "read_plan"] {
            assert!(core.contains(said), "the core says {said:?}");
        }
        for later in ["data-design-id", "resolves", "create_handoff", "review_build"] {
            assert!(!core.contains(later), "{later:?} waits for its stage");
        }
        assert!(core.len() < 2_500, "the core is {} bytes", core.len());
    }

    #[test]
    fn sora_builds_previews_but_never_invents_a_link_or_the_apps_code() {
        let rules = DESIGN_PREVIEW_RULES;
        assert!(rules.contains("save_preview") && rules.contains("data-design-id") && rules.contains("finished"));
        assert!(rules.contains("canvas.pen") && rules.contains("OpenPencil"));
        assert!(rules.contains("do not build raw HTML"));
        assert!(rules.contains("Design Studio") && rules.contains("exactly as it is"));
        assert!(rules.contains("never write the app's code"));
    }

    #[test]
    fn sora_refines_from_feedback_without_redoing_the_design() {
        let sora = DesignAgent::new();
        for said in ["read_preview", "resolves", "only what the feedback asks", "never mark feedback you didn't act on", "Approve"] {
            assert!(DESIGN_CHANGE_RULES.contains(said), "her change rules say {said:?}");
        }
        assert!(!sora.tool_needs_approval(tools::READ_PREVIEW), "reading her own preview needs no OK");
    }

    #[test]
    fn only_the_person_approves_and_sora_is_told_what_they_approved() {
        let sora = DesignAgent::new();
        let prompt = sora.system_prompt();
        assert!(prompt.contains("Only the person approves") && prompt.contains("never say a design is approved"), "{prompt}");
        let approves: Vec<String> = sora.tools().into_iter().filter(|tool| tool.name.contains("approv")).map(|tool| tool.name).collect();
        assert_eq!(approves, [tools::APPROVE_VERSION], "her one approving tool records what the person said in the chat");
        assert!(prompt.contains("approve_version"), "she records an approval given in the chat");
        let approval = nomi_agent_core::design_approval::Approval { version: 2, note: None, user_id: Uuid::nil(), created_at: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH };
        let context = approval_context(&approval, 3);
        assert!(context.starts_with("# Approved design\nThe person approved version 2"), "{context}");
        assert!(context.contains("Version 3 is newer and not approved"));
        assert!(NOT_APPROVED.starts_with("No version of the design is approved yet") && NOT_APPROVED.contains("approve_version"));
    }

    #[test]
    fn an_approved_design_goes_to_koda_as_a_handoff() {
        let sora = DesignAgent::new();
        let rules = DESIGN_HANDOFF_RULES;
        assert!(rules.contains("create_handoff") && rules.contains("delegate_to_agent"), "{rules}");
        assert!(!sora.tool_needs_approval(tools::CREATE_HANDOFF), "the person already approved the design");
    }

    #[test]
    fn sora_checks_the_build_against_the_design_and_says_what_she_couldnt_see() {
        let sora = DesignAgent::new();
        let rules = DESIGN_HANDOFF_RULES;
        assert!(rules.contains("review_build") && rules.contains("never say a build matches the design"), "{rules}");
        assert!(!sora.tool_needs_approval(tools::REVIEW_BUILD), "reading the build changes nothing");
    }

    #[test]
    fn sora_is_told_to_carry_on_when_research_fails() {
        let prompt = DesignAgent::new().system_prompt();
        for said in ["propose_directions", "web_search", "read_web_page", "list_connected_tools", "use_connected_tool", "carry on"] {
            assert!(prompt.contains(said), "her prompt still says {said:?}");
        }
    }

    #[test]
    fn the_classifier_is_told_design_is_about_the_look_not_the_build() {
        let description = DesignAgent::new().intent_description();
        assert!(description.contains("design") && description.contains("look"));
        assert!(!description.contains("built"), "building stays Rena's and Koda's");
    }
}
