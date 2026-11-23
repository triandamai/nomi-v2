//! Pins which tools each built-in agent is offered, which of its own tools ask for approval,
//! which tools the engine gates and which post to the chat, and every tool's description and
//! schema, so moving tools between crates can't change any of it unnoticed.
//! `UPDATE_SNAPSHOTS=1 cargo test -p nomi-server --test tool_offer_snapshot` re-records it.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use nomi_agent_core::engine::{engine_tools_for, is_gateable, is_quiet, OfferInputs};
use nomi_agent_core::tools::ToolSwitches;
use serde_json::{json, Value};

const SNAPSHOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots/tool_offer.json");
/// Never connected to: the registry only needs a pool to hand to its project helpers.
const UNUSED_DATABASE_URL: &str = "postgres://localhost/unused";

fn scenarios() -> Vec<(&'static str, OfferInputs)> {
    let ready = |switches: ToolSwitches, not_ready: &[&str], files: bool| OfferInputs {
        off: HashSet::new(),
        switches,
        not_ready: not_ready.iter().map(|n| n.to_string()).collect(),
        mentions_files: files,
    };
    vec![
        ("all_ready", ready(ToolSwitches::default(), &[], true)),
        ("nothing_ready", ready(ToolSwitches::default(), &["web_search", "speak"], false)),
        ("web_search_off", ready(ToolSwitches::with_off(["web_search"]), &[], true)),
        ("voice_off", ready(ToolSwitches::with_off(["speak"]), &[], true)),
    ]
}

fn names(tools: &[nomi_llm::ToolDefinition]) -> Vec<String> {
    tools.iter().map(|t| t.name.clone()).collect()
}

fn remember(definitions: &mut BTreeMap<String, Value>, tools: &[nomi_llm::ToolDefinition]) {
    for t in tools {
        definitions.insert(t.name.clone(), json!({ "description": t.description, "input_schema": t.input_schema }));
    }
}

#[tokio::test]
async fn offered_tools_match_snapshot() {
    let pool = sqlx::postgres::PgPoolOptions::new().connect_lazy(UNUSED_DATABASE_URL).unwrap();
    let storage = nomi_test_support::test_project_storage();
    let catalog = nomi_server::build_tool_catalog(storage.clone());
    let registry = nomi_server::build_agent_registry(&pool, storage, catalog.clone());

    let mut agents = BTreeMap::new();
    let mut definitions = BTreeMap::new();
    remember(&mut definitions, &catalog.definitions());
    let mut every_name: BTreeSet<String> = catalog.known_tool_names().into_iter().map(str::to_string).collect();
    for agent in registry.agents() {
        let own_tools = agent.tools();
        remember(&mut definitions, &own_tools);
        let own = names(&own_tools);
        every_name.extend(own.iter().cloned());
        let needs_approval: BTreeMap<&str, bool> =
            own.iter().map(|n| (n.as_str(), agent.tool_needs_approval(n))).collect();
        let mut offers = BTreeMap::new();
        for (scenario, inputs) in scenarios() {
            let engine_tools = engine_tools_for(agent.as_ref(), &registry, &inputs);
            remember(&mut definitions, &engine_tools);
            let engine = names(&engine_tools);
            every_name.extend(engine.iter().cloned());
            offers.insert(scenario, engine);
        }
        agents.insert(agent.agent_type().to_string(), json!({ "own": own, "own_needs_approval": needs_approval, "engine": offers }));
    }
    let crew_tools = nomi_agent_core::crew_tool_definitions(&registry);
    remember(&mut definitions, &crew_tools);
    every_name.extend(names(&crew_tools));

    let tools: BTreeMap<String, Value> =
        every_name.iter().map(|n| (n.clone(), json!({ "gateable": is_gateable(Some(catalog.as_ref()), n), "quiet": is_quiet(Some(catalog.as_ref()), n) }))).collect();
    let catalog_groups: BTreeMap<String, &str> = catalog.entries().into_iter().map(|e| (e.name, e.group)).collect();
    let actual = json!({ "agents": agents, "tools": tools, "catalog": catalog_groups, "definitions": definitions });
    let actual_text = serde_json::to_string_pretty(&actual).unwrap() + "\n";

    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        std::fs::write(SNAPSHOT, &actual_text).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(SNAPSHOT).expect("no snapshot yet: run with UPDATE_SNAPSHOTS=1");
    assert_eq!(expected, actual_text, "offered tools changed; diff tests/snapshots/tool_offer.json");
}
