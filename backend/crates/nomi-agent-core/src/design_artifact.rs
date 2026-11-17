//! A design preview: the static pages (HTML, CSS, JS, SVG) Sora builds for one direction, with a
//! manifest of its pages and of the elements feedback can point at. Elements carry a stable
//! `data-design-id`, kept from one version to the next. Each save is a new, immutable version
//! of the chat's previews (`design_artifacts`, `design_artifact_files`).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Connection, PgConnection};
use uuid::Uuid;

use crate::design_directions::slug;

/// The attribute an element is pointed at by.
pub const ID_ATTRIBUTE: &str = "data-design-id";
/// The page a preview opens on.
pub const ENTRY_PAGE: &str = "index.html";
/// The most files in one preview.
pub const MAX_FILES: usize = 20;
/// The largest one file may be, and all of them together.
pub const MAX_FILE_BYTES: usize = 200_000;
pub const MAX_TOTAL_BYTES: usize = 600_000;
/// The most characters in a path, a title or a label.
pub const MAX_PATH_CHARS: usize = 120;
pub const MAX_LABEL_CHARS: usize = 120;
/// The most elements one preview's manifest lists.
pub const MAX_ELEMENTS: usize = 300;
/// The characters in a version's preview token.
pub const PREVIEW_TOKEN_CHARS: usize = 64;
/// The file kinds a preview holds, each with what it is served as.
const KINDS: [(&str, &str); 4] =
    [("html", "text/html; charset=utf-8"), ("css", "text/css; charset=utf-8"), ("js", "text/javascript; charset=utf-8"), ("svg", "image/svg+xml")];
const HTML: &str = "html";

/// What an element is, for feedback about a section, a component or one element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementKind {
    Section,
    Component,
    Element,
}

