/// The speak tool's prompt text, owned by core's prompts and handed in by the server.
#[derive(Debug, Clone, Copy)]
pub struct SpeakCopy {
    pub description: &'static str,
    pub done: &'static str,
}

use async_trait::async_trait;
use nomi_llm::ToolDefinition;
use nomi_tool::{Approval, AutoOffer, Tool, ToolContext, ToolOutcome};
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres};

/// The catalog area this tool is listed under.
pub const GROUP: &str = "voice";

/// `speak`, offered only once voice is set up. Its card (the recording) is the outcome's block.
pub fn tools(copy: SpeakCopy) -> Vec<Box<dyn Tool>> {
    vec![Box::new(Speak { copy })]
}

pub struct Speak {
    copy: SpeakCopy,
}

#[async_trait]
impl Tool for Speak {
    fn definition(&self) -> ToolDefinition {
        crate::speak_tool_definition(self.copy.description)
    }
    fn group(&self) -> &'static str {
        GROUP
    }
    async fn ready(&self, conn: &mut PgConnection) -> bool {
        crate::is_ready(conn).await
    }
    fn approval(&self) -> Approval {
        Approval::Never
    }
    fn auto_offer(&self) -> AutoOffer {
        AutoOffer::Everyone
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (display_text, block) = crate::speak(conn, ctx.user_id, &input, self.copy.done).await?;
        Ok(ToolOutcome { display_text, block: Some(block) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nomi_tool::{Approval, AutoOffer};

    const COPY: SpeakCopy = SpeakCopy { description: "Say it out loud.", done: "Said." };

    #[test]
    fn speak_properties() {
        let tools = tools(COPY);
        assert_eq!(tools.len(), 1);
        let t = &tools[0];
        let def = t.definition();
        assert_eq!(def.name, crate::SPEAK_TOOL_NAME);
        assert_eq!(def.description, COPY.description);
        assert_eq!(t.approval(), Approval::Never);
        assert_eq!(t.auto_offer(), AutoOffer::Everyone);
        assert!(t.posts_activity());
    }
}
