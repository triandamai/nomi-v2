//! Koda, the coding agent: builds projects with Nomi's stack (Vite, TypeScript, Tailwind;
//! SvelteKit first) from his guides, writes the code into the project's storage, and runs
//! commands in the project's live preview in the user's browser.

pub mod brief;
pub mod files;
pub mod guides;
pub mod relay;
pub mod report;
pub mod runtime;

use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::project_ops::{publish_from_tool, publish_tool_definition, NoProjectOps, SharedProjectOps, PUBLISH_TOOL_NAME};
use nomi_agent_core::relay_ops::{NoRelayOps, SharedRelayOps, RELAY_TOOL_NAMES};
use nomi_agent_core::prompts::CODING_SYSTEM_PROMPT;
use nomi_agent_core::{SubAgent, ToolOutcome};
use nomi_llm::ToolDefinition;
use nomi_storage::ProjectStore;

pub use files::{
    delete_file, delete_file_tool_definition, edit_file, edit_file_tool_definition, guess_content_type, list_files,
    list_files_tool_definition, project_file_key, read_file, read_file_tool_definition, search_files, search_files_tool_definition,
    validate_path, write_file, write_file_tool_definition, write_files, write_files_tool_definition,
};

pub const CODING_AGENT_TYPE: &str = "coding";

/// Every tool Koda has, by name (also used by the tool catalog for custom agents).
pub const TOOL_NAMES: &[&str] =
    &["read_guide", "list_files", "read_file", "search_files", "write_files", "write_file", "edit_file", "delete_file", "run_command"];

pub fn tool_definition(name: &str) -> Option<ToolDefinition> {
    Some(match name {
        "read_guide" => guides::read_guide_tool_definition(),
        "list_files" => list_files_tool_definition(),
        "read_file" => read_file_tool_definition(),
        "search_files" => search_files_tool_definition(),
        "write_files" => write_files_tool_definition(),
        "write_file" => write_file_tool_definition(),
        "edit_file" => edit_file_tool_definition(),
        "delete_file" => delete_file_tool_definition(),
        "run_command" => runtime::run_command_tool_definition(),
        _ => return None,
    })
}

/// Runs one of Koda's tools.
pub async fn execute(conn: &mut PoolConnection<Postgres>, storage: &ProjectStore, user_id: Uuid, name: &str, input: Value) -> Result<ToolOutcome, String> {
    match name {
        "read_guide" => guides::read_guide(&input).map(ToolOutcome::text),
        "list_files" => list_files(conn, user_id, input).await.map(ToolOutcome::text),
        "read_file" => read_file(conn, storage, user_id, input).await.map(ToolOutcome::text),
        "search_files" => search_files(conn, storage, user_id, input).await.map(ToolOutcome::text),
        "write_files" => write_files(conn, storage, user_id, input).await,
        "write_file" => write_file(conn, storage, user_id, input).await,
        "edit_file" => edit_file(conn, storage, user_id, input).await,
        "delete_file" => delete_file(conn, storage, user_id, input).await,
        "run_command" => runtime::run_command(conn, user_id, input).await.map(ToolOutcome::text),
        other => Err(format!("unknown tool: {other}")),
    }
}

/// Checks whether `task` starts with the "Project <uuid>: ..." prefix CODING_SYSTEM_PROMPT
/// requires — used by validate_delegation_task to reject a delegation before it's created,
/// rather than let this agent hallucinate a project_id when the planning agent skipped
/// create_project/write_plan and delegated without ever making a project (a real failure mode:
/// the model doesn't always follow the create-then-delegate sequence its own prompt asks for).
fn has_project_prefix(task: &str) -> bool {
    task.strip_prefix("Project ")
        .and_then(|rest| rest.split_once(':'))
        .map(|(id, _)| id.trim().parse::<Uuid>().is_ok())
        .unwrap_or(false)
}

pub struct CodingAgent {
    storage: ProjectStore,
    project_ops: SharedProjectOps,
    relay_ops: SharedRelayOps,
}

impl CodingAgent {
    pub fn new(storage: ProjectStore) -> Self {
        Self { storage, project_ops: std::sync::Arc::new(NoProjectOps), relay_ops: std::sync::Arc::new(NoRelayOps) }
    }

    /// Migrations and the backend's schema for projects on Relay (see `relay`).
    pub fn with_relay_ops(mut self, relay_ops: SharedRelayOps) -> Self {
        self.relay_ops = relay_ops;
        self
    }

    /// Publishing (`publish_project`) through the server's Shipyard access.
    pub fn with_project_ops(mut self, project_ops: SharedProjectOps) -> Self {
        self.project_ops = project_ops;
        self
    }
}

