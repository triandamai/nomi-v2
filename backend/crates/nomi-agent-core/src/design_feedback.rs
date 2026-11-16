//! What the person says about a design preview in the Design Studio: about the whole design, a
//! page, a section, a component, one element, or how something behaves. Each piece of feedback
//! points at a version, and at a page and a `data-design-id` when it is about one. It is kept
//! for good (`design_feedback`): a later version can say it dealt with it, nothing more.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::design_artifact::{ElementKind, Manifest};

/// The most characters in one piece of feedback.
pub const MAX_BODY_CHARS: usize = 2000;
/// The most feedback one chat may have waiting on Sora at a time.
pub const MAX_OPEN_PER_CHAT: i64 = 200;
/// The most characters in what a version says it did about a piece of feedback.
pub const MAX_NOTE_CHARS: usize = 500;
/// A pin's x and y run from one edge of what it is on to the other.
const PIN_MIN: f64 = 0.0;
const PIN_MAX: f64 = 1.0;
const PERCENT: f32 = 100.0;
const NO_LEVEL: &str = "level is global, page, section, component, element or interaction";
const NO_PIN: &str = "pin is a point inside the target: x and y from 0 to 1";
const OPEN_HEADING: &str = "# Open feedback from the Design Studio";

/// What a piece of feedback is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Global,
    Page,
    Section,
    Component,
    Element,
    Interaction,
}

impl Level {
    const ALL: [(&'static str, Level); 6] = [
        ("global", Self::Global),
        ("page", Self::Page),
        ("section", Self::Section),
        ("component", Self::Component),
        ("element", Self::Element),
        ("interaction", Self::Interaction),
    ];

    pub fn name(self) -> &'static str {
        Self::ALL.iter().find(|(_, level)| *level == self).map(|(name, _)| *name).unwrap_or_default()
    }

    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.iter().find(|(known, _)| *known == name).map(|(_, level)| *level)
    }

    fn of(kind: ElementKind) -> Self {
        match kind {
            ElementKind::Section => Self::Section,
            ElementKind::Component => Self::Component,
            ElementKind::Element => Self::Element,
        }
    }
}

/// Whether a later version dealt with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Open,
    Resolved,
}

impl Status {
    const OPEN: &'static str = "open";
    const RESOLVED: &'static str = "resolved";

    fn named(name: &str) -> Self {
        if name == Self::RESOLVED {
            Self::Resolved
        } else {
            Self::Open
        }
    }
}

/// A point inside what the feedback is about: x across and y down, each from 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pin {
    pub x: f32,
    pub y: f32,
}

/// Feedback as the Studio sends it, checked against the version it is about.
#[derive(Debug, Clone, PartialEq)]
pub struct NewFeedback {
    pub version: i32,
    pub level: Level,
    pub page: Option<String>,
    pub element_id: Option<String>,
    pub body: String,
    pub pin: Option<Pin>,
}

fn text_of<'a>(input: &'a Value, field: &str) -> Option<&'a str> {
    input.get(field).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty())
}

fn pin_of(input: &Value) -> Result<Option<Pin>, String> {
    let Some(pin) = input.get("pin").filter(|pin| !pin.is_null()) else {
        return Ok(None);
    };
    let part = |field: &str| pin.get(field).and_then(Value::as_f64).filter(|v| (PIN_MIN..=PIN_MAX).contains(v));
    match (part("x"), part("y")) {
        (Some(x), Some(y)) => Ok(Some(Pin { x: x as f32, y: y as f32 })),
        _ => Err(NO_PIN.to_string()),
    }
}

