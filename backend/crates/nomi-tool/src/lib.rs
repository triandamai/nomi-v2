//! The one `Tool` trait every agent tool implements, and what it needs: the context a call runs
//! in, the content blocks and views a tool can return, the catalog custom agents are granted
//! tools from, and Admin → Tools switches. Depends on no agent crate.

pub mod catalog;
pub mod content_block;
pub mod context;
pub mod crew_names;
pub mod locale;
pub mod tools;
pub mod views;

pub use catalog::{find_tool, tool_definitions, Tool, ToolCatalog, ToolInfo};
pub use content_block::{ContentBlock, ToolMedia, ToolOutcome};
pub use context::{Approval, AutoOffer, OfferContext, ToolContext};

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use serde_json::{json, Value};
    use sqlx::pool::PoolConnection;
    use sqlx::Postgres;

    use super::*;

    struct Plain;

    #[async_trait]
    impl Tool for Plain {
        fn definition(&self) -> nomi_llm::ToolDefinition {
            nomi_llm::ToolDefinition { name: "plain".into(), description: "A plain tool.".into(), input_schema: json!({"type": "object"}) }
        }
        fn group(&self) -> &'static str {
            "test"
        }
        async fn execute(&self, _conn: &mut PoolConnection<Postgres>, _ctx: &ToolContext<'_>, _input: Value) -> Result<ToolOutcome, String> {
            Ok(ToolOutcome::text("ok"))
        }
    }

    #[test]
    fn default_trait_properties() {
        let tool = Plain;
        assert_eq!(tool.approval(), Approval::Checked);
        assert_eq!(tool.auto_offer(), AutoOffer::GrantOnly);
        assert!(tool.posts_activity());
        let ctx = OfferContext { reminder_targets: &[], mentions_files: false, skills: &[] };
        assert_eq!(tool.offer(&ctx).map(|d| d.name), Some("plain".to_string()));
    }

    #[test]
    fn catalog_get_and_tools_sorted() {
        struct Named(&'static str);
        #[async_trait]
        impl Tool for Named {
            fn definition(&self) -> nomi_llm::ToolDefinition {
                nomi_llm::ToolDefinition { name: self.0.into(), description: "x".into(), input_schema: json!({"type": "object"}) }
            }
            fn group(&self) -> &'static str {
                "test"
            }
            async fn execute(&self, _conn: &mut PoolConnection<Postgres>, _ctx: &ToolContext<'_>, _input: Value) -> Result<ToolOutcome, String> {
                Ok(ToolOutcome::text(self.0))
            }
        }
        let catalog = ToolCatalog::from_tools(vec![Box::new(Named("zeta")), Box::new(Named("alpha"))]);
        assert_eq!(catalog.get("zeta").map(|t| t.group()), Some("test"));
        assert!(catalog.get("missing").is_none());
        let names: Vec<String> = catalog.tools().map(|t| t.definition().name).collect();
        assert_eq!(names, ["alpha", "zeta"]);
        let registered: Vec<String> = catalog.in_registration_order().map(|t| t.definition().name).collect();
        assert_eq!(registered, ["zeta", "alpha"]);
    }
}
