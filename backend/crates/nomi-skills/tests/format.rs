use nomi_skills::*;

fn skill(name: &str, description: &str, body: &str) -> String {
    format!("---\nname: {name}\ndescription: {description}\n---\n{body}")
}

const DESC: &str = "Use when planning a menu for the week.";

fn reference(name: &str, text: &str) -> Reference {
    Reference { name: name.to_string(), text: text.to_string() }
}

// Copied from PR #356 (nomi-agent-design/src/library.rs), unchanged.
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
fn crlf_and_bom_parse() {
    let text = "\u{feff}---\r\nname: menu\r\ndescription: x\r\n---\r\n# Menu\r\nBody";
    let (meta, body) = frontmatter(text).unwrap();
    assert_eq!(meta.name, "menu");
    assert_eq!(meta.description, "x");
    assert_eq!(body, "# Menu\nBody");
}

#[test]
fn repeated_key_keeps_last() {
    let (meta, _) = frontmatter("---\nname: a\nname: b\n---\n").unwrap();
    assert_eq!(meta.name, "b");
}

#[test]
fn a_valid_skill_passes() {
    let parsed = validate(&skill("menu-plan", DESC, "# Menu"), vec![reference("Week.md", "w")]).unwrap();
    assert_eq!(parsed.meta.name, "menu-plan");
    assert_eq!(parsed.references[0].name, "week", "a reference name drops .md and is lowercased");
}

#[test]
fn no_frontmatter_is_refused() {
    assert_eq!(validate("# just text", vec![]).unwrap_err(), SkillError::NoFrontmatter);
}

#[test]
fn name_rules() {
    let ok = "a".repeat(MAX_NAME_CHARS);
    assert!(validate(&skill(&ok, DESC, ""), vec![]).is_ok());
    let long = "a".repeat(MAX_NAME_CHARS + 1);
    for bad in [long.as_str(), "1abc", "Menu", "menu plan", "", "menu_plan"] {
        assert_eq!(validate(&skill(bad, DESC, ""), vec![]).unwrap_err(), SkillError::BadName, "{bad:?}");
    }
}

#[test]
fn description_length() {
    let short = "x".repeat(MIN_DESCRIPTION_CHARS - 1);
    let min = "x".repeat(MIN_DESCRIPTION_CHARS);
    let max = "é".repeat(MAX_DESCRIPTION_CHARS);
    let long = "x".repeat(MAX_DESCRIPTION_CHARS + 1);
    assert!(validate(&skill("a", &min, ""), vec![]).is_ok());
    assert!(validate(&skill("a", &max, ""), vec![]).is_ok(), "counted in chars, not bytes");
    assert_eq!(validate(&skill("a", &short, ""), vec![]).unwrap_err(), SkillError::DescriptionLength);
    assert_eq!(validate(&skill("a", &long, ""), vec![]).unwrap_err(), SkillError::DescriptionLength);
}

#[test]
fn skill_size() {
    let head = skill("a", DESC, "");
    let fits = format!("{head}{}", "x".repeat(MAX_SKILL_BYTES - head.len()));
    assert!(validate(&fits, vec![]).is_ok());
    let over = format!("{fits}x");
    assert_eq!(validate(&over, vec![]).unwrap_err(), SkillError::TooLarge);
}

#[test]
fn reference_size() {
    let text = skill("a", DESC, "");
    assert!(validate(&text, vec![reference("r", &"x".repeat(MAX_SKILL_BYTES))]).is_ok());
    assert_eq!(validate(&text, vec![reference("r", &"x".repeat(MAX_SKILL_BYTES + 1))]).unwrap_err(), SkillError::TooLarge);
}

#[test]
fn reference_count() {
    let text = skill("a", DESC, "");
    let refs = |n: usize| (0..n).map(|i| reference(&format!("r{i}"), "x")).collect::<Vec<_>>();
    assert!(validate(&text, refs(MAX_REFERENCES)).is_ok());
    assert_eq!(validate(&text, refs(MAX_REFERENCES + 1)).unwrap_err(), SkillError::TooManyReferences);
}

#[test]
fn reference_names() {
    let text = skill("a", DESC, "");
    for bad in ["../x", "a/b", "..", "", "x y"] {
        assert_eq!(validate(&text, vec![reference(bad, "x")]).unwrap_err(), SkillError::BadReferenceName, "{bad:?}");
    }
}

#[test]
fn duplicate_reference_names_are_refused() {
    let text = skill("a", DESC, "");
    assert_eq!(validate(&text, vec![reference("r", "x"), reference("R.md", "y")]).unwrap_err(), SkillError::BadReferenceName);
}

#[test]
fn index_line_cuts_on_a_char_boundary() {
    let entry = SkillEntry { name: "menu".into(), description: "é".repeat(INDEX_DESCRIPTION_CHARS + 10) };
    let line = index_line(&entry);
    assert!(line.starts_with("- menu: "));
    assert_eq!(line.chars().count(), "- menu: ".len() + INDEX_DESCRIPTION_CHARS + 1, "cut, then an ellipsis");
    let short = SkillEntry { name: "menu".into(), description: "short".into() };
    assert_eq!(index_line(&short), "- menu: short");
}

#[test]
fn body_drops_frontmatter() {
    assert_eq!(body(&skill("a", DESC, "\n# Menu\n")), "# Menu");
    assert_eq!(body("  plain  "), "plain");
}
