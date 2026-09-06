pub mod app;
pub mod bootstrap;
pub mod delegation_worker;
pub mod routes;
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