impl ElementKind {
    const ALL: [(&'static str, ElementKind); 3] = [("section", Self::Section), ("component", Self::Component), ("element", Self::Element)];

    fn name(self) -> &'static str {
        Self::ALL.iter().find(|(_, kind)| *kind == self).map(|(name, _)| *name).unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    pub path: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Element {
    /// The value of its `data-design-id`: the same in every page and every later version.
    pub id: String,
    pub kind: ElementKind,
    pub label: String,
    /// The pages it is on.
    pub pages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactFile {
    pub path: String,
    pub content: String,
}

/// What a preview is, without its files' contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// The direction (`design_directions::Direction::id`) the preview is of.
    pub direction_id: String,
    pub title: String,
    pub pages: Vec<Page>,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignArtifact {
    pub manifest: Manifest,
    pub files: Vec<ArtifactFile>,
}

fn text_of<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty())
}

fn extension(path: &str) -> Option<&str> {
    path.rsplit('/').next()?.rsplit_once('.').map(|(_, extension)| extension)
}

/// What a file at `path` is served as; `None` when a preview can't hold it.
pub fn content_type_of(path: &str) -> Option<&'static str> {
    let extension = extension(path)?;
    KINDS.iter().find(|(kind, _)| *kind == extension).map(|(_, content_type)| *content_type)
}

fn is_html(path: &str) -> bool {
    extension(path) == Some(HTML)
}

/// Whether `path` is a plain relative path: lowercase segments, none empty, hidden or `..`.
fn inside_the_preview(path: &str) -> bool {
    path.chars().count() <= MAX_PATH_CHARS
        && path.split('/').all(|segment| {
            !segment.is_empty() && !segment.starts_with('.') && segment.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-' | '_'))
        })
}

fn check_path(path: &str) -> Result<(), String> {
    if !inside_the_preview(path) {
        return Err(format!("\"{path}\" isn't a path inside the preview"));
    }
    if content_type_of(path).is_none() {
        let kinds: Vec<&str> = KINDS.iter().map(|(kind, _)| *kind).collect();
        return Err(format!("\"{path}\" isn't a file a preview can hold ({})", kinds.join(", ")));
    }
    Ok(())
}

/// The ids on a page's elements, in order. A mention that isn't an attribute of a tag (a CSS
/// selector, say) is not one.
fn ids_in(path: &str, html: &str) -> Result<Vec<String>, String> {
    let mut ids: Vec<String> = Vec::new();
    let mut rest = html;
    while let Some(at) = rest.find(ID_ATTRIBUTE) {
        let on_a_tag = rest[..at].chars().next_back().is_some_and(char::is_whitespace);
        rest = &rest[at + ID_ATTRIBUTE.len()..];
        let Some(value) = rest.trim_start().strip_prefix('=').filter(|_| on_a_tag) else { continue };
        let value = value.trim_start();
        let quoted = value.chars().next().filter(|c| matches!(c, '"' | '\'')).and_then(|quote| value[1..].split_once(quote));
        let Some((id, after)) = quoted else {
            return Err(format!("{path}: a {ID_ATTRIBUTE} needs a quoted value"));
        };
        if id.is_empty() || id != slug(id) {
            return Err(format!("{path}: the id \"{id}\" must be lowercase letters, digits and dashes, like \"{}\"", slug(id)));
        }
        if ids.iter().any(|seen| seen == id) {
            return Err(format!("{path} uses the id \"{id}\" twice"));
        }
        ids.push(id.to_string());
        rest = after;
    }
    Ok(ids)
}

/// A page's `<title>`, or its path when it has none.
fn title_of(path: &str, html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let title = lower.find("<title>").map(|at| at + "<title>".len()).and_then(|start| lower[start..].find("</title>").map(|len| html[start..start + len].trim()));
    title.filter(|t| !t.is_empty()).unwrap_or(path).chars().take(MAX_LABEL_CHARS).collect()
}

fn has_viewport(html: &str) -> bool {
    let lower = html.to_ascii_lowercase();
    lower.contains("name=\"viewport\"") || lower.contains("name='viewport'")
}

/// The files of one `save_preview` call, each checked on its own: a piece of a preview may leave
/// out the entry page and the rest, which the whole preview is checked for when it is finished.
pub fn parse_file_list(input: &Value) -> Result<Vec<ArtifactFile>, String> {
    let list = input.get("files").and_then(Value::as_array).ok_or("files must be a list")?;
    if list.len() > MAX_FILES {
        return Err(format!("a preview has at most {MAX_FILES} files"));
    }
    let mut files: Vec<ArtifactFile> = Vec::new();
    for file in list {
        let path = text_of(file, "path").ok_or("each file needs a path")?;
        check_path(path)?;
        if files.iter().any(|f| f.path == path) {
            return Err(format!("\"{path}\" is in the files twice"));
        }
        let content = file.get("content").and_then(Value::as_str).ok_or_else(|| format!("\"{path}\" needs its content"))?;
        if content.len() > MAX_FILE_BYTES {
            return Err(format!("\"{path}\" is larger than {MAX_FILE_BYTES} bytes"));
        }
        files.push(ArtifactFile { path: path.to_string(), content: content.to_string() });
    }
    Ok(files)
}

/// The paths a finished `save_preview` call drops from the last version; none when left out.
pub fn removals(input: &Value) -> Result<Vec<String>, String> {
    let Some(list) = input.get("remove") else { return Ok(Vec::new()) };
    let list = list.as_array().ok_or("remove must be a list of paths")?;
    list.iter()
        .map(|path| {
            let path = path.as_str().ok_or("remove must be a list of paths")?;
            check_path(path)?;
            Ok(path.to_string())
        })
        .collect()
}

/// The checks only a whole preview has to pass: how many files, how large together, and its entry page.
fn check_whole(files: &[ArtifactFile]) -> Result<(), String> {
    if files.len() > MAX_FILES {
        return Err(format!("a preview has at most {MAX_FILES} files"));
    }
    let total: usize = files.iter().map(|f| f.content.len()).sum();
    if total > MAX_TOTAL_BYTES {
        return Err(format!("the preview's files are larger than {MAX_TOTAL_BYTES} bytes together"));
    }
    if !files.iter().any(|f| f.path == ENTRY_PAGE) {
        return Err(format!("a preview needs an {ENTRY_PAGE}"));
    }
    Ok(())
}

/// How Sora described an element: its kind and label.
fn described(input: &Value) -> Result<Vec<(String, ElementKind, Option<String>)>, String> {
    let list = input.get("elements").and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default();
    list.iter()
        .map(|element| {
            let id = text_of(element, "id").ok_or("each element needs an id")?.to_string();
            let kind = match text_of(element, "kind") {
                None => ElementKind::Element,
                Some(name) => ElementKind::ALL.iter().find(|(known, _)| *known == name).map(|(_, kind)| *kind).ok_or_else(|| format!("{id}: kind is section, component or element"))?,
            };
            Ok((id, kind, text_of(element, "label").map(|label| label.chars().take(MAX_LABEL_CHARS).collect())))
        })
        .collect()
}

impl DesignArtifact {
    /// Reads and checks a preview as the `save_preview` tool takes it. The error says what to fix.
    pub fn parse(input: &Value) -> Result<Self, String> {
        Self::from_files(input, parse_file_list(input)?)
    }

