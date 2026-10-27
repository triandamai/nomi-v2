//! What `nomi/backend.json` says about a project's files and live tables: its buckets, with who
//! may read and write each, and the tables whose changes are sent live.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An upload's size limit when the bucket doesn't set one.
pub const DEFAULT_MAX_SIZE_MB: u32 = 50;
/// The largest limit a bucket may set.
pub const MAX_SIZE_MB_CAP: u32 = 100;
const BYTES_PER_MB: u64 = 1024 * 1024;
/// Bucket names: lowercase letters, digits, `-` and `_`, starting with a letter.
const MAX_NAME_LEN: usize = 63;
/// The longest file path in a bucket.
pub const MAX_PATH_LEN: usize = 1024;

/// Who may read or write a bucket's files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    /// Any caller with the project's key.
    Anyone,
    /// A signed-in user.
    #[default]
    SignedIn,
    /// A signed-in user whose id is the path's first folder.
    Owner,
    /// Only the service key.
    Service,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BucketConfig {
    pub name: String,
    /// Files are also served without a key at `/storage/public/<bucket>/<path>`.
    #[serde(default)]
    pub public: bool,
    #[serde(default)]
    pub read: Rule,
    #[serde(default)]
    pub write: Rule,
    #[serde(default = "default_max_size_mb")]
    pub max_size_mb: u32,
    /// Content types an upload may have (`image/png`, `image/*`); empty allows any.
    #[serde(default)]
    pub types: Vec<String>,
}

fn default_max_size_mb() -> u32 {
    DEFAULT_MAX_SIZE_MB
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default)]
    pub buckets: Vec<BucketConfig>,
}

/// Who's asking, as far as a bucket rule cares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Who {
    pub service: bool,
    pub user: Option<Uuid>,
}

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    name.len() <= MAX_NAME_LEN
        && chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

impl BucketConfig {
    pub fn max_bytes(&self) -> u64 {
        u64::from(self.max_size_mb) * BYTES_PER_MB
    }

    /// Whether an upload with this content type may go in.
    pub fn accepts_type(&self, content_type: &str) -> bool {
        if self.types.is_empty() {
            return true;
        }
        let given = content_type.split(';').next().unwrap_or_default().trim().to_ascii_lowercase();
        self.types.iter().any(|pattern| {
            let pattern = pattern.trim().to_ascii_lowercase();
            match pattern.strip_suffix("/*") {
                Some(family) => given.split('/').next() == Some(family) && given.contains('/'),
                None => given == pattern,
            }
        })
    }
}

impl StorageConfig {
    /// Every problem with it, in words Koda can act on; empty when it's fine.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for bucket in &self.buckets {
            if !valid_name(&bucket.name) {
                out.push(format!("bucket name {:?}: use lowercase letters, digits, - and _, starting with a letter", bucket.name));
            }
            if !seen.insert(bucket.name.as_str()) {
                out.push(format!("bucket {} is listed twice", bucket.name));
            }
            if bucket.max_size_mb == 0 || bucket.max_size_mb > MAX_SIZE_MB_CAP {
                out.push(format!("bucket {}: max_size_mb must be between 1 and {MAX_SIZE_MB_CAP}", bucket.name));
            }
            if bucket.types.iter().any(|t| !t.contains('/')) {
                out.push(format!("bucket {}: types are content types like image/png or image/*", bucket.name));
            }
        }
        out
    }

    pub fn bucket(&self, name: &str) -> Option<&BucketConfig> {
        self.buckets.iter().find(|b| b.name == name)
    }
}

/// A file path inside a bucket, or `None` when it could reach outside it or isn't a file name:
/// no empty, `.` or `..` segments, no backslashes or control characters.
pub fn clean_path(path: &str) -> Option<String> {
    let trimmed = path.trim_start_matches('/');
    if trimmed.is_empty() || trimmed.len() > MAX_PATH_LEN || trimmed.chars().any(|c| c == '\\' || c.is_control()) {
        return None;
    }
    let ok = trimmed.split('/').all(|segment| !segment.is_empty() && segment != "." && segment != "..");
    ok.then(|| trimmed.to_string())
}

/// A folder to list under: empty for the top, otherwise a clean path.
pub fn clean_prefix(prefix: &str) -> Option<String> {
    let trimmed = prefix.trim_matches('/');
    if trimmed.is_empty() {
        Some(String::new())
    } else {
        clean_path(trimmed)
    }
}

/// Whether `who` may act on `path` under `rule`. The service key passes every rule.
pub fn allows(rule: Rule, who: Who, path: &str) -> bool {
    if who.service {
        return true;
    }
    match rule {
        Rule::Anyone => true,
        Rule::SignedIn => who.user.is_some(),
        Rule::Owner => who.user.is_some_and(|user| path.split('/').next() == Some(user.to_string().as_str())),
        Rule::Service => false,
    }
}

