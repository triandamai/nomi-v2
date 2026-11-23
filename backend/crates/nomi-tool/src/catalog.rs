use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres};

use nomi_llm::ToolDefinition;

use crate::content_block::ToolOutcome;
use crate::context::{Approval, AutoOffer, OfferContext, ToolContext};

/// One tool that lives outside the agent loop, implemented in the crate that owns it. The
/// built-in agents run their own tools through it, and the ones that can run for any agent are
/// collected into the `ToolCatalog` custom agents are granted tools from. Implementors ignore
/// whichever of `session_id`/`agent_session_id`/`input` they don't need.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Name, description and input schema, as the model sees them.
    fn definition(&self) -> ToolDefinition;

    /// The area that owns the tool, e.g. "money", "coding", "workspace".
    fn group(&self) -> &'static str;

    /// Extra words the tool is found by in `ToolCatalog::search`, beyond its name and description.
    fn keywords(&self) -> &'static [&'static str] {
        &[]
    }

    /// The definition to offer this turn, or `None` to leave the tool out of it.
    fn offer(&self, _ctx: &OfferContext<'_>) -> Option<ToolDefinition> {
        Some(self.definition())
    }

    /// Whether the tool can run right now (configured, has its key). Not ready: not offered.
    async fn ready(&self, _conn: &mut PgConnection) -> bool {
        true
    }

    /// Whether a call goes through the person's permission levels and the approval card.
    fn approval(&self) -> Approval {
        Approval::Checked
    }

    /// Which agents get the tool without a grant.
    fn auto_offer(&self) -> AutoOffer {
        AutoOffer::GrantOnly
    }

    /// Whether an agent that surfaces its activity posts a message for each call.
    fn posts_activity(&self) -> bool {
        true
    }

    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String>;
}

/// The definitions of `tools`, in order: a built-in agent's `SubAgent::tools()`.
pub fn tool_definitions(tools: &[Box<dyn Tool>]) -> Vec<ToolDefinition> {
    tools.iter().map(|tool| tool.definition()).collect()
}

/// The tool in `tools` named `name`, for a built-in agent's `SubAgent::execute_tool`.
pub fn find_tool<'a>(tools: &'a [Box<dyn Tool>], name: &str) -> Option<&'a dyn Tool> {
    tools.iter().find(|tool| tool.definition().name == name).map(|tool| tool.as_ref())
}

/// What the catalog says about one tool, for listing and searching it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub group: &'static str,
    pub keywords: Vec<&'static str>,
}

struct Entry {
    definition: ToolDefinition,
    tool: Box<dyn Tool>,
}

impl Entry {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: self.definition.name.clone(),
            description: self.definition.description.clone(),
            group: self.tool.group(),
            keywords: self.tool.keywords().to_vec(),
        }
    }
}

/// How well a tool matches a search, best first (see `ToolCatalog::search`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Match {
    ExactName,
    NamePrefix,
    NameContains,
    Keyword,
    Description,
}

fn match_of(info: &ToolInfo, query: &str) -> Option<Match> {
    let name = info.name.to_lowercase();
    if name == query {
        Some(Match::ExactName)
    } else if name.starts_with(query) {
        Some(Match::NamePrefix)
    } else if name.contains(query) {
        Some(Match::NameContains)
    } else if info.keywords.iter().any(|keyword| keyword.to_lowercase().contains(query)) {
        Some(Match::Keyword)
    } else if info.description.to_lowercase().contains(query) {
        Some(Match::Description)
    } else {
        None
    }
}

/// A fixed, Rust-reviewed catalog of tools a `DynamicAgent` can be granted — never arbitrary
/// code, never admin-configurable beyond picking from this set. Built once at startup (see
/// `nomi_server::build_tool_catalog`, which collects each agent crate's `catalog_tools`) and
/// shared behind an `Arc` everywhere it's needed.
pub struct ToolCatalog {
    entries: HashMap<String, Entry>,
}