    /// Checks `files` as a whole preview, with the title, direction and element names from `input`.
    pub fn from_files(input: &Value, files: Vec<ArtifactFile>) -> Result<Self, String> {
        check_whole(&files)?;
        let title: String = text_of(input, "title").ok_or("title is required")?.chars().take(MAX_LABEL_CHARS).collect();
        let direction_id = text_of(input, "direction_id").ok_or("direction_id is required")?.to_lowercase();
        if direction_id != slug(&direction_id) {
            return Err(format!("direction_id must be a direction's id, like \"{}\"", slug(&direction_id)));
        }
        let mut pages = Vec::new();
        let mut elements: Vec<Element> = Vec::new();
        for file in files.iter().filter(|f| is_html(&f.path)) {
            if !has_viewport(&file.content) {
                return Err(format!("{} has no viewport meta tag, so it wouldn't adapt to a phone", file.path));
            }
            let ids = ids_in(&file.path, &file.content)?;
            if ids.is_empty() {
                return Err(format!("{} has no {ID_ATTRIBUTE} attributes: give each section and component one", file.path));
            }
            for id in ids {
                match elements.iter_mut().find(|e| e.id == id) {
                    Some(element) => element.pages.push(file.path.clone()),
                    None => elements.push(Element { label: id.clone(), id, kind: ElementKind::Element, pages: vec![file.path.clone()] }),
                }
            }
            pages.push(Page { path: file.path.clone(), title: title_of(&file.path, &file.content) });
        }
        if elements.len() > MAX_ELEMENTS {
            return Err(format!("a preview has at most {MAX_ELEMENTS} ids"));
        }
        for (id, kind, label) in described(input)? {
            let element = elements.iter_mut().find(|e| e.id == id).ok_or_else(|| format!("elements describes \"{id}\", which no page has"))?;
            element.kind = kind;
            if let Some(label) = label {
                element.label = label;
            }
        }
        Ok(Self { manifest: Manifest { direction_id, title, pages, elements }, files })
    }

    /// Keeps what `earlier` said an element is (its kind and label) for every element this version
    /// still has and `input` doesn't describe again: a change that sends only some files keeps the rest as they were.
    pub fn inherit_descriptions(&mut self, earlier: &Manifest, input: &Value) {
        let described_now: Vec<&str> =
            input.get("elements").and_then(Value::as_array).map(|list| list.iter().filter_map(|e| text_of(e, "id")).collect()).unwrap_or_default();
        for element in self.manifest.elements.iter_mut().filter(|e| !described_now.contains(&e.id.as_str())) {
            if let Some(before) = earlier.elements.iter().find(|e| e.id == element.id) {
                element.kind = before.kind;
                element.label = before.label.clone();
            }
        }
    }

    pub fn file(&self, path: &str) -> Option<&ArtifactFile> {
        self.files.iter().find(|f| f.path == path)
    }
}

impl Manifest {
    /// The ids `earlier` had that this one no longer has: feedback on them has nowhere to point.
    pub fn missing_since(&self, earlier: &Manifest) -> Vec<String> {
        earlier.elements.iter().filter(|before| !self.elements.iter().any(|now| now.id == before.id)).map(|e| e.id.clone()).collect()
    }

