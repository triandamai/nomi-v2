//! Sora's design library: what she knows about design, compiled into the binary and read one
//! skill at a time with `read_design_skill`, so her instructions stay short.
//!
//! It follows the Agent Skills convention (SKILL.md): each skill is a folder
//! `library/<area>/<skill>/` with a `SKILL.md` (frontmatter `name`, `description`, `license`,
//! `source`, then the body) and, when it has more than one read's worth, `references/*.md` read on
//! demand. `build.rs` lists the folders, so adding a skill is adding its folder. Taste Skill is her
//! rulebook. Where each skill came from, and under which licence, is in `library/SOURCES.md`.

use serde_json::Value;

/// One reference file of a skill, read on demand.
pub struct Reference {
    pub name: &'static str,
    pub text: &'static str,
}

/// One skill: its area (the parent folder), its name (its folder) and its SKILL.md.
pub struct Skill {
    pub area: &'static str,
    pub name: &'static str,
    pub text: &'static str,
    pub references: &'static [Reference],
}

include!(concat!(env!("OUT_DIR"), "/library.rs"));

/// The longest a SKILL.md or a reference may be: a read puts the whole of it in front of Sora.
pub const MAX_SKILL_BYTES: usize = 16_000;

/// The library's areas, in the order Sora sees them, with what each holds.
pub const AREAS: &[(&str, &str)] = &[
    ("design", "the canvas format and how you work"),
    ("taste", "Taste Skill, your rulebook: house rules, layout, sections, copy, colour, type, checks"),
    ("styles", "the anti-slop audit, the design loop and named styles to offer as directions"),
    ("systems", "complete design systems to learn from and start a direction with"),
    ("ux", "states, UX laws, accessibility, motion timing, references, critique, brand extraction"),
    ("craft", "visual modes, polish, motion design and charts"),
    ("practice", "design practice: briefs, reviews, copywriting, platforms, conversion"),
    ("components", "the shadcn-svelte component kit Koda builds from"),
    ("decks", "slide decks: layouts, themes and how a deck tells a story"),
    ("video", "motion graphics and video compositions"),
    ("images", "images: briefs for generated pictures and how to use them"),
    ("templates", "marketing pieces: posters, social cards, documents"),
    ("references", "patterns learned from other libraries and how to use references"),
];

/// The skills listed whole in the tool, the ones most steps start from.
pub const START_HERE: &[&str] = &["taste-core", "styles-index", "pen-format", "components-kit", "taste-check", "slop-audit", "design-loop"];

/// A skill's frontmatter.
#[derive(Debug, Clone, PartialEq)]
pub struct Frontmatter {
    pub name: String,
    pub description: String,
    pub license: String,
    pub source: String,
}

/// A value as written after `key:`, without the quotes around it.
fn unquote(value: &str) -> String {
    let value = value.trim();
    match value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
        Some(inner) => inner.replace("\\\"", "\"").replace("\\\\", "\\"),
        None => value.to_string(),
    }
}

/// The frontmatter between the leading `---` lines, and the body after it; `None` without one.
pub fn frontmatter(text: &str) -> Option<(Frontmatter, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    let mut meta = Frontmatter { name: String::new(), description: String::new(), license: String::new(), source: String::new() };
    for line in rest[..end].lines() {
        let Some((key, value)) = line.split_once(':') else { continue };
        let value = unquote(value);
        match key.trim() {
            "name" => meta.name = value,
            "description" => meta.description = value,
            "license" => meta.license = value,
            "source" => meta.source = value,
            _ => {}
        }
    }
    Some((meta, &rest[end + "\n---\n".len()..]))
}

fn description(skill: &Skill) -> String {
    frontmatter(skill.text).map(|(meta, _)| meta.description).unwrap_or_default()
}

fn area_names() -> String {
    AREAS.iter().map(|(area, _)| *area).filter(|area| LIBRARY.iter().any(|s| s.area == *area)).collect::<Vec<_>>().join(", ")
}

