//! Design directions: the two to four clearly different ways a site or app could look that Sora
//! proposes from the design brief, each with why it fits and what it trades off. A set is
//! structured data (colours, type, layout), so a preview and a handoff can be made from it, and
//! each save is a new, immutable version (`design_direction_sets`).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::views::{DecisionOption, DecisionView};

/// The fewest and the most directions in one set.
pub const MIN_DIRECTIONS: usize = 2;
pub const MAX_DIRECTIONS: usize = 4;
/// The fewest and the most colours in a direction's palette.
pub const MIN_PALETTE_COLOURS: usize = 2;
pub const MAX_PALETTE_COLOURS: usize = 8;
/// The most trade-offs one direction lists.
pub const MAX_TRADE_OFFS: usize = 6;
/// The most characters in one of a direction's texts.
pub const MAX_TEXT_CHARS: usize = 800;
/// The digits in a colour after its `#`.
const HEX_DIGITS: usize = 6;
/// What the person is asked above the directions.
pub const CHOICE_QUESTION: &str = "Which direction should we take?";
/// The colour shown as the example in error messages.
const EXAMPLE_COLOUR: &str = "#B5532A";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Colour {
    /// Its role or name ("Paper", "Accent").
    pub name: String,
    /// `#RRGGBB`, uppercase.
    pub hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Typography {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Direction {
    /// A stable slug: previews, feedback and the handoff refer to the direction by it.
    pub id: String,
    pub name: String,
    /// What it looks like, in a sentence or two.
    pub summary: String,
    /// Why it fits this brief.
    pub rationale: String,
    /// What it costs or risks.
    pub trade_offs: Vec<String>,
    pub palette: Vec<Colour>,
    pub typography: Typography,
    /// How pages are arranged.
    pub layout: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imagery: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub recommended: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignDirections {
    pub directions: Vec<Direction>,
}

fn text_of<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty())
}

fn clipped(text: &str) -> String {
    text.chars().take(MAX_TEXT_CHARS).collect()
}

/// `text` as a slug: lowercase letters and digits, everything else a single dash.
pub(crate) fn slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

fn parse_colour(who: &str, colour: &Value) -> Result<Colour, String> {
    let hex = text_of(colour, "hex").unwrap_or_default();
    let digits = hex.strip_prefix('#').unwrap_or_default();
    if digits.len() != HEX_DIGITS || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("{who}: \"{hex}\" isn't a colour like {EXAMPLE_COLOUR}"));
    }
    let name = text_of(colour, "name").ok_or_else(|| format!("{who}: each colour needs a name"))?;
    Ok(Colour { name: clipped(name), hex: hex.to_uppercase() })
}

fn parse_direction(index: usize, input: &Value) -> Result<Direction, String> {
    let name = text_of(input, "name").ok_or_else(|| format!("direction {}: name is required", index + 1))?;
    let who = name;
    let required = |field: &str| text_of(input, field).map(clipped).ok_or_else(|| format!("{who}: {field} is required"));
    let raw_id = text_of(input, "id").ok_or_else(|| format!("{who}: id is required"))?.to_lowercase();
    if raw_id != slug(&raw_id) {
        return Err(format!("{who}: id must be lowercase letters, digits and dashes, like \"{}\"", slug(&raw_id)));
    }
    let summary = required("summary")?;
    let rationale = required("rationale")?;
    let trade_offs: Vec<String> = input
        .get("trade_offs")
        .and_then(Value::as_array)
        .map(|list| list.iter().filter_map(|t| t.as_str().map(str::trim).filter(|t| !t.is_empty())).take(MAX_TRADE_OFFS).map(clipped).collect())
        .unwrap_or_default();
    if trade_offs.is_empty() {
        return Err(format!("{who}: give at least one trade-off"));
    }
    let layout = required("layout")?;
    let typography = input.get("typography").unwrap_or(&Value::Null);
    let (Some(heading), Some(body)) = (text_of(typography, "heading"), text_of(typography, "body")) else {
        return Err(format!("{who}: typography needs a heading and a body font"));
    };
    let colours = input.get("palette").and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default();
    let palette = colours.iter().map(|c| parse_colour(who, c)).collect::<Result<Vec<_>, _>>()?;
    if !(MIN_PALETTE_COLOURS..=MAX_PALETTE_COLOURS).contains(&palette.len()) {
        return Err(format!("{who}: a palette has {MIN_PALETTE_COLOURS} to {MAX_PALETTE_COLOURS} colours"));
    }
    Ok(Direction {
        id: raw_id,
        name: clipped(name),
        summary,
        rationale,
        trade_offs,
        palette,
        typography: Typography { heading: clipped(heading), body: clipped(body) },
        layout,
        imagery: text_of(input, "imagery").map(clipped),
        recommended: input.get("recommended").and_then(Value::as_bool).unwrap_or(false),
    })
}

