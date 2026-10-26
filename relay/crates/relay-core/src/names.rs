//! Names Relay gives things: a project's environments and tier, its schema and roles in a
//! database, its public address.

use uuid::Uuid;

/// Postgres truncates identifiers longer than this.
pub const MAX_IDENTIFIER_LEN: usize = 63;
/// How much of a project's id ends its address, so two "todo" apps don't collide.
const SLUG_ID_CHARS: usize = 6;
/// Keeps an address well under DNS's 63-character label limit with the "-dev" suffix.
const MAX_SLUG_NAME: usize = 40;

/// A project's two backends: `dev`, which the preview uses, and `production`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Env {
    Dev,
    Production,
}

impl Env {
    pub const ALL: [Env; 2] = [Env::Dev, Env::Production];

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "dev" => Some(Env::Dev),
            "production" => Some(Env::Production),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Env::Dev => "dev",
            Env::Production => "production",
        }
    }
}

/// Free projects share databases (a schema each); dedicated ones have their own databases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Free,
    Dedicated,
}

impl Tier {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "free" => Some(Tier::Free),
            "dedicated" => Some(Tier::Dedicated),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Tier::Free => "free",
            Tier::Dedicated => "dedicated",
        }
    }
}

/// The roles a project's database work runs as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleKind {
    /// Owns the project's tables: migrations run as it.
    Owner,
    /// Requests with the public key and no signed-in user.
    Anon,
    /// Requests from a signed-in user.
    Authenticated,
    /// Requests with the service key; bypasses row-level security.
    Service,
}

impl RoleKind {
    pub const ALL: [RoleKind; 4] = [RoleKind::Owner, RoleKind::Anon, RoleKind::Authenticated, RoleKind::Service];

    fn suffix(self) -> &'static str {
        match self {
            RoleKind::Owner => "owner",
            RoleKind::Anon => "anon",
            RoleKind::Authenticated => "authenticated",
            RoleKind::Service => "service",
        }
    }
}

fn prefix(project: Uuid) -> String {
    format!("app_{}", project.simple())
}

/// Where a project's own tables live: its schema in a shared database, or `public` in its own.
pub fn schema_for(project: Uuid, tier: Tier) -> String {
    match tier {
        Tier::Free => prefix(project),
        Tier::Dedicated => "public".to_string(),
    }
}

pub fn role(project: Uuid, kind: RoleKind) -> String {
    format!("{}_{}", prefix(project), kind.suffix())
}

/// "Bakery Site!" → "bakery-site-3f9a2c": readable, and unique by the id's start.
pub fn slug(name: &str, project: Uuid) -> String {
    let mut words = String::new();
    for c in name.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            words.push(c);
        } else if !words.ends_with('-') && !words.is_empty() {
            words.push('-');
        }
    }
    let words: String = words.trim_matches('-').chars().take(MAX_SLUG_NAME).collect();
    let words = words.trim_end_matches('-');
    let id: String = project.simple().to_string().chars().take(SLUG_ID_CHARS).collect();
    format!("{}-{id}", if words.is_empty() { "app" } else { words })
}

/// `https://<slug>.<domain>` for production, `https://<slug>-dev.<domain>` for dev.
pub fn public_url(domain: &str, slug: &str, env: Env) -> String {
    match env {
        Env::Production => format!("https://{slug}.{domain}"),
        Env::Dev => format!("https://{slug}-dev.{domain}"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn environments_are_dev_and_production_only() {
        assert_eq!(Env::parse("dev"), Some(Env::Dev));
        assert_eq!(Env::parse("production"), Some(Env::Production));
        assert_eq!(Env::parse("staging"), None);
        assert_eq!(Env::Dev.as_str(), "dev");
        assert_eq!(Env::ALL, [Env::Dev, Env::Production]);
    }

    #[test]
    fn a_free_project_gets_its_own_schema_and_roles() {
        let project = Uuid::parse_str("4f2c1a7e-9b3d-4c8e-a1f0-2b3c4d5e6f70").unwrap();
        assert_eq!(schema_for(project, Tier::Free), "app_4f2c1a7e9b3d4c8ea1f02b3c4d5e6f70");
        assert_eq!(schema_for(project, Tier::Dedicated), "public");
        assert_eq!(role(project, RoleKind::Owner), "app_4f2c1a7e9b3d4c8ea1f02b3c4d5e6f70_owner");
        assert_eq!(role(project, RoleKind::Authenticated), "app_4f2c1a7e9b3d4c8ea1f02b3c4d5e6f70_authenticated");
        for kind in RoleKind::ALL {
            assert!(role(project, kind).len() <= MAX_IDENTIFIER_LEN);
        }
        assert_eq!(Tier::parse("free"), Some(Tier::Free));
        assert_eq!(Tier::parse("dedicated"), Some(Tier::Dedicated));
    }

    #[test]
    fn each_environment_has_its_own_address() {
        assert_eq!(public_url("api.relay.test", "bakery-3f9a2c", Env::Production), "https://bakery-3f9a2c.api.relay.test");
        assert_eq!(public_url("api.relay.test", "bakery-3f9a2c", Env::Dev), "https://bakery-3f9a2c-dev.api.relay.test");
        assert_eq!(slug("Bakery Site!", Uuid::parse_str("3f9a2c1b-0000-0000-0000-000000000000").unwrap()), "bakery-site-3f9a2c");
        assert_eq!(slug("   ", Uuid::parse_str("abcdef12-0000-0000-0000-000000000000").unwrap()), "app-abcdef");
    }
}
