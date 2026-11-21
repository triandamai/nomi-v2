//! Sora's own tools: proposing directions, building a preview of one, and the tools on the MCP
//! servers the person connected (visual references, moodboards, whatever they have), through
//! `McpOps`.

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::mcp_ops::{format_listing, McpOps, SharedMcpOps, CARRY_ON};
use nomi_agent_core::design_artifact::{self, ArtifactFile, DesignArtifact, Manifest, ID_ATTRIBUTE, PEN_ENTRY};
use nomi_agent_core::design_approval;
use nomi_agent_core::design_draft;
use nomi_agent_core::design_feedback::{self, Status};
use nomi_agent_core::design_handoff::{self, FeedbackOutcome, Package, Spec};
use nomi_agent_core::design_review::{self, Review, SharedProjectSource};
use nomi_agent_core::design_brief;
use nomi_agent_core::static_publish::{self, PublishRequest, Published, SharedStaticPublisher};
use nomi_agent_core::design_directions::{self, DesignDirections};
use nomi_agent_core::{ContentBlock, Tool, ToolOutcome, View};
use nomi_llm::ToolDefinition;

pub const PROPOSE_DIRECTIONS: &str = "propose_directions";
pub const SAVE_PREVIEW: &str = "save_preview";
pub const READ_PREVIEW: &str = "read_preview";
pub const CREATE_HANDOFF: &str = "create_handoff";
pub const REVIEW_BUILD: &str = "review_build";
pub const APPROVE_VERSION: &str = "approve_version";
/// What Sora is told when she tries to hand over a design the person hasn't approved.
pub const NOT_APPROVED_YET: &str = "the person hasn't approved a version yet: ask them to approve the one they want built, here in the chat or with Approve in the Design Studio, then hand it over";
/// The fewest characters of the person's words an approval quotes: enough to be their reply, not one letter.
const MIN_QUOTE_CHARS: usize = 2;
pub const LIST_CONNECTED_TOOLS: &str = "list_connected_tools";
pub const USE_CONNECTED_TOOL: &str = "use_connected_tool";
/// The tools' group in listings.
const GROUP: &str = "design";

/// What is on the person's servers, as Sora reads it.
pub async fn list_connected(mcp: &dyn McpOps, user_id: Uuid) -> String {
    format_listing(&mcp.tools(user_id).await)
}

/// Calls the tool the input names. A failure comes back as words for Sora, never as a failed turn.
pub async fn use_connected(mcp: &dyn McpOps, user_id: Uuid, input: &Value) -> Result<String, String> {
    let text = |field: &str| input.get(field).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty());
    let server = text("server").ok_or("server is required")?;
    let tool = text("tool").ok_or("tool is required")?;
    let arguments = match input.get("arguments") {
        None | Some(Value::Null) => json!({}),
        Some(arguments) if arguments.is_object() => arguments.clone(),
        Some(_) => return Err("arguments must be an object".to_string()),
    };
    mcp.call(user_id, server, tool, arguments).await.map_err(|why| format!("{tool} on {server} didn't work: {why}. {CARRY_ON}"))
}

/// `list_connected_tools`.
pub struct ListConnectedTools {
    pub mcp: SharedMcpOps,
}

#[async_trait]
impl Tool for ListConnectedTools {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: LIST_CONNECTED_TOOLS.to_string(),
            description: "List the tools on the MCP servers the person connected (visual references, moodboards, their own design \
                          tools), each with its server and inputs. Call it before use_connected_tool."
                .to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        }
    }

    fn group(&self) -> &'static str {
        GROUP
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["mcp", "connected", "references"]
    }

    async fn execute(&self, _conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, _input: Value) -> Result<ToolOutcome, String> {
        Ok(ToolOutcome::text(list_connected(self.mcp.as_ref(), user_id).await))
    }
}

/// `use_connected_tool`.
pub struct UseConnectedTool {
    pub mcp: SharedMcpOps,
}

#[async_trait]
impl Tool for UseConnectedTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: USE_CONNECTED_TOOL.to_string(),
            description: "Call one tool from list_connected_tools: its server and tool exactly as listed, and arguments that fit \
                          its inputs. The person approves it first. If it fails, carry on without it."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server": {"type": "string", "description": "The server's name, as listed"},
                    "tool": {"type": "string", "description": "The tool's name, as listed"},
                    "arguments": {"type": "object", "description": "The tool's inputs"}
                },
                "required": ["server", "tool"]
            }),
        }
    }

    fn group(&self) -> &'static str {
        GROUP
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["mcp", "connected", "references"]
    }

    async fn execute(&self, _conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        use_connected(self.mcp.as_ref(), user_id, &input).await.map(ToolOutcome::text)
    }
}

/// What Sora is told after a set was saved and shown.
pub fn proposed_message(version: i32, brief_version: Option<i32>, set: &DesignDirections) -> String {
    let from = brief_version.map(|v| format!(", from design brief version {v}")).unwrap_or_default();
    let names: Vec<String> = set.directions.iter().map(|d| format!("{} ({})", d.name, d.id)).collect();
    format!(
        "Directions saved as version {version}{from}: {}. The person now sees them as a choice card under your reply, with each one's name, look, reason, \
         trade-offs and your pick marked: don't list or describe them again. Say in one sentence what to weigh \
         between them, then wait for their answer.",
        names.join(", ")
    )
}