/// The colours and fonts of a direction, for telling two directions apart.
fn look(direction: &Direction) -> (Vec<&str>, String, String) {
    (direction.palette.iter().map(|c| c.hex.as_str()).collect(), direction.typography.heading.to_lowercase(), direction.typography.body.to_lowercase())
}

impl DesignDirections {
    /// Reads and checks a set as the `propose_directions` tool takes it. The error says what to
    /// fix: how many, what each needs, and that they differ.
    pub fn parse(input: &Value) -> Result<Self, String> {
        let list = input.get("directions").and_then(Value::as_array).ok_or("directions must be a list")?;
        if !(MIN_DIRECTIONS..=MAX_DIRECTIONS).contains(&list.len()) {
            return Err(format!("propose {MIN_DIRECTIONS} to {MAX_DIRECTIONS} directions, not {}", list.len()));
        }
        let directions = list.iter().enumerate().map(|(i, d)| parse_direction(i, d)).collect::<Result<Vec<_>, _>>()?;
        for (index, direction) in directions.iter().enumerate() {
            for earlier in &directions[..index] {
                if earlier.id == direction.id {
                    return Err(format!("two directions share the id \"{}\"", direction.id));
                }
                if earlier.name.to_lowercase() == direction.name.to_lowercase() {
                    return Err(format!("two directions share the name \"{}\"", direction.name.to_lowercase()));
                }
                if look(earlier) == look(direction) {
                    return Err(format!("{} and {} have the same colours and fonts: make them clearly different", earlier.name, direction.name));
                }
            }
        }
        if directions.iter().filter(|d| d.recommended).count() > 1 {
            return Err("recommend one direction, not several".to_string());
        }
        Ok(Self { directions })
    }

    /// The direction with this id, however it was capitalised.
    pub fn find(&self, id: &str) -> Option<&Direction> {
        let id = id.trim().to_lowercase();
        self.directions.iter().find(|d| d.id == id)
    }

    /// The set as a choice for the person: each direction with why it fits and what it costs.
    pub fn to_decision(&self) -> DecisionView {
        DecisionView {
            question: CHOICE_QUESTION.to_string(),
            options: self
                .directions
                .iter()
                .map(|d| DecisionOption { label: d.name.clone(), detail: Some(d.summary.clone()), pros: vec![d.rationale.clone()], cons: d.trade_offs.clone(), recommended: d.recommended })
                .collect(),
        }
    }

    /// The set as an agent reads it.
    pub fn to_markdown(&self, version: i32) -> String {
        let mut text = format!("# Design directions (version {version})\n");
        for d in &self.directions {
            let recommended = if d.recommended { ", recommended" } else { "" };
            let colours: Vec<String> = d.palette.iter().map(|c| format!("{} {}", c.name, c.hex)).collect();
            text.push_str(&format!(
                "\n## {} (id: {}{recommended})\n{}\nWhy: {}\nColours: {}\nFonts: {} (headings), {} (body)\nLayout: {}\n",
                d.name,
                d.id,
                d.summary,
                d.rationale,
                colours.join(", "),
                d.typography.heading,
                d.typography.body,
                d.layout
            ));
            if let Some(imagery) = &d.imagery {
                text.push_str(&format!("Imagery: {imagery}\n"));
            }
            text.push_str("Trade-offs:\n");
            for trade_off in &d.trade_offs {
                text.push_str(&format!("- {trade_off}\n"));
            }
        }
        text.trim_end().to_string()
    }
}

/// The input schema of the tool that saves a set of directions.
pub fn directions_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "directions": {
                "type": "array",
                "minItems": MIN_DIRECTIONS,
                "maxItems": MAX_DIRECTIONS,
                "description": "Clearly different directions: different colours, fonts and layout, not variations of one",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "A short stable slug, lowercase with dashes; keep it when you refine the direction"},
                        "name": {"type": "string"},
                        "summary": {"type": "string", "description": "What it looks like, in a sentence or two"},
                        "rationale": {"type": "string", "description": "Why it fits this brief"},
                        "trade_offs": {"type": "array", "minItems": 1, "maxItems": MAX_TRADE_OFFS, "items": {"type": "string"}, "description": "What it costs or risks"},
                        "palette": {
                            "type": "array",
                            "minItems": MIN_PALETTE_COLOURS,
                            "maxItems": MAX_PALETTE_COLOURS,
                            "items": {"type": "object", "properties": {"name": {"type": "string"}, "hex": {"type": "string", "description": "#RRGGBB"}}, "required": ["name", "hex"]}
                        },
                        "typography": {"type": "object", "properties": {"heading": {"type": "string"}, "body": {"type": "string"}}, "required": ["heading", "body"]},
                        "layout": {"type": "string", "description": "How the pages are arranged"},
                        "imagery": {"type": "string", "description": "Photos, illustration or none, and their style"},
                        "recommended": {"type": "boolean", "description": "true on the one you would pick"}
                    },
                    "required": ["id", "name", "summary", "rationale", "trade_offs", "palette", "typography", "layout"]
                }
            }
        },
        "required": ["directions"]
    })
}

