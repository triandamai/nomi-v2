//! Sora's design library: what she knows about design, compiled into the binary and read one
//! skill at a time with `read_design_skill`, so her instructions stay short. Taste Skill is her
//! rulebook; the rest is the canvas format, styles to offer as directions, the shadcn component
//! vocabulary Koda builds from, and reference patterns. Where each skill came from, and under
//! which licence, is in `library/SOURCES.md`.

use serde_json::Value;

/// One skill: its topic (the file's name) and its Markdown, which starts with `# Title` and a
/// one-paragraph summary.
pub struct Skill {
    pub topic: &'static str,
    pub text: &'static str,
}

/// The longest a skill may be: a read puts the whole of it in front of Sora.
pub const MAX_SKILL_BYTES: usize = 16_000;

macro_rules! skill {
    ($topic:literal, $path:literal) => {
        Skill { topic: $topic, text: include_str!(concat!("../library/", $path)) }
    };
}

/// Every skill in the library, in the order the index lists them.
pub static LIBRARY: &[Skill] = &[
    skill!("pen-format", "pen-format.md"),
    skill!("taste-core", "taste/taste-core.md"),
    skill!("taste-layout", "taste/taste-layout.md"),
    skill!("taste-sections", "taste/taste-sections.md"),
    skill!("taste-copy", "taste/taste-copy.md"),
    skill!("taste-color", "taste/taste-color.md"),
    skill!("taste-type", "taste/taste-type.md"),
    skill!("taste-cards-icons", "taste/taste-cards-icons.md"),
    skill!("taste-photos-motion", "taste/taste-photos-motion.md"),
    skill!("taste-app-screens", "taste/taste-app-screens.md"),
    skill!("taste-check", "taste/taste-check.md"),
    skill!("slop-audit", "styles/slop-audit.md"),
    skill!("design-loop", "styles/design-loop.md"),
    skill!("styles-index", "styles/styles-index.md"),
    skill!("style-details", "styles/style-details.md"),
    skill!("pattern-landing-pricing", "styles/pattern-landing-pricing.md"),
    skill!("style-clean-minimal-beige", "styles/style-clean-minimal-beige.md"),
    skill!("style-editorial-service-booking", "styles/style-editorial-service-booking.md"),
    skill!("style-orange-paper-saas", "styles/style-orange-paper-saas.md"),
    skill!("style-book-serif-index", "styles/style-book-serif-index.md"),
    skill!("style-image-first-grid", "styles/style-image-first-grid.md"),
    skill!("style-portfolio-chapters", "styles/style-portfolio-chapters.md"),
    skill!("style-split-technical", "styles/style-split-technical.md"),
    skill!("style-framed-grid", "styles/style-framed-grid.md"),
    skill!("style-nested-agency", "styles/style-nested-agency.md"),
    skill!("style-agency-grid-minimal", "styles/style-agency-grid-minimal.md"),
    skill!("style-documentary-brutalist", "styles/style-documentary-brutalist.md"),
    skill!("style-tactile-hardware", "styles/style-tactile-hardware.md"),
    skill!("style-neoclassical-ledger", "styles/style-neoclassical-ledger.md"),
    skill!("style-field-guide", "styles/style-field-guide.md"),
    skill!("style-product-proof-saas", "styles/style-product-proof-saas.md"),
    skill!("style-tech-green-dark", "styles/style-tech-green-dark.md"),
    skill!("components-kit", "components/components-kit.md"),
    skill!("components-catalog", "components/components-catalog.md"),
    skill!("components-catalog-display", "components/components-catalog-display.md"),
    skill!("components-handoff", "components/components-handoff.md"),
    skill!("references-patterns", "references/references-patterns.md"),
    skill!("references-rules", "references/references-rules.md"),
];

/// A skill's summary: the paragraph under its title, on one line.
pub fn summary(text: &str) -> String {
    let mut paragraphs = text.split("\n\n").map(str::trim).filter(|p| !p.is_empty());
    let _title = paragraphs.next();
    match paragraphs.next() {
        Some(p) if !p.starts_with('#') => p.split_whitespace().collect::<Vec<_>>().join(" "),
        _ => String::new(),
    }
}