impl NewFeedback {
    /// Reads feedback about `version`, whose pages and elements are `manifest`'s. What an
    /// element is (section, component, element) comes from the manifest, not from the asker.
    pub fn parse(version: i32, manifest: &Manifest, input: &Value) -> Result<Self, String> {
        let level = text_of(input, "level").and_then(Level::named).ok_or(NO_LEVEL)?;
        let body = text_of(input, "body").ok_or("body is required")?;
        if body.chars().count() > MAX_BODY_CHARS {
            return Err(format!("feedback is at most {MAX_BODY_CHARS} characters"));
        }
        if level == Level::Global {
            return Ok(Self { version, level, page: None, element_id: None, body: body.to_string(), pin: None });
        }
        let page = text_of(input, "page").ok_or("page is required")?;
        if !manifest.pages.iter().any(|p| p.path == page) {
            return Err(format!("this version has no page \"{page}\""));
        }
        let pin = pin_of(input)?;
        let element_id = text_of(input, "element_id");
        let element = match element_id {
            Some(id) => Some(
                manifest
                    .elements
                    .iter()
                    .find(|e| e.id == id && e.pages.iter().any(|p| p == page))
                    .ok_or_else(|| format!("{page} has no element \"{id}\" in this version"))?,
            ),
            None => None,
        };
        let level = match (level, element) {
            (Level::Page, _) => return Ok(Self { version, level, page: Some(page.to_string()), element_id: None, body: body.to_string(), pin }),
            (Level::Interaction, _) => Level::Interaction,
            (_, Some(element)) => Level::of(element.kind),
            (_, None) => return Err("element_id is required".to_string()),
        };
        Ok(Self { version, level, page: Some(page.to_string()), element_id: element_id.map(str::to_string), body: body.to_string(), pin })
    }
}

/// One piece of feedback as it is kept. `number` is its place in the chat's feedback, the name
/// Sora and the person call it by.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Feedback {
    pub id: Uuid,
    pub number: i32,
    /// The version it was given on.
    pub version: i32,
    pub level: Level,
    pub page: Option<String>,
    pub element_id: Option<String>,
    pub body: String,
    pub pin: Option<Pin>,
    pub status: Status,
    /// The version that dealt with it, and what that version did about it.
    pub resolved_in_version: Option<i32>,
    pub resolution: Option<String>,
    pub user_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Feedback {
    /// Where it points, in words.
    pub fn target(&self) -> String {
        let page = self.page.as_deref().unwrap_or_default();
        match (self.level, self.element_id.as_deref()) {
            (Level::Global, _) => "the whole design".to_string(),
            (Level::Page, _) => format!("page {page}"),
            (Level::Interaction, Some(id)) => format!("how {id} behaves on {page}"),
            (Level::Interaction, None) => format!("how page {page} behaves"),
            (level, id) => format!("{} {} on {page}", level.name(), id.unwrap_or_default()),
        }
    }

    fn line(&self) -> String {
        let pin = self
            .pin
            .map(|pin| format!(", at {:.0}% across and {:.0}% down", pin.x * PERCENT, pin.y * PERCENT))
            .unwrap_or_default();
        let body = self.body.split_whitespace().collect::<Vec<_>>().join(" ");
        format!("- #{} (version {}, {}{pin}): {body}", self.number, self.version, self.target())
    }
}

/// The feedback still waiting on a new version, as Sora reads it; `None` when there is none.
pub fn open_markdown(feedback: &[Feedback]) -> Option<String> {
    let lines: Vec<String> = feedback.iter().filter(|f| f.status == Status::Open).map(Feedback::line).collect();
    (!lines.is_empty()).then(|| format!("{OPEN_HEADING}\n{}", lines.join("\n")))
}

#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    number: i32,
    artifact_version: i32,
    level: String,
    page: Option<String>,
    element_id: Option<String>,
    body: String,
    pin_x: Option<f32>,
    pin_y: Option<f32>,
    status: String,
    resolved_in_version: Option<i32>,
    resolution: Option<String>,
    user_id: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<Row> for Feedback {
    fn from(row: Row) -> Self {
        Self {
            id: row.id,
            number: row.number,
            version: row.artifact_version,
            level: Level::named(&row.level).unwrap_or(Level::Global),
            page: row.page,
            element_id: row.element_id,
            body: row.body,
            pin: row.pin_x.zip(row.pin_y).map(|(x, y)| Pin { x, y }),
            status: Status::named(&row.status),
            resolved_in_version: row.resolved_in_version,
            resolution: row.resolution,
            user_id: row.user_id,
            created_at: row.created_at,
            resolved_at: row.resolved_at,
        }
    }
}

