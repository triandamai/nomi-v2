use async_trait::async_trait;
use nomi_llm::ToolDefinition;
use nomi_tool::{Approval, AutoOffer, Tool, ToolContext, ToolOutcome};
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;

/// The record tools, in the order agents are offered them. They run straight away (no approval
/// card) and store under the calling agent's type, so one agent never sees another's records.
pub fn tools() -> Vec<Box<dyn Tool>> {
    crate::record_tool_definitions().into_iter().map(|definition| Box::new(RecordTool { definition }) as Box<dyn Tool>).collect()
}

pub struct RecordTool {
    definition: ToolDefinition,
}

#[async_trait]
impl Tool for RecordTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }
    fn group(&self) -> &'static str {
        crate::GROUP
    }
    fn approval(&self) -> Approval {
        Approval::Never
    }
    fn auto_offer(&self) -> AutoOffer {
        AutoOffer::IfUsesRecords
    }
    fn posts_activity(&self) -> bool {
        false
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let name = self.definition.name.as_str();
        match crate::execute(conn, ctx.agent_type, ctx.user_id, name, &input).await {
            Some(result) => result.map(ToolOutcome::text),
            None => Err(format!("unknown tool: {name}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nomi_tool::{Approval, AutoOffer};

    #[test]
    fn record_tools_properties() {
        let tools = tools();
        let names: Vec<String> = tools.iter().map(|t| t.definition().name).collect();
        assert_eq!(names, crate::RECORD_TOOL_NAMES);
        for t in &tools {
            assert_eq!(t.approval(), Approval::Never);
            assert_eq!(t.auto_offer(), AutoOffer::IfUsesRecords);
            assert!(!t.posts_activity());
            assert_eq!(t.group(), crate::GROUP);
        }
    }
}
