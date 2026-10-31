//! GitHub, for pushing a project's code to a repository: the person's own GitHub (connected
//! through Nomi's OAuth app, or with a token they made), and the Git Data API calls that make one
//! commit of the project's files on the repository's default branch.

use base64::Engine;
use reqwest::{Method, StatusCode};
use serde_json::{json, Value};

/// GitHub's API and sign-in pages; `GITHUB_API_URL` and `GITHUB_OAUTH_URL` point elsewhere
/// (development and tests).
const DEFAULT_API: &str = "https://api.github.com";
const DEFAULT_OAUTH: &str = "https://github.com";
/// What Nomi asks GitHub for: writing to the person's repositories.
const OAUTH_SCOPE: &str = "repo";
/// GitHub refuses requests without a User-Agent.
const USER_AGENT: &str = "nomi";
/// A regular (non-executable) file in a Git tree.
const FILE_MODE: &str = "100644";
/// GitHub's own limits on account and repository names.
const MAX_OWNER_CHARS: usize = 39;
const MAX_REPO_CHARS: usize = 100;

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).ok().map(|v| v.trim().trim_end_matches('/').to_string()).filter(|v| !v.is_empty()).unwrap_or_else(|| default.to_string())
}

pub fn api_base() -> String {
    env_or("GITHUB_API_URL", DEFAULT_API)
}

fn oauth_base() -> String {
    env_or("GITHUB_OAUTH_URL", DEFAULT_OAUTH)
}

/// Nomi's GitHub OAuth app, as Admin → Configuration has it (the environment fills the rest).
/// `None` until a client id, secret and callback address are all set.
pub struct OAuthApp {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl OAuthApp {
    pub async fn current() -> Option<Self> {
        let github = nomi_settings::config::current(&nomi_settings::config::GITHUB).await;
        let get = |field: &str| github.get(field).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        Some(Self { client_id: get("client_id")?, client_secret: get("client_secret")?, redirect_uri: get("redirect_uri")? })
    }

    /// The GitHub page that asks the person to let Nomi in; it comes back to `redirect_uri`.
    pub fn authorize_url(&self, state: &str) -> String {
        let mut url = reqwest::Url::parse(&format!("{}/login/oauth/authorize", oauth_base())).expect("a valid GitHub address");
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("scope", OAUTH_SCOPE)
            .append_pair("state", state);
        url.to_string()
    }

    /// The access token for the `code` GitHub sent back.
    pub async fn exchange(&self, http: &reqwest::Client, code: &str) -> Result<String, GithubError> {
        let response = http
            .post(format!("{}/login/oauth/access_token", oauth_base()))
            .header("accept", "application/json")
            .header("user-agent", USER_AGENT)
            .form(&[("client_id", self.client_id.as_str()), ("client_secret", self.client_secret.as_str()), ("code", code), ("redirect_uri", self.redirect_uri.as_str())])
            .send()
            .await
            .map_err(|e| GithubError::Unreachable(e.to_string()))?;
        let body: Value = response.json().await.map_err(|e| GithubError::Unreachable(e.to_string()))?;
        // GitHub answers 200 with `error` when the code is wrong or used.
        match body["access_token"].as_str() {
            Some(token) if !token.is_empty() => Ok(token.to_string()),
            _ => Err(GithubError::Refused(body["error_description"].as_str().unwrap_or("GitHub didn't accept the sign-in").to_string())),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GithubError {
    /// The token is wrong, expired or revoked.
    TokenRejected,
    NotFound(String),
    /// GitHub understood but won't (no access, a name taken, the branch moved on).
    Refused(String),
    Unreachable(String),
}

impl std::fmt::Display for GithubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GithubError::TokenRejected => write!(f, "GitHub didn't accept the token"),
            GithubError::NotFound(what) | GithubError::Refused(what) | GithubError::Unreachable(what) => write!(f, "{what}"),
        }
    }
}

/// `owner/name` from what the person typed or pasted (a github.com link works too).
pub fn parse_repo(input: &str) -> Option<(String, String)> {
    let trimmed = input.trim().trim_end_matches('/').trim_end_matches(".git");
    let path = ["https://github.com/", "http://github.com/", "github.com/"].iter().find_map(|p| trimmed.strip_prefix(p)).unwrap_or(trimmed);
    let (owner, name) = path.split_once('/')?;
    let owner_ok = !owner.is_empty() && owner.len() <= MAX_OWNER_CHARS && owner.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    let name_ok = !name.is_empty()
        && name.len() <= MAX_REPO_CHARS
        && name != "."
        && name != ".."
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    (owner_ok && name_ok).then(|| (owner.to_string(), name.to_string()))
}

/// What a push made.
#[derive(Debug, serde::Serialize)]
pub struct Pushed {
    pub url: String,
    pub commit: String,
    pub branch: String,
    /// Whether the repository was made for this push.
    pub created: bool,
}

/// GitHub as one person.
pub struct Github<'a> {
    http: &'a reqwest::Client,
    api: String,
    token: &'a str,
}

impl<'a> Github<'a> {
    pub fn new(http: &'a reqwest::Client, token: &'a str) -> Self {
        Self { http, api: api_base(), token }
    }

    async fn call(&self, method: Method, path: &str, body: Option<Value>) -> Result<(StatusCode, Value), GithubError> {
        let mut request = self
            .http
            .request(method, format!("{}{path}", self.api))
            .bearer_auth(self.token)
            .header("accept", "application/vnd.github+json")
            .header("user-agent", USER_AGENT)
            .header("x-github-api-version", "2022-11-28");
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.map_err(|e| GithubError::Unreachable(e.to_string()))?;
        let status = response.status();
        let body = response.json::<Value>().await.unwrap_or(Value::Null);
        Ok((status, body))
    }

