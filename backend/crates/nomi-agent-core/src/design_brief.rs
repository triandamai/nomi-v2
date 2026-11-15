//! The design brief: what Rena found out about a site or app before it is designed, as
//! structured data rather than prose. Every entry says where it came from (the person, research,
//! a recommendation or an assumption), so nobody downstream mistakes a guess for a fact. Each
//! save is a new, immutable version (`design_briefs`). Only what the design can't start without
//! is required; the rest may stay unspecified.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgConnection;
use uuid::Uuid;

/// The most entries one section holds.
pub const MAX_ENTRIES_PER_SECTION: usize = 20;
/// The most characters in an entry's value or note, or in an open question.
pub const MAX_ENTRY_CHARS: usize = 600;
/// The most open questions a brief keeps.
pub const MAX_OPEN_QUESTIONS: usize = 20;

/// One section of the brief.
pub struct SectionDef {
    /// Its key in the tool input and in storage.
    pub key: &'static str,
    /// Its heading when the brief is read.
    pub title: &'static str,
    /// What goes in it, for the tool's schema.
    pub holds: &'static str,
    /// Whether design can't start without it.
    pub critical: bool,
}

const fn section(key: &'static str, title: &'static str, holds: &'static str, critical: bool) -> SectionDef {
    SectionDef { key, title, holds, critical }
}

/// The brief's sections, in the order they are read.
pub const SECTIONS: [SectionDef; 9] = [
    section("company", "Company", "Who the company or person is and what they offer", true),
    section("positioning", "Positioning", "How it sets itself apart, and what competitors do", false),
    section("audience", "Audience", "Who the site or app is for", true),
    section("brand_personality", "Brand personality", "How the brand should feel and sound", false),
    section("visual_preferences", "Visual preferences", "Looks, colours, fonts and sites they like or dislike", false),
    section("objectives", "Objectives", "What the site or app is for and what a visitor should do", true),
    section("features", "Features", "The product's features and the pages it needs", false),
    section("assets", "Assets", "What exists already: logo, colours, fonts, photos, copy", false),
    section("references", "References", "Sites and materials that were looked at", false),
];

const OPEN_KEY: &str = "open";

/// Where an entry came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The person said it.
    Confirmed,
    /// Found on a page or in a file; the note says where.
    Research,
    /// Rena's proposal; the note says why.
    Recommended,
    /// Decided without asking.
    Assumed,
}

impl Source {
    pub const ALL: [Source; 4] = [Source::Confirmed, Source::Research, Source::Recommended, Source::Assumed];

    pub fn as_str(self) -> &'static str {
        match self {
            Source::Confirmed => "confirmed",
            Source::Research => "research",
            Source::Recommended => "recommended",
            Source::Assumed => "assumed",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.as_str() == text)
    }
}

/// One thing the brief says, and where it came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub value: String,
    pub source: Source,
    /// Where research was found, why something is recommended, or any other remark.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DesignBrief {
    /// Entries by section key; a section left out is unspecified.
    #[serde(default)]
    pub sections: BTreeMap<String, Vec<Entry>>,
    /// What is still unspecified and doesn't block the design.
    #[serde(default)]
    pub open: Vec<String>,
}

