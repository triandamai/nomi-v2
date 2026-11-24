//! Skills that come from somewhere else: an uploaded `SKILL.md`, a `.zip` of a skill folder, or a
//! link (a GitHub folder or file, or any `https` URL to a `.md`).
//!
//! Nothing touches the disk: a zip is read in memory, and what is read counts against a budget as
//! it is decompressed, never by the sizes the archive claims. Every request of a link import goes
//! through the web tools' public-host guard first (redirects included), so a link can't reach the
//! server's own network.

use std::io::{Cursor, Read};
use std::time::Duration;

use nomi_skills::{validate, ParsedSkill, Reference, SkillError, MAX_REFERENCES, MAX_SKILL_BYTES};
use reqwest::{StatusCode, Url};
use serde::Deserialize;

/// The most entries a skill's zip may hold.
pub const MAX_ZIP_ENTRIES: usize = 64;
/// The most a skill's zip may unpack to.
pub const MAX_ZIP_BYTES: u64 = 512 * 1024;
/// How long one request of a link import may take.
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
/// Requests one link import may make: two folder listings, the SKILL.md and every reference.
pub const MAX_FETCHES: usize = 3 + MAX_REFERENCES;
/// The most redirects one request follows.
const MAX_REDIRECTS: usize = 3;
/// The most a folder listing may be.
const MAX_LISTING_BYTES: usize = 256 * 1024;
/// GitHub's API, where folders are listed.
pub const GITHUB_API: &str = "https://api.github.com";
/// GitHub's raw file host.
pub const GITHUB_RAW: &str = "https://raw.githubusercontent.com";
const GITHUB_HOST: &str = "github.com";
const GITHUB_RAW_HOST: &str = "raw.githubusercontent.com";
const SKILL_FILE: &str = "SKILL.md";
const REFERENCES_DIR: &str = "references";
const MARKDOWN: &str = ".md";
const USER_AGENT: &str = "nomi-skill-import";

/// Why an import didn't give a skill.
#[derive(Debug)]
pub enum ImportError {
    /// Not a `.md` or a `.zip`.
    UnsupportedFile,
    /// Whatever it is, it isn't a skill (no SKILL.md, two of them, not text, not a skill link…).
    NotASkill(String),
    /// A SKILL.md that breaks a rule.
    Invalid(SkillError),
    /// More than a skill may be.
    TooLarge,
    /// The link couldn't be read.
    FetchFailed(String),
    /// The link points at the server's own network.
    PrivateAddress,
    /// GitHub turned the request away for now.
    RateLimited,
}

impl From<SkillError> for ImportError {
    fn from(e: SkillError) -> Self {
        match e {
            SkillError::TooLarge => ImportError::TooLarge,
            other => ImportError::Invalid(other),
        }
    }
}

/// A skill ready to save, and the files that came with it but aren't part of a skill.
#[derive(Debug)]
pub struct Imported {
    pub skill: ParsedSkill,
    pub skipped: Vec<String>,
}

fn text_of(bytes: Vec<u8>, what: &str) -> Result<String, ImportError> {
    String::from_utf8(bytes).map_err(|_| ImportError::NotASkill(format!("{what} isn't text")))
}

/// A single SKILL.md.
pub fn from_markdown(bytes: &[u8]) -> Result<Imported, ImportError> {
    if bytes.len() > MAX_SKILL_BYTES {
        return Err(ImportError::TooLarge);
    }
    let text = text_of(bytes.to_vec(), "that file")?;
    Ok(Imported { skill: validate(&text, vec![])?, skipped: vec![] })
}

/// A path inside an archive that could point outside it.
fn unsafe_path(name: &str) -> bool {
    name.starts_with('/') || name.contains('\\') || name.contains(':') || name.split('/').any(|part| part == "..")
}

fn is_markdown(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(MARKDOWN)
}