/// `propose_directions`: checks the set, saves it as the chat's next version and shows the
/// person the choice.
pub struct ProposeDirections;

#[async_trait]
impl Tool for ProposeDirections {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: PROPOSE_DIRECTIONS.to_string(),
            description: "Propose two to four clearly different design directions for the person to choose from, each with what \
                          it looks like, why it fits the brief, its trade-offs, colours, fonts and layout. It saves them as a \
                          new version and shows the person the choice. Call it again with the whole set (same ids for the \
                          directions you keep) when they ask for changes."
                .to_string(),
            input_schema: design_directions::directions_input_schema(),
        }
    }

    fn group(&self) -> &'static str {
        GROUP
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["design", "directions", "look"]
    }

    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        let set = DesignDirections::parse(&input)?;
        let brief_version = design_brief::latest(&mut **conn, session_id).await.ok().flatten().map(|(version, _)| version);
        let version = design_directions::save(&mut **conn, session_id, agent_session_id, user_id, brief_version, &set).await.map_err(|e| {
            tracing::error!(error = %e, "design directions: save failed");
            "the directions couldn't be saved right now".to_string()
        })?;
        Ok(ToolOutcome { display_text: proposed_message(version, brief_version, &set), block: Some(ContentBlock::View(View::Decision(set.to_decision()))) })
    }
}

/// Whether `direction_id` is one of the directions proposed in this chat; the error tells Sora
/// what to do instead.
pub fn known_direction(set: Option<&DesignDirections>, direction_id: &str) -> Result<(), String> {
    let Some(set) = set else {
        return Err(format!("propose directions with {PROPOSE_DIRECTIONS} first, then build the one the person picks"));
    };
    if set.find(direction_id).is_some() {
        return Ok(());
    }
    let ids: Vec<&str> = set.directions.iter().map(|d| d.id.as_str()).collect();
    Err(format!("there is no direction \"{direction_id}\": the directions are {}", ids.join(", ")))
}

fn counted(count: usize, one: &str) -> String {
    let plural = if count == 1 { "" } else { "s" };
    format!("{count} {one}{plural}")
}

/// Where a version of the chat's previews opens in the Design Studio: a path inside Nomi.
pub fn studio_link(session_id: Uuid, version: i32) -> String {
    format!("/chat/{session_id}/design?version={version}")
}

/// What Sora is told after a preview was saved: its version, the ids that went missing since
/// the direction's last version, its Design Studio link, and its published address when it has one.
/// Added to why a finishing `save_preview` was refused when files were waiting in the draft.
pub use nomi_agent_core::prompts::DESIGN_DRAFT_KEPT as DRAFT_KEPT;

/// Added to a saved version built with files from the draft, so a forgotten piece doesn't slip in unnoticed.
pub fn from_draft_message(paths: &[String]) -> String {
    if paths.is_empty() {
        return String::new();
    }
    format!(" It includes these files from the draft: {}.", paths.join(", "))
}

/// What Sora is told after a piece of a preview went into the draft.
pub fn staged_message(direction_id: &str, paths: &[&str]) -> String {
    format!(
        "Saved to the draft of direction {direction_id}, which now has: {}. The person sees nothing yet: send the rest, \
         and set finished to true on the last call.",
        paths.join(", ")
    )
}

pub fn saved_message(session_id: Uuid, version: i32, manifest: &Manifest, missing: &[String], published: Option<&Result<Published, String>>) -> String {
    let mut said = format!(
        "Preview saved as version {version} of direction {}: {}, {}.",
        manifest.direction_id,
        counted(manifest.pages.len(), "page"),
        counted(manifest.elements.len(), "element id")
    );
    if !missing.is_empty() {
        said.push_str(&format!(
            " These ids from the last version of this direction are gone: {}. Feedback on them has nowhere to point now: keep an element's id for as long as the element exists.",
            missing.join(", ")
        ));
    }
    said.push_str(&format!(
        " The person can look at it in the Design Studio, at phone, tablet and desktop width: give them this link exactly as it is: [Open in Design Studio]({}).",
        studio_link(session_id, version)
    ));
    match published {
        Some(Ok(published)) => said.push_str(&format!(" It is also published at {}, a link they can pass on.", published.url)),
        Some(Err(why)) => said.push_str(&format!(" It isn't published outside Nomi ({why}), so that is the only link: never give another.")),
        None => said.push_str(" That is the only link: never give another."),
    }
    said
}

/// What Sora is told about the feedback a saved version dealt with, and what is still waiting.
pub fn resolved_message(version: i32, resolved: &[i32], still_open: usize) -> String {
    let waiting = match still_open {
        1 => "1 piece of feedback is".to_string(),
        many => format!("{many} pieces of feedback are"),
    };
    if resolved.is_empty() {
        return if still_open == 0 { String::new() } else { format!(" {waiting} still waiting: name the ones a version deals with in resolves.") };
    }
    let numbers: Vec<String> = resolved.iter().map(|n| format!("#{n}")).collect();
    let left = if still_open == 0 { "no feedback is waiting now".to_string() } else { format!("{waiting} still waiting") };
    format!(" Feedback {} is marked as dealt with in version {version}; {left}.", numbers.join(", "))
}

