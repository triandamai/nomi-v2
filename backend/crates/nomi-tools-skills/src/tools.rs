//! `read_skill`: a member reads one of its skills (or one of a skill's references) when a task
//! calls for it. Offered only to a member that has skills; its description carries the index.

use async_trait::async_trait;
use nomi_llm::ToolDefinition;
use nomi_skills::{body, index_line, BuiltinSkill, Reference, SkillEntry};
use nomi_tool::{Approval, AutoOffer, OfferContext, Tool, ToolContext, ToolOutcome};
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;

use crate::store;

/// The catalog area this tool is listed under.
pub const GROUP: &str = "skills";
/// The tool's name.
pub const READ_SKILL_TOOL_NAME: &str = "read_skill";
/// Where the index goes in the description.
const INDEX_SLOT: &str = "{index}";

/// The tool's wording, which lives with the other prompts (`nomi-agent-core/src/prompts.rs`).
#[derive(Debug, Clone, Copy)]
pub struct ReadSkillCopy {
    /// Holds `{index}`, replaced with the member's skills.
    pub description: &'static str,
}

pub fn tools(copy: ReadSkillCopy) -> Vec<Box<dyn Tool>> {
    vec![Box::new(ReadSkill { copy })]
}

/// The member's index: the person's skills first, then the built-in ones they don't shadow.
pub fn merged_index(builtin: &[BuiltinSkill], mine: Vec<SkillEntry>) -> Vec<SkillEntry> {
    let mut merged = mine;
    for skill in builtin {
        if !merged.iter().any(|m| m.name == skill.name) {
            merged.push(skill.entry());
        }
    }
    merged
}

pub struct ReadSkill {
    copy: ReadSkillCopy,
}

impl ReadSkill {
    fn definition_with(&self, index: &str) -> ToolDefinition {
        ToolDefinition {
            name: READ_SKILL_TOOL_NAME.to_string(),
            description: self.copy.description.replace(INDEX_SLOT, index),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "The skill's name, from your list of skills." },
                    "reference": { "type": "string", "description": "One of the skill's extra files, as the skill names it." }
                },
                "required": ["name"]
            }),
        }
    }
}

/// The skill's body and the names of its references, or one reference whole.
fn answer(name: &str, text: &str, references: &[Reference], wanted: Option<&str>) -> Result<String, String> {
    let names = || references.iter().map(|r| r.name.as_str()).collect::<Vec<_>>().join(", ");
    if let Some(wanted) = wanted {
        let lower = wanted.to_lowercase();
        let wanted = lower.strip_suffix(".md").unwrap_or(&lower);
        return references
            .iter()
            .find(|r| r.name == wanted)
            .map(|r| r.text.clone())
            .ok_or_else(|| format!("{name} has no reference \"{wanted}\". It has: {}", if references.is_empty() { "none".to_string() } else { names() }));
    }
    let body = body(text);
    Ok(if references.is_empty() { body } else { format!("{body}\n\nMore in this skill (read_skill with reference): {}", names()) })
}

#[async_trait]
impl Tool for ReadSkill {
    fn definition(&self) -> ToolDefinition {
        self.definition_with("")
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["skill", "know-how", "guide", "instructions"]
    }
    fn group(&self) -> &'static str {
        GROUP
    }
    fn offer(&self, ctx: &OfferContext<'_>) -> Option<ToolDefinition> {
        if ctx.skills.is_empty() {
            return None;
        }
        let index = ctx.skills.iter().map(index_line).collect::<Vec<_>>().join("\n");
        Some(self.definition_with(&index))
    }
    fn approval(&self) -> Approval {
        Approval::Never
    }
    fn auto_offer(&self) -> AutoOffer {
        AutoOffer::Everyone
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let field = |key: &str| input.get(key).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty());
        let reference = field("reference");
        let mine = store::index(&mut **conn, ctx.user_id, ctx.agent_type).await.map_err(|e| e.to_string())?;
        let available = || merged_index(ctx.builtin_skills, mine.clone()).into_iter().map(|e| e.name).collect::<Vec<_>>().join(", ");
        let Some(name) = field("name").map(str::to_lowercase) else {
            return Err(format!("name is required: one of your skills ({})", available()));
        };
        if let Some((row, references)) = store::find_enabled(&mut **conn, ctx.user_id, ctx.agent_type, &name).await.map_err(|e| e.to_string())? {
            return answer(&row.name, &row.text, &references, reference).map(ToolOutcome::text);
        }
        if let Some(skill) = ctx.builtin_skills.iter().find(|s| s.name == name) {
            let references: Vec<Reference> = skill.references.iter().map(|(n, t)| Reference { name: n.to_string(), text: t.to_string() }).collect();
            return answer(skill.name, skill.text, &references, reference).map(ToolOutcome::text);
        }
        Err(format!("you have no skill \"{name}\". Your skills: {}", available()))
    }
}
