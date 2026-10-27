//! The admin areas a person can be given access to, and the three levels of access.
//!
//! Each area is a resource in the `admin` scope (`nomi:admin:<resource>:[...]`). A level
//! includes the ones below it: `manage` can also `write` and `view`.

/// Look at the area.
pub const VIEW: &str = "view";
/// Change what's in it.
pub const WRITE: &str = "write";
/// Everything, including what can't be undone (deleting, keys, secrets, who has access).
pub const MANAGE: &str = "manage";

/// Lowest to highest.
pub const LEVELS: [&str; 3] = [VIEW, WRITE, MANAGE];

/// Every admin area, in the order the admin console lists them.
pub const ADMIN_RESOURCES: [&str; 11] = [
    "dashboard",
    "user",
    "plans",
    "notifications",
    "models",
    "tools",
    "agents",
    "support",
    "maintenance",
    "integrations",
    "configuration",
];

/// Where a level sits (0 for view), or None when it isn't one.
pub fn rank(level: &str) -> Option<usize> {
    LEVELS.iter().position(|l| *l == level)
}

/// The level and every one below it, e.g. `write` → `[view, write]`.
pub fn up_to(level: &str) -> Option<Vec<String>> {
    rank(level).map(|r| LEVELS[..=r].iter().map(|s| s.to_string()).collect())
}

/// The highest level among some actions.
pub fn highest<'a>(actions: impl IntoIterator<Item = &'a str>) -> Option<&'static str> {
    actions.into_iter().filter_map(rank).max().map(|r| LEVELS[r])
}

pub fn is_admin_resource(resource: &str) -> bool {
    ADMIN_RESOURCES.contains(&resource)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_include_the_ones_below() {
        assert_eq!(up_to("write").unwrap(), vec!["view", "write"]);
        assert_eq!(up_to("manage").unwrap().len(), 3);
        assert!(up_to("own").is_none());
    }

    #[test]
    fn highest_picks_the_top_level() {
        assert_eq!(highest(["view", "manage"]), Some("manage"));
        assert_eq!(highest(["write"]), Some("write"));
        assert_eq!(highest(["other"]), None);
    }
}
