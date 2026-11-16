//! Checking what was built against the approved design: the handoff's colours, fonts and
//! element ids looked for in the project's own source. It finds what can be found in text (a
//! colour that was never used, a section that isn't there); it can't judge spacing or feel, and
//! says so. Each review is kept as a new version (`design_reviews`).

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::design_artifact::ID_ATTRIBUTE;
use crate::design_handoff::Package;

/// What the check says it didn't look at.
pub const NOT_CHECKED: &str = "Not checked: spacing, sizes, layout and how it feels. This only looks for the handoff's colours, fonts and element ids in the project's files; look at the running app for the rest.";
/// What a review is told when nothing can read the project's files.
pub const NOT_READABLE: &str = "the project's files can't be read here";
/// The most source files one review reads, and the largest one it reads.
pub const MAX_SOURCE_FILES: usize = 400;
pub const MAX_SOURCE_FILE_BYTES: i32 = 200_000;

/// One file of the project that was built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub content: String,
}

/// Where a review reads the built project from. Optional and replaceable: without one, a review
/// says the build can't be read.
#[async_trait]
pub trait ProjectSource: Send + Sync {
    /// The project's text files, when this person can open the project.
    async fn files(&self, user_id: Uuid, project_id: Uuid) -> Result<Vec<SourceFile>, String>;
}

pub type SharedProjectSource = Arc<dyn ProjectSource>;

pub struct NoProjectSource;