impl ToolCatalog {
    /// No entries — for tests that need a `&ToolCatalog`/`Arc<ToolCatalog>` but never exercise
    /// dynamic-agent tool execution.
    pub fn empty() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Every tool in `tools`. Panics on two tools with the same name (see `register`).
    pub fn from_tools(tools: Vec<Box<dyn Tool>>) -> Self {
        let mut catalog = Self::empty();
        for tool in tools {
            catalog.register(tool);
        }
        catalog
    }

    /// Adds `tool` under its definition's name. Two tools with one name is a wiring mistake, so
    /// it stops the server at startup rather than letting one silently shadow the other.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let definition = tool.definition();
        if let Some(existing) = self.entries.get(&definition.name) {
            panic!(
                "tool catalog: \"{}\" is registered twice (groups \"{}\" and \"{}\"); tool names must be unique",
                definition.name,
                existing.tool.group(),
                tool.group()
            );
        }
        self.entries.insert(definition.name.clone(), Entry { definition, tool });
    }

    /// Every registered tool name, sorted — admin CRUD write-time validation rejects any
    /// `granted_tools` entry not in this list before it's ever saved.
    pub fn known_tool_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.entries.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    /// Every catalog tool's definition, by name.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut defs: Vec<ToolDefinition> = self.entries.values().map(|entry| entry.definition.clone()).collect();
        defs.sort_by(|a, b| a.name.cmp(&b.name));
        defs
    }

    /// The definitions of `granted`, in its order. Tolerates unknown names by skipping them
    /// (unlike write-time validation, which rejects them) — covers a tool being retired from the
    /// Rust catalog after an agent was already granted it: that agent should degrade gracefully,
    /// not break. See the design spec's "Write-time validation, runtime tolerance" note.
    pub fn definitions_for(&self, granted: &[String]) -> Vec<ToolDefinition> {
        granted.iter().filter_map(|name| self.entries.get(name)).map(|entry| entry.definition.clone()).collect()
    }

    /// Every tool's name, description, group and keywords, sorted by name.
    pub fn entries(&self) -> Vec<ToolInfo> {
        let mut infos: Vec<ToolInfo> = self.entries.values().map(Entry::info).collect();
        infos.sort_by(|a, b| a.name.cmp(&b.name));
        infos
    }

    /// Up to `limit` tools matching `query` (case-insensitive), best first: the exact name, then
    /// names starting with it, names containing it, a keyword containing it, and finally the
    /// description containing it; ties by name. An empty query lists every tool by name.
    pub fn search(&self, query: &str, limit: usize) -> Vec<ToolInfo> {
        let query = query.trim().to_lowercase();
        let mut ranked: Vec<(Match, ToolInfo)> = self
            .entries()
            .into_iter()
            .filter_map(|info| if query.is_empty() { Some((Match::ExactName, info)) } else { match_of(&info, &query).map(|m| (m, info)) })
            .collect();
        ranked.sort_by(|(a_match, a), (b_match, b)| a_match.cmp(b_match).then_with(|| a.name.cmp(&b.name)));
        ranked.into_iter().take(limit).map(|(_, info)| info).collect()
    }

    /// The tool named `name`.
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.entries.get(name).map(|entry| entry.tool.as_ref())
    }

    /// Every tool, sorted by name.
    pub fn tools(&self) -> impl Iterator<Item = &dyn Tool> {
        let mut entries: Vec<&Entry> = self.entries.values().collect();
        entries.sort_by(|a, b| a.definition.name.cmp(&b.definition.name));
        entries.into_iter().map(|entry| entry.tool.as_ref())
    }

    pub async fn execute(&self, name: &str, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        match self.entries.get(name) {
            Some(entry) => entry.tool.execute(conn, ctx, input).await,
            None => Err(format!("tool not granted: {name}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// More than every test catalog holds, so a search returns all its matches.
    const NO_LIMIT: usize = 100;
    /// A limit smaller than the test catalog.
    const SMALL_LIMIT: usize = 2;

    struct Fake {
        name: &'static str,
        description: &'static str,
        group: &'static str,
        keywords: &'static [&'static str],
    }

    #[async_trait]
    impl Tool for Fake {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition { name: self.name.to_string(), description: self.description.to_string(), input_schema: json!({"type": "object"}) }
        }
        fn group(&self) -> &'static str {
            self.group
        }
        fn keywords(&self) -> &'static [&'static str] {
            self.keywords
        }
        async fn execute(&self, _: &mut PoolConnection<Postgres>, _: &ToolContext<'_>, _: Value) -> Result<ToolOutcome, String> {
            Ok(ToolOutcome::text(self.name))
        }
    }

    fn fake(name: &'static str, description: &'static str, keywords: &'static [&'static str]) -> Box<dyn Tool> {
        Box::new(Fake { name, description, group: "test", keywords })
    }

    fn catalog() -> ToolCatalog {
        ToolCatalog::from_tools(vec![
            fake("list_budgets", "List the monthly budgets.", &["money", "spending"]),
            fake("budget", "One budget.", &[]),
            fake("set_budget", "Set a monthly budget.", &["limit"]),
            fake("read_file", "Read one file of the budget app.", &["code"]),
            fake("gmail_send", "Send an email.", &["mail", "budget report"]),
            fake("budgeting_tips", "Tips.", &[]),
        ])
    }

    fn names(infos: Vec<ToolInfo>) -> Vec<String> {
        infos.into_iter().map(|info| info.name).collect()
    }

    #[test]
    fn search_ranks_name_then_keyword_then_description() {
        assert_eq!(
            names(catalog().search("Budget", NO_LIMIT)),
            ["budget", "budgeting_tips", "list_budgets", "set_budget", "gmail_send", "read_file"]
        );
        assert_eq!(names(catalog().search("MAIL", NO_LIMIT)), ["gmail_send"]);
        assert_eq!(names(catalog().search("spending", NO_LIMIT)), ["list_budgets"]);
        assert_eq!(names(catalog().search("monthly", NO_LIMIT)), ["list_budgets", "set_budget"]);
        assert!(catalog().search("nothing like it", NO_LIMIT).is_empty());
    }

    #[test]
    fn search_with_an_empty_query_lists_everything_up_to_the_limit() {
        assert_eq!(names(catalog().search("  ", NO_LIMIT)), names(catalog().entries()));
        assert_eq!(names(catalog().search("", SMALL_LIMIT)), ["budget", "budgeting_tips"]);
        assert_eq!(catalog().search("budget", SMALL_LIMIT).len(), SMALL_LIMIT);
    }

    #[test]
    fn entries_are_sorted_with_their_group_and_keywords() {
        let entries = catalog().entries();
        assert_eq!(names(entries.clone()), ["budget", "budgeting_tips", "gmail_send", "list_budgets", "read_file", "set_budget"]);
        let list = entries.iter().find(|e| e.name == "list_budgets").unwrap();
        assert_eq!(list.group, "test");
        assert_eq!(list.keywords, ["money", "spending"]);
        assert_eq!(list.description, "List the monthly budgets.");
    }

    #[test]
    #[should_panic(expected = "\"set_budget\" is registered twice")]
    fn registering_a_name_twice_panics() {
        let mut catalog = catalog();
        catalog.register(fake("set_budget", "Another one.", &[]));
    }

    #[test]
    fn definitions_for_keeps_the_granted_order_and_skips_unknown_names() {
        let granted = ["set_budget".to_string(), "retired_tool".to_string(), "budget".to_string()];
        let defs = catalog().definitions_for(&granted);
        assert_eq!(defs.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(), ["set_budget", "budget"]);
        assert_eq!(catalog().known_tool_names(), ["budget", "budgeting_tips", "gmail_send", "list_budgets", "read_file", "set_budget"]);
    }

    #[test]
    fn find_tool_and_tool_definitions_follow_the_list() {
        let tools = vec![fake("b", "", &[]), fake("a", "", &[])];
        assert_eq!(tool_definitions(&tools).iter().map(|d| d.name.as_str()).collect::<Vec<_>>(), ["b", "a"]);
        assert!(find_tool(&tools, "a").is_some());
        assert!(find_tool(&tools, "c").is_none());
    }
}