/// Table names for `realtime`: plain identifiers, each once.
pub fn realtime_problems(tables: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for table in tables {
        let plain = table.chars().next().is_some_and(|c| c.is_ascii_lowercase() || c == '_')
            && table.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && table.len() <= MAX_NAME_LEN;
        if !plain {
            out.push(format!("realtime table {table:?}: use the table's plain name, like message"));
        }
        if !seen.insert(table.as_str()) {
            out.push(format!("realtime table {table} is listed twice"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn bucket(value: serde_json::Value) -> BucketConfig {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn a_bucket_says_only_its_name_and_gets_signed_in_rules_and_50_mb() {
        let b = bucket(json!({ "name": "avatars" }));
        assert!(!b.public);
        assert_eq!((b.read, b.write), (Rule::SignedIn, Rule::SignedIn));
        assert_eq!(b.max_bytes(), 50 * 1024 * 1024);
        let b = bucket(json!({ "name": "docs", "read": "anyone", "write": "owner", "public": true, "max_size_mb": 5 }));
        assert_eq!((b.read, b.write, b.public, b.max_size_mb), (Rule::Anyone, Rule::Owner, true, 5));
    }

    #[test]
    fn rules_decide_by_who_is_asking_and_the_paths_first_folder() {
        let user = Uuid::new_v4();
        let other = Uuid::new_v4();
        let anon = Who { service: false, user: None };
        let signed_in = Who { service: false, user: Some(user) };
        let service = Who { service: true, user: None };
        let own = format!("{user}/avatar.png");
        assert!(allows(Rule::Anyone, anon, "a.png"));
        assert!(!allows(Rule::SignedIn, anon, "a.png"));
        assert!(allows(Rule::SignedIn, signed_in, "a.png"));
        assert!(allows(Rule::Owner, signed_in, &own));
        assert!(!allows(Rule::Owner, signed_in, &format!("{other}/avatar.png")));
        assert!(!allows(Rule::Owner, signed_in, &format!("x{user}/avatar.png")));
        assert!(!allows(Rule::Owner, anon, &own));
        assert!(!allows(Rule::Service, signed_in, &own));
        for rule in [Rule::Anyone, Rule::SignedIn, Rule::Owner, Rule::Service] {
            assert!(allows(rule, service, "anything"));
        }
    }

    #[test]
    fn paths_can_never_leave_the_bucket() {
        assert_eq!(clean_path("/u/a.png").as_deref(), Some("u/a.png"));
        assert_eq!(clean_path("u 1/é.png").as_deref(), Some("u 1/é.png"));
        for bad in ["", "/", "../x", "u/../../x", "u//a", "u/./a", "u\\a", "a\u{0}b", "u/"] {
            assert_eq!(clean_path(bad), None, "{bad:?}");
        }
        assert_eq!(clean_path(&"a".repeat(MAX_PATH_LEN + 1)), None);
        assert_eq!(clean_prefix("").as_deref(), Some(""));
        assert_eq!(clean_prefix("/u/").as_deref(), Some("u"));
        assert_eq!(clean_prefix("u/.."), None);
    }

    #[test]
    fn upload_types_match_exactly_or_by_family() {
        let b = bucket(json!({ "name": "a", "types": ["image/*", "application/pdf"] }));
        assert!(b.accepts_type("image/png"));
        assert!(b.accepts_type("IMAGE/JPEG; charset=binary"));
        assert!(b.accepts_type("application/pdf"));
        assert!(!b.accepts_type("text/html"));
        assert!(!b.accepts_type("image"));
        assert!(!b.accepts_type(""));
        assert!(bucket(json!({ "name": "a" })).accepts_type("text/html"));
    }

    #[test]
    fn bad_settings_are_named() {
        let config: StorageConfig = serde_json::from_value(json!({ "buckets": [
            { "name": "Avatars" }, { "name": "a" }, { "name": "a" }, { "name": "big", "max_size_mb": 0 }, { "name": "t", "types": ["png"] }
        ]}))
        .unwrap();
        let problems = config.problems();
        assert_eq!(problems.len(), 4, "{problems:?}");
        assert!(problems[0].contains("Avatars"));
        assert!(StorageConfig::default().problems().is_empty());
        assert!(realtime_problems(&["message".into(), "task_2".into()]).is_empty());
        assert_eq!(realtime_problems(&["public.message".into(), "Task".into(), "a".into(), "a".into()]).len(), 3);
    }

    #[test]
    fn rules_are_written_in_snake_case() {
        assert_eq!(serde_json::to_value(Rule::SignedIn).unwrap(), json!("signed_in"));
        assert!(serde_json::from_value::<BucketConfig>(json!({ "name": "a", "read": "everyone" })).is_err());
    }
}
