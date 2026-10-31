//! A project's tools beyond the workspace itself (`routes/{domain,deploy,project_database,
//! project_relay,showcase,projects}.rs`): its address, how production is doing, its data, and the
//! project's settings (showcase, offline, transfer, delete).

use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Services, NO_BODY};
use crate::error::NomiError;

const PROJECTS: &str = "projects";
const DOMAIN: &str = "domain";
const CHECK: &str = "check";
const DEPLOY: &str = "deploy";
const USAGE: &str = "usage";
const METRICS: &str = "metrics";
const LOGS: &str = "logs";
const ENVIRONMENTS: &str = "environments";
const PRODUCTION: &str = "production";
const DATABASE: &str = "database";
const TABLES: &str = "tables";
const QUERY: &str = "query";
const RELAY: &str = "relay";
const RETRY: &str = "retry";
const SHOWCASE: &str = "showcase";
const TRANSFER: &str = "transfer";
const ORGS: &str = "orgs";
const PARAM_SUBDOMAIN: &str = "subdomain";
const PARAM_RANGE: &str = "range";
const PARAM_TAIL: &str = "tail";
const PARAM_SOURCE: &str = "source";
const PARAM_QUERY: &str = "q";
/// How many of production's latest log lines the monitor reads, as the web's does.
const LOG_TAIL: &str = "200";

/// The project's production address. `hostname` is unset until the first publish gives it one.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ProjectDomain {
    pub hostname: Option<String>,
    pub subdomain: Option<String>,
    pub url: Option<String>,
    pub can_edit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct DomainSuggestion {
    pub subdomain: String,
    pub hostname: String,
}

/// Whether a name is free; when it isn't, why (`taken` or `invalid`) and free names instead.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct DomainCheck {
    #[serde(default)]
    pub subdomain: String,
    pub hostname: Option<String>,
    pub available: bool,
    pub reason: Option<String>,
    pub message: Option<String>,
    #[serde(default)]
    pub suggestions: Vec<DomainSuggestion>,
}

/// The outcome of a rename: the new address, or (refused) the check that says why.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum DomainChange {
    Changed { hostname: String },
    Refused { check: DomainCheck },
}

#[derive(Serialize)]
struct SubdomainBody<'a> {
    subdomain: &'a str,
}

#[derive(Deserialize)]
struct Changed {
    hostname: String,
}

/// One running container in production and what it uses right now (unset while it reports nothing).
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct ContainerUsage {
    pub service: String,
    pub cpu_percent: Option<f64>,
    pub memory_usage_bytes: Option<i64>,
    pub memory_limit_bytes: Option<i64>,
}

#[derive(Deserialize)]
struct UsageAnswer {
    #[serde(default)]
    containers: Vec<RawContainer>,
}

#[derive(Deserialize)]
struct RawContainer {
    service: String,
    stats: Option<RawStats>,
}

#[derive(Deserialize)]
struct RawStats {
    cpu_percent: Option<f64>,
    memory_usage_bytes: Option<f64>,
    memory_limit_bytes: Option<f64>,
}

/// Traffic over a period; any figure the backend can't tell is unset.
#[derive(Debug, Clone, PartialEq, Deserialize, uniffi::Record)]
pub struct TrafficCounters {
    pub visitors: Option<f64>,
    pub requests: Option<f64>,
    pub errors_5xx: Option<f64>,
    pub errors_4xx: Option<f64>,
    pub p50_ms: Option<f64>,
    pub p95_ms: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, uniffi::Record)]
pub struct TrafficBucket {
    pub at: String,
    pub requests: Option<f64>,
    pub errors_5xx: Option<f64>,
}

/// Production's traffic over `range` (`1h`, `24h` or `7d`), against the period before.
#[derive(Debug, Clone, PartialEq, Deserialize, uniffi::Record)]
pub struct ProjectTraffic {
    pub range: String,
    #[serde(default)]
    pub buckets: Vec<TrafficBucket>,
    pub totals: TrafficCounters,
    pub previous_totals: TrafficCounters,
}

/// Production's latest log lines, and a note when there are none to read.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ProjectLogs {
    #[serde(default)]
    pub lines: Vec<String>,
    pub note: Option<String>,
}

/// Which copy of the project's data: the one it's built against, or production's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum DataSource {
    Development,
    Production,
}