/// A `.zip` of one skill: `SKILL.md` at the root or in one top folder, `references/*.md` beside it.
/// Anything else is skipped and named.
pub fn from_zip(bytes: &[u8]) -> Result<Imported, ImportError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| ImportError::NotASkill("that isn't a zip archive".into()))?;
    if archive.len() > MAX_ZIP_ENTRIES {
        return Err(ImportError::TooLarge);
    }
    let mut files = Vec::new();
    for i in 0..archive.len() {
        let entry = archive.by_index_raw(i).map_err(|_| ImportError::NotASkill("that zip archive is damaged".into()))?;
        let name = entry.name().to_string();
        if unsafe_path(&name) {
            return Err(ImportError::NotASkill(format!("{name} points outside the archive")));
        }
        if !entry.is_dir() {
            files.push((i, name));
        }
    }
    let skill_files: Vec<&(usize, String)> = files
        .iter()
        .filter(|(_, name)| {
            let parts: Vec<&str> = name.split('/').collect();
            parts.len() <= 2 && parts.last().is_some_and(|last| last.eq_ignore_ascii_case(SKILL_FILE))
        })
        .collect();
    let (skill_index, skill_name) = match skill_files.as_slice() {
        [one] => (one.0, one.1.clone()),
        [] => return Err(ImportError::NotASkill("there's no SKILL.md at the top of the archive or in its one folder".into())),
        _ => return Err(ImportError::NotASkill("the archive holds more than one SKILL.md".into())),
    };
    let prefix = skill_name.rsplit_once('/').map(|(dir, _)| format!("{dir}/")).unwrap_or_default();
    let references_prefix = format!("{prefix}{REFERENCES_DIR}/");

    let mut budget = MAX_ZIP_BYTES;
    let mut read = |archive: &mut zip::ZipArchive<Cursor<&[u8]>>, index: usize, name: &str| -> Result<String, ImportError> {
        let entry = archive.by_index(index).map_err(|_| ImportError::NotASkill(format!("{name} can't be unpacked")))?;
        let mut out = Vec::new();
        entry.take(budget + 1).read_to_end(&mut out).map_err(|_| ImportError::NotASkill(format!("{name} can't be unpacked")))?;
        let len = out.len() as u64;
        if len > budget {
            return Err(ImportError::TooLarge);
        }
        budget -= len;
        text_of(out, name)
    };

    let text = read(&mut archive, skill_index, &skill_name)?;
    let mut references = Vec::new();
    let mut skipped = Vec::new();
    for (index, name) in &files {
        if *index == skill_index {
            continue;
        }
        match name.strip_prefix(&references_prefix) {
            Some(file) if !file.contains('/') && is_markdown(file) => {
                references.push(Reference { name: file.to_string(), text: read(&mut archive, *index, name)? });
            }
            _ => skipped.push(name.clone()),
        }
    }
    Ok(Imported { skill: validate(&text, references)?, skipped })
}

/// What a link points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkKind {
    /// `github.com/<owner>/<repo>/tree/<ref>/<path>`: a skill folder.
    GithubFolder { owner: String, repo: String, git_ref: String, path: String },
    /// One file on GitHub's raw host (a `blob` link is turned into one).
    RawFile { url: String },
    /// Any other `https` link to a `.md`.
    Other { url: String },
}

/// Which kind of skill link `url` is; links that can't be one are refused before any request.
pub fn classify(url: &str) -> Result<LinkKind, ImportError> {
    let parsed = Url::parse(url.trim()).map_err(|_| ImportError::NotASkill("that isn't a link".into()))?;
    if parsed.scheme() != "https" {
        return Err(ImportError::NotASkill("a skill link starts with https://".into()));
    }
    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    let parts: Vec<&str> = parsed.path_segments().map(|s| s.filter(|p| !p.is_empty()).collect()).unwrap_or_default();
    match host.as_str() {
        GITHUB_HOST => match parts.as_slice() {
            [owner, repo, "tree", git_ref, path @ ..] => {
                Ok(LinkKind::GithubFolder { owner: owner.to_string(), repo: repo.to_string(), git_ref: git_ref.to_string(), path: path.join("/") })
            }
            [owner, repo, "blob", git_ref, path @ ..] if !path.is_empty() => {
                Ok(LinkKind::RawFile { url: format!("{GITHUB_RAW}/{owner}/{repo}/{git_ref}/{}", path.join("/")) })
            }
            _ => Err(ImportError::NotASkill("a GitHub link to a skill is a folder (…/tree/…) or a file (…/blob/…)".into())),
        },
        GITHUB_RAW_HOST => Ok(LinkKind::RawFile { url: parsed.to_string() }),
        _ if is_markdown(parsed.path()) => Ok(LinkKind::Other { url: parsed.to_string() }),
        _ => Err(ImportError::NotASkill("a skill link points at a folder on GitHub or at a .md file".into())),
    }
}

/// A skill from a link, against GitHub itself.
pub async fn from_link(url: &str, github_token: Option<&str>) -> Result<Imported, ImportError> {
    from_link_with_bases(url, github_token, GITHUB_API, GITHUB_RAW).await
}

#[derive(Deserialize)]
struct ListedEntry {
    name: String,
    #[serde(rename = "type")]
    kind: String,
}

/// One link import's requests, counted and guarded.
struct Fetcher {
    client: reqwest::Client,
    left: usize,
}