/// One line per skill, as Sora picks what to read.
pub fn index() -> String {
    LIBRARY.iter().map(|s| format!("- {}: {}", s.topic, summary(s.text))).collect::<Vec<_>>().join("\n")
}

/// `read_design_skill`: the whole of one skill, or why there is none (with the topics there are).
pub fn read_design_skill(input: &Value) -> Result<String, String> {
    let topics = || LIBRARY.iter().map(|s| s.topic).collect::<Vec<_>>().join(", ");
    let wanted = input.get("topic").and_then(Value::as_str).map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty());
    let Some(wanted) = wanted else {
        return Err(format!("topic is required: one of {}", topics()));
    };
    LIBRARY
        .iter()
        .find(|s| s.topic == wanted)
        .map(|s| s.text.to_string())
        .ok_or_else(|| format!("there is no design skill \"{wanted}\": the topics are {}", topics()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::{Path, PathBuf};

    fn library_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("library")
    }

    /// Every Markdown skill file under library/, as its topic (the file stem).
    fn skill_files(dir: &Path, found: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                skill_files(&path, found);
            } else if path.extension().is_some_and(|e| e == "md") && path.file_name().is_some_and(|n| n != "SOURCES.md") {
                found.push(path.file_stem().unwrap().to_string_lossy().into_owned());
            }
        }
    }

    #[test]
    fn every_skill_file_in_the_library_is_one_sora_can_read() {
        let mut files = Vec::new();
        skill_files(&library_dir(), &mut files);
        files.sort();
        let mut topics: Vec<String> = LIBRARY.iter().map(|s| s.topic.to_string()).collect();
        topics.sort();
        assert_eq!(topics, files, "library/*.md and LIBRARY must list the same skills");
    }

    #[test]
    fn topics_are_unique_slugs_with_a_title_and_a_summary() {
        let mut seen = std::collections::HashSet::new();
        for skill in LIBRARY {
            assert!(seen.insert(skill.topic), "{} is listed twice", skill.topic);
            assert!(skill.topic.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'), "{}", skill.topic);
            assert!(skill.text.starts_with("# "), "{} starts with its title", skill.topic);
            let summary = summary(skill.text);
            assert!(summary.len() > 20 && summary.len() < 400 && !summary.starts_with('#'), "{}: {summary:?}", skill.topic);
        }
    }

    #[test]
    fn a_skill_fits_in_one_read_and_the_library_stays_small() {
        for skill in LIBRARY {
            assert!(skill.text.len() <= MAX_SKILL_BYTES, "{} is {} bytes", skill.topic, skill.text.len());
        }
        let total: usize = LIBRARY.iter().map(|s| s.text.len()).sum();
        assert!(total <= 400_000, "the library is {total} bytes");
    }

    #[test]
    fn the_summary_is_the_paragraph_under_the_title() {
        assert_eq!(summary("# Title\n\nWhat this is, in a line.\nStill the line.\n\n## Next\n\nBody"), "What this is, in a line. Still the line.");
        assert_eq!(summary("# Only a title"), "");
    }

    #[test]
    fn reading_a_skill_gives_its_whole_text() {
        let pen = LIBRARY.iter().find(|s| s.topic == "pen-format").expect("the canvas format is in the library");
        assert_eq!(read_design_skill(&json!({"topic": "pen-format"})).unwrap(), pen.text);
        assert_eq!(read_design_skill(&json!({"topic": " Pen-Format "})).unwrap(), pen.text, "case and spaces don't matter");
    }

    #[test]
    fn an_unknown_or_missing_topic_names_the_topics_there_are() {
        let unknown = read_design_skill(&json!({"topic": "cobol"})).unwrap_err();
        assert!(unknown.contains("no design skill \"cobol\"") && unknown.contains("pen-format"), "{unknown}");
        let missing = read_design_skill(&json!({})).unwrap_err();
        assert!(missing.contains("topic is required") && missing.contains("pen-format"), "{missing}");
    }

    #[test]
    fn the_index_lists_every_topic_with_its_summary() {
        let index = index();
        for skill in LIBRARY {
            assert!(index.contains(&format!("- {}: {}", skill.topic, summary(skill.text))), "{}", skill.topic);
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
