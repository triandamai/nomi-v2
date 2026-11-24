pub mod configuration;
pub mod app;
pub mod ask_built;
pub mod attachment_worker;
pub mod background_worker;
pub mod billing;
pub mod billing_worker;
pub mod bootstrap;
pub mod chat_events;
pub mod delegation_worker;
pub mod design_project_source;
pub mod dns;
pub mod github;
pub mod import_review;
pub mod routes;
pub mod issue_worker;
pub mod memory_worker;
pub mod notifications;
pub mod presence;
pub mod project_events;
pub mod problem;
pub mod mcp_ops;
pub mod project_ops;
pub mod public_url;
pub mod project_relays;
pub mod relay_backend;
pub mod relay_connection;
pub mod relay_ops;
pub mod push;
pub mod quota;
pub mod reminders_worker;
pub mod sandbox_cleanup;
pub mod sandbox_devserver;
pub mod sandbox_manager;
pub mod sandbox_ops;
pub mod sandbox_runs;
pub mod sandbox_steps;
pub mod sandbox_store;
pub mod sandbox_sync;
pub mod sandbox_usage;
pub mod sandbox_worker;
pub mod scheduler_worker;
pub mod shipyard;
pub mod static_publisher;
pub mod sign_in_codes;
pub mod user_events;
pub mod worker;

/// The one place a new agent gets wired in. Adding an agent: implement `SubAgent` in its
/// own crate (see nomi-agent-money or nomi-agent-chitchat for the shape), add it as a
/// dependency of this crate's Cargo.toml, and add one line here.
/// Every built-in agent. Rena and Koda publish projects, and Iris inspects them, through
/// `project_ops::ServerProjectOps`; Rena makes Relay backends, and Koda works on them, through
/// `relay_ops::ServerRelayOps`.
/// The registry shares the tool catalog (built once by `build_tool_catalog`, in `AppState.tool_catalog`), for `find_tools` and `create_agent`.
pub fn build_agent_registry(pool: &sqlx::PgPool, project_storage: nomi_storage::ProjectStore, catalog: std::sync::Arc<nomi_agent_core::ToolCatalog>) -> nomi_agent_core::AgentRegistry {
    let project_ops = project_ops::ServerProjectOps::shared(pool.clone(), project_storage.clone());
    let relay_ops = relay_ops::ServerRelayOps::shared(pool.clone(), project_storage.clone());
    let project_source = design_project_source::StoreProjectSource::shared(pool.clone(), project_storage.clone());
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
        Box::new(nomi_agent_coding::CodingAgent::new(project_storage).with_project_ops(project_ops.clone()).with_relay_ops(relay_ops).with_sandbox_ops(sandbox_ops::ServerSandboxOps::shared(pool.clone()))),
        Box::new(nomi_agent_inspector::InspectorAgent::new().with_project_ops(project_ops)),
        // Sora: off for everyone until Admin → Configuration → Design rolls her out.
        Box::new(
            nomi_agent_design::DesignAgent::new()
                .with_mcp_ops(mcp_ops::ServerMcpOps::shared(pool.clone()))
                .with_static_publisher(static_publisher::ShipyardStaticPublisher::shared(pool.clone()))
                .with_project_source(project_source),
        ),
    ])
    .with_tool_catalog(catalog)
}

/// Writes the tool catalog into `tool_catalog_entries` (a failure is logged, never fatal), then
/// embeds the rows that need it in the background, so startup never waits on the embedding
/// provider.
pub async fn sync_tool_catalog(pool: &sqlx::PgPool, catalog: &nomi_agent_core::ToolCatalog, settings_key: [u8; 32], http_client: reqwest::Client) {
    let synced = match pool.acquire().await {
        Ok(mut conn) => nomi_agent_core::tool_finder::sync_catalog_table(&mut conn, catalog).await,
        Err(e) => Err(e),
    };
    if let Err(e) = synced {
        tracing::warn!(error = %e, "tool catalog: couldn't sync the catalog table; find_tools matches by words alone");
        return;
    }
    let pool = pool.clone();
    tokio::spawn(async move {
        let provider = bootstrap::build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client, None).await;
        let embedded = nomi_agent_core::tool_finder::embed_catalog_entries(&pool, provider.as_ref()).await;
        tracing::info!(embedded, "tool catalog: embeddings up to date");
    });
}

/// The one place a crate's grantable tools get wired into `ToolCatalog`: implement
/// `nomi_agent_core::Tool` for the tool in its own agent crate, list it in that crate's
/// `catalog_tools`, and (for a new crate) add its `catalog_tools` here. Two tools with one name
/// stop the server at startup.
pub fn build_tool_catalog(project_storage: nomi_storage::ProjectStore) -> std::sync::Arc<nomi_agent_core::ToolCatalog> {
    let tools = [
        nomi_agent_money::catalog_tools(),
        nomi_agent_personality::catalog_tools(),
        nomi_agent_planning::catalog_tools(),
        nomi_agent_supervisor::catalog_tools(),
        nomi_agent_reminders::catalog_tools(),
        nomi_agent_coding::catalog_tools(project_storage),
        nomi_agent_health::catalog_tools(&nomi_agent_health::HealthAgent::from_env()),
        nomi_agent_workspace::catalog_tools(&nomi_agent_workspace::WorkspaceAgent::from_env()),
        nomi_agent_core::engine::engine_catalog_tools(),
    ];
    std::sync::Arc::new(nomi_agent_core::ToolCatalog::from_tools(tools.into_iter().flatten().collect()))
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