impl Fetcher {
    async fn get(&mut self, url: &str, token: Option<&str>, limit: usize) -> Result<String, ImportError> {
        let mut url = Url::parse(url).map_err(|_| ImportError::FetchFailed(format!("{url} isn't a link")))?;
        let mut token = token;
        for _ in 0..=MAX_REDIRECTS {
            if self.left == 0 {
                return Err(ImportError::TooLarge);
            }
            self.left -= 1;
            let host = url.host_str().unwrap_or_default().to_string();
            let port = url.port_or_known_default().unwrap_or_default();
            if let Err(reason) = nomi_tools_web::ensure_public_host(&host, port).await {
                return Err(if reason.contains("private") { ImportError::PrivateAddress } else { ImportError::FetchFailed(reason) });
            }
            let mut request = self.client.get(url.clone()).header(reqwest::header::USER_AGENT, USER_AGENT);
            if let Some(token) = token {
                request = request.bearer_auth(token);
            }
            let response = request.send().await.map_err(|e| ImportError::FetchFailed(e.without_url().to_string()))?;
            let status = response.status();
            if status.is_redirection() {
                let location = response.headers().get(reqwest::header::LOCATION).and_then(|v| v.to_str().ok()).unwrap_or_default().to_string();
                url = url.join(&location).map_err(|_| ImportError::FetchFailed("a redirect without a place to go".into()))?;
                // The token is for the API only, never for wherever a redirect leads.
                token = None;
                continue;
            }
            if status == StatusCode::FORBIDDEN || status == StatusCode::TOO_MANY_REQUESTS {
                return Err(ImportError::RateLimited);
            }
            if !status.is_success() {
                return Err(ImportError::FetchFailed(format!("{} answered {status}", url.path())));
            }
            return read_capped(response, limit).await.and_then(|bytes| text_of(bytes, url.path()));
        }
        Err(ImportError::FetchFailed("too many redirects".into()))
    }
}

async fn read_capped(mut response: reqwest::Response, limit: usize) -> Result<Vec<u8>, ImportError> {
    let mut out = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| ImportError::FetchFailed(e.without_url().to_string()))? {
        out.extend_from_slice(&chunk);
        if out.len() > limit {
            return Err(ImportError::TooLarge);
        }
    }
    Ok(out)
}

/// `from_link` with GitHub's hosts replaced (`api_base`, `raw_base`): the tests' stub.
pub async fn from_link_with_bases(url: &str, github_token: Option<&str>, api_base: &str, raw_base: &str) -> Result<Imported, ImportError> {
    let kind = classify(url)?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(FETCH_TIMEOUT)
        .build()
        .map_err(|e| ImportError::FetchFailed(e.to_string()))?;
    let mut fetcher = Fetcher { client, left: MAX_FETCHES };
    let on_raw = |url: &str| url.replacen(GITHUB_RAW, raw_base, 1);
    match kind {
        LinkKind::RawFile { url } => {
            let text = fetcher.get(&on_raw(&url), None, MAX_SKILL_BYTES).await?;
            Ok(Imported { skill: validate(&text, vec![])?, skipped: vec![] })
        }
        LinkKind::Other { url } => {
            let text = fetcher.get(&url, None, MAX_SKILL_BYTES).await?;
            Ok(Imported { skill: validate(&text, vec![])?, skipped: vec![] })
        }
        LinkKind::GithubFolder { owner, repo, git_ref, path } => {
            let joined = |name: &str| if path.is_empty() { name.to_string() } else { format!("{path}/{name}") };
            let list = |path: &str| format!("{api_base}/repos/{owner}/{repo}/contents/{path}?ref={git_ref}");
            let raw = |path: &str| format!("{raw_base}/{owner}/{repo}/{git_ref}/{path}");
            let parse = |text: String| -> Result<Vec<ListedEntry>, ImportError> {
                serde_json::from_str(&text).map_err(|_| ImportError::NotASkill("that GitHub link isn't a folder".into()))
            };

            let entries = parse(fetcher.get(&list(&path), github_token, MAX_LISTING_BYTES).await?)?;
            let Some(skill) = entries.iter().find(|e| e.kind == "file" && e.name.eq_ignore_ascii_case(SKILL_FILE)) else {
                return Err(ImportError::NotASkill("that folder has no SKILL.md".into()));
            };
            let mut skipped = Vec::new();
            let mut reference_files = Vec::new();
            for entry in &entries {
                if entry.name == skill.name {
                    continue;
                }
                if entry.kind == "dir" && entry.name == REFERENCES_DIR {
                    let references_path = joined(REFERENCES_DIR);
                    for listed in parse(fetcher.get(&list(&references_path), github_token, MAX_LISTING_BYTES).await?)? {
                        let path = format!("{references_path}/{}", listed.name);
                        if listed.kind == "file" && is_markdown(&listed.name) {
                            reference_files.push((listed.name, path));
                        } else {
                            skipped.push(path);
                        }
                    }
                } else {
                    skipped.push(joined(&entry.name));
                }
            }
            if reference_files.len() > MAX_REFERENCES {
                return Err(ImportError::Invalid(SkillError::TooManyReferences));
            }
            let text = fetcher.get(&raw(&joined(&skill.name)), None, MAX_SKILL_BYTES).await?;
            let mut references = Vec::new();
            for (name, path) in reference_files {
                references.push(Reference { name, text: fetcher.get(&raw(&path), None, MAX_SKILL_BYTES).await? });
            }
            Ok(Imported { skill: validate(&text, references)?, skipped })
        }
    }
}
