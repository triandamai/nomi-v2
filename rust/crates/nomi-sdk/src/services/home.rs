//! The person, their crew, their appearance preferences, and the home page's status
//! (`routes/{profile,agents,home}.rs`).

use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::Services;
use crate::error::NomiError;

const AGENTS: &str = "agents";
const PREFERENCES: &str = "preferences";
const HOME: &str = "home";
const DEFAULT_THEME: &str = "system";
const DEFAULT_ACCENT: &str = "canopy";
const DEFAULT_TIMEZONE: &str = "UTC";
const DEFAULT_RECENTS_LIMIT: u32 = 10;

/// One crew member. `state` is `working`, `waiting` (on the person), `done` (in the last day) or
/// `idle`, with `status` the line to go with it. `shape`/`tone`/`motion` are set for dynamic agents.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct CrewMember {
    pub agent_type: String,
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub is_dynamic: bool,
    pub shape: Option<String>,
    pub tone: Option<String>,
    pub motion: Option<String>,
    pub state: String,
    pub status: String,
}

#[derive(Deserialize)]
struct CrewResponse {
    agents: Vec<CrewMember>,
}

/// The part of `PreferencesResponse` the apps use.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct Preferences {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_accent")]
    pub accent_color: String,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    pub home_layout: Option<String>,
    pub drawer_pins: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub drawer_recents: bool,
    #[serde(default = "default_recents_limit")]
    pub drawer_recents_limit: u32,
}

/// `UpdatePreferencesRequest`: only the fields set are changed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, uniffi::Record)]
pub struct PreferencesUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_layout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drawer_pins: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drawer_recents: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drawer_recents_limit: Option<u32>,
}

/// Something the crew did while the person was away.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct OutItem {
    pub kind: String,
    pub agent: String,
    pub title: String,
    pub detail: String,
    pub session_id: Option<String>,
    pub at: String,
}

/// Something scheduled for today.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct TodayItem {
    pub id: String,
    pub run_at: String,
    pub label: String,
    pub agent: String,
    pub recurrence: Option<String>,
}

/// A plan in progress: `done` of `total` steps.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct PlanItem {
    pub kind: String,
    pub title: String,
    pub agent: String,
    pub done: i64,
    pub total: i64,
    pub session_id: String,
    pub updated_at: String,
}

/// The home page's status. The lists are capped; `*_total` says how many there are.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct HomeSummary {
    #[serde(default)]
    pub since: String,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    #[serde(default)]
    pub while_you_were_out: Vec<OutItem>,
    #[serde(default)]
    pub today: Vec<TodayItem>,
    #[serde(default)]
    pub plans: Vec<PlanItem>,
    #[serde(default)]
    pub while_you_were_out_total: u32,
    #[serde(default)]
    pub today_total: u32,
    #[serde(default)]
    pub plans_total: u32,
}

fn default_theme() -> String {
    DEFAULT_THEME.to_owned()
}

fn default_accent() -> String {
    DEFAULT_ACCENT.to_owned()
}

fn default_timezone() -> String {
    DEFAULT_TIMEZONE.to_owned()
}

fn default_true() -> bool {
    true
}

fn default_recents_limit() -> u32 {
    DEFAULT_RECENTS_LIMIT
}

impl Services {
    pub async fn crew(&self) -> Result<Vec<CrewMember>, NomiError> {
        Ok(self.get::<CrewResponse>(&[AGENTS], &[]).await?.agents)
    }

    pub async fn preferences(&self) -> Result<Preferences, NomiError> {
        self.get(&[PREFERENCES], &[]).await
    }

    pub async fn update_preferences(
        &self,
        update: PreferencesUpdate,
    ) -> Result<Preferences, NomiError> {
        self.call_for(Method::PUT, &[PREFERENCES], Some(&update))
            .await
    }

    pub async fn home_summary(&self) -> Result<HomeSummary, NomiError> {
        self.get(&[HOME], &[]).await
    }
}