/// A version's files as Sora reads them back: all of them under the version's manifest, or the one at `path`.
pub fn read_message(version: i32, manifest: &Manifest, files: &[ArtifactFile], path: Option<&str>) -> Result<String, String> {
    match path {
        Some(path) => match files.iter().find(|f| f.path == path) {
            Some(file) => Ok(format!("## {} (version {version})\n{}", file.path, file.content)),
            None => {
                let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
                Err(format!("version {version} has no file \"{path}\": its files are {}", paths.join(", ")))
            }
        },
        None => {
            let whole: Vec<String> = files.iter().map(|f| format!("## {}\n{}", f.path, f.content)).collect();
            Ok(format!("{}\n\n{}", manifest.to_markdown(version), whole.join("\n\n")))
        }
    }
}

/// `read_preview`: the files of one of the chat's preview versions, to change from.
pub struct ReadPreview;

#[async_trait]
impl Tool for ReadPreview {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: READ_PREVIEW.to_string(),
            description: "Read the files of a preview you built in this chat, as they were saved: the newest version unless you \
                          name one, every file unless you name a path. Call it before changing a preview, so you change from \
                          what is there instead of writing it again from memory."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "version": {"type": "integer", "description": "The preview version; the newest when left out"},
                    "path": {"type": "string", "description": "One file's path; every file when left out"}
                }
            }),
        }
    }

    fn group(&self) -> &'static str {
        GROUP
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["design", "preview", "files"]
    }

    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, _agent_session_id: Uuid, _user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        let unread = |e: sqlx::Error| {
            tracing::error!(error = %e, "design preview: read failed");
            "the preview couldn't be read right now".to_string()
        };
        let asked = input.get("version").and_then(Value::as_i64).and_then(|v| i32::try_from(v).ok());
        let (version, manifest) = match asked {
            Some(version) => {
                let manifest = design_artifact::version(&mut **conn, session_id, version).await.map_err(unread)?;
                (version, manifest.ok_or_else(|| format!("this chat has no preview version {version}"))?)
            }
            None => design_artifact::latest(&mut **conn, session_id).await.map_err(unread)?.ok_or_else(|| format!("there is no preview in this chat yet: build one with {SAVE_PREVIEW}"))?,
        };
        let files = design_artifact::files(&mut **conn, session_id, version).await.map_err(unread)?;
        let path = input.get("path").and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty());
        read_message(version, &manifest, &files, path).map(ToolOutcome::text)
    }
}

/// `save_preview`: checks the files, saves them as the chat's next preview version, and
/// publishes it when a provider is set up. A preview that can't be published is still saved.
pub struct SavePreview {
    pub publisher: SharedStaticPublisher,
}