    /// The preview as an agent reads it.
    pub fn to_markdown(&self, version: i32) -> String {
        let mut text = format!("# Design preview (version {version}, direction: {})\nTitle: {}\nPages:\n", self.direction_id, self.title);
        for page in &self.pages {
            text.push_str(&format!("- {}: {}\n", page.path, page.title));
        }
        text.push_str("Element ids:\n");
        for element in &self.elements {
            text.push_str(&format!("- {} ({}, \"{}\"): {}\n", element.id, element.kind.name(), element.label, element.pages.join(", ")));
        }
        text.trim_end().to_string()
    }
}

/// The input schema of the tool that saves a preview.
pub fn artifact_input_schema() -> Value {
    let kinds: Vec<&str> = ElementKind::ALL.iter().map(|(name, _)| *name).collect();
    json!({
        "type": "object",
        "properties": {
            "direction_id": {"type": "string", "description": "The id of the direction this is a preview of"},
            "title": {"type": "string", "description": "What this version is, in a few words"},
            "files": {
                "type": "array",
                "minItems": 1,
                "maxItems": MAX_FILES,
                "description": "Every file of the preview, whole: index.html first, then other pages, css, js and svg. Relative lowercase paths",
                "items": {"type": "object", "properties": {"path": {"type": "string"}, "content": {"type": "string"}}, "required": ["path", "content"]}
            },
            "elements": {
                "type": "array",
                "description": "What the ids in the pages are, for the ones worth naming",
                "items": {
                    "type": "object",
                    "properties": {"id": {"type": "string"}, "kind": {"type": "string", "enum": kinds}, "label": {"type": "string"}},
                    "required": ["id"]
                }
            },
            "resolves": crate::design_feedback::resolves_schema(),
            "finished": {
                "type": "boolean",
                "description": "false keeps these files in a draft the person doesn't see yet, to send the rest in later calls; \
                                true (the default) saves the version from the last version of this direction, the draft and these files"
            },
            "remove": {
                "type": "array",
                "description": "Paths to drop from the last version when this call finishes it",
                "items": {"type": "string"}
            }
        },
        "required": ["direction_id", "title", "files"]
    })
}

/// Saves `artifact` as the chat's next preview version, made from `directions_version` of the
/// directions, and returns its version. Earlier versions are never changed.
pub async fn save(conn: &mut PgConnection, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, directions_version: Option<i32>, artifact: &DesignArtifact) -> Result<i32, sqlx::Error> {
    let mut tx = conn.begin().await?;
    let (id, version): (Uuid, i32) = sqlx::query_as(
        "INSERT INTO design_artifacts (session_id, agent_session_id, user_id, direction_id, directions_version, version, manifest) \
         SELECT $1, $2, $3, $4, $5, COALESCE(MAX(version), 0) + 1, $6 FROM design_artifacts WHERE session_id = $1 \
         RETURNING id, version",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(user_id)
    .bind(&artifact.manifest.direction_id)
    .bind(directions_version)
    .bind(sqlx::types::Json(&artifact.manifest))
    .fetch_one(&mut *tx)
    .await?;
    for file in &artifact.files {
        sqlx::query("INSERT INTO design_artifact_files (artifact_id, path, content_type, content) VALUES ($1, $2, $3, $4)")
            .bind(id)
            .bind(&file.path)
            .bind(content_type_of(&file.path).unwrap_or_default())
            .bind(&file.content)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(version)
}

/// The chat's newest preview: its version and manifest.
pub async fn latest(conn: &mut PgConnection, session_id: Uuid) -> Result<Option<(i32, Manifest)>, sqlx::Error> {
    let row: Option<(i32, sqlx::types::Json<Manifest>)> =
        sqlx::query_as("SELECT version, manifest FROM design_artifacts WHERE session_id = $1 ORDER BY version DESC LIMIT 1").bind(session_id).fetch_optional(conn).await?;
    Ok(row.map(|(version, manifest)| (version, manifest.0)))
}

/// The chat's newest preview of one direction.
pub async fn latest_of_direction(conn: &mut PgConnection, session_id: Uuid, direction_id: &str) -> Result<Option<(i32, Manifest)>, sqlx::Error> {
    let row: Option<(i32, sqlx::types::Json<Manifest>)> =
        sqlx::query_as("SELECT version, manifest FROM design_artifacts WHERE session_id = $1 AND direction_id = $2 ORDER BY version DESC LIMIT 1")
            .bind(session_id)
            .bind(direction_id)
            .fetch_optional(conn)
            .await?;
    Ok(row.map(|(version, manifest)| (version, manifest.0)))
}

/// One version of the chat's previews.
pub async fn version(conn: &mut PgConnection, session_id: Uuid, version: i32) -> Result<Option<Manifest>, sqlx::Error> {
    let row: Option<(sqlx::types::Json<Manifest>,)> =
        sqlx::query_as("SELECT manifest FROM design_artifacts WHERE session_id = $1 AND version = $2").bind(session_id).bind(version).fetch_optional(conn).await?;
    Ok(row.map(|(manifest,)| manifest.0))
}

/// Every file of one version, by path.
pub async fn files(conn: &mut PgConnection, session_id: Uuid, version: i32) -> Result<Vec<ArtifactFile>, sqlx::Error> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT f.path, f.content FROM design_artifact_files f JOIN design_artifacts a ON a.id = f.artifact_id \
         WHERE a.session_id = $1 AND a.version = $2 ORDER BY f.path",
    )
    .bind(session_id)
    .bind(version)
    .fetch_all(conn)
    .await?;
    Ok(rows.into_iter().map(|(path, content)| ArtifactFile { path, content }).collect())
}

/// One stored version, as the Design Studio lists it.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredVersion {
    pub version: i32,
    pub manifest: Manifest,
    /// The unguessable name its files are served under (see `file_by_token`).
    pub preview_token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Where it was published last, when it was.
    pub published_url: Option<String>,
    /// Why it isn't published, when its newest publish failed.
    pub publish_error: Option<String>,
}

