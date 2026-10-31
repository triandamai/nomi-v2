pub mod configuration;
pub mod app;
pub mod attachment_worker;
pub mod background_worker;
pub mod bootstrap;
pub mod chat_events;
pub mod delegation_worker;
pub mod github;
pub mod routes;
pub mod memory_worker;
pub mod notifications;
pub mod project_events;
pub mod problem;
pub mod project_ops;
pub mod public_url;
pub mod project_relays;
pub mod relay_backend;
pub mod relay_connection;
pub mod relay_ops;
pub mod push;
pub mod quota;
pub mod reminders_worker;
pub mod scheduler_worker;
pub mod shipyard;
pub mod sign_in_codes;
pub mod user_events;
mod tool_catalog_adapters;
pub mod worker;

/// The one place a new agent gets wired in. Adding an agent: implement `SubAgent` in its
/// own crate (see nomi-agent-money or nomi-agent-chitchat for the shape), add it as a
/// dependency of this crate's Cargo.toml, and add one line here.
/// Every built-in agent. Rena and Koda publish projects, and Iris inspects them, through
/// `project_ops::ServerProjectOps`; Rena makes Relay backends, and Koda works on them, through
/// `relay_ops::ServerRelayOps`.
pub fn build_agent_registry(pool: &sqlx::PgPool, project_storage: nomi_storage::ProjectStore) -> nomi_agent_core::AgentRegistry {
    let project_ops = project_ops::ServerProjectOps::shared(pool.clone(), project_storage.clone());
    let relay_ops = relay_ops::ServerRelayOps::shared(pool.clone(), project_storage.clone());
    nomi_agent_core::AgentRegistry::new(vec![
        Box::new(nomi_agent_chitchat::ChitchatAgent),
        Box::new(nomi_agent_money::MoneyAgent),
        Box::new(nomi_agent_reminders::RemindersAgent),
        Box::new(nomi_agent_files::FilesAgent),
        Box::new(nomi_agent_workspace::WorkspaceAgent::from_env()),
        Box::new(nomi_agent_health::HealthAgent::from_env()),
        Box::new(nomi_agent_personality::PersonalityAgent),
        Box::new(nomi_agent_supervisor::SupervisorAgent),
        Box::new(nomi_agent_planning::PlanningAgent::new().with_project_ops(project_ops.clone()).with_relay_ops(relay_ops.clone())),
        Box::new(nomi_agent_coding::CodingAgent::new(project_storage).with_project_ops(project_ops.clone()).with_relay_ops(relay_ops)),
        Box::new(nomi_agent_inspector::InspectorAgent::new().with_project_ops(project_ops)),
    ])
}

/// The one place a new grantable tool gets wired into `ToolCatalog`: implement (or reuse) a
/// free function in the owning agent crate, add its `CatalogTool` adapter to
/// `tool_catalog_adapters`, and add one entry here.
pub fn build_tool_catalog(project_storage: nomi_storage::ProjectStore) -> std::sync::Arc<nomi_agent_core::ToolCatalog> {
    let mut entries = tool_catalog_adapters::non_coding_entries();
    entries.extend(tool_catalog_adapters::coding_entries(project_storage));
    std::sync::Arc::new(nomi_agent_core::ToolCatalog::new(entries))
}

/// The HTTP client for model, embedding and other provider calls. No overall timeout (a long
/// answer streams for minutes), but a provider that never connects, or stops sending
/// mid-stream, fails the call instead of hanging the turn.
pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(20))
        .read_timeout(std::time::Duration::from_secs(180))
        .build()
        .expect("failed to build the HTTP client")
}
