//! What the server and the worker take from Admin → Configuration at start. Most settings are
//! read on use (attachments, Google, email, the agent loop); file storage is set up once, so a
//! change to it applies the next time Nomi starts.

use nomi_settings::config;

/// S3 as saved in Admin → Configuration, field by field over the environment's `S3_*` and
/// `AWS_*`; `None` keeps files on disk.
pub async fn storage() -> Option<nomi_storage::S3Config> {
    let storage = config::current(&config::STORAGE).await;
    nomi_storage::build_from(|field| storage.get(field)).await
}

/// The domain every published app is addressed under (Admin → Configuration → Domains), with no
/// environment fallback; `None` leaves the address to Shipyard's own apps domain.
pub async fn apps_base_domain() -> Option<String> {
    config::current(&config::DOMAINS).await.get("apps_base_domain")
}

/// Xendit payment gateway configuration as saved in Admin → Configuration or environment.
pub async fn xendit_config() -> Option<nomi_payment::xendit::XenditConfig> {
    let xendit = config::current(&config::XENDIT).await;
    let secret_key = xendit.get("secret_key")?;
    Some(nomi_payment::xendit::XenditConfig {
        api_key: secret_key,
        webhook_token: xendit.get("webhook_token"),
        base_url: xendit.get("base_url"),
    })
}


/// boat.dev's API when `api_base` isn't set.
const DEFAULT_API_BASE: &str = "https://boat.dev/api/v1";
/// How long a sandbox may sit idle before it is stopped, in seconds.
pub(crate) const DEFAULT_IDLE_STOP_SECS: u64 = 600;
/// Sandbox starts allowed per minute, under boat.dev's own limit of 12.
const DEFAULT_STARTS_PER_MINUTE: u32 = 10;
/// Sandbox starts allowed per hour, under boat.dev's own limit of 60.
const DEFAULT_STARTS_PER_HOUR: u32 = 50;
/// Sandbox minutes a user may use per month.
const DEFAULT_MONTHLY_MINUTES_PER_USER: u32 = 600;

pub use config::Rollout;

/// What the orphan sweep does with a provider sandbox no project owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReaperMode {
    /// Only log what would be deleted.
    #[default]
    DryRun,
    /// Delete it.
    Delete,
}

/// The sandbox runtime as saved in Admin -> Configuration -> Sandbox.
#[derive(Clone)]
pub struct SandboxConfig {
    pub api_base: String,
    pub api_key: String,
    pub size: nomi_sandbox::Size,
    pub idle_stop_secs: u64,
    pub rollout: Rollout,
    pub starts_per_minute: u32,
    pub starts_per_hour: u32,
    pub monthly_minutes_per_user: u32,
    pub reaper_mode: ReaperMode,
}

// The key never reaches a log.
impl std::fmt::Debug for SandboxConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SandboxConfig").field("api_base", &self.api_base).field("api_key", &"<redacted>").field("size", &self.size).field("rollout", &self.rollout).finish_non_exhaustive()
    }
}

/// The sandbox settings, or `None` (use WebContainer) unless the switch is on and a key is set.
pub async fn sandbox_config() -> Option<SandboxConfig> {
    let section = config::current(&config::SANDBOX).await;
    if section.get("enabled").as_deref() != Some("on") {
        return None;
    }
    parse_sandbox(&section)
}

/// The sandbox settings whatever the switch says, or `None` when no key is saved. The worker
/// uses it to keep winding sandboxes down after the switch is turned off: with the switch off
/// it may only stop and delete, never create or resume.
pub async fn sandbox_credentials() -> Option<SandboxConfig> {
    parse_sandbox(&config::current(&config::SANDBOX).await)
}

fn parse_sandbox(section: &config::Section) -> Option<SandboxConfig> {
    let api_key = section.get("api_key")?;
    let size = match section.get("size").as_deref() {
        Some("default") => nomi_sandbox::Size::Default,
        Some("large") => nomi_sandbox::Size::Large,
        _ => nomi_sandbox::Size::Small,
    };
    let rollout = Rollout::from_section(section);
    let count = |field: &str, default: u32| section.number(field).map_or(default, |n| u32::try_from(n).unwrap_or(u32::MAX));
    Some(SandboxConfig {
        api_base: section.get("api_base").unwrap_or_else(|| DEFAULT_API_BASE.to_string()),
        api_key,
        size,
        idle_stop_secs: section.number("idle_stop_secs").unwrap_or(DEFAULT_IDLE_STOP_SECS),
        rollout,
        starts_per_minute: count("starts_per_minute", DEFAULT_STARTS_PER_MINUTE),
        starts_per_hour: count("starts_per_hour", DEFAULT_STARTS_PER_HOUR),
        monthly_minutes_per_user: count("monthly_minutes_per_user", DEFAULT_MONTHLY_MINUTES_PER_USER),
        reaper_mode: if section.get("reaper_mode").as_deref() == Some("delete") { ReaperMode::Delete } else { ReaperMode::DryRun },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn rollout_includes_matches_allowlist_and_percent() {
        let a = Uuid::from_bytes([0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // bucket 5
        let b = Uuid::from_bytes([0, 0, 0, 80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // bucket 80
        assert!(!Rollout::Off.includes(a));
        assert!(Rollout::All.includes(b));
        assert!(Rollout::Allowlist(vec![a]).includes(a));
        assert!(!Rollout::Allowlist(vec![a]).includes(b));
        assert!(Rollout::Percent(10).includes(a));
        assert!(!Rollout::Percent(10).includes(b));
        assert!(!Rollout::Percent(0).includes(a));
        assert!(Rollout::Percent(100).includes(b));
    }
}