/// What the tool lists: the areas with how many skills each holds, then the skills to start with.
pub fn tool_index() -> String {
    let areas: Vec<String> = AREAS
        .iter()
        .filter_map(|(area, holds)| {
            let count = LIBRARY.iter().filter(|s| s.area == *area).count();
            (count > 0).then(|| format!("{area} ({count}): {holds}"))
        })
        .collect();
    let start: Vec<String> = START_HERE
        .iter()
        .filter_map(|name| LIBRARY.iter().find(|s| s.name == *name))
        .map(|s| format!("- {}: {}", s.name, description(s)))
        .collect();
    format!("Areas (read index:<area> for its skills):\n{}\n\nStart with:\n{}", areas.join("\n"), start.join("\n"))
}

/// One area's skills, one line each; `None` for an area with none.
pub fn area_index(area: &str) -> Option<String> {
    let lines: Vec<String> = LIBRARY.iter().filter(|s| s.area == area).map(|s| format!("- {}: {}", s.name, description(s))).collect();
    (!lines.is_empty()).then(|| lines.join("\n"))
}

/// A skill's body, followed by the names of its references; or one reference, whole.
pub fn read_skill(skill: &Skill, reference: Option<&str>) -> Result<String, String> {
    let names = || skill.references.iter().map(|r| r.name).collect::<Vec<_>>().join(", ");
    if let Some(wanted) = reference.map(|r| r.trim().to_lowercase()).filter(|r| !r.is_empty()) {
        return skill
            .references
            .iter()
            .find(|r| r.name == wanted)
            .map(|r| r.text.to_string())
            .ok_or_else(|| format!("{} has no reference \"{wanted}\": it has {}", skill.name, names()));
    }
    let body = frontmatter(skill.text).map(|(_, body)| body).unwrap_or(skill.text).trim();
    Ok(if skill.references.is_empty() { body.to_string() } else { format!("{body}\n\nMore in this skill (read_design_skill with reference): {}", names()) })
}