const COLUMNS: &str = "id, number, artifact_version, level, page, element_id, body, pin_x, pin_y, status, resolved_in_version, resolution, user_id, created_at, resolved_at";

/// Keeps `feedback` as the chat's next piece of feedback.
pub async fn add(conn: &mut PgConnection, session_id: Uuid, user_id: Uuid, feedback: &NewFeedback) -> Result<Feedback, sqlx::Error> {
    let row: Row = sqlx::query_as(&format!(
        "INSERT INTO design_feedback (session_id, user_id, number, artifact_version, level, page, element_id, body, pin_x, pin_y) \
         SELECT $1, $2, COALESCE(MAX(number), 0) + 1, $3, $4, $5, $6, $7, $8, $9 FROM design_feedback WHERE session_id = $1 \
         RETURNING {COLUMNS}"
    ))
    .bind(session_id)
    .bind(user_id)
    .bind(feedback.version)
    .bind(feedback.level.name())
    .bind(&feedback.page)
    .bind(&feedback.element_id)
    .bind(&feedback.body)
    .bind(feedback.pin.map(|pin| pin.x))
    .bind(feedback.pin.map(|pin| pin.y))
    .fetch_one(conn)
    .await?;
    Ok(row.into())
}

/// All of the chat's feedback, oldest first.
pub async fn list(conn: &mut PgConnection, session_id: Uuid) -> Result<Vec<Feedback>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(&format!("SELECT {COLUMNS} FROM design_feedback WHERE session_id = $1 ORDER BY number")).bind(session_id).fetch_all(conn).await?;
    Ok(rows.into_iter().map(Feedback::from).collect())
}

/// How much of the chat's feedback is still waiting on a new version.
pub async fn open_count(conn: &mut PgConnection, session_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM design_feedback WHERE session_id = $1 AND status = $2").bind(session_id).bind(Status::OPEN).fetch_one(conn).await
}

/// What a new version did about one piece of feedback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    /// The feedback's number in the chat.
    pub number: i32,
    pub note: String,
}

/// The schema of `resolves`, for the tool that saves a preview.
pub fn resolves_schema() -> Value {
    serde_json::json!({
        "type": "array",
        "description": "The open feedback this version dealt with: each one's number and, in a sentence, what you changed for it. Leave out feedback you didn't act on",
        "items": {
            "type": "object",
            "properties": {"feedback": {"type": "integer", "description": "The feedback's number"}, "note": {"type": "string", "maxLength": MAX_NOTE_CHARS}},
            "required": ["feedback", "note"]
        }
    })
}

/// The feedback a new version says it dealt with (`resolves`); none when it doesn't say.
pub fn parse_resolutions(input: &Value) -> Result<Vec<Resolution>, String> {
    let items = match input.get("resolves") {
        None | Some(Value::Null) => return Ok(Vec::new()),
        Some(Value::Array(items)) => items,
        Some(_) => return Err("resolves must be a list".to_string()),
    };
    let mut resolutions: Vec<Resolution> = Vec::new();
    for item in items {
        let number = item.get("feedback").and_then(Value::as_i64).and_then(|n| i32::try_from(n).ok()).ok_or("each of resolves names a feedback number")?;
        let note = text_of(item, "note").ok_or_else(|| format!("feedback #{number}: note says what you changed"))?;
        if note.chars().count() > MAX_NOTE_CHARS {
            return Err(format!("feedback #{number}: note is at most {MAX_NOTE_CHARS} characters"));
        }
        if resolutions.iter().any(|r| r.number == number) {
            return Err(format!("feedback #{number} is in resolves twice"));
        }
        resolutions.push(Resolution { number, note: note.to_string() });
    }
    Ok(resolutions)
}