#[async_trait]
impl ProjectSource for NoProjectSource {
    async fn files(&self, _user_id: Uuid, _project_id: Uuid) -> Result<Vec<SourceFile>, String> {
        Err(NOT_READABLE.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckKind {
    Color,
    Font,
    Element,
}

/// One thing the handoff says, and where the build has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Check {
    pub kind: CheckKind,
    /// What was looked for: a colour's value, a font's name, an element's id.
    pub expected: String,
    /// What the handoff calls it.
    pub name: String,
    /// The first file that has it; `None` when no file does.
    pub found_in: Option<String>,
}

impl Check {
    fn line(&self) -> String {
        let what = match self.kind {
            CheckKind::Color => format!("Colour {} {}", self.name, self.expected),
            CheckKind::Font => format!("Font {} ({})", self.expected, self.name),
            CheckKind::Element => format!("Element id \"{}\" ({})", self.expected, self.name),
        };
        let became = match (&self.found_in, self.kind) {
            (Some(path), _) => path.clone(),
            (None, CheckKind::Color) => "not used in the project's files".to_string(),
            (None, CheckKind::Font) => "not named in the project's files".to_string(),
            (None, CheckKind::Element) => format!("no {ID_ATTRIBUTE}=\"{}\" in the project's files", self.expected),
        };
        format!("- {what}: {became}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Review {
    pub checks: Vec<Check>,
}

fn first_with(files: &[SourceFile], has: impl Fn(&str) -> bool) -> Option<String> {
    files.iter().find(|file| has(&file.content)).map(|file| file.path.clone())
}

impl Review {
    /// Looks for what `package` says in `files`, the project as it was built.
    pub fn of(package: &Package, files: &[SourceFile]) -> Self {
        let spec = &package.spec;
        let mut checks: Vec<Check> = Vec::new();
        for color in &spec.design_system.colors {
            let wanted = color.value.to_lowercase();
            checks.push(Check { kind: CheckKind::Color, expected: color.value.clone(), name: color.name.clone(), found_in: first_with(files, |content| content.to_lowercase().contains(&wanted)) });
        }
        for style in &spec.design_system.typography {
            if checks.iter().any(|c| c.kind == CheckKind::Font && c.expected == style.font) {
                continue;
            }
            let wanted = style.font.to_lowercase();
            checks.push(Check { kind: CheckKind::Font, expected: style.font.clone(), name: style.role.clone(), found_in: first_with(files, |content| content.to_lowercase().contains(&wanted)) });
        }
        let ids = spec.pages.iter().flat_map(|page| page.sections.iter()).chain(spec.components.iter().map(|c| &c.id));
        for id in ids {
            if checks.iter().any(|c| c.kind == CheckKind::Element && &c.expected == id) {
                continue;
            }
            let name = spec.components.iter().find(|c| &c.id == id).map(|c| c.name.clone()).unwrap_or_else(|| id.clone());
            let quoted = [format!("{ID_ATTRIBUTE}=\"{id}\""), format!("{ID_ATTRIBUTE}='{id}'")];
            checks.push(Check { kind: CheckKind::Element, expected: id.clone(), name, found_in: first_with(files, |content| quoted.iter().any(|q| content.contains(q))) });
        }
        Self { checks }
    }

    /// The checks the build doesn't meet.
    pub fn deviations(&self) -> Vec<&Check> {
        self.checks.iter().filter(|check| check.found_in.is_none()).collect()
    }

    /// The review as Sora reads it.
    pub fn to_markdown(&self, version: i32, handoff_version: i32) -> String {
        let deviations = self.deviations();
        let mut text = format!(
            "# Design check (version {version}, against handoff version {handoff_version})\n{} of {} checks match.\n",
            self.checks.len() - deviations.len(),
            self.checks.len()
        );
        if !deviations.is_empty() {
            text.push_str("\n## Not found in the build\n");
            for check in &deviations {
                text.push_str(&format!("{}\n", check.line()));
            }
        }
        text.push_str("\n## Found\n");
        for check in self.checks.iter().filter(|check| check.found_in.is_some()) {
            text.push_str(&format!("{}\n", check.line()));
        }
        format!("{text}\n{NOT_CHECKED}")
    }
}

/// Keeps `review` of `project_id` as the chat's next review and returns its version.
pub async fn save(conn: &mut PgConnection, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, handoff_version: i32, project_id: Uuid, review: &Review) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO design_reviews (session_id, agent_session_id, user_id, version, handoff_version, project_id, review) \
         SELECT $1, $2, $3, COALESCE(MAX(version), 0) + 1, $4, $5, $6 FROM design_reviews WHERE session_id = $1 RETURNING version",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(user_id)
    .bind(handoff_version)
    .bind(project_id)
    .bind(sqlx::types::Json(review))
    .fetch_one(conn)
    .await
}

/// The chat's newest review: its version, the handoff version it was against, and the review.
pub async fn latest(conn: &mut PgConnection, session_id: Uuid) -> Result<Option<(i32, i32, Review)>, sqlx::Error> {
    let row: Option<(i32, i32, sqlx::types::Json<Review>)> =
        sqlx::query_as("SELECT version, handoff_version, review FROM design_reviews WHERE session_id = $1 ORDER BY version DESC LIMIT 1").bind(session_id).fetch_optional(conn).await?;
    Ok(row.map(|(version, handoff_version, review)| (version, handoff_version, review.0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design_handoff::tests::spec;
    use crate::design_handoff::{Package, Spec};
    use serde_json::json;

    fn package() -> Package {
        let mut input = spec();
        input["pages"] = json!([{"path": "index.html", "purpose": "Landing", "sections": ["site-header", "hero"]}]);
        input["sitemap"] = json!([{"path": "index.html", "title": "Home", "purpose": "Landing"}]);
        let html = "<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>Home</title></head><body><header data-design-id=\"site-header\"></header><section data-design-id=\"hero\"></section></body></html>";
        let manifest = crate::design_artifact::DesignArtifact::parse(&json!({"direction_id": "editorial", "title": "Approved", "files": [{"path": "index.html", "content": html}]})).unwrap().manifest;
        Package {
            spec: Spec::parse(&input, &manifest).unwrap(),
            artifact_version: 2,
            direction_id: "editorial".into(),
            studio_link: "/chat/c1/design?version=2".into(),
            published_url: None,
            brief_version: None,
            brief: None,
            feedback: vec![],
        }
    }

    fn source(path: &str, content: &str) -> SourceFile {
        SourceFile { path: path.to_string(), content: content.to_string() }
    }

    #[test]
    fn a_build_that_kept_the_design_has_nothing_to_report() {
        let files = [
            source("src/app.css", ":root { --paper: #f6efe4; --accent: #B5532A; font-family: 'Inter', sans-serif }\nh1 { font-family: Fraunces, serif }"),
            source("src/routes/+page.svelte", "<header data-design-id=\"site-header\"></header><section data-design-id='hero'></section>"),
        ];
        let review = Review::of(&package(), &files);
        assert_eq!(review.checks.len(), 6, "two colours, two fonts, two ids");
        assert!(review.deviations().is_empty(), "{:?}", review.deviations());
        assert_eq!(review.checks[0], Check { kind: CheckKind::Color, expected: "#F6EFE4".into(), name: "Paper".into(), found_in: Some("src/app.css".into()) });
    }

    #[test]
    fn what_the_build_left_out_is_named() {
        let files = [source("src/app.css", "body { background: #FFFFFF; font-family: Inter }"), source("src/routes/+page.svelte", "<section data-design-id=\"hero\"></section><p>site-header</p>")];
        let review = Review::of(&package(), &files);
        let missing: Vec<(CheckKind, &str)> = review.deviations().iter().map(|c| (c.kind, c.expected.as_str())).collect();
        assert_eq!(missing, [(CheckKind::Color, "#F6EFE4"), (CheckKind::Color, "#B5532A"), (CheckKind::Font, "Fraunces"), (CheckKind::Element, "site-header")]);
    }

    #[test]
    fn a_review_reads_as_what_matches_what_doesnt_and_what_wasnt_checked() {
        let files = [source("src/app.css", "h1 { color: #b5532a; font-family: Fraunces } body { font-family: Inter; background: #F6EFE4 }")];
        let text = Review::of(&package(), &files).to_markdown(3, 1);
        assert!(text.starts_with("# Design check (version 3, against handoff version 1)\n4 of 6 checks match.\n\n## Not found in the build\n"), "{text}");
        assert!(text.contains("- Element id \"site-header\" (Site header): no data-design-id=\"site-header\" in the project's files\n"), "{text}");
        assert!(text.contains("## Found\n- Colour Paper #F6EFE4: src/app.css\n"));
        assert!(text.ends_with(NOT_CHECKED));
        assert!(NOT_CHECKED.contains("spacing"));
        let clean = [source("a.css", "#F6EFE4 #B5532A Fraunces Inter"), source("a.svelte", "data-design-id=\"site-header\" data-design-id=\"hero\"")];
        assert!(Review::of(&package(), &clean).to_markdown(1, 1).contains("6 of 6 checks match.\n\n## Found\n"));
    }

    #[test]
    fn a_review_survives_being_stored() {
        let review = Review::of(&package(), &[]);
        let stored = serde_json::to_value(&review).unwrap();
        assert_eq!(stored["checks"][0]["kind"], "color");
        assert_eq!(serde_json::from_value::<Review>(stored).unwrap(), review);
    }

    #[tokio::test]
    async fn without_a_source_the_build_cant_be_read() {
        assert_eq!(NoProjectSource.files(Uuid::nil(), Uuid::nil()).await.unwrap_err(), NOT_READABLE);
    }
}
