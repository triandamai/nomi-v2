//! The SKILL.md format (the Agent Skills convention), shared by every member's skills: a `SKILL.md`
//! with frontmatter (`name`, `description`, `license`, `source`) and a body, plus optional
//! `references/*.md` read on demand. The parser is PR #356's (Sora's library), so her library can
//! move onto this crate unchanged; it also takes CRLF line endings and a leading BOM.

/// The longest a SKILL.md or a reference may be: a read puts the whole of it in front of a member.
pub const MAX_SKILL_BYTES: usize = 16_000;
/// The most reference files one skill may carry.
pub const MAX_REFERENCES: usize = 20;
/// The most skills a person may add to one member, switched-off ones included.
pub const MAX_SKILLS_PER_MEMBER: i64 = 30;
/// The longest a skill's or a reference's name may be.
pub const MAX_NAME_CHARS: usize = 64;
/// The shortest description that still says when to use the skill.
pub const MIN_DESCRIPTION_CHARS: usize = 20;
/// The longest description.
pub const MAX_DESCRIPTION_CHARS: usize = 1024;
/// How much of a description goes into a member's skill index.
pub const INDEX_DESCRIPTION_CHARS: usize = 150;

const FENCE: &str = "---\n";
const CLOSING_FENCE: &str = "\n---\n";
const BOM: char = '\u{feff}';

/// A skill's frontmatter.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Frontmatter {
    pub name: String,
    pub description: String,
    pub license: String,
    pub source: String,
}

/// One reference file of a skill.
#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    pub name: String,
    pub text: String,
}

/// A skill that passed every check: its frontmatter, the SKILL.md as written (line endings
/// normalised) and its references (names normalised).
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSkill {
    pub meta: Frontmatter,
    pub text: String,
    pub references: Vec<Reference>,
}

/// Which rule a skill broke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillError {
    NoFrontmatter,
    BadName,
    DescriptionLength,
    TooLarge,
    TooManyReferences,
    BadReferenceName,
}

impl SkillError {
    /// What went wrong, in a line a person can act on.
    pub fn detail(self) -> String {
        match self {
            SkillError::NoFrontmatter => "a SKILL.md starts with a frontmatter block between --- lines".to_string(),
            SkillError::BadName => format!("name is 1–{MAX_NAME_CHARS} lowercase letters, digits or -, starting with a letter"),
            SkillError::DescriptionLength => format!("description is {MIN_DESCRIPTION_CHARS}–{MAX_DESCRIPTION_CHARS} characters"),
            SkillError::TooLarge => format!("a SKILL.md and each reference are at most {MAX_SKILL_BYTES} bytes"),
            SkillError::TooManyReferences => format!("a skill has at most {MAX_REFERENCES} references"),
            SkillError::BadReferenceName => format!("a reference is named with 1–{MAX_NAME_CHARS} lowercase letters, digits or -, once per skill"),
        }
    }
}

/// A skill compiled into a member (Sora's library, for one).
#[derive(Debug, Clone, Copy)]
pub struct BuiltinSkill {
    pub name: &'static str,
    pub text: &'static str,
    /// `(name, text)` of each reference.
    pub references: &'static [(&'static str, &'static str)],
}

/// One line of a member's skill index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
}

impl BuiltinSkill {
    pub fn entry(&self) -> SkillEntry {
        SkillEntry { name: self.name.to_string(), description: frontmatter(self.text).map(|(m, _)| m.description).unwrap_or_default() }
    }
}

/// A value as written after `key:`, without the quotes around it.
fn unquote(value: &str) -> String {
    let value = value.trim();
    match value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
        Some(inner) => inner.replace("\\\"", "\"").replace("\\\\", "\\"),
        None => value.to_string(),
    }
}

/// `text` with CRLF line endings turned into LF and a leading BOM dropped.
fn normalise(text: &str) -> String {
    text.strip_prefix(BOM).unwrap_or(text).replace("\r\n", "\n")
}

/// The frontmatter between the leading `---` lines, and the body after it; `None` without one.
pub fn frontmatter(text: &str) -> Option<(Frontmatter, String)> {
    let text = normalise(text);
    let rest = text.strip_prefix(FENCE)?;
    // An empty block closes on the very next line.
    let (block, body) = match rest.strip_prefix(FENCE) {
        Some(body) => ("", body),
        None => {
            let end = rest.find(CLOSING_FENCE)?;
            (&rest[..end], &rest[end + CLOSING_FENCE.len()..])
        }
    };
    let mut meta = Frontmatter::default();
    for line in block.lines() {
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
    Some((meta, body.to_string()))
}

/// A skill's text without its frontmatter, trimmed.
pub fn body(text: &str) -> String {
    match frontmatter(text) {
        Some((_, body)) => body.trim().to_string(),
        None => text.trim().to_string(),
    }
}

/// 1–`MAX_NAME_CHARS` of `[a-z0-9-]`, starting with a letter.
fn good_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().count() <= MAX_NAME_CHARS
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Checks a skill and its references against every rule. Reference names drop a trailing `.md`
/// and are lowercased.
pub fn validate(text: &str, references: Vec<Reference>) -> Result<ParsedSkill, SkillError> {
    let text = normalise(text);
    if text.len() > MAX_SKILL_BYTES {
        return Err(SkillError::TooLarge);
    }
    let (meta, _) = frontmatter(&text).ok_or(SkillError::NoFrontmatter)?;
    if !good_name(&meta.name) {
        return Err(SkillError::BadName);
    }
    let description_chars = meta.description.chars().count();
    if !(MIN_DESCRIPTION_CHARS..=MAX_DESCRIPTION_CHARS).contains(&description_chars) {
        return Err(SkillError::DescriptionLength);
    }
    if references.len() > MAX_REFERENCES {
        return Err(SkillError::TooManyReferences);
    }
    let mut seen = Vec::with_capacity(references.len());
    let mut normalised = Vec::with_capacity(references.len());
    for reference in references {
        let lower = reference.name.trim().to_lowercase();
        let name = lower.strip_suffix(".md").unwrap_or(&lower).to_string();
        if !good_name(&name) || seen.contains(&name) {
            return Err(SkillError::BadReferenceName);
        }
        let text = normalise(&reference.text);
        if text.len() > MAX_SKILL_BYTES {
            return Err(SkillError::TooLarge);
        }
        seen.push(name.clone());
        normalised.push(Reference { name, text });
    }
    Ok(ParsedSkill { meta, text, references: normalised })
}

/// `- name: description`, the description cut to `INDEX_DESCRIPTION_CHARS` characters.
pub fn index_line(entry: &SkillEntry) -> String {
    let description = entry.description.trim();
    if description.chars().count() <= INDEX_DESCRIPTION_CHARS {
        return format!("- {}: {description}", entry.name);
    }
    let cut: String = description.chars().take(INDEX_DESCRIPTION_CHARS).collect();
    format!("- {}: {cut}…", entry.name)
}