/// "a", "a and b", "a, b and c".
fn listed(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => (*one).to_string(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

fn source_names() -> String {
    let names: Vec<&str> = Source::ALL.iter().map(|s| s.as_str()).collect();
    match names.as_slice() {
        [rest @ .., last] => format!("{} or {last}", rest.join(", ")),
        [] => String::new(),
    }
}

fn parse_entry(section: &str, entry: &Value) -> Result<Entry, String> {
    let text = |field: &str| entry.get(field).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty());
    let value = text("value").ok_or_else(|| format!("{section}: an entry has no value"))?;
    let source = text("source")
        .and_then(Source::parse)
        .ok_or_else(|| format!("{section}: each entry needs a source: {}", source_names()))?;
    let note = text("note");
    if value.chars().count() > MAX_ENTRY_CHARS || note.is_some_and(|n| n.chars().count() > MAX_ENTRY_CHARS) {
        return Err(format!("{section}: an entry is longer than {MAX_ENTRY_CHARS} characters"));
    }
    match (source, note) {
        (Source::Research, None) => return Err(format!("{section}: a research entry needs a note saying where it was found")),
        (Source::Recommended, None) => return Err(format!("{section}: a recommended entry needs a note saying why")),
        _ => {}
    }
    Ok(Entry { value: value.to_string(), source, note: note.map(str::to_string) })
}

impl DesignBrief {
    /// Reads and checks a brief as the `save_design_brief` tool takes it: an object of sections,
    /// each a list of `{value, source, note?}`, plus `open`. The error says what to fix.
    pub fn parse(input: &Value) -> Result<Self, String> {
        let object = input.as_object().ok_or("the brief must be an object of sections")?;
        if let Some(unknown) = object.keys().find(|k| *k != OPEN_KEY && !SECTIONS.iter().any(|s| s.key == k.as_str())) {
            let keys: Vec<&str> = SECTIONS.iter().map(|s| s.key).collect();
            return Err(format!("unknown section \"{unknown}\": the sections are {}", listed(&keys)));
        }
        let mut sections = BTreeMap::new();
        for def in &SECTIONS {
            let Some(value) = object.get(def.key).filter(|v| !v.is_null()) else { continue };
            let entries = value.as_array().ok_or_else(|| format!("{} must be a list of {{value, source, note?}}", def.key))?;
            if entries.len() > MAX_ENTRIES_PER_SECTION {
                return Err(format!("{}: at most {MAX_ENTRIES_PER_SECTION} entries", def.key));
            }
            let entries = entries.iter().map(|e| parse_entry(def.key, e)).collect::<Result<Vec<_>, _>>()?;
            if !entries.is_empty() {
                sections.insert(def.key.to_string(), entries);
            }
        }
        let open = match object.get(OPEN_KEY).filter(|v| !v.is_null()) {
            None => Vec::new(),
            Some(open) => open
                .as_array()
                .ok_or("open must be a list of questions")?
                .iter()
                .filter_map(|q| q.as_str().map(str::trim).filter(|q| !q.is_empty()))
                .take(MAX_OPEN_QUESTIONS)
                .map(|q| q.chars().take(MAX_ENTRY_CHARS).collect())
                .collect(),
        };
        let missing: Vec<&str> = SECTIONS.iter().filter(|s| s.critical && !sections.contains_key(s.key)).map(|s| s.key).collect();
        if !missing.is_empty() {
            return Err(format!("the brief needs {}: ask the person or note what you assume", listed(&missing)));
        }
        Ok(Self { sections, open })
    }

    /// A section's entries (none when it is unspecified).
    pub fn section(&self, key: &str) -> &[Entry] {
        self.sections.get(key).map(Vec::as_slice).unwrap_or_default()
    }

    /// The keys of the sections left unspecified, in reading order.
    pub fn unspecified(&self) -> Vec<&'static str> {
        SECTIONS.iter().filter(|s| self.section(s.key).is_empty()).map(|s| s.key).collect()
    }

    /// The brief as an agent reads it: each filled section, every entry marked with where it
    /// came from, then what is open and what was left unspecified.
    pub fn to_markdown(&self, version: i32) -> String {
        let mut text = format!("# Design brief (version {version})\n");
        for def in &SECTIONS {
            let entries = self.section(def.key);
            if entries.is_empty() {
                continue;
            }
            text.push_str(&format!("\n## {}\n", def.title));
            for entry in entries {
                let origin = match (entry.source, entry.note.as_deref()) {
                    (Source::Confirmed, None) => "confirmed by the person".to_string(),
                    (Source::Confirmed, Some(note)) => format!("confirmed by the person; {note}"),
                    (Source::Research, note) => format!("from research: {}", note.unwrap_or("source not recorded")),
                    (Source::Recommended, note) => format!("recommended: {}", note.unwrap_or("no reason recorded")),
                    (Source::Assumed, None) => "assumed, not confirmed".to_string(),
                    (Source::Assumed, Some(note)) => format!("assumed, not confirmed; {note}"),
                };
                text.push_str(&format!("- {} ({origin})\n", entry.value));
            }
        }
        if !self.open.is_empty() {
            text.push_str("\n## Still open\n");
            for question in &self.open {
                text.push_str(&format!("- {question}\n"));
            }
        }
        let unspecified: Vec<String> = SECTIONS.iter().filter(|s| self.section(s.key).is_empty()).map(|s| s.title.to_lowercase()).collect();
        if !unspecified.is_empty() {
            text.push_str(&format!("\nNot specified: {}\n", unspecified.join(", ")));
        }
        text.trim_end().to_string()
    }
}