#[async_trait]
impl SubAgent for CodingAgent {
    /// Remembers durable facts the user mentions here and recalls them in later turns.
    fn uses_memory(&self) -> bool {
        true
    }

    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(CODING_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(CODING_SYSTEM_PROMPT)
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            guides::read_guide_tool_definition(),
            list_files_tool_definition(),
            read_file_tool_definition(),
            search_files_tool_definition(),
            write_files_tool_definition(),
            write_file_tool_definition(),
            edit_file_tool_definition(),
            delete_file_tool_definition(),
            runtime::run_command_tool_definition(),
            publish_tool_definition(),
        ]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        if name == PUBLISH_TOOL_NAME {
            return publish_from_tool(self.project_ops.as_ref(), user_id, &self.display_name(), &input).await.map(ToolOutcome::text);
        }
        if RELAY_TOOL_NAMES.contains(&name) {
            return relay::execute(conn, &self.storage, self.relay_ops.as_ref(), user_id, name, &input).await.map(ToolOutcome::text);
        }
        execute(conn, &self.storage, user_id, name, input).await
    }

    /// The Relay tools, for a project whose backend is Relay.
    async fn chat_tools(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        user_id: Uuid,
        messages: &[nomi_llm::LlmMessage],
    ) -> Vec<ToolDefinition> {
        relay::chat_tools(conn, session_id, user_id, messages).await
    }

    /// The project, its files, Rena's plan and the stack's guides, so a build starts from them
    /// (see `brief`).
    async fn domain_context(
        &self,
        conn: &mut PoolConnection<Postgres>,
        s3: Option<&nomi_storage::S3Config>,
        session_id: Uuid,
        user_id: Uuid,
        messages: &[nomi_llm::LlmMessage],
    ) -> Option<String> {
        brief::task_brief(conn, s3, session_id, user_id, messages).await
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(CODING_AGENT_TYPE)
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("writing or editing code for a project — not reachable directly, only via delegation from planning")
    }

    /// Koda's tool results (command output, file listings, guides) aren't posted as messages:
    /// they're steps in his reply's activity, with command output there to expand. File changes
    /// still show as their own cards (rich blocks always post).
    fn surfaces_activity(&self) -> bool {
        false
    }

    fn reads_plans(&self) -> bool {
        true
    }

    fn supports_todos(&self) -> bool {
        true
    }

    /// Koda's tools only touch the project's own files, its own dev backend on Relay, and run
    /// commands in the person's own browser preview: building shouldn't stop for an approval at
    /// every file.
    fn tool_needs_approval(&self, tool_name: &str) -> bool {
        !TOOL_NAMES.contains(&tool_name) && !RELAY_TOOL_NAMES.contains(&tool_name)
    }

    fn supports_plans(&self) -> bool {
        true
    }

    /// Building a project takes a while: it runs in the background and reports back.
    fn works_in_background(&self) -> bool {
        true
    }

    /// Setting up, installing, checking and fixing a project takes many rounds of tools.
    fn max_tool_turns(&self) -> u32 {
        60
    }

    /// The same failing command (or tool error) three times means the fixes aren't working:
    /// stop and say so, rather than spend the rest of the 60 rounds on it.
    fn repeated_failure_limit(&self) -> Option<u32> {
        Some(3)
    }

    /// run_command reports a command that ran and failed as output ("exit code 1"), not an error.
    fn tool_output_is_failure(&self, tool_name: &str, output: &str) -> bool {
        tool_name == "run_command"
            && output.lines().nth(1).is_some_and(|line| {
                line.strip_prefix("exit code ").is_some_and(|code| code.trim() != "0") || line.starts_with("stopped waiting after")
            })
    }

    fn validate_delegation_task(&self, task: &str) -> Result<(), String> {
        if has_project_prefix(task) {
            Ok(())
        } else {
            Err(
                "This task has no 'Project <uuid>: ...' prefix, so there's no project to write \
                 files into. Call create_project (and write_plan) first, then delegate again \
                 with a task formatted exactly as 'Project <the real project id>: <summary>'."
                    .to_string(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_that_exits_non_zero_or_never_finishes_is_a_failure() {
        let agent = CodingAgent::new(nomi_storage::ProjectStore::at(std::env::temp_dir()));
        assert_eq!(agent.repeated_failure_limit(), Some(3));
        assert!(agent.tool_output_is_failure("run_command", "$ npm run check\nexit code 1\nType error"));
        assert!(agent.tool_output_is_failure("run_command", "$ npm run build\nexit code ?\n"));
        assert!(agent.tool_output_is_failure("run_command", "$ npm run build\nstopped waiting after 180s with no result."));
        assert!(!agent.tool_output_is_failure("run_command", "$ npm run check\nexit code 0\n0 errors"));
        assert!(!agent.tool_output_is_failure("run_command", runtime::NOT_OPEN));
        assert!(!agent.tool_output_is_failure("read_file", "exit code 1"));
    }

    #[test]
    fn every_tool_koda_has_is_named_and_defined() {
        let agent = CodingAgent::new(nomi_storage::ProjectStore::at(std::env::temp_dir()));
        let names: Vec<String> = agent.tools().into_iter().map(|t| t.name).collect();
        // Publishing is his too, but stays out of TOOL_NAMES: it asks first, and isn't a tool
        // custom agents can be given.
        let mut expected: Vec<&str> = TOOL_NAMES.to_vec();
        expected.push(PUBLISH_TOOL_NAME);
        assert_eq!(names, expected);
        assert!(TOOL_NAMES.iter().all(|n| tool_definition(n).is_some()));
        assert!(agent.tool_needs_approval(PUBLISH_TOOL_NAME));
    }

    #[test]
    fn rejects_a_delegation_task_with_no_project_prefix() {
        let agent = CodingAgent::new(nomi_storage::ProjectStore::at(std::env::temp_dir()));
        let result = agent.validate_delegation_task("Build an advanced calculator app with trig functions.");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_a_delegation_task_with_a_malformed_project_id() {
        let agent = CodingAgent::new(nomi_storage::ProjectStore::at(std::env::temp_dir()));
        assert!(agent.validate_delegation_task("Project not-a-real-uuid: build it").is_err());
    }

    #[test]
    fn accepts_a_delegation_task_with_a_real_project_prefix() {
        let agent = CodingAgent::new(nomi_storage::ProjectStore::at(std::env::temp_dir()));
        let task = format!("Project {}: build a calculator", Uuid::new_v4());
        assert!(agent.validate_delegation_task(&task).is_ok());
    }
}
