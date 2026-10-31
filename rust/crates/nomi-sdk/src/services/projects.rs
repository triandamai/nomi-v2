//! One project's workspace (`routes/{projects,deploy,project_issues,project_members}.rs`): the
//! project behind a chat, its files, where it's published, its issues and who works on it.

use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{Services, NO_BODY};
use crate::error::NomiError;

const PROJECTS: &str = "projects";
const BY_SESSION: &str = "by-session";
const FILES: &str = "files";
const DEPLOY: &str = "deploy";
const PUBLISH: &str = "publish";
const ISSUES: &str = "issues";
const MEMBERS: &str = "members";
const PARAM_STATUS: &str = "status";
const PATH_SEPARATOR: char = '/';

/// A file in a project, as listed with it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ProjectFile {
    pub path: String,
    pub content_type: String,
    pub size_bytes: i64,
}

/// A project with everything its workspace header needs. `status` is `planning`, `building`,
/// `ready` or `failed`; `backend` is `drizzle` or `relay`; `session_id` is the project's chat.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ProjectDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub stack: String,
    pub files: Vec<ProjectFile>,
    pub session_id: String,
    pub org_id: String,
    pub org_name: String,
    pub open_issues: i64,
    pub backend: String,
}

/// One publish of the project. `status` is `queued`, `building`, `deploying`, `live`, `failed` or
/// `superseded`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ProjectPublish {
    pub id: String,
    pub environment: String,
    pub status: String,
    pub url: Option<String>,
    pub error: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
    pub finished_at: Option<String>,
}

/// The project's production backend while it's still being set up.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct BackendSetup {
    pub state: String,
    pub step: Option<String>,
    pub error: Option<String>,
}

/// Where the project is published. `available` is whether this Nomi can publish at all; `role` is
/// the person's (`admin` publishes); `url` is the newest successful production address.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct DeployOverview {
    pub available: bool,
    pub role: String,
    pub url: Option<String>,
    pub publishes: Vec<ProjectPublish>,
    pub backend_setup: Option<BackendSetup>,
}

/// An issue filed on the project. `kind` is `bug`, `feature` or `task`; `severity` `low`,
/// `medium`, `high` or `critical`; `status` `open` or `closed`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ProjectIssue {
    pub number: i32,
    pub title: String,
    pub body: String,
    pub kind: String,
    pub severity: String,
    pub status: String,
    pub created_by: Option<String>,
    pub mine: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct IssueCounts {
    pub open: i64,
    pub closed: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ProjectIssues {
    pub issues: Vec<ProjectIssue>,
    pub counts: IssueCounts,
}

/// An issue to file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, uniffi::Record)]
pub struct NewProjectIssue {
    pub title: String,
    pub body: String,
    pub kind: String,
    pub severity: String,
}

#[derive(Serialize)]
struct IssueStatusChange<'a> {
    status: &'a str,
}

/// Someone in the project's organization. `project_role` is set when they were added to it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ProjectPerson {
    pub user_id: String,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub org_role: String,
    pub project_role: Option<String>,
    pub is_owner: bool,
}

/// Who works on the project: `access` is `org` (everyone in the organization) or `members`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, uniffi::Record)]
pub struct ProjectMembers {
    pub access: String,
    pub my_role: String,
    pub people: Vec<ProjectPerson>,
}

impl Services {
    /// The project whose chat is `session_id`; `NomiError::Http` 404 for an ordinary chat.
    pub async fn project_for_chat(&self, session_id: &str) -> Result<ProjectDetail, NomiError> {
        self.get(&[PROJECTS, BY_SESSION, session_id], &[]).await
    }

    /// A project file's content, as stored.
    pub async fn project_file(&self, project_id: &str, path: &str) -> Result<Vec<u8>, NomiError> {
        let mut segments = vec![PROJECTS, project_id, FILES];
        segments.extend(path.split(PATH_SEPARATOR).filter(|part| !part.is_empty()));
        self.get_bytes(&segments).await
    }

    pub async fn project_deploy(&self, project_id: &str) -> Result<DeployOverview, NomiError> {
        self.get(&[PROJECTS, project_id, DEPLOY], &[]).await
    }

    /// Publishes the project to production; the overview follows its progress.
    pub async fn publish_project(&self, project_id: &str) -> Result<(), NomiError> {
        self.call(
            Method::POST,
            &[PROJECTS, project_id, DEPLOY, PUBLISH],
            NO_BODY,
        )
        .await
    }

    /// Issues with `status` `open`, `closed` or `all`, newest first.
    pub async fn project_issues(
        &self,
        project_id: &str,
        status: &str,
    ) -> Result<ProjectIssues, NomiError> {
        self.get(&[PROJECTS, project_id, ISSUES], &[(PARAM_STATUS, status)])
            .await
    }

    pub async fn file_project_issue(
        &self,
        project_id: &str,
        issue: &NewProjectIssue,
    ) -> Result<ProjectIssue, NomiError> {
        self.call_for(Method::POST, &[PROJECTS, project_id, ISSUES], Some(issue))
            .await
    }

    /// Closes or reopens issue `number` (`status` is `closed` or `open`).
    pub async fn set_project_issue_status(
        &self,
        project_id: &str,
        number: i32,
        status: &str,
    ) -> Result<ProjectIssue, NomiError> {
        let number = number.to_string();
        self.call_for(
            Method::PATCH,
            &[PROJECTS, project_id, ISSUES, &number],
            Some(&IssueStatusChange { status }),
        )
        .await
    }

    pub async fn project_members(&self, project_id: &str) -> Result<ProjectMembers, NomiError> {
        self.get(&[PROJECTS, project_id, MEMBERS], &[]).await
    }
}