#[async_trait]
impl Tool for SavePreview {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: SAVE_PREVIEW.to_string(),
            description: format!(
                "Build a preview of one proposed direction as an OpenPencil scene graph ({PEN_ENTRY}): \
                 do not build raw HTML pages. Include design variables ($color-*, typography, spacing), a design \
                 system frame (colours, type scale, buttons, form fields, components), and app flow screen frames, \
                 with a {ID_ATTRIBUTE} / id slug on each frame, section and component (a lowercase slug, unique in its \
                 screen/frame, the same across screens that share the element). A preview can be sent in pieces if large: \
                 send calls with finished false, and the last call with finished true, which saves the preview as a new version \
                 (describe elements on that last call; what you said about an element before carries over). To change a preview, \
                 send the updated {PEN_ENTRY} (the rest stay as they were, with the same ids; drop a file with remove), and name in \
                 resolves the open feedback the new version dealt with."
            ),
            input_schema: design_artifact::artifact_input_schema(),
        }
    }

    fn group(&self) -> &'static str {
        GROUP
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["design", "preview", "mockup"]
    }

    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        let sent = design_artifact::parse_file_list(&input)?;
        let remove = design_artifact::removals(&input)?;
        let finished = input.get("finished").and_then(Value::as_bool).unwrap_or(true);
        let direction_id = input.get("direction_id").and_then(Value::as_str).ok_or("direction_id is required")?.trim().to_lowercase();
        let resolutions = design_feedback::parse_resolutions(&input)?;
        let unsaved = |e: sqlx::Error| {
            tracing::error!(error = %e, "design preview: save failed");
            "the preview couldn't be saved right now".to_string()
        };
        let directions = design_directions::latest(&mut **conn, session_id).await.map_err(unsaved)?;
        known_direction(directions.as_ref().map(|(_, _, set)| set), &direction_id)?;
        // The first preview waits for the person to pick a direction.
        nomi_agent_core::build_flow::facts(&mut **conn, session_id).await.map_err(unsaved)?.preview_allowed()?;
        let draft = design_draft::draft(&mut **conn, session_id, &direction_id).await.map_err(unsaved)?;
        if !finished {
            let mut paths: Vec<&str> = draft.iter().map(|f| f.path.as_str()).collect();
            paths.extend(sent.iter().map(|f| f.path.as_str()).filter(|path| !draft.iter().any(|f| f.path == *path)));
            if paths.len() > design_artifact::MAX_FILES {
                return Err(format!("a preview has at most {} files", design_artifact::MAX_FILES));
            }
            let message = staged_message(&direction_id, &paths);
            design_draft::stage(&mut **conn, session_id, &direction_id, &sent).await.map_err(unsaved)?;
            return Ok(ToolOutcome::text(message));
        }
        let before = design_artifact::latest_of_direction(&mut **conn, session_id, &direction_id).await.map_err(unsaved)?;
        let previous = match &before {
            Some((version, _)) => design_artifact::files(&mut **conn, session_id, *version).await.map_err(unsaved)?,
            None => Vec::new(),
        };
        let drafted: Vec<String> = draft.iter().map(|f| f.path.clone()).collect();
        let merged = design_draft::merge(previous, draft, sent, &remove);
        let mut artifact = DesignArtifact::from_files(&input, merged).map_err(|e| if drafted.is_empty() { e } else { format!("{e}. {DRAFT_KEPT}") })?;
        if let Some((_, earlier)) = &before {
            artifact.inherit_descriptions(earlier, &input);
        }
        let from_draft: Vec<String> = drafted.iter().filter(|path| artifact.file(path).is_some()).cloned().collect();
        let missing = before.map(|(_, earlier)| artifact.manifest.missing_since(&earlier)).unwrap_or_default();
        let feedback = design_feedback::list(&mut **conn, session_id).await.map_err(unsaved)?;
        design_feedback::check_open(&resolutions, &feedback)?;
        let directions_version = directions.map(|(version, _, _)| version);
        let version = design_artifact::save(&mut **conn, session_id, agent_session_id, user_id, directions_version, &artifact).await.map_err(unsaved)?;
        if let Err(e) = design_draft::clear(&mut **conn, session_id, &direction_id, &drafted).await {
            // The version is saved; a draft left behind is only merged into the next one again.
            tracing::error!(error = %e, "design preview: the draft couldn't be cleared");
        }

        // A .pen preview is a canvas for the Design Studio, not a website: only HTML pages are published.
        let published = if artifact.files.iter().any(|f| f.path.ends_with(".html")) {
            let request = PublishRequest { user_id, session_id, version, title: &artifact.manifest.title, files: &artifact.files };
            Some(static_publish::publish_and_record(&mut **conn, self.publisher.as_ref(), &request).await)
        } else {
            None
        };
        let resolved = design_feedback::resolve(&mut **conn, session_id, version, &resolutions).await.map_err(|e| {
            tracing::error!(error = %e, "design preview: feedback couldn't be marked");
            format!("version {version} is saved, but the feedback it dealt with couldn't be marked")
        })?;
        let still_open = feedback.iter().filter(|f| f.status == Status::Open && !resolved.contains(&f.number)).count();
        let mut said = saved_message(session_id, version, &artifact.manifest, &missing, published.as_ref());
        said.push_str(&from_draft_message(&from_draft));
        said.push_str(&resolved_message(version, &resolved, still_open));
        Ok(ToolOutcome::text(said))
    }
}

/// Text folded for comparing what someone said: lowercase, with runs of whitespace as one space.
fn folded(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Whether `quote` is words of `said` (the person's latest message), ignoring case and spacing.
pub fn quoted_in(said: &str, quote: &str) -> bool {
    let quote = folded(quote);
    quote.chars().count() >= MIN_QUOTE_CHARS && folded(said).contains(&quote)
}

/// `approve_version`: records the approval the person just gave in the chat. It only works with
/// their own words from their latest message, so Sora can't approve on her own.
pub struct ApproveVersion;

#[async_trait]
impl Tool for ApproveVersion {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: APPROVE_VERSION.to_string(),
            description: "Record that the person approved a preview version, when they said so in this chat. Quote their \
                          latest message's words exactly in quote; it is refused when their latest message doesn't contain \
                          them. Never call it on your own judgement or on an unclear reply: ask instead."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "version": {"type": "integer", "description": "The preview version they approved; the newest when they didn't name one"},
                    "quote": {"type": "string", "description": "The words of their latest message that approve it, exactly as they wrote them"},
                    "note": {"type": "string", "description": "Anything they asked for with the approval, in a sentence"}
                },
                "required": ["version", "quote"]
            }),
        }
    }

    fn group(&self) -> &'static str {
        GROUP
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["design", "approve", "approval"]
    }

    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        let unsaved = |e: sqlx::Error| {
            tracing::error!(error = %e, "design approval in chat: failed");
            "the approval couldn't be recorded right now".to_string()
        };
        let version = input
            .get("version")
            .and_then(Value::as_i64)
            .and_then(|v| i32::try_from(v).ok())
            .ok_or("version must be a preview version number")?;
        let quote = input.get("quote").and_then(Value::as_str).unwrap_or_default();
        let note = design_approval::note_of(&input)?;
        let said: Option<String> = sqlx::query_scalar(
            "SELECT content FROM messages WHERE session_id = $1 AND sender_channel_identity_id IS NOT NULL \
             ORDER BY created_at DESC, id DESC LIMIT 1",
        )
        .bind(session_id)
        .fetch_optional(&mut **conn)
        .await
        .map_err(unsaved)?;
        if !said.as_deref().is_some_and(|said| quoted_in(said, quote)) {
            return Err(nomi_agent_core::prompts::DESIGN_APPROVAL_NOT_SAID.to_string());
        }
        if design_artifact::version(&mut **conn, session_id, version).await.map_err(unsaved)?.is_none() {
            return Err(format!("there is no preview version {version} in this chat"));
        }
        design_approval::approve(&mut **conn, session_id, user_id, version, note.as_deref()).await.map_err(unsaved)?;
        Ok(ToolOutcome::text(nomi_agent_core::prompts::DESIGN_APPROVED_IN_CHAT.replace("{version}", &version.to_string())))
    }
}