/// Every version of the chat's previews, oldest first.
pub async fn list(conn: &mut PgConnection, session_id: Uuid) -> Result<Vec<StoredVersion>, sqlx::Error> {
    type Row = (i32, sqlx::types::Json<Manifest>, String, chrono::DateTime<chrono::Utc>, Option<String>, Option<String>);
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT a.version, a.manifest, a.preview_token, a.created_at, \
                (SELECT p.url FROM design_artifact_publications p WHERE p.artifact_id = a.id ORDER BY p.created_at DESC, p.id DESC LIMIT 1), \
                (SELECT f.reason FROM design_artifact_publish_failures f WHERE f.artifact_id = a.id \
                   AND f.created_at >= COALESCE((SELECT MAX(p.created_at) FROM design_artifact_publications p WHERE p.artifact_id = a.id), '-infinity') \
                 ORDER BY f.created_at DESC, f.id DESC LIMIT 1) \
         FROM design_artifacts a WHERE a.session_id = $1 ORDER BY a.version",
    )
    .bind(session_id)
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(version, manifest, preview_token, created_at, published_url, publish_error)| StoredVersion {
            version,
            manifest: manifest.0,
            preview_token,
            created_at,
            published_url,
            publish_error,
        })
        .collect())
}

/// One file of the version named by `token`: what it is served as, and its content. The token
/// is the whole permission, so nothing else about the asker is checked.
pub async fn file_by_token(conn: &mut PgConnection, token: &str, path: &str) -> Result<Option<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT f.content_type, f.content FROM design_artifact_files f JOIN design_artifacts a ON a.id = f.artifact_id \
         WHERE a.preview_token = $1 AND f.path = $2",
    )
    .bind(token)
    .bind(path)
    .fetch_optional(conn)
    .await
}

