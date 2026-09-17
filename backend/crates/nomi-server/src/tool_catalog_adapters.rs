use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{CatalogTool, ToolOutcome};
use nomi_llm::ToolDefinition;
use nomi_storage::LocalFsStore;

struct ListTransactions;
#[async_trait]
impl CatalogTool for ListTransactions {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_money::list_transactions(conn, user_id, input).await.map(ToolOutcome::text)
    }
}

struct SummarizeBudget;
#[async_trait]
impl CatalogTool for SummarizeBudget {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_money::summarize_budget(conn, user_id, input).await.map(ToolOutcome::text)
    }
}

struct CreateProject;
#[async_trait]
impl CatalogTool for CreateProject {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_planning::create_project(conn, session_id, user_id, input).await.map(ToolOutcome::text)
    }
}

struct SetPersonality;
#[async_trait]
impl CatalogTool for SetPersonality {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_personality::set_personality(conn, session_id, agent_session_id, user_id, input).await.map(ToolOutcome::text)
    }
}

struct ListPersonalityVersions;
#[async_trait]
impl CatalogTool for ListPersonalityVersions {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, _input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_personality::list_personality_versions(conn, user_id).await.map(ToolOutcome::text)
    }
}

struct RollbackPersonality;
#[async_trait]
impl CatalogTool for RollbackPersonality {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_personality::rollback_personality(conn, session_id, agent_session_id, user_id, input).await.map(ToolOutcome::text)
    }
}

struct ListRecentAgentActivity;
#[async_trait]
impl CatalogTool for ListRecentAgentActivity {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, _agent_session_id: Uuid, _user_id: Uuid, _input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_supervisor::list_recent_agent_activity(conn, session_id).await.map(ToolOutcome::text)
    }
}

/// Every non-coding entry for `build_tool_catalog` — coding's four entries are added by that
/// function directly (Task 4), since they need a `LocalFsStore` this function doesn't have.
pub fn non_coding_entries() -> HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)> {
    let mut entries: HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)> = HashMap::new();
    entries.insert("list_transactions", (nomi_agent_money::list_transactions_tool_definition(), Box::new(ListTransactions)));
    entries.insert("summarize_budget", (nomi_agent_money::summarize_budget_tool_definition(), Box::new(SummarizeBudget)));
    entries.insert("create_project", (nomi_agent_planning::create_project_tool_definition(), Box::new(CreateProject)));
    entries.insert("set_personality", (nomi_agent_personality::set_personality_tool_definition(), Box::new(SetPersonality)));
    entries.insert(
        "list_personality_versions",
        (nomi_agent_personality::list_personality_versions_tool_definition(), Box::new(ListPersonalityVersions)),
    );
    entries.insert(
        "rollback_personality",
        (nomi_agent_personality::rollback_personality_tool_definition(), Box::new(RollbackPersonality)),
    );
    entries.insert(
        "list_recent_agent_activity",
        (nomi_agent_supervisor::list_recent_agent_activity_tool_definition(), Box::new(ListRecentAgentActivity)),
    );
    entries
}

struct WriteFile {
    storage: LocalFsStore,
}
#[async_trait]
impl CatalogTool for WriteFile {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_coding::write_file(conn, &self.storage, user_id, input).await
    }
}

struct ReadFile {
    storage: LocalFsStore,
}
#[async_trait]
impl CatalogTool for ReadFile {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_coding::read_file(conn, &self.storage, user_id, input).await.map(ToolOutcome::text)
    }
}

struct ListFiles;
#[async_trait]
impl CatalogTool for ListFiles {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_coding::list_files(conn, user_id, input).await.map(ToolOutcome::text)
    }
}

struct DeleteFile {
    storage: LocalFsStore,
}
#[async_trait]
impl CatalogTool for DeleteFile {
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        nomi_agent_coding::delete_file(conn, &self.storage, user_id, input).await
    }
}

/// Coding's four entries, needing `project_storage` — combined with `non_coding_entries()` by
/// `build_tool_catalog` in `lib.rs`.
pub fn coding_entries(project_storage: LocalFsStore) -> HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)> {
    let mut entries: HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)> = HashMap::new();
    entries.insert(
        "write_file",
        (nomi_agent_coding::write_file_tool_definition(), Box::new(WriteFile { storage: project_storage.clone() })),
    );
    entries.insert(
        "read_file",
        (nomi_agent_coding::read_file_tool_definition(), Box::new(ReadFile { storage: project_storage.clone() })),
    );
    entries.insert("list_files", (nomi_agent_coding::list_files_tool_definition(), Box::new(ListFiles)));
    entries.insert(
        "delete_file",
        (nomi_agent_coding::delete_file_tool_definition(), Box::new(DeleteFile { storage: project_storage })),
    );
    entries
}