/// What Sora is told after a handoff was saved: how it reaches Koda.
pub fn handoff_message(version: i32, artifact_version: i32, project: Option<Uuid>) -> String {
    let next = match project {
        Some(project_id) => format!(
            "Now pass it on: call delegate_to_agent with target coding and a task that starts exactly \"Project {project_id}: \" followed by \
             one line on what to build. Koda gets the whole package in his brief, so don't repeat it in the task."
        ),
        None => "There is no project in this chat yet, so Koda has nowhere to build it: tell the person the design is ready and that \
                 Rena sets the project up; Koda gets the whole package in his brief once she hands the build over."
            .to_string(),
    };
    format!("Handoff saved as version {version}, for approved preview version {artifact_version}. {next}")
}

/// `create_handoff`: writes down the approved design for Koda, with the brief it came from, what
/// became of the feedback and the approved preview's version and address.
pub struct CreateHandoff;

#[async_trait]
impl Tool for CreateHandoff {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: CREATE_HANDOFF.to_string(),
            description: "Write the handoff for the version the person approved, so Koda can build it without guessing: the design \
                          system, sitemap, user flows, every page and component (by their data-design-id), interactions, responsive \
                          rules, accessibility, assets and the decisions made. The brief, the feedback's outcome and the approved \
                          preview's version and address are added for you. It only works once the person has approved a version."
                .to_string(),
            input_schema: design_handoff::handoff_input_schema(),
        }
    }

    fn group(&self) -> &'static str {
        GROUP
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["design", "handoff", "build"]
    }

    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        let unsaved = |e: sqlx::Error| {
            tracing::error!(error = %e, "design handoff: save failed");
            "the handoff couldn't be saved right now".to_string()
        };
        let approval = design_approval::current(&mut **conn, session_id).await.map_err(unsaved)?.ok_or(NOT_APPROVED_YET)?;
        let manifest = design_artifact::version(&mut **conn, session_id, approval.version).await.map_err(unsaved)?.ok_or(NOT_APPROVED_YET)?;
        let spec = Spec::parse(&input, &manifest)?;
        let brief = design_brief::latest(&mut **conn, session_id).await.map_err(unsaved)?;
        let feedback = design_feedback::list(&mut **conn, session_id).await.map_err(unsaved)?;
        let published = static_publish::latest(&mut **conn, session_id, approval.version).await.map_err(unsaved)?;
        let package = Package {
            spec,
            artifact_version: approval.version,
            direction_id: manifest.direction_id,
            studio_link: studio_link(session_id, approval.version),
            published_url: published.map(|published| published.url),
            brief_version: brief.as_ref().map(|(version, _)| *version),
            brief: brief.map(|(_, brief)| brief),
            feedback: feedback.iter().map(FeedbackOutcome::from).collect(),
        };
        let version = design_handoff::save(&mut **conn, session_id, agent_session_id, user_id, &package).await.map_err(unsaved)?;
        let project: Option<Uuid> = sqlx::query_scalar("SELECT id FROM projects WHERE id = chat_project($1)").bind(session_id).fetch_optional(&mut **conn).await.ok().flatten();
        Ok(ToolOutcome::text(handoff_message(version, approval.version, project)))
    }
}

/// What Sora is told after a build was checked: the review, and what to do about it.
pub fn review_message(review: &str, project_id: Uuid, deviations: usize) -> String {
    let next = if deviations == 0 {
        "There is nothing to fix from this check: tell the person the build has the design's colours, fonts and parts, and what the check can't see.".to_string()
    } else {
        format!(
            "Tell the person in plain words what differs from the design they approved. If they want it put right, hand it to Koda: \
             delegate_to_agent with target coding and a task that starts exactly \"Project {project_id}: \" and lists what to fix."
        )
    };
    format!("{review}\n\n{next}")
}

/// `review_build`: checks the project Koda built against the handoff he built it from.
pub struct ReviewBuild {
    pub source: SharedProjectSource,
}

