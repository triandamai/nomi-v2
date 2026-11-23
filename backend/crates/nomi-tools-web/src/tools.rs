use async_trait::async_trait;
use nomi_llm::ToolDefinition;
use nomi_tool::{Approval, AutoOffer, Tool, ToolContext, ToolOutcome};
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres};

/// The catalog area these tools are listed under.
pub const GROUP: &str = "web";

/// `web_search` (offered only once a search provider is set up) and `read_web_page`. Both run
/// straight away and post nothing to the chat.
pub fn tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(WebSearch), Box::new(ReadWebPage)]
}

async fn run(conn: &mut PgConnection, name: &str, input: &Value) -> Result<ToolOutcome, String> {
    match crate::execute(conn, name, input).await {
        Some(result) => result.map(ToolOutcome::text),
        None => Err(format!("unknown tool: {name}")),
    }
}

pub struct WebSearch;

#[async_trait]
impl Tool for WebSearch {
    fn definition(&self) -> ToolDefinition {
        crate::web_search_tool_definition()
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["internet", "search", "lookup", "news", "google"]
    }
    fn group(&self) -> &'static str {
        GROUP
    }
    async fn ready(&self, conn: &mut PgConnection) -> bool {
        crate::search_setup(conn).await.is_some()
    }
    fn approval(&self) -> Approval {
        Approval::Never
    }
    fn auto_offer(&self) -> AutoOffer {
        AutoOffer::Everyone
    }
    fn posts_activity(&self) -> bool {
        false
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        run(conn, crate::WEB_SEARCH_TOOL_NAME, &input).await
    }
}

pub struct ReadWebPage;

#[async_trait]
impl Tool for ReadWebPage {
    fn definition(&self) -> ToolDefinition {
        crate::read_web_page_tool_definition()
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["url", "website", "link", "browse", "fetch"]
    }
    fn group(&self) -> &'static str {
        GROUP
    }
    fn approval(&self) -> Approval {
        Approval::Never
    }
    fn auto_offer(&self) -> AutoOffer {
        AutoOffer::Everyone
    }
    fn posts_activity(&self) -> bool {
        false
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, _ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        run(conn, crate::READ_WEB_PAGE_TOOL_NAME, &input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nomi_tool::{Approval, AutoOffer};

    #[test]
    fn web_tools_properties() {
        let tools = tools();
        let names: Vec<String> = tools.iter().map(|t| t.definition().name).collect();
        assert_eq!(names, [crate::WEB_SEARCH_TOOL_NAME, crate::READ_WEB_PAGE_TOOL_NAME]);
        for t in &tools {
            assert_eq!(t.approval(), Approval::Never);
            assert_eq!(t.auto_offer(), AutoOffer::Everyone);
            assert!(!t.posts_activity());
        }
    }
}