/// The input schema of the tool that saves a brief: every section, with the critical ones
/// required.
pub fn brief_input_schema() -> Value {
    let sources: Vec<&str> = Source::ALL.iter().map(|s| s.as_str()).collect();
    let entry = json!({
        "type": "object",
        "properties": {
            "value": {"type": "string", "description": "One fact, preference or decision"},
            "source": {"type": "string", "enum": sources, "description": "confirmed: the person said it; research: found on a page or in a file; recommended: your proposal; assumed: decided without asking"},
            "note": {"type": "string", "description": "Required for research (where it was found) and recommended (why); optional otherwise"}
        },
        "required": ["value", "source"]
    });
    let mut properties = serde_json::Map::new();
    for def in &SECTIONS {
        properties.insert(def.key.to_string(), json!({"type": "array", "maxItems": MAX_ENTRIES_PER_SECTION, "description": def.holds, "items": entry}));
    }
    properties.insert(
        OPEN_KEY.to_string(),
        json!({"type": "array", "maxItems": MAX_OPEN_QUESTIONS, "items": {"type": "string"}, "description": "What is still unspecified and doesn't block the design"}),
    );
    let required: Vec<&str> = SECTIONS.iter().filter(|s| s.critical).map(|s| s.key).collect();
    json!({"type": "object", "properties": properties, "required": required})
}

/// Saves `brief` as the chat's next version and returns that version. Earlier versions are
/// never changed.
pub async fn save(conn: &mut PgConnection, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, brief: &DesignBrief) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO design_briefs (session_id, agent_session_id, user_id, version, content) \
         SELECT $1, $2, $3, COALESCE(MAX(version), 0) + 1, $4 FROM design_briefs WHERE session_id = $1 \
         RETURNING version",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(user_id)
    .bind(sqlx::types::Json(brief))
    .fetch_one(conn)
    .await
}

/// The chat's newest brief and its version.
pub async fn latest(conn: &mut PgConnection, session_id: Uuid) -> Result<Option<(i32, DesignBrief)>, sqlx::Error> {
    let row: Option<(i32, sqlx::types::Json<DesignBrief>)> =
        sqlx::query_as("SELECT version, content FROM design_briefs WHERE session_id = $1 ORDER BY version DESC LIMIT 1")
            .bind(session_id)
            .fetch_optional(conn)
            .await?;
    Ok(row.map(|(version, brief)| (version, brief.0)))
}