#[async_trait]
impl Tool for ReviewBuild {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: REVIEW_BUILD.to_string(),
            description: "Check what Koda built against the design that was handed over: it looks for the handoff's colours, fonts and \
                          element ids in the project's files and says which are there and which aren't. It can't judge spacing or \
                          feel. Checks this chat's project unless you name another."
                .to_string(),
            input_schema: json!({"type": "object", "properties": {"project_id": {"type": "string", "description": "The project to check; this chat's project when left out"}}}),
        }
    }

    fn group(&self) -> &'static str {
        GROUP
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["design", "review", "build"]
    }

    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        let unsaved = |e: sqlx::Error| {
            tracing::error!(error = %e, "design review: failed");
            "the build couldn't be checked right now".to_string()
        };
        let (handoff_version, package) =
            design_handoff::latest(&mut **conn, session_id).await.map_err(unsaved)?.ok_or(format!("there is no handoff in this chat yet: a build is checked against the handoff it was built from ({CREATE_HANDOFF})"))?;
        let named = match input.get("project_id").and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty()) {
            Some(id) => Some(id.parse::<Uuid>().map_err(|_| "project_id isn't a project's id".to_string())?),
            None => sqlx::query_scalar("SELECT id FROM projects WHERE id = chat_project($1)").bind(session_id).fetch_optional(&mut **conn).await.map_err(unsaved)?,
        };
        let project_id = named.ok_or("there is no project in this chat to check")?;
        let files = self.source.files(user_id, project_id).await.map_err(|why| format!("the build couldn't be checked: {why}"))?;
        let review = Review::of(&package, &files);
        let version = design_review::save(&mut **conn, session_id, agent_session_id, user_id, handoff_version, project_id, &review).await.map_err(unsaved)?;
        Ok(ToolOutcome::text(review_message(&review.to_markdown(version, handoff_version), project_id, review.deviations().len())))
    }
}