/// Saves `set` as the chat's next version, made from `brief_version` of the brief (when there
/// was one), and returns the set's version. Earlier versions are never changed.
pub async fn save(conn: &mut PgConnection, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, brief_version: Option<i32>, set: &DesignDirections) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO design_direction_sets (session_id, agent_session_id, user_id, brief_version, version, content) \
         SELECT $1, $2, $3, $4, COALESCE(MAX(version), 0) + 1, $5 FROM design_direction_sets WHERE session_id = $1 \
         RETURNING version",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(user_id)
    .bind(brief_version)
    .bind(sqlx::types::Json(set))
    .fetch_one(conn)
    .await
}

/// The chat's newest set: its version, the brief version it was made from, and the set.
pub async fn latest(conn: &mut PgConnection, session_id: Uuid) -> Result<Option<(i32, Option<i32>, DesignDirections)>, sqlx::Error> {
    let row: Option<(i32, Option<i32>, sqlx::types::Json<DesignDirections>)> =
        sqlx::query_as("SELECT version, brief_version, content FROM design_direction_sets WHERE session_id = $1 ORDER BY version DESC LIMIT 1")
            .bind(session_id)
            .fetch_optional(conn)
            .await?;
    Ok(row.map(|(version, brief_version, set)| (version, brief_version, set.0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn direction(id: &str, name: &str, accent: &str) -> serde_json::Value {
        json!({
            "id": id,
            "name": name,
            "summary": "Serif headings on warm paper tones, photos of the day's bakes up top.",
            "rationale": "The owner talks about freshness and craft, not price.",
            "trade_offs": ["Needs good photos to work"],
            "palette": [{"name": "Paper", "hex": "#F6EFE4"}, {"name": "Accent", "hex": accent}],
            "typography": {"heading": "Fraunces", "body": "Inter"},
            "layout": "One long page: hero, today's bakes, pre-order, hours and map."
        })
    }

    fn set() -> serde_json::Value {
        json!({"directions": [direction("editorial", "Editorial", "#B5532A"), direction("swiss", "Swiss utility", "#1F4FFF")]})
    }

    #[test]
    fn a_set_of_two_distinct_directions_is_read() {
        let directions = DesignDirections::parse(&set()).unwrap();
        assert_eq!(directions.directions.len(), 2);
        let editorial = &directions.directions[0];
        assert_eq!((editorial.id.as_str(), editorial.name.as_str()), ("editorial", "Editorial"));
        assert_eq!(editorial.palette[1].hex, "#B5532A");
        assert_eq!(editorial.typography.heading, "Fraunces");
        assert!(!editorial.recommended && editorial.imagery.is_none());
        assert_eq!(directions.find("SWISS").map(|d| d.name.as_str()), Some("Swiss utility"));
        assert_eq!(directions.find("nope"), None);
    }

    #[test]
    fn there_are_two_to_four_directions() {
        let one = json!({"directions": [direction("editorial", "Editorial", "#B5532A")]});
        assert_eq!(DesignDirections::parse(&one).unwrap_err(), format!("propose {MIN_DIRECTIONS} to {MAX_DIRECTIONS} directions, not 1"));
        let five: Vec<_> = (0..=MAX_DIRECTIONS).map(|n| direction(&format!("d{n}"), &format!("Direction {n}"), &format!("#00000{n}"))).collect();
        assert!(DesignDirections::parse(&json!({"directions": five})).unwrap_err().starts_with("propose 2 to 4 directions"));
        assert_eq!(DesignDirections::parse(&json!({})).unwrap_err(), "directions must be a list");
    }

    #[test]
    fn directions_must_differ_from_each_other() {
        let mut same_id = set();
        same_id["directions"][1]["id"] = json!("Editorial");
        assert_eq!(DesignDirections::parse(&same_id).unwrap_err(), "two directions share the id \"editorial\"");
        let mut same_name = set();
        same_name["directions"][1]["name"] = json!(" editorial ");
        assert_eq!(DesignDirections::parse(&same_name).unwrap_err(), "two directions share the name \"editorial\"");
        let mut same_look = set();
        same_look["directions"][1]["palette"] = same_look["directions"][0]["palette"].clone();
        same_look["directions"][1]["typography"] = same_look["directions"][0]["typography"].clone();
        assert_eq!(DesignDirections::parse(&same_look).unwrap_err(), "Editorial and Swiss utility have the same colours and fonts: make them clearly different");
    }

    #[test]
    fn each_direction_says_why_and_what_it_costs() {
        for (field, value, error) in [
            ("rationale", json!(" "), "Editorial: rationale is required"),
            ("trade_offs", json!([]), "Editorial: give at least one trade-off"),
            ("summary", json!(null), "Editorial: summary is required"),
            ("layout", json!(""), "Editorial: layout is required"),
            ("typography", json!({"heading": "Fraunces"}), "Editorial: typography needs a heading and a body font"),
        ] {
            let mut input = set();
            input["directions"][0][field] = value;
            assert_eq!(DesignDirections::parse(&input).unwrap_err(), error, "{field}");
        }
    }

    #[test]
    fn ids_are_stable_slugs() {
        let mut input = set();
        input["directions"][0]["id"] = json!("Warm Editorial!");
        assert_eq!(DesignDirections::parse(&input).unwrap_err(), "Editorial: id must be lowercase letters, digits and dashes, like \"warm-editorial\"");
        input["directions"][0]["id"] = json!("warm-editorial-2");
        assert!(DesignDirections::parse(&input).is_ok());
    }

    #[test]
    fn colours_are_six_digit_hex_and_there_are_enough_of_them() {
        let mut input = set();
        input["directions"][0]["palette"][1]["hex"] = json!("orange");
        assert_eq!(DesignDirections::parse(&input).unwrap_err(), "Editorial: \"orange\" isn't a colour like #B5532A");
        input["directions"][0]["palette"] = json!([{"name": "Paper", "hex": "#f6efe4"}]);
        assert_eq!(DesignDirections::parse(&input).unwrap_err(), format!("Editorial: a palette has {MIN_PALETTE_COLOURS} to {MAX_PALETTE_COLOURS} colours"));
        input["directions"][0]["palette"] = json!([{"name": "Paper", "hex": "#f6efe4"}, {"name": "Ink", "hex": "#1b1b1b"}]);
        assert_eq!(DesignDirections::parse(&input).unwrap().directions[0].palette[0].hex, "#F6EFE4", "stored uppercase");
    }

    #[test]
    fn at_most_one_direction_is_recommended() {
        let mut input = set();
        input["directions"][0]["recommended"] = json!(true);
        assert!(DesignDirections::parse(&input).unwrap().directions[0].recommended);
        input["directions"][1]["recommended"] = json!(true);
        assert_eq!(DesignDirections::parse(&input).unwrap_err(), "recommend one direction, not several");
    }

    #[test]
    fn a_set_survives_being_stored() {
        let directions = DesignDirections::parse(&set()).unwrap();
        let stored = serde_json::to_value(&directions).unwrap();
        assert_eq!(serde_json::from_value::<DesignDirections>(stored).unwrap(), directions);
    }

    #[test]
    fn the_person_gets_a_choice_with_each_directions_reason_and_trade_offs() {
        let mut input = set();
        input["directions"][1]["recommended"] = json!(true);
        let view = DesignDirections::parse(&input).unwrap().to_decision();
        assert_eq!(view.question, CHOICE_QUESTION);
        assert_eq!(view.options.iter().map(|o| o.label.as_str()).collect::<Vec<_>>(), ["Editorial", "Swiss utility"]);
        let swiss = &view.options[1];
        assert!(swiss.recommended && !view.options[0].recommended);
        assert!(swiss.detail.as_deref().unwrap().starts_with("Serif headings on warm paper tones"));
        assert_eq!(swiss.pros, ["The owner talks about freshness and craft, not price."]);
        assert_eq!(swiss.cons, ["Needs good photos to work"]);
    }

    #[test]
    fn it_reads_as_markdown_with_colours_and_fonts() {
        let text = DesignDirections::parse(&set()).unwrap().to_markdown(2);
        assert!(text.starts_with("# Design directions (version 2)"), "{text}");
        assert!(text.contains("## Editorial (id: editorial)"));
        assert!(text.contains("Colours: Paper #F6EFE4, Accent #B5532A"));
        assert!(text.contains("Fonts: Fraunces (headings), Inter (body)"));
        assert!(text.contains("Trade-offs:\n- Needs good photos to work"));
    }

    #[test]
    fn the_tool_schema_asks_for_two_to_four_directions() {
        let schema = directions_input_schema();
        let directions = &schema["properties"]["directions"];
        assert_eq!((directions["minItems"].as_u64(), directions["maxItems"].as_u64()), (Some(MIN_DIRECTIONS as u64), Some(MAX_DIRECTIONS as u64)));
        assert_eq!(directions["items"]["required"], json!(["id", "name", "summary", "rationale", "trade_offs", "palette", "typography", "layout"]));
    }
}