/// One version of the chat's brief.
pub async fn version(conn: &mut PgConnection, session_id: Uuid, version: i32) -> Result<Option<DesignBrief>, sqlx::Error> {
    let row: Option<(sqlx::types::Json<DesignBrief>,)> = sqlx::query_as("SELECT content FROM design_briefs WHERE session_id = $1 AND version = $2")
        .bind(session_id)
        .bind(version)
        .fetch_optional(conn)
        .await?;
    Ok(row.map(|(brief,)| brief.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn minimal() -> serde_json::Value {
        json!({
            "company": [{"value": "Roti Pagi, a neighbourhood bakery in Bandung", "source": "confirmed"}],
            "audience": [{"value": "People living within walking distance", "source": "assumed"}],
            "objectives": [{"value": "Get visitors to pre-order for pickup", "source": "confirmed"}]
        })
    }

    #[test]
    fn a_brief_needs_only_its_critical_sections() {
        let brief = DesignBrief::parse(&minimal()).unwrap();
        assert_eq!(brief.section("company").len(), 1);
        assert_eq!(brief.section("company")[0].source, Source::Confirmed);
        assert_eq!(brief.section("audience")[0].source, Source::Assumed);
        assert!(brief.section("visual_preferences").is_empty(), "left unspecified");
        assert!(brief.open.is_empty());
        assert_eq!(brief.unspecified(), ["positioning", "brand_personality", "visual_preferences", "features", "assets", "references"]);
    }

    #[test]
    fn a_missing_critical_section_is_named() {
        let mut input = minimal();
        input.as_object_mut().unwrap().remove("audience");
        input["objectives"] = json!([]);
        assert_eq!(DesignBrief::parse(&input).unwrap_err(), "the brief needs audience and objectives: ask the person or note what you assume");
    }

    #[test]
    fn every_entry_says_where_it_came_from() {
        let mut input = minimal();
        input["features"] = json!([{"value": "Online pre-order"}]);
        assert_eq!(DesignBrief::parse(&input).unwrap_err(), "features: each entry needs a source: confirmed, research, recommended or assumed");
        input["features"] = json!([{"value": "Online pre-order", "source": "guessed"}]);
        assert!(DesignBrief::parse(&input).unwrap_err().starts_with("features: each entry needs a source"));
        input["features"] = json!([{"value": "  ", "source": "confirmed"}]);
        assert_eq!(DesignBrief::parse(&input).unwrap_err(), "features: an entry has no value");
    }

    #[test]
    fn research_names_its_page_and_a_recommendation_its_reason() {
        let mut input = minimal();
        input["positioning"] = json!([{"value": "Competitors lead with price", "source": "research"}]);
        assert_eq!(DesignBrief::parse(&input).unwrap_err(), "positioning: a research entry needs a note saying where it was found");
        input["positioning"] = json!([{"value": "Lead with freshness, not price", "source": "recommended", "note": " "}]);
        assert_eq!(DesignBrief::parse(&input).unwrap_err(), "positioning: a recommended entry needs a note saying why");
        input["positioning"] = json!([
            {"value": "Competitors lead with price", "source": "research", "note": "https://example.com/bakeries"},
            {"value": "Lead with freshness, not price", "source": "recommended", "note": "It's what the owner talks about"}
        ]);
        let brief = DesignBrief::parse(&input).unwrap();
        assert_eq!(brief.section("positioning")[0].note.as_deref(), Some("https://example.com/bakeries"));
    }

    #[test]
    fn unknown_sections_and_wrong_shapes_are_refused() {
        let mut input = minimal();
        input["colour"] = json!([{"value": "Blue", "source": "confirmed"}]);
        assert!(DesignBrief::parse(&input).unwrap_err().starts_with("unknown section \"colour\""));
        let mut input = minimal();
        input["company"] = json!("Roti Pagi");
        assert_eq!(DesignBrief::parse(&input).unwrap_err(), "company must be a list of {value, source, note?}");
        assert_eq!(DesignBrief::parse(&json!("brief")).unwrap_err(), "the brief must be an object of sections");
    }

    #[test]
    fn a_section_holds_only_so_many_entries_of_so_many_characters() {
        let mut input = minimal();
        input["features"] = json!((0..=MAX_ENTRIES_PER_SECTION).map(|n| json!({"value": format!("Feature {n}"), "source": "confirmed"})).collect::<Vec<_>>());
        assert_eq!(DesignBrief::parse(&input).unwrap_err(), format!("features: at most {MAX_ENTRIES_PER_SECTION} entries"));
        let mut input = minimal();
        input["features"] = json!([{"value": "x".repeat(MAX_ENTRY_CHARS + 1), "source": "confirmed"}]);
        assert_eq!(DesignBrief::parse(&input).unwrap_err(), format!("features: an entry is longer than {MAX_ENTRY_CHARS} characters"));
    }

    #[test]
    fn open_questions_are_kept_trimmed_and_without_blanks() {
        let mut input = minimal();
        input["open"] = json!(["  Is there a logo?  ", "", "Which photos can be used?"]);
        assert_eq!(DesignBrief::parse(&input).unwrap().open, ["Is there a logo?", "Which photos can be used?"]);
        input["open"] = json!("a logo?");
        assert_eq!(DesignBrief::parse(&input).unwrap_err(), "open must be a list of questions");
    }

    #[test]
    fn a_brief_survives_being_stored() {
        let mut input = minimal();
        input["open"] = json!(["Is there a logo?"]);
        let brief = DesignBrief::parse(&input).unwrap();
        let stored = serde_json::to_value(&brief).unwrap();
        assert_eq!(serde_json::from_value::<DesignBrief>(stored).unwrap(), brief);
    }

    #[test]
    fn it_reads_as_markdown_that_keeps_facts_and_guesses_apart() {
        let mut input = minimal();
        input["positioning"] = json!([{"value": "Competitors lead with price", "source": "research", "note": "https://example.com/bakeries"}]);
        input["open"] = json!(["Is there a logo?"]);
        let text = DesignBrief::parse(&input).unwrap().to_markdown(3);
        assert!(text.starts_with("# Design brief (version 3)"), "{text}");
        assert!(text.contains("## Company\n- Roti Pagi, a neighbourhood bakery in Bandung (confirmed by the person)"));
        assert!(text.contains("## Audience\n- People living within walking distance (assumed, not confirmed)"));
        assert!(text.contains("## Positioning\n- Competitors lead with price (from research: https://example.com/bakeries)"));
        assert!(text.contains("## Still open\n- Is there a logo?"));
        assert!(text.contains("Not specified: brand personality, visual preferences, features, assets, references"));
        assert!(!text.contains("## Features"), "an empty section has no heading");
    }

    #[test]
    fn the_tool_schema_lists_every_section_and_what_is_required() {
        let schema = brief_input_schema();
        for section in SECTIONS {
            assert_eq!(schema["properties"][section.key]["type"], "array", "{}", section.key);
        }
        assert_eq!(schema["properties"]["open"]["type"], "array");
        assert_eq!(schema["required"], json!(["company", "audience", "objectives"]));
        assert_eq!(schema["properties"]["company"]["items"]["properties"]["source"]["enum"], json!(["confirmed", "research", "recommended", "assumed"]));
    }
}