/// Whether every resolution is about feedback that is still waiting; the error says what is.
pub fn check_open(resolutions: &[Resolution], feedback: &[Feedback]) -> Result<(), String> {
    let open: Vec<i32> = feedback.iter().filter(|f| f.status == Status::Open).map(|f| f.number).collect();
    match resolutions.iter().find(|r| !open.contains(&r.number)) {
        None => Ok(()),
        Some(stray) if open.is_empty() => Err(format!("feedback #{} isn't waiting: no feedback is open", stray.number)),
        Some(stray) => {
            let numbers: Vec<String> = open.iter().map(|n| format!("#{n}")).collect();
            Err(format!("feedback #{} isn't waiting: the feedback still open is {}", stray.number, numbers.join(", ")))
        }
    }
}

/// Marks the feedback `version` dealt with, each with what the version did. What was said stays
/// as it was; feedback already dealt with is left alone. Returns the numbers it marked.
pub async fn resolve(conn: &mut PgConnection, session_id: Uuid, version: i32, resolutions: &[Resolution]) -> Result<Vec<i32>, sqlx::Error> {
    let mut marked = Vec::new();
    for resolution in resolutions {
        let done = sqlx::query(
            "UPDATE design_feedback SET status = $1, resolved_in_version = $2, resolution = $3, resolved_at = now() \
             WHERE session_id = $4 AND number = $5 AND status = $6",
        )
        .bind(Status::RESOLVED)
        .bind(version)
        .bind(&resolution.note)
        .bind(session_id)
        .bind(resolution.number)
        .bind(Status::OPEN)
        .execute(&mut *conn)
        .await?;
        if done.rows_affected() > 0 {
            marked.push(resolution.number);
        }
    }
    Ok(marked)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design_artifact::DesignArtifact;
    use serde_json::json;

    fn manifest() -> Manifest {
        let page = |title: &str, body: &str| format!("<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>{title}</title></head><body>{body}</body></html>");
        DesignArtifact::parse(&json!({
            "direction_id": "editorial",
            "title": "First pass",
            "files": [
                {"path": "index.html", "content": page("Home", "<header data-design-id=\"site-header\"></header><section data-design-id=\"hero\"><a data-design-id=\"order-button\">Order</a></section>")},
                {"path": "menu/index.html", "content": page("Menu", "<header data-design-id=\"site-header\"></header><ul data-design-id=\"menu-list\"></ul>")}
            ],
            "elements": [{"id": "hero", "kind": "section", "label": "Hero"}, {"id": "site-header", "kind": "component", "label": "Site header"}]
        }))
        .unwrap()
        .manifest
    }

    fn parsed(input: serde_json::Value) -> Result<NewFeedback, String> {
        NewFeedback::parse(2, &manifest(), &input)
    }

    #[test]
    fn feedback_about_the_whole_design_points_at_nothing() {
        let feedback = parsed(json!({"level": "global", "body": "  Warmer overall  ", "page": "index.html", "element_id": "hero"})).unwrap();
        assert_eq!(feedback, NewFeedback { version: 2, level: Level::Global, page: None, element_id: None, body: "Warmer overall".into(), pin: None });
    }

    #[test]
    fn feedback_about_a_page_names_a_page_the_version_has() {
        let feedback = parsed(json!({"level": "page", "page": "menu/index.html", "body": "Too long"})).unwrap();
        assert_eq!((feedback.level, feedback.page.as_deref(), feedback.element_id), (Level::Page, Some("menu/index.html"), None));
        assert_eq!(parsed(json!({"level": "page", "body": "Too long"})).unwrap_err(), "page is required");
        assert_eq!(parsed(json!({"level": "page", "page": "about.html", "body": "x"})).unwrap_err(), "this version has no page \"about.html\"");
    }

    #[test]
    fn feedback_about_an_element_takes_its_level_from_what_the_element_is() {
        let hero = parsed(json!({"level": "element", "page": "index.html", "element_id": "hero", "body": "Shorter"})).unwrap();
        assert_eq!(hero.level, Level::Section, "the manifest says what it is, not the asker");
        let header = parsed(json!({"level": "section", "page": "menu/index.html", "element_id": "site-header", "body": "Sticky"})).unwrap();
        assert_eq!(header.level, Level::Component);
        let button = parsed(json!({"level": "component", "page": "index.html", "element_id": "order-button", "body": "Bigger"})).unwrap();
        assert_eq!((button.level, button.element_id.as_deref()), (Level::Element, Some("order-button")));
        assert_eq!(parsed(json!({"level": "element", "page": "index.html", "body": "x"})).unwrap_err(), "element_id is required");
        assert_eq!(
            parsed(json!({"level": "element", "page": "index.html", "element_id": "menu-list", "body": "x"})).unwrap_err(),
            "index.html has no element \"menu-list\" in this version"
        );
    }

    #[test]
    fn feedback_about_how_something_behaves_is_on_a_page_or_an_element() {
        let on_page = parsed(json!({"level": "interaction", "page": "index.html", "body": "The menu should slide in"})).unwrap();
        assert_eq!((on_page.level, on_page.element_id), (Level::Interaction, None));
        let on_element = parsed(json!({"level": "interaction", "page": "index.html", "element_id": "order-button", "body": "Show a spinner"})).unwrap();
        assert_eq!((on_element.level, on_element.element_id.as_deref()), (Level::Interaction, Some("order-button")));
        assert_eq!(parsed(json!({"level": "interaction", "body": "x"})).unwrap_err(), "page is required");
    }

    #[test]
    fn feedback_needs_words_and_a_known_level() {
        assert_eq!(parsed(json!({"level": "global", "body": "  "})).unwrap_err(), "body is required");
        assert_eq!(parsed(json!({"level": "global", "body": "x".repeat(MAX_BODY_CHARS + 1)})).unwrap_err(), format!("feedback is at most {MAX_BODY_CHARS} characters"));
        assert_eq!(parsed(json!({"level": "vibe", "body": "x"})).unwrap_err(), "level is global, page, section, component, element or interaction");
        assert_eq!(parsed(json!({"body": "x"})).unwrap_err(), "level is global, page, section, component, element or interaction");
    }

    #[test]
    fn a_pin_is_a_point_inside_what_the_feedback_is_about() {
        let pinned = parsed(json!({"level": "element", "page": "index.html", "element_id": "hero", "body": "Here", "pin": {"x": 0.25, "y": 1}})).unwrap();
        assert_eq!(pinned.pin, Some(Pin { x: 0.25, y: 1.0 }));
        for pin in [json!({"x": 1.2, "y": 0}), json!({"x": -0.1, "y": 0}), json!({"x": 0.5}), json!("top left")] {
            assert_eq!(
                parsed(json!({"level": "page", "page": "index.html", "body": "Here", "pin": pin})).unwrap_err(),
                "pin is a point inside the target: x and y from 0 to 1"
            );
        }
        assert_eq!(parsed(json!({"level": "global", "body": "x", "pin": {"x": 0.5, "y": 0.5}})).unwrap().pin, None, "the whole design has no point");
    }

    fn stored(number: i32, new: NewFeedback) -> Feedback {
        Feedback {
            id: Uuid::nil(),
            number,
            version: new.version,
            level: new.level,
            page: new.page,
            element_id: new.element_id,
            body: new.body,
            pin: new.pin,
            status: Status::Open,
            resolved_in_version: None,
            resolution: None,
            user_id: Uuid::nil(),
            created_at: chrono::DateTime::<chrono::Utc>::UNIX_EPOCH,
            resolved_at: None,
        }
    }

    #[test]
    fn feedback_says_where_it_points() {
        let at = |input: serde_json::Value| stored(1, parsed(input).unwrap()).target();
        assert_eq!(at(json!({"level": "global", "body": "x"})), "the whole design");
        assert_eq!(at(json!({"level": "page", "page": "menu/index.html", "body": "x"})), "page menu/index.html");
        assert_eq!(at(json!({"level": "element", "page": "index.html", "element_id": "hero", "body": "x"})), "section hero on index.html");
        assert_eq!(at(json!({"level": "interaction", "page": "index.html", "element_id": "order-button", "body": "x"})), "how order-button behaves on index.html");
        assert_eq!(at(json!({"level": "interaction", "page": "index.html", "body": "x"})), "how page index.html behaves");
    }

    #[test]
    fn open_feedback_reads_as_a_list_sora_can_answer_by_number() {
        let mut dealt_with = stored(1, parsed(json!({"level": "global", "body": "Warmer"})).unwrap());
        dealt_with.status = Status::Resolved;
        let open = stored(2, parsed(json!({"level": "element", "page": "index.html", "element_id": "hero", "body": "Make it\nshorter", "pin": {"x": 0.5, "y": 0.1}})).unwrap());
        let text = open_markdown(&[dealt_with, open]).unwrap();
        assert_eq!(text, "# Open feedback from the Design Studio\n- #2 (version 2, section hero on index.html, at 50% across and 10% down): Make it shorter");
        assert_eq!(open_markdown(&[]), None);
    }

    #[test]
    fn feedback_goes_to_the_studio_with_its_level_and_state_in_words() {
        let feedback = stored(3, parsed(json!({"level": "interaction", "page": "index.html", "body": "Slide in"})).unwrap());
        let sent = serde_json::to_value(&feedback).unwrap();
        assert_eq!((sent["number"].as_i64(), sent["level"].as_str(), sent["status"].as_str()), (Some(3), Some("interaction"), Some("open")));
        assert_eq!(sent["pin"], json!(null));
        assert_eq!(sent["resolved_in_version"], json!(null));
    }

    #[test]
    fn a_version_names_the_feedback_it_dealt_with_and_what_it_did() {
        let input = json!({"resolves": [{"feedback": 2, "note": "  Hero is half the height  "}, {"feedback": 3, "note": "Warmer paper colour"}]});
        assert_eq!(
            parse_resolutions(&input).unwrap(),
            [Resolution { number: 2, note: "Hero is half the height".into() }, Resolution { number: 3, note: "Warmer paper colour".into() }]
        );
        assert_eq!(parse_resolutions(&json!({})).unwrap(), []);
        assert_eq!(parse_resolutions(&json!({"resolves": null})).unwrap(), []);
        for (input, error) in [
            (json!({"resolves": "all"}), "resolves must be a list".to_string()),
            (json!({"resolves": [{"note": "x"}]}), "each of resolves names a feedback number".to_string()),
            (json!({"resolves": [{"feedback": 2}]}), "feedback #2: note says what you changed".to_string()),
            (json!({"resolves": [{"feedback": 2, "note": "x".repeat(MAX_NOTE_CHARS + 1)}]}), format!("feedback #2: note is at most {MAX_NOTE_CHARS} characters")),
            (json!({"resolves": [{"feedback": 2, "note": "a"}, {"feedback": 2, "note": "b"}]}), "feedback #2 is in resolves twice".to_string()),
        ] {
            assert_eq!(parse_resolutions(&input).unwrap_err(), error);
        }
    }

    #[test]
    fn only_feedback_still_waiting_can_be_dealt_with() {
        let mut done = stored(1, parsed(json!({"level": "global", "body": "Warmer"})).unwrap());
        done.status = Status::Resolved;
        let all = [done, stored(2, parsed(json!({"level": "global", "body": "Bigger type"})).unwrap()), stored(4, parsed(json!({"level": "global", "body": "x"})).unwrap())];
        let resolution = |number: i32| Resolution { number, note: "done".into() };
        assert_eq!(check_open(&[resolution(2), resolution(4)], &all), Ok(()));
        assert_eq!(check_open(&[], &all), Ok(()));
        assert_eq!(check_open(&[resolution(1)], &all).unwrap_err(), "feedback #1 isn't waiting: the feedback still open is #2, #4");
        assert_eq!(check_open(&[resolution(9)], &all[..1]).unwrap_err(), "feedback #9 isn't waiting: no feedback is open");
    }
}
