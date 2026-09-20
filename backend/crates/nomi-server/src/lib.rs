pub mod app;
pub mod bootstrap;
pub mod delegation_worker;
pub mod routes;
pub mod scheduler_worker;
mod tool_catalog_adapters;
pub mod web_identity;
pub mod worker;

/// The one place a new agent gets wired in. Adding an agent: implement `SubAgent` in its
/// own crate (see nomi-agent-money or nomi-agent-chitchat for the shape), add it as a
/// dependency of this crate's Cargo.toml, and add one line here.
pub fn build_agent_registry(project_storage: nomi_storage::LocalFsStore) -> nomi_agent_core::AgentRegistry {
    nomi_agent_core::AgentRegistry::new(vec![
        Box::new(nomi_agent_chitchat::ChitchatAgent),
        Box::new(nomi_agent_money::MoneyAgent),
        Box::new(nomi_agent_personality::PersonalityAgent),
        Box::new(nomi_agent_supervisor::SupervisorAgent),
        Box::new(nomi_agent_planning::PlanningAgent::new()),
        Box::new(nomi_agent_coding::CodingAgent::new(project_storage)),
    ])
}

/// The one place a new grantable tool gets wired into `ToolCatalog`: implement (or reuse) a
/// free function in the owning agent crate, add its `CatalogTool` adapter to
/// `tool_catalog_adapters`, and add one entry here.
pub fn build_tool_catalog(project_storage: nomi_storage::LocalFsStore) -> std::sync::Arc<nomi_agent_core::ToolCatalog> {
    let mut entries = tool_catalog_adapters::non_coding_entries();
    entries.extend(tool_catalog_adapters::coding_entries(project_storage));
    std::sync::Arc::new(nomi_agent_core::ToolCatalog::new(entries))
}