/// Sora's own tools.
pub fn own_tools(mcp: &SharedMcpOps, publisher: &SharedStaticPublisher, source: &SharedProjectSource) -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(ProposeDirections),
        Box::new(SavePreview { publisher: publisher.clone() }),
        Box::new(ReadPreview),
        Box::new(ApproveVersion),
        Box::new(CreateHandoff),
        Box::new(ReviewBuild { source: source.clone() }),
        Box::new(ListConnectedTools { mcp: mcp.clone() }),
        Box::new(UseConnectedTool { mcp: mcp.clone() }),
    ]
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use async_trait::async_trait;
    use nomi_agent_core::mcp_ops::{McpListing, McpOps, McpToolInfo, CARRY_ON, NOTHING_CONNECTED};
    use serde_json::json;
    use std::sync::Mutex;

    #[test]
    fn a_version_says_which_files_came_from_the_draft() {
        assert_eq!(from_draft_message(&[]), "");
        let said = from_draft_message(&["about.html".to_string(), "styles.css".to_string()]);
        assert!(said.contains("about.html, styles.css"), "{said}");
    }

    #[test]
    fn a_refused_finish_says_how_to_drop_a_bad_draft_file() {
        assert!(DRAFT_KEPT.contains("remove"));
    }

    /// The person's servers as a test wants them, recording what was called.
    #[derive(Default)]
    struct FakeMcp {
        listing: McpListing,
        answer: Option<Result<String, String>>,
        called: Mutex<Vec<(String, String, Value)>>,
    }

    #[async_trait]
    impl McpOps for FakeMcp {
        async fn tools(&self, _: Uuid) -> McpListing {
            self.listing.clone()
        }
        async fn call(&self, _: Uuid, server: &str, tool: &str, arguments: Value) -> Result<String, String> {
            self.called.lock().unwrap().push((server.to_string(), tool.to_string(), arguments));
            self.answer.clone().unwrap_or(Ok("done".to_string()))
        }
    }

    pub(crate) fn two_directions() -> Value {
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

    #[test]
    fn proposing_tells_sora_the_version_the_brief_it_came_from_and_to_wait() {
        let set = DesignDirections::parse(&two_directions()).unwrap();
        let said = proposed_message(3, Some(2), &set);
        assert!(said.starts_with("Directions saved as version 3, from design brief version 2: Editorial (editorial), Swiss utility (swiss)."), "{said}");
        assert!(said.contains("wait for their answer"));
        assert!(proposed_message(1, None, &set).starts_with("Directions saved as version 1: Editorial"));
    }

    #[test]
    fn the_propose_tool_takes_the_directions_schema() {
        let tool = ProposeDirections.definition();
        assert_eq!(tool.name, PROPOSE_DIRECTIONS);
        assert_eq!(tool.input_schema, nomi_agent_core::design_directions::directions_input_schema());
    }

    fn manifest() -> nomi_agent_core::design_artifact::Manifest {
        let html = "<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>Home</title></head><body><h1 data-design-id=\"hero\">Hi</h1></body></html>";
        nomi_agent_core::design_artifact::DesignArtifact::parse(&json!({"direction_id": "editorial", "title": "First pass", "files": [{"path": "index.html", "content": html}]}))
            .unwrap()
            .manifest
    }

    #[test]
    fn a_saved_preview_gives_sora_the_studio_link_and_no_other() {
        let said = saved_message(Uuid::nil(), 2, &manifest(), &[], Some(&Err(nomi_agent_core::static_publish::NOT_SET_UP.to_string())));
        assert!(said.starts_with("Preview saved as version 2 of direction editorial: 1 page, 1 element id."), "{said}");
        assert!(said.contains("[Open in Design Studio](/chat/00000000-0000-0000-0000-000000000000/design?version=2)"), "{said}");
        assert!(said.contains("It isn't published outside Nomi (publishing previews isn't set up)"));
        assert!(said.contains("never give another"));
    }

    #[test]
    fn a_published_preview_tells_sora_its_address() {
        let published = nomi_agent_core::static_publish::Published { provider: "fake".into(), url: "https://previews.example/a".into() };
        let said = saved_message(Uuid::nil(), 1, &manifest(), &[], Some(&Ok(published)));
        assert!(said.contains("It is also published at https://previews.example/a"), "{said}");
        assert!(said.contains("/design?version=1"));
        assert!(!said.contains("isn't published"));
    }

    #[test]
    fn ids_that_went_missing_are_pointed_out() {
        let said = saved_message(Uuid::nil(), 3, &manifest(), &["menu-list".to_string(), "footer".to_string()], Some(&Err("down".into())));
        assert!(said.contains("These ids from the last version of this direction are gone: menu-list, footer."), "{said}");
        assert!(said.contains("keep an element's id"));
    }

    #[test]
    fn sora_reads_a_versions_files_whole_or_one_at_a_time() {
        use nomi_agent_core::design_artifact::ArtifactFile;
        let files = [ArtifactFile { path: "index.html".into(), content: "<h1>Hi</h1>".into() }, ArtifactFile { path: "styles.css".into(), content: "h1 { margin: 0 }".into() }];
        let all = read_message(2, &manifest(), &files, None).unwrap();
        assert!(all.starts_with("# Design preview (version 2, direction: editorial)"), "{all}");
        assert!(all.contains("\n## index.html\n<h1>Hi</h1>\n") && all.ends_with("## styles.css\nh1 { margin: 0 }"), "{all}");
        assert_eq!(read_message(2, &manifest(), &files, Some("styles.css")).unwrap(), "## styles.css (version 2)\nh1 { margin: 0 }");
        assert_eq!(read_message(2, &manifest(), &files, Some("app.js")).unwrap_err(), "version 2 has no file \"app.js\": its files are index.html, styles.css");
    }

    #[test]
    fn the_read_tool_asks_for_nothing_and_takes_a_version_and_a_path() {
        let tool = ReadPreview.definition();
        assert_eq!(tool.name, READ_PREVIEW);
        assert_eq!(tool.input_schema["properties"].as_object().unwrap().keys().collect::<Vec<_>>(), ["path", "version"]);
        assert!(tool.input_schema.get("required").is_none());
    }

    #[test]
    fn a_version_that_dealt_with_feedback_says_which_and_what_is_left() {
        assert_eq!(resolved_message(3, &[2, 4], 1), " Feedback #2, #4 is marked as dealt with in version 3; 1 piece of feedback is still waiting.");
        assert_eq!(resolved_message(3, &[2], 0), " Feedback #2 is marked as dealt with in version 3; no feedback is waiting now.");
        assert_eq!(resolved_message(3, &[], 2), " 2 pieces of feedback are still waiting: name the ones a version deals with in resolves.");
        assert_eq!(resolved_message(3, &[], 0), "");
    }

    #[test]
    fn the_preview_tool_lets_a_version_name_the_feedback_it_dealt_with() {
        let schema = SavePreview { publisher: std::sync::Arc::new(nomi_agent_core::static_publish::NoStaticPublisher) }.definition().input_schema;
        assert_eq!(schema["properties"]["resolves"]["items"]["required"], json!(["feedback", "note"]));
        assert_eq!(schema["required"], json!(["direction_id", "title", "files"]), "resolves stays optional");
    }

    #[test]
    fn a_saved_handoff_tells_sora_how_to_pass_it_to_koda() {
        let project = Uuid::parse_str("0193309a-4f5b-7b00-8cb5-123456789abc").unwrap();
        let said = handoff_message(2, 3, Some(project));
        assert!(said.starts_with("Handoff saved as version 2, for approved preview version 3."), "{said}");
        assert!(said.contains("delegate_to_agent") && said.contains("\"Project 0193309a-4f5b-7b00-8cb5-123456789abc: "), "{said}");
        assert!(said.contains("Koda gets the whole package in his brief"));
        let nowhere = handoff_message(1, 3, None);
        assert!(nowhere.contains("There is no project in this chat yet") && !nowhere.contains("\"Project "), "{nowhere}");
    }

    #[test]
    fn a_handoff_waits_for_the_persons_approval() {
        assert!(NOT_APPROVED_YET.contains("approve") && NOT_APPROVED_YET.contains("Design Studio"));
        let tool = CreateHandoff.definition();
        assert_eq!(tool.name, CREATE_HANDOFF);
        assert_eq!(tool.input_schema, nomi_agent_core::design_handoff::handoff_input_schema());
        assert!(tool.description.contains("approved"));
    }

    #[test]
    fn a_review_tells_sora_what_to_do_about_what_it_found() {
        let project = Uuid::parse_str("0193309a-4f5b-7b00-8cb5-123456789abc").unwrap();
        let off = review_message("# Design check (version 1, against handoff version 1)", project, 2);
        assert!(off.starts_with("# Design check (version 1, against handoff version 1)\n\n"), "{off}");
        assert!(off.contains("delegate_to_agent") && off.contains("\"Project 0193309a-4f5b-7b00-8cb5-123456789abc: "), "{off}");
        let clean = review_message("# Design check", project, 0);
        assert!(clean.contains("nothing to fix") && !clean.contains("delegate_to_agent"), "{clean}");
    }

    #[test]
    fn the_review_tool_checks_the_chats_project_unless_told_another() {
        let tool = ReviewBuild { source: std::sync::Arc::new(nomi_agent_core::design_review::NoProjectSource) }.definition();
        assert_eq!(tool.name, REVIEW_BUILD);
        assert!(tool.input_schema["properties"]["project_id"].is_object() && tool.input_schema.get("required").is_none());
        assert!(tool.description.contains("colours, fonts and element ids"));
    }

    #[test]
    fn the_studio_link_is_inside_nomi_and_names_the_version() {
        let session_id = Uuid::parse_str("0193309a-4f5b-7b00-8cb5-123456789abc").unwrap();
        assert_eq!(studio_link(session_id, 4), "/chat/0193309a-4f5b-7b00-8cb5-123456789abc/design?version=4");
    }

    #[test]
    fn a_preview_is_of_a_direction_that_was_proposed() {
        let set = DesignDirections::parse(&two_directions()).unwrap();
        assert_eq!(known_direction(Some(&set), "swiss"), Ok(()));
        assert_eq!(known_direction(Some(&set), "brutalist").unwrap_err(), "there is no direction \"brutalist\": the directions are editorial, swiss");
        assert_eq!(known_direction(None, "swiss").unwrap_err(), "propose directions with propose_directions first, then build the one the person picks");
    }

    #[test]
    fn the_preview_tool_takes_the_artifact_schema() {
        let tool = SavePreview { publisher: std::sync::Arc::new(nomi_agent_core::static_publish::NoStaticPublisher) }.definition();
        assert_eq!(tool.name, SAVE_PREVIEW);
        assert_eq!(tool.input_schema, nomi_agent_core::design_artifact::artifact_input_schema());
        assert!(tool.description.contains("data-design-id"));
    }

    #[tokio::test]
    async fn listing_shows_what_is_connected_or_says_nothing_is() {
        assert_eq!(list_connected(&FakeMcp::default(), Uuid::nil()).await, NOTHING_CONNECTED);
        let refs = McpToolInfo { server: "Refs".into(), tool: "search_references".into(), description: None, input_schema: json!({"type": "object"}) };
        let listed = list_connected(&FakeMcp { listing: McpListing { tools: vec![refs], unavailable: vec![] }, ..Default::default() }, Uuid::nil()).await;
        assert!(listed.contains("server \"Refs\", tool \"search_references\""));
    }

    #[tokio::test]
    async fn using_a_tool_passes_the_server_tool_and_arguments_through() {
        let mcp = FakeMcp { answer: Some(Ok("Three references".into())), ..Default::default() };
        let answer = use_connected(&mcp, Uuid::nil(), &json!({"server": " Refs ", "tool": "search_references", "arguments": {"query": "bakery"}})).await;
        assert_eq!(answer.as_deref(), Ok("Three references"));
        assert_eq!(*mcp.called.lock().unwrap(), [("Refs".to_string(), "search_references".to_string(), json!({"query": "bakery"}))]);
    }

    #[tokio::test]
    async fn arguments_left_out_are_an_empty_object() {
        let mcp = FakeMcp::default();
        use_connected(&mcp, Uuid::nil(), &json!({"server": "Refs", "tool": "list_boards"})).await.unwrap();
        assert_eq!(mcp.called.lock().unwrap()[0].2, json!({}));
    }

    #[tokio::test]
    async fn a_call_without_its_server_or_tool_is_not_made() {
        let mcp = FakeMcp::default();
        assert_eq!(use_connected(&mcp, Uuid::nil(), &json!({"tool": "x"})).await.unwrap_err(), "server is required");
        assert_eq!(use_connected(&mcp, Uuid::nil(), &json!({"server": "Refs", "tool": " "})).await.unwrap_err(), "tool is required");
        assert_eq!(use_connected(&mcp, Uuid::nil(), &json!({"server": "Refs", "tool": "x", "arguments": "nope"})).await.unwrap_err(), "arguments must be an object");
        assert!(mcp.called.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_tool_that_fails_tells_sora_why_and_to_carry_on() {
        let mcp = FakeMcp { answer: Some(Err("the server couldn't be reached (no connection)".into())), ..Default::default() };
        let failed = use_connected(&mcp, Uuid::nil(), &json!({"server": "Refs", "tool": "search_references"})).await.unwrap_err();
        assert!(failed.starts_with("search_references on Refs didn't work: the server couldn't be reached (no connection)."), "{failed}");
        assert!(failed.ends_with(CARRY_ON));
    }
    #[test]
    fn an_approval_quotes_the_persons_own_words() {
        assert!(quoted_in("Looks great,  APPROVE it please", "approve it"));
        assert!(quoted_in("yes v3", "Yes v3"));
        assert!(!quoted_in("can you make the header bigger?", "approve"));
        assert!(!quoted_in("ok", ""), "an empty quote is no approval");
        assert!(!quoted_in("ok", "o"), "one letter is no approval");
    }
}