    /// `call`, with anything but success as an error.
    async fn ok(&self, method: Method, path: &str, body: Option<Value>) -> Result<Value, GithubError> {
        let (status, body) = self.call(method, path, body).await?;
        if status.is_success() {
            return Ok(body);
        }
        let message = body["message"].as_str().unwrap_or("GitHub refused the request").to_string();
        Err(match status {
            StatusCode::UNAUTHORIZED => GithubError::TokenRejected,
            StatusCode::NOT_FOUND => GithubError::NotFound(message),
            s if s.is_server_error() => GithubError::Unreachable(message),
            _ => GithubError::Refused(message),
        })
    }

    /// Whose token it is.
    pub async fn login(&self) -> Result<String, GithubError> {
        let user = self.ok(Method::GET, "/user", None).await?;
        user["login"].as_str().map(str::to_string).ok_or_else(|| GithubError::Unreachable("GitHub didn't say who this is".to_string()))
    }

    /// One commit of `files` (path, contents) on the default branch of `owner/name`, as the whole
    /// tree: what the project doesn't have any more goes from the repository too. With `create`,
    /// a repository that doesn't exist yet is made first (`private` or not).
    pub async fn push(&self, owner: &str, name: &str, create: bool, private: bool, files: &[(String, Vec<u8>)], message: &str) -> Result<Pushed, GithubError> {
        let base64 = base64::engine::general_purpose::STANDARD;
        let repo_path = format!("/repos/{owner}/{name}");
        let (repo, created) = match self.ok(Method::GET, &repo_path, None).await {
            Ok(repo) => (repo, false),
            Err(GithubError::NotFound(_)) if create => {
                let login = self.login().await?;
                let path = if owner.eq_ignore_ascii_case(&login) { "/user/repos".to_string() } else { format!("/orgs/{owner}/repos") };
                // A first commit (its README) gives the new repository a branch to commit onto.
                (self.ok(Method::POST, &path, Some(json!({ "name": name, "private": private, "auto_init": true }))).await?, true)
            }
            Err(GithubError::NotFound(_)) => return Err(GithubError::NotFound(format!("{owner}/{name} doesn't exist, or this GitHub can't see it"))),
            Err(e) => return Err(e),
        };
        let branch = repo["default_branch"].as_str().unwrap_or("main").to_string();
        let url = repo["html_url"].as_str().unwrap_or_default().to_string();
        let ref_path = format!("{repo_path}/git/ref/heads/{branch}");

        // An empty repository has no branch, and the Git Data API won't write to it: one file
        // through the contents API makes the first commit.
        let parent = match self.call(Method::GET, &ref_path, None).await? {
            (status, body) if status.is_success() => body["object"]["sha"].as_str().unwrap_or_default().to_string(),
            (StatusCode::NOT_FOUND | StatusCode::CONFLICT, _) => {
                let (path, content) = files.first().ok_or_else(|| GithubError::Refused("there's nothing to push".to_string()))?;
                let made = self
                    .ok(Method::PUT, &format!("{repo_path}/contents/{path}"), Some(json!({ "message": message, "content": base64.encode(content), "branch": branch })))
                    .await?;
                made["commit"]["sha"].as_str().unwrap_or_default().to_string()
            }
            (StatusCode::UNAUTHORIZED, _) => return Err(GithubError::TokenRejected),
            (_, body) => return Err(GithubError::Refused(body["message"].as_str().unwrap_or("GitHub refused the request").to_string())),
        };

        let mut tree = Vec::with_capacity(files.len());
        for (path, content) in files {
            let blob = self.ok(Method::POST, &format!("{repo_path}/git/blobs"), Some(json!({ "content": base64.encode(content), "encoding": "base64" }))).await?;
            tree.push(json!({ "path": path, "mode": FILE_MODE, "type": "blob", "sha": blob["sha"] }));
        }
        let tree = self.ok(Method::POST, &format!("{repo_path}/git/trees"), Some(json!({ "tree": tree }))).await?;
        let commit = self
            .ok(Method::POST, &format!("{repo_path}/git/commits"), Some(json!({ "message": message, "tree": tree["sha"], "parents": [parent] })))
            .await?;
        let sha = commit["sha"].as_str().unwrap_or_default().to_string();
        // Not forced: if someone pushed meanwhile, GitHub refuses rather than lose their work.
        self.ok(Method::PATCH, &format!("{repo_path}/git/refs/heads/{branch}"), Some(json!({ "sha": sha, "force": false }))).await?;
        Ok(Pushed { url, commit: sha, branch, created })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_owner_and_name_from_what_was_pasted() {
        assert_eq!(parse_repo("ana/todo"), Some(("ana".into(), "todo".into())));
        assert_eq!(parse_repo(" https://github.com/acme-co/site.v2.git/ "), Some(("acme-co".into(), "site.v2".into())));
        assert_eq!(parse_repo("github.com/ana/my_app"), Some(("ana".into(), "my_app".into())));
    }

    #[test]
    fn refuses_what_isnt_a_repository() {
        for input in ["ana", "ana/", "/todo", "ana/to do", "ana/todo/tree/main", "an a/todo", "ana/..", &format!("{}/x", "a".repeat(40))] {
            assert_eq!(parse_repo(input), None, "{input}");
        }
    }
}
