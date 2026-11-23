use async_trait::async_trait;
use nomi_llm::ToolDefinition;
use nomi_tool::{Approval, AutoOffer, OfferContext, Tool, ToolContext, ToolOutcome};
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;

/// The catalog area this tool is listed under.
pub const GROUP: &str = "attachments";

/// `read_attachment`, offered only in a conversation that holds uploaded files.
pub fn tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(ReadAttachment)]
}

pub struct ReadAttachment;

#[async_trait]
impl Tool for ReadAttachment {
    fn definition(&self) -> ToolDefinition {
        crate::read_attachment_tool_definition()
    }
    fn group(&self) -> &'static str {
        GROUP
    }
    fn offer(&self, ctx: &OfferContext<'_>) -> Option<ToolDefinition> {
        ctx.mentions_files.then(|| self.definition())
    }
    fn approval(&self) -> Approval {
        Approval::Never
    }
    fn auto_offer(&self) -> AutoOffer {
        AutoOffer::Everyone
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        crate::read_attachment(conn, ctx.user_id, &input).await.map(ToolOutcome::text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nomi_tool::{Approval, AutoOffer, OfferContext};

    #[test]
    fn read_attachment_properties() {
        let tools = tools();
        assert_eq!(tools.len(), 1);
        let t = &tools[0];
        assert_eq!(t.definition().name, crate::READ_ATTACHMENT_TOOL_NAME);
        assert_eq!(t.approval(), Approval::Never);
        assert_eq!(t.auto_offer(), AutoOffer::Everyone);
        assert!(t.posts_activity());
    }

    #[test]
    fn read_attachment_hidden_without_files() {
        let tools = tools();
        let t = &tools[0];
        assert!(t.offer(&OfferContext { reminder_targets: &[], mentions_files: false }).is_none());
        assert!(t.offer(&OfferContext { reminder_targets: &[], mentions_files: true }).is_some());
    }
}
