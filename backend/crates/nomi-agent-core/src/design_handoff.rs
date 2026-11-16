//! The package Koda builds from: what Sora writes down about an approved design (its design
//! system, sitemap, flows, pages, components, interactions, responsive rules, accessibility,
//! assets and decisions), together with the brief it came from, what became of the feedback,
//! and the approved preview's version and address. Each handoff is a new version
//! (`design_handoffs`); a saved one is never changed.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::design_artifact::Manifest;
use crate::design_brief::DesignBrief;
use crate::design_feedback::Feedback;

/// The most bytes in what Sora writes for one handoff.
pub const MAX_PACKAGE_BYTES: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub name: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeStyle {
    pub role: String,
    pub font: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesignSystem {
    pub colors: Vec<Color>,
    pub typography: Vec<TypeStyle>,
    #[serde(default)]
    pub spacing: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SitemapEntry {
    pub path: String,
    pub title: String,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flow {
    pub name: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageSpec {
    pub path: String,
    pub purpose: String,
    /// The `data-design-id`s of its sections, top to bottom.
    pub sections: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentSpec {
    /// Its `data-design-id` in the approved preview.
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub states: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interaction {
    pub target: String,
    pub trigger: String,
    pub result: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub kind: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub decision: String,
    pub reason: String,
}

/// What Sora writes down about the approved design.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec {
    pub design_system: DesignSystem,
    pub sitemap: Vec<SitemapEntry>,
    pub user_flows: Vec<Flow>,
    pub pages: Vec<PageSpec>,
    pub components: Vec<ComponentSpec>,
    #[serde(default)]
    pub interactions: Vec<Interaction>,
    pub responsive: Vec<String>,
    pub accessibility: Vec<String>,
    #[serde(default)]
    pub assets: Vec<Asset>,
    pub decisions: Vec<Decision>,
}

fn filled(part: &str, count: usize) -> Result<(), String> {
    if count == 0 {
        return Err(format!("{part} needs at least one entry"));
    }
    Ok(())
}

impl Spec {
    /// Reads a handoff and checks it against `approved`, the approved version's manifest: every
    /// page of the version is described, and nothing is described that the version doesn't have.
    pub fn parse(input: &Value, approved: &Manifest) -> Result<Self, String> {
        if input.to_string().len() > MAX_PACKAGE_BYTES {
            return Err(format!("the handoff is larger than {MAX_PACKAGE_BYTES} bytes: say it more briefly"));
        }
        let spec: Spec = serde_json::from_value(input.clone()).map_err(|e| format!("the handoff isn't complete: {e}"))?;
        for (part, count) in [
            ("design_system.colors", spec.design_system.colors.len()),
            ("design_system.typography", spec.design_system.typography.len()),
            ("sitemap", spec.sitemap.len()),
            ("user_flows", spec.user_flows.len()),
            ("pages", spec.pages.len()),
            ("components", spec.components.len()),
            ("responsive", spec.responsive.len()),
            ("accessibility", spec.accessibility.len()),
            ("decisions", spec.decisions.len()),
        ] {
            filled(part, count)?;
        }
        for page in &spec.pages {
            if !approved.pages.iter().any(|p| p.path == page.path) {
                return Err(format!("pages describes {}, which the approved version doesn't have", page.path));
            }
            for id in &page.sections {
                if !approved.elements.iter().any(|e| &e.id == id && e.pages.contains(&page.path)) {
                    return Err(format!("{} has no element \"{id}\" in the approved version", page.path));
                }
            }
        }
        if let Some(missing) = approved.pages.iter().find(|p| !spec.pages.iter().any(|page| page.path == p.path)) {
            return Err(format!("pages doesn't describe {}, a page of the approved version", missing.path));
        }
        if let Some(stray) = spec.components.iter().find(|c| !approved.elements.iter().any(|e| e.id == c.id)) {
            return Err(format!("components names \"{}\", which isn't a data-design-id of the approved version", stray.id));
        }
        Ok(spec)
    }
}

/// What became of one piece of feedback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackOutcome {
    pub number: i32,
    pub target: String,
    pub body: String,
    pub resolved_in_version: Option<i32>,
    pub resolution: Option<String>,
}

impl From<&Feedback> for FeedbackOutcome {
    fn from(feedback: &Feedback) -> Self {
        Self { number: feedback.number, target: feedback.target(), body: feedback.body.clone(), resolved_in_version: feedback.resolved_in_version, resolution: feedback.resolution.clone() }
    }
}

/// A whole handoff as it is kept: what Sora wrote, and what it stands on, as they were then.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub spec: Spec,
    /// The approved preview version, its direction and where it can be looked at.
    pub artifact_version: i32,
    pub direction_id: String,
    pub studio_link: String,
    pub published_url: Option<String>,
    pub brief_version: Option<i32>,
    pub brief: Option<DesignBrief>,
    pub feedback: Vec<FeedbackOutcome>,
}

fn bullets(text: &mut String, heading: &str, lines: Vec<String>) {
    text.push_str(&format!("\n## {heading}\n"));
    for line in lines {
        text.push_str(&format!("- {line}\n"));
    }
}

impl Package {
    /// The package as Koda reads it.
    pub fn to_markdown(&self, version: i32) -> String {
        let spec = &self.spec;
        let published = match &self.published_url {
            Some(url) => format!("Published at: {url}"),
            None => "Not published outside Nomi.".to_string(),
        };
        let mut text = format!(
            "# Design handoff (version {version})\nApproved preview: version {} of direction {}\nDesign Studio: {}\n{published}\n",
            self.artifact_version, self.direction_id, self.studio_link
        );
        if let (Some(brief), Some(brief_version)) = (&self.brief, self.brief_version) {
            text.push_str(&format!("\n{}\n", brief.to_markdown(brief_version)));
        }
        let mut system: Vec<String> = spec.design_system.colors.iter().map(|c| format!("{}: {}{}", c.name, c.value, c.usage.as_deref().map(|u| format!(" ({u})")).unwrap_or_default())).collect();
        system.extend(spec.design_system.typography.iter().map(|t| {
            let size = t.size.as_deref().map(|s| format!(", {s}")).unwrap_or_default();
            let weight = t.weight.as_deref().map(|w| format!(", weight {w}")).unwrap_or_default();
            format!("{}: {}{size}{weight}", t.role, t.font)
        }));
        system.extend(spec.design_system.spacing.iter().cloned());
        bullets(&mut text, "Design system", system);
        bullets(&mut text, "Sitemap", spec.sitemap.iter().map(|s| format!("{} \"{}\": {}", s.path, s.title, s.purpose)).collect());
        bullets(&mut text, "User flows", spec.user_flows.iter().map(|f| format!("{}: {}", f.name, f.steps.join(" → "))).collect());
        bullets(
            &mut text,
            "Pages",
            spec.pages.iter().map(|p| format!("{} ({}): {}{}", p.path, p.purpose, p.sections.join(", "), p.notes.as_deref().map(|n| format!(". {n}")).unwrap_or_default())).collect(),
        );
        bullets(
            &mut text,
            "Components",
            spec.components
                .iter()
                .map(|c| format!("{} ({}): {}{}", c.id, c.name, c.description, if c.states.is_empty() { String::new() } else { format!(". States: {}", c.states.join("; ")) }))
                .collect(),
        );
        bullets(&mut text, "Interactions", spec.interactions.iter().map(|i| format!("{}, on {}: {}", i.target, i.trigger, i.result)).collect());
        bullets(&mut text, "Responsive rules", spec.responsive.clone());
        bullets(&mut text, "Accessibility", spec.accessibility.clone());
        bullets(&mut text, "Assets", spec.assets.iter().map(|a| format!("{} ({}): {}", a.name, a.kind, a.source)).collect());
        text.push_str("\n## Feedback resolution\n");
        if self.feedback.is_empty() {
            text.push_str("No feedback was given in the Design Studio.\n");
        }
        for outcome in &self.feedback {
            let became = match (outcome.resolved_in_version, outcome.resolution.as_deref()) {
                (Some(version), Some(note)) => format!("dealt with in version {version}: {note}"),
                (Some(version), None) => format!("dealt with in version {version}"),
                _ => "not dealt with".to_string(),
            };
            text.push_str(&format!("- #{} ({}) \"{}\": {became}\n", outcome.number, outcome.target, outcome.body));
        }
        bullets(&mut text, "Design decisions", spec.decisions.iter().map(|d| format!("{}: {}", d.decision, d.reason)).collect());
        text.trim_end().to_string()
    }
}

fn list_of(description: &str, fields: &[&str], required: &[&str]) -> Value {
    let properties: serde_json::Map<String, Value> = fields.iter().map(|field| (field.to_string(), json!({"type": "string"}))).collect();
    json!({"type": "array", "description": description, "items": {"type": "object", "properties": properties, "required": required}})
}

fn strings(description: &str) -> Value {
    json!({"type": "array", "description": description, "items": {"type": "string"}})
}

/// The input schema of the tool that writes a handoff.
pub fn handoff_input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "design_system": {
                "type": "object",
                "properties": {
                    "colors": list_of("Every colour: its name, its value (hex) and what it is used for", &["name", "value", "usage"], &["name", "value"]),
                    "typography": list_of("Every text style: its role, font, size/line height and weight", &["role", "font", "size", "weight"], &["role", "font"]),
                    "spacing": strings("Spacing, radius and grid rules")
                },
                "required": ["colors", "typography"]
            },
            "sitemap": list_of("Every page: its path in the preview, its title and what it is for", &["path", "title", "purpose"], &["path", "title", "purpose"]),
            "user_flows": {
                "type": "array",
                "description": "What people do, step by step",
                "items": {"type": "object", "properties": {"name": {"type": "string"}, "steps": {"type": "array", "items": {"type": "string"}}}, "required": ["name", "steps"]}
            },
            "pages": {
                "type": "array",
                "description": "Every page of the approved preview: its path, purpose, and the data-design-id of each section top to bottom",
                "items": {
                    "type": "object",
                    "properties": {"path": {"type": "string"}, "purpose": {"type": "string"}, "sections": {"type": "array", "items": {"type": "string"}}, "notes": {"type": "string"}},
                    "required": ["path", "purpose", "sections"]
                }
            },
            "components": {
                "type": "array",
                "description": "Each reusable component: its data-design-id in the preview, name, what it is, and its states (hover, empty, error, loading)",
                "items": {
                    "type": "object",
                    "properties": {"id": {"type": "string"}, "name": {"type": "string"}, "description": {"type": "string"}, "states": {"type": "array", "items": {"type": "string"}}},
                    "required": ["id", "name", "description"]
                }
            },
            "interactions": list_of("What happens when something is used: its target, the trigger and the result", &["target", "trigger", "result"], &["target", "trigger", "result"]),
            "responsive": strings("How the layout changes from a phone to a desktop, with the widths"),
            "accessibility": strings("Contrast, focus order, labels, motion and touch target rules"),
            "assets": list_of("Images, icons and fonts: the name, the kind and where each comes from", &["name", "kind", "source"], &["name", "kind", "source"]),
            "decisions": list_of("The design decisions that were made and why", &["decision", "reason"], &["decision", "reason"])
        },
        "required": ["design_system", "sitemap", "user_flows", "pages", "components", "responsive", "accessibility", "decisions"]
    })
}

