//! The rest of the person's account: what Nomi remembers about them, their connected services,
//! their usage allowance, and their projects (`routes/{memory,connections,usage,projects}.rs`).

use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{Services, NO_BODY};
use crate::error::NomiError;

const MEMORY: &str = "memory";
const CONFIRM: &str = "confirm";
const CONNECTIONS: &str = "connections";
const USAGE: &str = "usage";
const BRIEF: &str = "brief";
const PROJECTS: &str = "projects";
const PARAM_ORG: &str = "org_id";
const FREE_PLAN: &str = "Free";

/// One memory. `kind` is `preference`, `person`, `routine`, `goal` or `fact`; `needs_check` marks
/// a strong memory nobody has confirmed in a long while. `embedding` is its raw vector, for the
/// similarity map.
#[derive(Debug, Clone, PartialEq, Deserialize, uniffi::Record)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub weight: f64,
    pub kind: String,
    pub created_at: String,
    pub updated_at: String,
    pub uses: i64,
    pub last_used_at: Option<String>,
    #[serde(default)]
    pub needs_check: bool,
    #[serde(default)]
    pub embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct MemoryList {
    memories: Vec<Memory>,
}

#[derive(Serialize)]
struct EditMemoryBody<'a> {
    content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct GoogleConnection {
    /// Whether Google sign-in is set up on this backend at all.
    pub configured: bool,
    pub email: Option<String>,
    #[serde(default)]
    pub services: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct HealthConnection {
    #[serde(default)]
    pub connected: bool,
    pub account_name: Option<String>,
    pub connected_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ConnectionsOverview {
    pub google: GoogleConnection,
    pub hevy: Option<HealthConnection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct UsagePlan {
    #[serde(default = "free_plan")]
    pub name: String,
    #[serde(default)]
    pub monthly_tokens: i64,
}

/// This month's allowance and how much of it is used.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct UsageBrief {
    pub plan: UsagePlan,
    #[serde(default)]
    pub month: String,
    #[serde(default)]
    pub tokens_used: i64,
    #[serde(default)]
    pub can_upgrade: bool,
}

/// A project in the shown organization.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct Project {
    pub id: String,
    pub session_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
    pub open_issues: i64,
}

#[derive(Deserialize)]
struct ProjectList {
    projects: Vec<Project>,
}

fn free_plan() -> String {
    FREE_PLAN.to_owned()
}

impl Services {
    pub async fn memories(&self) -> Result<Vec<Memory>, NomiError> {
        Ok(self.get::<MemoryList>(&[MEMORY], &[]).await?.memories)
    }

    /// Rewrites a memory (the backend re-embeds it); `kind` changes its kind too.
    pub async fn edit_memory(
        &self,
        id: String,
        content: String,
        kind: Option<String>,
    ) -> Result<(), NomiError> {
        let body = EditMemoryBody {
            content: &content,
            kind: kind.as_deref(),
        };
        self.call(Method::PUT, &[MEMORY, &id], Some(&body)).await
    }

    /// The person says the memory is still true.
    pub async fn confirm_memory(&self, id: String) -> Result<(), NomiError> {
        self.call(Method::POST, &[MEMORY, &id, CONFIRM], NO_BODY)
            .await
    }

    pub async fn delete_memory(&self, id: String) -> Result<(), NomiError> {
        self.call(Method::DELETE, &[MEMORY, &id], NO_BODY).await
    }

    pub async fn connections(&self) -> Result<ConnectionsOverview, NomiError> {
        self.get(&[CONNECTIONS], &[]).await
    }

    pub async fn usage_brief(&self) -> Result<UsageBrief, NomiError> {
        self.get(&[USAGE, BRIEF], &[]).await
    }

    /// Projects in `org_id`, or in the organization the person picked for projects.
    pub async fn projects(&self, org_id: Option<String>) -> Result<Vec<Project>, NomiError> {
        let list: ProjectList = match org_id.as_deref() {
            Some(org) => self.get(&[PROJECTS], &[(PARAM_ORG, org)]).await?,
            None => self.get(&[PROJECTS], &[]).await?,
        };
        Ok(list.projects)
    }
}