/// One file of one version: what it is served as, and its content.
pub async fn file(conn: &mut PgConnection, session_id: Uuid, version: i32, path: &str) -> Result<Option<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT f.content_type, f.content FROM design_artifact_files f JOIN design_artifacts a ON a.id = f.artifact_id \
         WHERE a.session_id = $1 AND a.version = $2 AND f.path = $3",
    )
    .bind(session_id)
    .bind(version)
    .bind(path)
    .fetch_optional(conn)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn page(title: &str, body: &str) -> String {
        format!(
            "<!doctype html><html><head><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
             <title>{title}</title><link rel=\"stylesheet\" href=\"styles.css\"></head><body>{body}</body></html>"
        )
    }

    fn home() -> String {
        page("Roti Pagi", "<header data-design-id=\"site-header\">Roti Pagi</header><section data-design-id='hero'><h1>Fresh at six</h1></section>")
    }

    fn preview() -> Value {
        json!({
            "direction_id": "Editorial",
            "title": "Editorial, first pass",
            "files": [
                {"path": "index.html", "content": home()},
                {"path": "styles.css", "content": "body { margin: 0 }"},
                {"path": "menu/index.html", "content": page("Menu", "<header data-design-id=\"site-header\">Roti Pagi</header><ul data-design-id=\"menu-list\"></ul>")}
            ],
            "elements": [{"id": "hero", "kind": "section", "label": "Hero"}, {"id": "site-header", "kind": "component", "label": "Site header"}]
        })
    }

    fn with_file(path: &str, content: &str) -> Value {
        let mut input = preview();
        input["files"].as_array_mut().unwrap().push(json!({"path": path, "content": content}));
        input
    }

    #[test]
    fn a_preview_is_read_with_its_pages_and_element_ids() {
        let artifact = DesignArtifact::parse(&preview()).unwrap();
        let manifest = &artifact.manifest;
        assert_eq!(manifest.direction_id, "editorial");
        assert_eq!(manifest.title, "Editorial, first pass");
        assert_eq!(manifest.pages, [Page { path: "index.html".into(), title: "Roti Pagi".into() }, Page { path: "menu/index.html".into(), title: "Menu".into() }]);
        assert_eq!(
            manifest.elements,
            [
                Element { id: "site-header".into(), kind: ElementKind::Component, label: "Site header".into(), pages: vec!["index.html".into(), "menu/index.html".into()] },
                Element { id: "hero".into(), kind: ElementKind::Section, label: "Hero".into(), pages: vec!["index.html".into()] },
                Element { id: "menu-list".into(), kind: ElementKind::Element, label: "menu-list".into(), pages: vec!["menu/index.html".into()] },
            ]
        );
        assert_eq!(artifact.files.len(), 3);
        assert_eq!(artifact.file("styles.css").map(|f| f.content.as_str()), Some("body { margin: 0 }"));
        assert_eq!(artifact.file("nope.css"), None);
    }

    #[test]
    fn a_preview_starts_at_its_index_page() {
        let mut input = preview();
        input["files"].as_array_mut().unwrap().remove(0);
        assert_eq!(DesignArtifact::parse(&input).unwrap_err(), "a preview needs an index.html");
        assert_eq!(DesignArtifact::parse(&json!({"direction_id": "editorial", "title": "x"})).unwrap_err(), "files must be a list");
        let mut untitled = preview();
        untitled["title"] = json!(" ");
        assert_eq!(DesignArtifact::parse(&untitled).unwrap_err(), "title is required");
        let mut no_direction = preview();
        no_direction["direction_id"] = json!("Warm one!");
        assert_eq!(DesignArtifact::parse(&no_direction).unwrap_err(), "direction_id must be a direction's id, like \"warm-one\"");
    }

    #[test]
    fn file_paths_stay_inside_the_preview_and_are_of_known_kinds() {
        for (path, error) in [
            ("../secrets.html", "\"../secrets.html\" isn't a path inside the preview"),
            ("/etc/index.html", "\"/etc/index.html\" isn't a path inside the preview"),
            ("a//b.css", "\"a//b.css\" isn't a path inside the preview"),
            (".env.js", "\".env.js\" isn't a path inside the preview"),
            ("My Page.html", "\"My Page.html\" isn't a path inside the preview"),
            ("server.php", "\"server.php\" isn't a file a preview can hold (html, css, js, svg)"),
            ("readme", "\"readme\" isn't a file a preview can hold (html, css, js, svg)"),
        ] {
            assert_eq!(DesignArtifact::parse(&with_file(path, "x")).unwrap_err(), error, "{path}");
        }
        assert_eq!(DesignArtifact::parse(&with_file("styles.css", "again")).unwrap_err(), "\"styles.css\" is in the files twice");
        assert!(DesignArtifact::parse(&with_file("img/logo.svg", "<svg/>")).is_ok());
    }

    #[test]
    fn a_piece_needs_no_index_page() {
        let files = parse_file_list(&json!({"files": [{"path": "styles.css", "content": "h1{}"}]})).unwrap();
        assert_eq!(files, vec![ArtifactFile { path: "styles.css".into(), content: "h1{}".into() }]);
    }

    #[test]
    fn a_piece_still_checks_each_file() {
        let twice = json!({"files": [{"path": "a.css", "content": ""}, {"path": "a.css", "content": ""}]});
        assert_eq!(parse_file_list(&twice).unwrap_err(), "\"a.css\" is in the files twice");
        let big = json!({"files": [{"path": "a.css", "content": "x".repeat(MAX_FILE_BYTES + 1)}]});
        assert_eq!(parse_file_list(&big).unwrap_err(), format!("\"a.css\" is larger than {MAX_FILE_BYTES} bytes"));
    }

    #[test]
    fn removals_are_optional_paths() {
        assert_eq!(removals(&json!({})).unwrap(), Vec::<String>::new());
        assert_eq!(removals(&json!({"remove": ["about.html"]})).unwrap(), vec!["about.html".to_string()]);
        assert!(removals(&json!({"remove": ["../x.html"]})).is_err());
    }

    #[test]
    fn from_files_checks_the_whole_preview() {
        let only_css = vec![ArtifactFile { path: "styles.css".into(), content: "h1{}".into() }];
        assert_eq!(DesignArtifact::from_files(&preview(), only_css).unwrap_err(), format!("a preview needs an {ENTRY_PAGE}"));
        let too_many = (0..=MAX_FILES).map(|n| ArtifactFile { path: format!("s{n}.css"), content: String::new() }).collect();
        assert_eq!(DesignArtifact::from_files(&preview(), too_many).unwrap_err(), format!("a preview has at most {MAX_FILES} files"));
    }

    #[test]
    fn previews_are_limited_in_size() {
        let big = "x".repeat(MAX_FILE_BYTES + 1);
        assert_eq!(DesignArtifact::parse(&with_file("big.js", &big)).unwrap_err(), format!("\"big.js\" is larger than {MAX_FILE_BYTES} bytes"));
        let mut many = preview();
        for n in 0..MAX_FILES {
            many["files"].as_array_mut().unwrap().push(json!({"path": format!("s{n}.css"), "content": ""}));
        }
        assert_eq!(DesignArtifact::parse(&many).unwrap_err(), format!("a preview has at most {MAX_FILES} files"));
        let mut heavy = preview();
        let chunk = "x".repeat(MAX_FILE_BYTES);
        for n in 0..=(MAX_TOTAL_BYTES / MAX_FILE_BYTES) {
            heavy["files"].as_array_mut().unwrap().push(json!({"path": format!("s{n}.js"), "content": chunk}));
        }
        assert_eq!(DesignArtifact::parse(&heavy).unwrap_err(), format!("the preview's files are larger than {MAX_TOTAL_BYTES} bytes together"));
    }

    #[test]
    fn every_page_is_responsive_and_has_ids_to_point_at() {
        let fixed = "<html><head><title>T</title></head><body><p data-design-id=\"p\"></p></body></html>";
        assert_eq!(
            DesignArtifact::parse(&with_file("about.html", fixed)).unwrap_err(),
            "about.html has no viewport meta tag, so it wouldn't adapt to a phone"
        );
        assert_eq!(
            DesignArtifact::parse(&with_file("about.html", &page("About", "<p>Hi</p>"))).unwrap_err(),
            "about.html has no data-design-id attributes: give each section and component one"
        );
    }

    #[test]
    fn ids_are_slugs_and_appear_once_in_a_page() {
        let twice = page("About", "<p data-design-id=\"intro\"></p><p data-design-id=\"intro\"></p>");
        assert_eq!(DesignArtifact::parse(&with_file("about.html", &twice)).unwrap_err(), "about.html uses the id \"intro\" twice");
        let shouty = page("About", "<p data-design-id=\"Intro Text\"></p>");
        assert_eq!(
            DesignArtifact::parse(&with_file("about.html", &shouty)).unwrap_err(),
            "about.html: the id \"Intro Text\" must be lowercase letters, digits and dashes, like \"intro-text\""
        );
        let unquoted = page("About", "<p data-design-id=intro></p>");
        assert_eq!(DesignArtifact::parse(&with_file("about.html", &unquoted)).unwrap_err(), "about.html: a data-design-id needs a quoted value");
        let styled = page("About", "<style>[data-design-id=\"intro\"] { color: red }</style><p data-design-id=\"intro\"></p>");
        assert!(DesignArtifact::parse(&with_file("about.html", &styled)).is_ok(), "a selector is not an element");
    }

    #[test]
    fn described_elements_must_be_in_the_pages() {
        let mut input = preview();
        input["elements"] = json!([{"id": "footer", "kind": "section", "label": "Footer"}]);
        assert_eq!(DesignArtifact::parse(&input).unwrap_err(), "elements describes \"footer\", which no page has");
        input["elements"] = json!([{"id": "hero", "kind": "banner", "label": "Hero"}]);
        assert_eq!(DesignArtifact::parse(&input).unwrap_err(), "hero: kind is section, component or element");
        input["elements"] = json!([{"id": "hero"}]);
        let hero = DesignArtifact::parse(&input).unwrap().manifest.elements.into_iter().find(|e| e.id == "hero").unwrap();
        assert_eq!((hero.kind, hero.label.as_str()), (ElementKind::Element, "hero"));
        input.as_object_mut().unwrap().remove("elements");
        assert_eq!(DesignArtifact::parse(&input).unwrap().manifest.elements.len(), 3);
    }

    #[test]
    fn a_page_without_a_title_is_known_by_its_path() {
        let untitled = "<html><head><meta name='viewport' content='width=device-width'></head><body><p data-design-id=\"p\"></p></body></html>";
        let artifact = DesignArtifact::parse(&with_file("about.html", untitled)).unwrap();
        assert_eq!(artifact.manifest.pages.last().unwrap().title, "about.html");
    }

    #[test]
    fn files_are_served_as_what_they_are() {
        for (path, content_type) in [
            ("index.html", "text/html; charset=utf-8"),
            ("a/styles.css", "text/css; charset=utf-8"),
            ("app.js", "text/javascript; charset=utf-8"),
            ("logo.svg", "image/svg+xml"),
        ] {
            assert_eq!(content_type_of(path), Some(content_type), "{path}");
        }
        assert_eq!(content_type_of("notes.txt"), None);
    }

    #[test]
    fn ids_that_went_missing_since_the_last_version_are_named() {
        let before = DesignArtifact::parse(&preview()).unwrap().manifest;
        let mut input = preview();
        input["files"][0]["content"] = json!(page("Roti Pagi", "<header data-design-id=\"site-header\"></header><section data-design-id=\"intro\"></section>"));
        input.as_object_mut().unwrap().remove("elements");
        let after = DesignArtifact::parse(&input).unwrap().manifest;
        assert_eq!(after.missing_since(&before), ["hero"]);
        assert!(before.missing_since(&before).is_empty());
    }

    #[test]
    fn the_manifest_survives_being_stored_and_reads_as_markdown() {
        let manifest = DesignArtifact::parse(&preview()).unwrap().manifest;
        let stored = serde_json::to_value(&manifest).unwrap();
        assert_eq!(stored["elements"][0]["kind"], "component");
        assert_eq!(serde_json::from_value::<Manifest>(stored).unwrap(), manifest);
        let text = manifest.to_markdown(3);
        assert!(text.starts_with("# Design preview (version 3, direction: editorial)\nTitle: Editorial, first pass"), "{text}");
        assert!(text.contains("Pages:\n- index.html: Roti Pagi\n- menu/index.html: Menu"));
        assert!(text.contains("- site-header (component, \"Site header\"): index.html, menu/index.html"));
    }

    #[test]
    fn the_tool_schema_asks_for_files_and_the_direction() {
        let schema = artifact_input_schema();
        assert_eq!(schema["required"], json!(["direction_id", "title", "files"]));
        assert_eq!(schema["properties"]["files"]["maxItems"].as_u64(), Some(MAX_FILES as u64));
        assert_eq!(schema["properties"]["elements"]["items"]["properties"]["kind"]["enum"], json!(["section", "component", "element"]));
    }

    #[test]
    fn descriptions_carry_over_to_a_version_that_does_not_repeat_them() {
        let mut described = preview();
        described["elements"] = json!([{"id": "site-header", "kind": "component", "label": "Site header"}, {"id": "menu-list", "kind": "section", "label": "Menu"}]);
        let earlier = DesignArtifact::parse(&described).unwrap().manifest;

        let mut next = preview();
        next["elements"] = json!([{"id": "menu-list", "kind": "component", "label": "Menu list"}]);
        let mut artifact = DesignArtifact::parse(&next).unwrap();
        artifact.inherit_descriptions(&earlier, &next);
        let of = |id: &str| artifact.manifest.elements.iter().find(|e| e.id == id).unwrap().clone();
        assert_eq!((of("site-header").kind, of("site-header").label), (ElementKind::Component, "Site header".to_string()), "kept from before");
        assert_eq!((of("menu-list").kind, of("menu-list").label), (ElementKind::Component, "Menu list".to_string()), "described again: the new one wins");
    }

    #[test]
    fn the_tool_schema_offers_finished_and_remove() {
        let schema = artifact_input_schema();
        assert_eq!(schema["properties"]["finished"]["type"], "boolean");
        assert_eq!(schema["properties"]["remove"]["items"]["type"], "string");
        assert_eq!(schema["required"], json!(["direction_id", "title", "files"]), "both are optional");
    }
}