/// Keeps `package` as the chat's next handoff and returns its version.
pub async fn save(conn: &mut PgConnection, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, package: &Package) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO design_handoffs (session_id, agent_session_id, user_id, version, artifact_version, brief_version, package) \
         SELECT $1, $2, $3, COALESCE(MAX(version), 0) + 1, $4, $5, $6 FROM design_handoffs WHERE session_id = $1 RETURNING version",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(user_id)
    .bind(package.artifact_version)
    .bind(package.brief_version)
    .bind(sqlx::types::Json(package))
    .fetch_one(conn)
    .await
}

/// The chat's newest handoff: its version and package.
pub async fn latest(conn: &mut PgConnection, session_id: Uuid) -> Result<Option<(i32, Package)>, sqlx::Error> {
    let row: Option<(i32, sqlx::types::Json<Package>)> =
        sqlx::query_as("SELECT version, package FROM design_handoffs WHERE session_id = $1 ORDER BY version DESC LIMIT 1").bind(session_id).fetch_optional(conn).await?;
    Ok(row.map(|(version, package)| (version, package.0)))
}

/// The chat's newest handoff as the builder's brief shows it, where Design is rolled out to
/// this person; nothing otherwise, so a brief without Design is what it always was.
pub async fn for_builder(conn: &mut PgConnection, session_id: Uuid, user_id: Uuid) -> Option<(i32, Package)> {
    if !nomi_settings::config::design_enabled_for(user_id).await {
        return None;
    }
    latest(conn, session_id).await.ok().flatten()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::design_artifact::DesignArtifact;
    use serde_json::json;

    fn manifest() -> Manifest {
        let page = |title: &str, body: &str| format!("<html><head><meta name=\"viewport\" content=\"width=device-width\"><title>{title}</title></head><body>{body}</body></html>");
        DesignArtifact::parse(&json!({
            "direction_id": "editorial",
            "title": "Approved pass",
            "files": [
                {"path": "index.html", "content": page("Home", "<header data-design-id=\"site-header\"></header><section data-design-id=\"hero\"></section>")},
                {"path": "menu/index.html", "content": page("Menu", "<header data-design-id=\"site-header\"></header><ul data-design-id=\"menu-list\"></ul>")}
            ]
        }))
        .unwrap()
        .manifest
    }

    pub(crate) fn spec() -> Value {
        json!({
            "design_system": {
                "colors": [{"name": "Paper", "value": "#F6EFE4", "usage": "Page background"}, {"name": "Accent", "value": "#B5532A"}],
                "typography": [{"role": "Heading", "font": "Fraunces", "size": "48/56", "weight": "600"}, {"role": "Body", "font": "Inter"}],
                "spacing": ["8px base unit", "Sections 96px apart on desktop, 56px on a phone"]
            },
            "sitemap": [{"path": "index.html", "title": "Home", "purpose": "Say what the bakery is and take pre-orders"}, {"path": "menu/index.html", "title": "Menu", "purpose": "Today's bread"}],
            "user_flows": [{"name": "Pre-order", "steps": ["Open Home", "Tap Order", "Pick bread and a time"]}],
            "pages": [
                {"path": "index.html", "purpose": "Landing", "sections": ["site-header", "hero"], "notes": "Hero fills the first screen"},
                {"path": "menu/index.html", "purpose": "Menu", "sections": ["site-header", "menu-list"]}
            ],
            "components": [{"id": "site-header", "name": "Site header", "description": "Logo left, links right", "states": ["Sticky after 80px"]}],
            "interactions": [{"target": "order-button", "trigger": "Tap", "result": "Opens the order sheet"}],
            "responsive": ["One column under 720px"],
            "accessibility": ["Body text contrast at least 4.5:1"],
            "assets": [{"name": "Logo", "kind": "svg", "source": "img/logo.svg in the preview"}],
            "decisions": [{"decision": "Serif headings", "reason": "The person asked for something warm"}]
        })
    }

    fn package() -> Package {
        Package {
            spec: Spec::parse(&spec(), &manifest()).unwrap(),
            artifact_version: 3,
            direction_id: "editorial".into(),
            studio_link: "/chat/c1/design?version=3".into(),
            published_url: Some("https://ship.example.com/dpl/d1".into()),
            brief_version: Some(2),
            brief: Some(crate::design_brief::DesignBrief::parse(&json!({"company": [{"value": "Roti Pagi", "source": "confirmed"}], "audience": [{"value": "Neighbours", "source": "assumed"}], "objectives": [{"value": "Pre-orders", "source": "confirmed"}]})).unwrap()),
            feedback: vec![
                FeedbackOutcome { number: 1, target: "section hero on index.html".into(), body: "Make it shorter".into(), resolved_in_version: Some(3), resolution: Some("Half the height".into()) },
                FeedbackOutcome { number: 2, target: "the whole design".into(), body: "Warmer".into(), resolved_in_version: None, resolution: None },
            ],
        }
    }

    #[test]
    fn a_handoff_is_read_whole() {
        let spec = Spec::parse(&spec(), &manifest()).unwrap();
        assert_eq!(spec.design_system.colors[1], Color { name: "Accent".into(), value: "#B5532A".into(), usage: None });
        assert_eq!(spec.pages[0].sections, ["site-header", "hero"]);
        assert_eq!(spec.components[0].states, ["Sticky after 80px"]);
        assert_eq!(spec.user_flows[0].steps.len(), 3);
    }

    #[test]
    fn a_handoff_leaves_no_part_out() {
        for part in ["design_system", "sitemap", "user_flows", "pages", "components", "responsive", "accessibility", "decisions"] {
            let mut input = spec();
            input.as_object_mut().unwrap().remove(part);
            let error = Spec::parse(&input, &manifest()).unwrap_err();
            assert!(error.starts_with("the handoff isn't complete:") && error.contains(part), "{part}: {error}");
        }
        for part in ["sitemap", "user_flows", "pages", "components", "responsive", "accessibility", "decisions"] {
            let mut input = spec();
            input[part] = json!([]);
            assert_eq!(Spec::parse(&input, &manifest()).unwrap_err(), format!("{part} needs at least one entry"), "{part}");
        }
        let mut colourless = spec();
        colourless["design_system"]["colors"] = json!([]);
        assert_eq!(Spec::parse(&colourless, &manifest()).unwrap_err(), "design_system.colors needs at least one entry");
        let mut no_interactions = spec();
        no_interactions.as_object_mut().unwrap().remove("interactions");
        no_interactions.as_object_mut().unwrap().remove("assets");
        assert!(Spec::parse(&no_interactions, &manifest()).is_ok(), "a design may have no interactions or assets to name");
    }

    #[test]
    fn a_handoff_describes_the_approved_version_and_nothing_else() {
        let mut missing = spec();
        missing["pages"].as_array_mut().unwrap().pop();
        assert_eq!(Spec::parse(&missing, &manifest()).unwrap_err(), "pages doesn't describe menu/index.html, a page of the approved version");
        let mut stray = spec();
        stray["pages"][0]["path"] = json!("about.html");
        assert_eq!(Spec::parse(&stray, &manifest()).unwrap_err(), "pages describes about.html, which the approved version doesn't have");
        let mut elsewhere = spec();
        elsewhere["pages"][0]["sections"] = json!(["menu-list"]);
        assert_eq!(Spec::parse(&elsewhere, &manifest()).unwrap_err(), "index.html has no element \"menu-list\" in the approved version");
        let mut invented = spec();
        invented["components"][0]["id"] = json!("footer");
        assert_eq!(Spec::parse(&invented, &manifest()).unwrap_err(), "components names \"footer\", which isn't a data-design-id of the approved version");
        let mut huge = spec();
        huge["responsive"] = json!(["x".repeat(MAX_PACKAGE_BYTES)]);
        assert_eq!(Spec::parse(&huge, &manifest()).unwrap_err(), format!("the handoff is larger than {MAX_PACKAGE_BYTES} bytes: say it more briefly"));
    }

    #[test]
    fn koda_reads_every_part_of_the_package() {
        let text = package().to_markdown(1);
        assert!(text.starts_with("# Design handoff (version 1)\nApproved preview: version 3 of direction editorial\nDesign Studio: /chat/c1/design?version=3\nPublished at: https://ship.example.com/dpl/d1\n"), "{text}");
        for heading in [
            "# Design brief (version 2)", "## Design system", "## Sitemap", "## User flows", "## Pages", "## Components", "## Interactions",
            "## Responsive rules", "## Accessibility", "## Assets", "## Feedback resolution", "## Design decisions",
        ] {
            assert!(text.contains(heading), "{heading} is in the package");
        }
        assert!(text.contains("- Paper: #F6EFE4 (Page background)\n- Accent: #B5532A\n"), "{text}");
        assert!(text.contains("- Heading: Fraunces, 48/56, weight 600\n- Body: Inter\n"));
        assert!(text.contains("- index.html (Landing): site-header, hero. Hero fills the first screen\n"));
        assert!(text.contains("- Pre-order: Open Home → Tap Order → Pick bread and a time\n"));
        assert!(text.contains("- site-header (Site header): Logo left, links right. States: Sticky after 80px\n"));
        assert!(text.contains("- #1 (section hero on index.html) \"Make it shorter\": dealt with in version 3: Half the height\n"));
        assert!(text.contains("- #2 (the whole design) \"Warmer\": not dealt with\n"));
        assert!(text.contains("- Serif headings: The person asked for something warm"));
    }

    #[test]
    fn a_package_without_a_brief_feedback_or_an_address_says_so() {
        let bare = Package { brief: None, brief_version: None, published_url: None, feedback: vec![], ..package() };
        let text = bare.to_markdown(2);
        assert!(text.contains("Not published outside Nomi.\n"));
        assert!(text.contains("## Feedback resolution\nNo feedback was given in the Design Studio.\n"));
        assert!(!text.contains("# Design brief"));
    }

    #[test]
    fn a_package_survives_being_stored() {
        let stored = serde_json::to_value(package()).unwrap();
        assert_eq!(serde_json::from_value::<Package>(stored).unwrap(), package());
    }

    #[test]
    fn the_tool_schema_asks_for_every_part() {
        let schema = handoff_input_schema();
        assert_eq!(schema["required"], json!(["design_system", "sitemap", "user_flows", "pages", "components", "responsive", "accessibility", "decisions"]));
        for part in ["interactions", "assets"] {
            assert!(schema["properties"][part].is_object(), "{part}");
        }
    }
}