impl DataSource {
    fn as_str(self) -> &'static str {
        match self {
            DataSource::Development => "development",
            DataSource::Production => PRODUCTION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct DatabaseTable {
    pub schema: String,
    pub name: String,
    pub rows_estimate: i64,
}

/// A query's answer. Cells come as text (`null` is unset); `truncated` when there were more rows.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<String>>>,
    pub truncated: bool,
    pub rows_affected: Option<i64>,
    pub took_ms: f64,
}

#[derive(Deserialize)]
struct RawQueryResult {
    #[serde(default)]
    columns: Vec<String>,
    #[serde(default)]
    rows: Vec<Vec<Value>>,
    #[serde(default)]
    truncated: bool,
    rows_affected: Option<i64>,
    #[serde(default)]
    took_ms: f64,
}

impl From<RawQueryResult> for QueryResult {
    fn from(raw: RawQueryResult) -> Self {
        let cell = |value: Value| match value {
            Value::Null => None,
            Value::String(text) => Some(text),
            other => Some(other.to_string()),
        };
        Self {
            columns: raw.columns,
            rows: raw
                .rows
                .into_iter()
                .map(|row| row.into_iter().map(cell).collect())
                .collect(),
            truncated: raw.truncated,
            rows_affected: raw.rows_affected,
            took_ms: raw.took_ms,
        }
    }
}

#[derive(Serialize)]
struct QueryBody<'a> {
    sql: &'a str,
    source: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct RelayColumn {
    pub name: String,
    #[serde(rename = "type")]
    pub column_type: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct RelayTable {
    pub name: String,
    pub rls_enabled: bool,
    #[serde(default)]
    pub columns: Vec<RelayColumn>,
}

/// A Relay backend still starting (`creating`, `sleeping`, `waking` or `failed`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct RelaySetup {
    pub state: String,
    pub step: Option<String>,
    pub error: Option<String>,
}

/// A project's Relay backend: still being set up, or its address and tables.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct RelayOverview {
    pub backend: String,
    pub setup: Option<RelaySetup>,
    pub url: Option<String>,
    #[serde(default)]
    pub tables: Vec<RelayTable>,
}

/// Whether the project is on the public showcase, with its blurb.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ProjectShowcase {
    pub showcase: bool,
    pub blurb: Option<String>,
    pub url: Option<String>,
    pub can_edit: bool,
}

#[derive(Serialize)]
struct ShowcaseBody<'a> {
    showcase: bool,
    blurb: Option<&'a str>,
}

/// An organization the project can move to.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct TransferOrg {
    pub id: String,
    pub name: String,
    pub is_personal: bool,
    pub owner_name: Option<String>,
}

#[derive(Serialize)]
struct TransferBody<'a> {
    target_org_id: &'a str,
    confirm_name: &'a str,
}

/// The backend's refusal of a rename, whose body says why.
const HTTP_CONFLICT: u16 = 409;

impl Services {
    pub async fn project_domain(&self, project_id: &str) -> Result<ProjectDomain, NomiError> {
        self.get(&[PROJECTS, project_id, DOMAIN], &[]).await
    }

    pub async fn check_project_domain(
        &self,
        project_id: &str,
        subdomain: &str,
    ) -> Result<DomainCheck, NomiError> {
        self.get(
            &[PROJECTS, project_id, DOMAIN, CHECK],
            &[(PARAM_SUBDOMAIN, subdomain)],
        )
        .await
    }

    /// Moves production to `subdomain`; a refusal comes back with why, not as an error.
    pub async fn change_project_domain(
        &self,
        project_id: &str,
        subdomain: &str,
    ) -> Result<DomainChange, NomiError> {
        let response = self
            .request(
                Method::PUT,
                &[PROJECTS, project_id, DOMAIN],
                Some(&SubdomainBody { subdomain }),
            )
            .await?;
        if response.status().as_u16() == HTTP_CONFLICT {
            let check = crate::network::http::json_any(response).await?;
            return Ok(DomainChange::Refused { check });
        }
        let changed: Changed = crate::network::http::json(response).await?;
        Ok(DomainChange::Changed {
            hostname: changed.hostname,
        })
    }

    pub async fn project_usage(&self, project_id: &str) -> Result<Vec<ContainerUsage>, NomiError> {
        let answer: UsageAnswer = self
            .get(&[PROJECTS, project_id, DEPLOY, USAGE], &[])
            .await?;
        Ok(answer
            .containers
            .into_iter()
            .map(|container| ContainerUsage {
                service: container.service,
                cpu_percent: container.stats.as_ref().and_then(|s| s.cpu_percent),
                memory_usage_bytes: container
                    .stats
                    .as_ref()
                    .and_then(|s| s.memory_usage_bytes)
                    .map(|bytes| bytes as i64),
                memory_limit_bytes: container
                    .stats
                    .as_ref()
                    .and_then(|s| s.memory_limit_bytes)
                    .map(|bytes| bytes as i64),
            })
            .collect())
    }