/// `read_design_skill`: a skill (or one of its references), an area's index (`index:<area>`),
/// or the list of areas (`index`).
pub fn read_design_skill(input: &Value) -> Result<String, String> {
    let text = |field: &str| input.get(field).and_then(Value::as_str).map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty());
    let Some(topic) = text("topic") else {
        return Err(format!("topic is required: a skill's name, index:<area> (areas: {}) or index", area_names()));
    };
    if topic == "index" {
        let lines: Vec<String> = AREAS
            .iter()
            .filter(|(area, _)| LIBRARY.iter().any(|s| s.area == *area))
            .map(|(area, holds)| format!("- {area}: {holds}"))
            .collect();
        return Ok(format!("The library's areas (read index:<area> for its skills):\n{}", lines.join("\n")));
    }
    if let Some(area) = topic.strip_prefix("index:") {
        return area_index(area.trim()).ok_or_else(|| format!("there is no area \"{}\": the areas are {}", area.trim(), area_names()));
    }
    let skill = LIBRARY
        .iter()
        .find(|s| s.name == topic)
        .ok_or_else(|| format!("there is no design skill \"{topic}\": read index (or index:<area>) for the skills there are"))?;
    read_skill(skill, text("reference").as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::{Path, PathBuf};

    fn library_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("library")
    }

    /// Every `<area>/<skill>/SKILL.md` on disk, as (area, skill).
    fn skill_folders() -> Vec<(String, String)> {
        let mut found = Vec::new();
        for area in std::fs::read_dir(library_dir()).unwrap().map(|e| e.unwrap().path()).filter(|p| p.is_dir()) {
            for skill in std::fs::read_dir(&area).unwrap().map(|e| e.unwrap().path()).filter(|p| p.is_dir()) {
                if skill.join("SKILL.md").is_file() {
                    let name = |p: &Path| p.file_name().unwrap().to_string_lossy().into_owned();
                    found.push((name(&area), name(&skill)));
                }
            }
        }
        found.sort();
        found
    }

    #[test]
    fn every_skill_folder_on_disk_is_in_the_library() {
        let mut listed: Vec<(String, String)> = LIBRARY.iter().map(|s| (s.area.to_string(), s.name.to_string())).collect();
        listed.sort();
        assert_eq!(listed, skill_folders());
        assert!(LIBRARY.len() >= 38, "the skills from the first library are all here");
    }

    #[test]
    fn a_skill_is_named_like_its_folder_and_says_when_to_use_it_and_where_it_came_from() {
        let mut seen = std::collections::HashSet::new();
        for skill in LIBRARY {
            let (meta, body) = frontmatter(skill.text).unwrap_or_else(|| panic!("{} has frontmatter", skill.name));
            assert_eq!(meta.name, skill.name, "the frontmatter name is the folder's");
            assert!(seen.insert(skill.name), "{} is in two areas", skill.name);
            assert!(skill.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'), "{}", skill.name);
            assert!((20..=1024).contains(&meta.description.len()), "{}: {:?}", skill.name, meta.description);
            assert!(!meta.license.is_empty() && !meta.source.is_empty(), "{} names its licence and source", skill.name);
            assert!(AREAS.iter().any(|(area, _)| *area == skill.area), "{} is in a known area", skill.area);
            assert!(body.trim_start().starts_with("# "), "{}'s body starts with its title", skill.name);
        }
    }

    #[test]
    fn frontmatter_is_read_from_the_lines_between_the_dashes() {
        let text = "---\nname: taste-core\ndescription: \"Read first: the house rules.\"\nlicense: MIT\nsource: https://example.com/a\n---\n# Taste\n\nBody";
        let (meta, body) = frontmatter(text).unwrap();
        assert_eq!(meta.name, "taste-core");
        assert_eq!(meta.description, "Read first: the house rules.", "a colon inside the value and quotes around it are fine");
        assert_eq!(meta.license, "MIT");
        assert_eq!(meta.source, "https://example.com/a");
        assert_eq!(body, "# Taste\n\nBody");
        assert!(frontmatter("# No frontmatter").is_none());
        assert!(frontmatter("---\nname: x\n").is_none(), "an unclosed block isn't frontmatter");
    }

    #[test]
    fn a_skill_and_each_of_its_references_fit_in_one_read() {
        let mut total = 0;
        for skill in LIBRARY {
            assert!(skill.text.len() <= MAX_SKILL_BYTES, "{} is {} bytes", skill.name, skill.text.len());
            total += skill.text.len();
            for reference in skill.references {
                assert!(reference.text.len() <= MAX_SKILL_BYTES, "{}/{} is {} bytes", skill.name, reference.name, reference.text.len());
                assert!(reference.text.trim_start().starts_with("# "), "{}/{} starts with its title", skill.name, reference.name);
                total += reference.text.len();
            }
        }
        assert!(total <= 2_500_000, "the library is {total} bytes");
    }

    #[test]
    fn reading_a_skill_gives_its_body_and_names_its_references() {
        let pen = LIBRARY.iter().find(|s| s.name == "pen-format").expect("the canvas format is in the library");
        let read = read_design_skill(&json!({"topic": " Pen-Format "})).unwrap();
        assert!(read.starts_with("# "), "the frontmatter isn't read out: {}", &read[..40]);
        assert_eq!(read.trim(), frontmatter(pen.text).unwrap().1.trim(), "no references, so only the body");
        let with_refs = Skill { area: "design", name: "demo", text: "---\nname: demo\ndescription: A demo skill for the test here.\nlicense: MIT\nsource: Nomi\n---\n# Demo\n\nBody", references: &[Reference { name: "palettes", text: "# Palettes\n\nRows" }] };
        let read = read_skill(&with_refs, None).unwrap();
        assert!(read.ends_with("More in this skill (read_design_skill with reference): palettes"), "{read}");
        assert_eq!(read_skill(&with_refs, Some("Palettes")).unwrap(), "# Palettes\n\nRows");
        assert!(read_skill(&with_refs, Some("nope")).unwrap_err().contains("palettes"));
    }

    #[test]
    fn an_unknown_or_missing_topic_says_where_to_look() {
        let unknown = read_design_skill(&json!({"topic": "cobol"})).unwrap_err();
        assert!(unknown.contains("no design skill \"cobol\"") && unknown.contains("index"), "{unknown}");
        let missing = read_design_skill(&json!({})).unwrap_err();
        assert!(missing.contains("topic is required") && missing.contains("index"), "{missing}");
        assert!(read_design_skill(&json!({"topic": "index:nowhere"})).unwrap_err().contains("taste"), "names the areas there are");
    }

    #[test]
    fn the_tool_index_is_short_names_every_area_and_the_skills_to_start_with() {
        let index = tool_index();
        assert!(index.len() <= 4_000, "the tool index is {} bytes", index.len());
        for (area, _) in AREAS {
            let count = LIBRARY.iter().filter(|s| s.area == *area).count();
            if count > 0 {
                assert!(index.contains(&format!("{area} ({count})")), "{area} is listed with its count");
            }
        }
        for name in START_HERE {
            let skill = LIBRARY.iter().find(|s| s.name == *name).unwrap_or_else(|| panic!("{name} exists"));
            assert!(index.contains(&format!("- {name}: {}", frontmatter(skill.text).unwrap().0.description)), "{name}");
        }
    }

    #[test]
    fn an_area_index_lists_every_skill_in_it_with_its_description() {
        let taste = read_design_skill(&json!({"topic": "index:taste"})).unwrap();
        for skill in LIBRARY.iter().filter(|s| s.area == "taste") {
            assert!(taste.contains(&format!("- {}: {}", skill.name, frontmatter(skill.text).unwrap().0.description)), "{}", skill.name);
        }
        let all = read_design_skill(&json!({"topic": "index"})).unwrap();
        for (area, holds) in AREAS.iter().filter(|(a, _)| LIBRARY.iter().any(|s| s.area == *a)) {
            assert!(all.contains(area) && all.contains(holds), "{area}");
        }
        for (area, _) in AREAS {
            let listed = area_index(area).unwrap_or_default();
            assert!(listed.len() <= MAX_SKILL_BYTES, "{area}'s index is {} bytes", listed.len());
        }
    }

    #[test]
    fn the_canvas_format_guide_says_what_the_studio_can_draw() {
        let pen = read_design_skill(&json!({"topic": "pen-format"})).unwrap();
        for said in [
            "\"type\": \"frame\"", "rectangle", "ellipse", "\"text\"", "path", "\"ref\"", "reusable", "descendants",
            "fill_container", "fit_content", "justifyContent", "alignItems", "space-between", "padding", "gap",
            "cornerRadius", "stroke", "shadow", "textGrowth", "fixed-width", "lineHeight", "themes", "\"theme\"",
            "solid", "lowercase", "300", "canvas.pen",
        ] {
            assert!(pen.contains(said), "the format guide says {said:?}");
        }
    }

    #[test]
    fn the_sources_name_every_library_and_its_licence() {
        let sources = std::fs::read_to_string(library_dir().join("SOURCES.md")).unwrap();
        for said in ["Taste Skill", "Leonxlnx/taste-skill", "mengto/skills", "shadcn", "shadcn-svelte", "beUI", "Beautiful UI", "rare-ui", "MIT", "Commons Clause"] {
            assert!(sources.contains(said), "SOURCES.md names {said:?}");
        }
        assert!(sources.contains("reference only"), "rare-ui is only a reference");
    }
}