    pub async fn project_traffic(
        &self,
        project_id: &str,
        range: &str,
    ) -> Result<ProjectTraffic, NomiError> {
        self.get(
            &[PROJECTS, project_id, DEPLOY, METRICS],
            &[(PARAM_RANGE, range)],
        )
        .await
    }

    pub async fn project_logs(&self, project_id: &str) -> Result<ProjectLogs, NomiError> {
        self.get(
            &[PROJECTS, project_id, DEPLOY, LOGS],
            &[(PARAM_TAIL, LOG_TAIL)],
        )
        .await
    }

    /// Takes production offline; the next publish brings it back.
    pub async fn take_project_offline(&self, project_id: &str) -> Result<(), NomiError> {
        self.call(
            Method::DELETE,
            &[PROJECTS, project_id, DEPLOY, ENVIRONMENTS, PRODUCTION],
            NO_BODY,
        )
        .await
    }

    pub async fn project_tables(
        &self,
        project_id: &str,
        source: DataSource,
    ) -> Result<Vec<DatabaseTable>, NomiError> {
        self.get(
            &[PROJECTS, project_id, DATABASE, TABLES],
            &[(PARAM_SOURCE, source.as_str())],
        )
        .await
    }

    /// Runs `sql` against the project's database (`relay` for a Relay backend).
    pub async fn query_project_data(
        &self,
        project_id: &str,
        sql: &str,
        source: DataSource,
        relay: bool,
    ) -> Result<QueryResult, NomiError> {
        let area = if relay { RELAY } else { DATABASE };
        let raw: RawQueryResult = self
            .call_for(
                Method::POST,
                &[PROJECTS, project_id, area, QUERY],
                Some(&QueryBody {
                    sql,
                    source: source.as_str(),
                }),
            )
            .await?;
        Ok(raw.into())
    }

    pub async fn project_relay(
        &self,
        project_id: &str,
        source: DataSource,
    ) -> Result<RelayOverview, NomiError> {
        self.get(
            &[PROJECTS, project_id, RELAY],
            &[(PARAM_SOURCE, source.as_str())],
        )
        .await
    }

    /// Starts a Relay backend that failed to set up again.
    pub async fn retry_project_relay(
        &self,
        project_id: &str,
        source: DataSource,
    ) -> Result<(), NomiError> {
        let mut url = self.auth.api_url(&[PROJECTS, project_id, RELAY, RETRY]);
        url.query_pairs_mut()
            .append_pair(PARAM_SOURCE, source.as_str());
        let response = self.auth.send(|client| client.post(url.clone())).await?;
        crate::network::http::success(response).await.map(drop)
    }

    pub async fn project_showcase(&self, project_id: &str) -> Result<ProjectShowcase, NomiError> {
        self.get(&[PROJECTS, project_id, SHOWCASE], &[]).await
    }

    pub async fn set_project_showcase(
        &self,
        project_id: &str,
        showcase: bool,
        blurb: Option<&str>,
    ) -> Result<ProjectShowcase, NomiError> {
        self.call_for(
            Method::PUT,
            &[PROJECTS, project_id, SHOWCASE],
            Some(&ShowcaseBody { showcase, blurb }),
        )
        .await
    }

    pub async fn transfer_orgs(
        &self,
        project_id: &str,
        query: &str,
    ) -> Result<Vec<TransferOrg>, NomiError> {
        self.get(
            &[PROJECTS, project_id, TRANSFER, ORGS],
            &[(PARAM_QUERY, query)],
        )
        .await
    }

    /// Moves the project to `org_id`; `confirm_name` must be the project's name.
    pub async fn transfer_project(
        &self,
        project_id: &str,
        org_id: &str,
        confirm_name: &str,
    ) -> Result<(), NomiError> {
        self.call(
            Method::POST,
            &[PROJECTS, project_id, TRANSFER],
            Some(&TransferBody {
                target_org_id: org_id,
                confirm_name,
            }),
        )
        .await
    }

    /// Deletes the project, its files and its publishes for good.
    pub async fn delete_project(&self, project_id: &str) -> Result<(), NomiError> {
        self.call(Method::DELETE, &[PROJECTS, project_id], NO_BODY)
            .await
    }
}
