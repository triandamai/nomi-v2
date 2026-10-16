//! Bringing existing code into a project: a zip archive the person uploads, or a public GitHub
//! repository. The files land in the project as if Koda had written them, its stack is read
//! from package.json, and the open project page picks them up.

use std::io::Read;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, project_role, ApiError};

/// The biggest archive accepted.
pub const MAX_ARCHIVE_BYTES: usize = 50 * 1024 * 1024;
const MAX_FILES: usize = 2000;
/// Larger files are skipped (lock files and assets rarely need editing in the project).
const MAX_FILE_BYTES: usize = 1024 * 1024;

#[derive(Serialize, Debug, PartialEq)]
pub struct ImportResult {
    pub imported: usize,
    /// Paths left out: folders that are never sent (node_modules, .git, build output), binary
    /// files and files over 1 MB.
    pub skipped: usize,
    pub stack: String,
}

/// The files of a zip, with a top folder every path shares removed (GitHub's `repo-main/`).
pub fn read_archive(bytes: &[u8]) -> Result<(Vec<(String, String)>, usize), String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|_| "that isn't a zip file".to_string())?;
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut skipped = 0;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("the archive is damaged: {e}"))?;
        if file.is_dir() {
            continue;
        }
        let Some(path) = file.enclosed_name().map(|p| p.to_string_lossy().replace('\\', "/")) else {
            skipped += 1;
            continue;
        };
        if file.size() as usize > MAX_FILE_BYTES {
            skipped += 1;
            continue;
        }
        let mut content = Vec::with_capacity(file.size() as usize);
        if (&mut file).take(MAX_FILE_BYTES as u64 + 1).read_to_end(&mut content).is_err() || content.len() > MAX_FILE_BYTES {
            skipped += 1;
            continue;
        }
        entries.push((path, content));
        if entries.len() > MAX_FILES * 4 {
            return Err(format!("the archive has too many files (at most {MAX_FILES})"));
        }
    }
    // One folder holding everything: strip it.
    let prefix = entries.first().and_then(|(p, _)| p.split_once('/').map(|(top, _)| format!("{top}/")));
    let strip = prefix.filter(|prefix| entries.iter().all(|(p, _)| p.starts_with(prefix.as_str())));

    let mut files = Vec::new();
    for (path, content) in entries {
        let path = match &strip {
            Some(prefix) => path[prefix.len()..].to_string(),
            None => path,
        };
        if path.is_empty() || nomi_shipyard::bundle::excluded(&path) || path.split('/').any(|part| part == ".DS_Store" || part == "__MACOSX") {
            skipped += 1;
            continue;
        }
        match String::from_utf8(content) {
            Ok(text) if !text.contains('\0') => files.push((path, text)),
            _ => skipped += 1,
        }
    }
    if files.len() > MAX_FILES {
        return Err(format!("the archive has too many files (at most {MAX_FILES})"));
    }
    if files.is_empty() {
        return Err("there's no code in that archive".to_string());
    }
    Ok((files, skipped))
}

/// The stack a project is built with, read from its files.
pub fn detect_stack(files: &[(String, String)]) -> &'static str {
    let package = files.iter().find(|(p, _)| p == "package.json").map(|(_, c)| c.as_str()).unwrap_or_default();
    if package.contains("@sveltejs/kit") {
        "sveltekit"
    } else if package.contains("\"astro\"") {
        "astro"
    } else if package.contains("\"vue\"") {
        "vue"
    } else if package.contains("\"react\"") {
        "react"
    } else if package.contains("\"svelte\"") {
        "svelte"
    } else {
        "static"
    }
}

async fn import_files(state: &AppState, project_id: Uuid, files: Vec<(String, String)>, skipped: usize) -> Result<ImportResult, ApiError> {
    let stack = detect_stack(&files);
    let mut conn = state.pool.acquire().await.map_err(internal("failed to import the code"))?;
    let mut imported = 0;
    let mut skipped = skipped;
    for (path, content) in &files {
        match nomi_agent_coding::files::save_file(&mut conn, &state.project_storage, project_id, path, content).await {
            Ok(_) => imported += 1,
            Err(e) => {
                tracing::debug!(%path, error = %e, "skipped an imported file");
                skipped += 1;
            }
        }
    }
    sqlx::query("UPDATE projects SET stack = $2 WHERE id = $1")
        .bind(project_id)
        .bind(stack)
        .execute(&mut *conn)
        .await
        .map_err(internal("failed to import the code"))?;
    nomi_agent_coding::files::bump_files_version(&mut conn, project_id).await.map_err(internal("failed to import the code"))?;
    Ok(ImportResult { imported, skipped, stack: stack.to_string() })
}

/// `POST /api/projects/:id/import` with the zip as the body (`application/zip`, up to 50 MB).
pub async fn import_archive(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ImportResult>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let content_type = headers.get("content-type").and_then(|v| v.to_str().ok()).unwrap_or_default();
    if !content_type.is_empty() && !content_type.contains("zip") && !content_type.contains("octet-stream") {
        return Err((StatusCode::UNSUPPORTED_MEDIA_TYPE, "send a .zip archive".to_string()));
    }
    let (files, skipped) = read_archive(&body).map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))?;
    Ok(Json(import_files(&state, project_id, files, skipped).await?))
}

#[derive(Deserialize)]
pub struct ImportedFile {
    pub path: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct FilesBatch {
    pub files: Vec<ImportedFile>,
    /// The last batch: the stack is read and the project picks the files up.
    #[serde(default)]
    pub finish: bool,
}

/// `POST /api/projects/:id/import/files`: files the browser unpacked from an archive, a batch
/// at a time (the web app's request size limit is small).
pub async fn import_batch(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(batch): Json<FilesBatch>,
) -> Result<Json<ImportResult>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    if batch.files.len() > 500 {
        return Err((StatusCode::BAD_REQUEST, "send at most 500 files at a time".to_string()));
    }
    let mut conn = state.pool.acquire().await.map_err(internal("failed to import the code"))?;
    let mut imported = 0;
    let mut skipped = 0;
    for file in &batch.files {
        let path = file.path.trim_start_matches('/');
        if nomi_shipyard::bundle::excluded(path) || file.content.len() > MAX_FILE_BYTES {
            skipped += 1;
            continue;
        }
        match nomi_agent_coding::files::save_file(&mut conn, &state.project_storage, project_id, path, &file.content).await {
            Ok(_) => imported += 1,
            Err(_) => skipped += 1,
        }
    }
    let mut stack = String::new();
    if batch.finish {
        let package = state
            .project_storage
            .get_object(&nomi_agent_coding::project_file_key(project_id, "package.json"))
            .await
            .ok()
            .flatten()
            .unwrap_or_default();
        let has_index: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM project_files WHERE project_id = $1 AND path = 'index.html')")
            .bind(project_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(internal("failed to import the code"))?;
        let mut probe = vec![("package.json".to_string(), package)];
        if has_index {
            probe.push(("index.html".to_string(), String::new()));
        }
        stack = detect_stack(&probe).to_string();
        sqlx::query("UPDATE projects SET stack = $2 WHERE id = $1")
            .bind(project_id)
            .bind(&stack)
            .execute(&mut *conn)
            .await
            .map_err(internal("failed to import the code"))?;
    }
    nomi_agent_coding::files::bump_files_version(&mut conn, project_id).await.map_err(internal("failed to import the code"))?;
    Ok(Json(ImportResult { imported, skipped, stack }))
}

#[derive(Deserialize)]
pub struct RepoRequest {
    /// `owner/repo` or a github.com URL.
    pub repo: String,
    /// A branch or tag; the default branch when left out.
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
}

/// `owner/repo` from what the person pasted; only public github.com repositories.
pub fn parse_repo(input: &str) -> Option<(String, String)> {
    let trimmed = input.trim().trim_end_matches('/').trim_end_matches(".git");
    let path = trimmed
        .strip_prefix("https://github.com/")
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
        .or_else(|| trimmed.strip_prefix("github.com/"))
        .unwrap_or(trimmed);
    let mut parts = path.split('/');
    let (owner, repo) = (parts.next()?, parts.next()?);
    let valid = |s: &str| !s.is_empty() && s.len() <= 100 && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    (valid(owner) && valid(repo) && !owner.starts_with('.') && !repo.starts_with('.')).then(|| (owner.to_string(), repo.to_string()))
}

/// `POST /api/projects/:id/import/github` `{ repo, ref? }`: downloads a public repository.
pub async fn import_github(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<RepoRequest>,
) -> Result<Json<ImportResult>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let (owner, repo) = parse_repo(&req.repo).ok_or((StatusCode::BAD_REQUEST, "use owner/repo or a github.com link".to_string()))?;
    let git_ref = req.git_ref.as_deref().map(str::trim).filter(|r| !r.is_empty());
    if git_ref.is_some_and(|r| !r.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))) {
        return Err((StatusCode::BAD_REQUEST, "that branch name isn't valid".to_string()));
    }
    let base = std::env::var("GITHUB_API_URL").unwrap_or_else(|_| "https://api.github.com".to_string());
    let url = match git_ref {
        Some(r) => format!("{base}/repos/{owner}/{repo}/zipball/{r}"),
        None => format!("{base}/repos/{owner}/{repo}/zipball"),
    };
    let response = state
        .http_client
        .get(url)
        .header("user-agent", "nomi")
        .header("accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|_| (StatusCode::BAD_GATEWAY, "GitHub couldn't be reached".to_string()))?;
    match response.status().as_u16() {
        200 => {}
        404 => return Err((StatusCode::NOT_FOUND, "that repository wasn't found, or it's private".to_string())),
        403 | 429 => return Err((StatusCode::TOO_MANY_REQUESTS, "GitHub is limiting requests; try again in a few minutes".to_string())),
        _ => return Err((StatusCode::BAD_GATEWAY, "GitHub couldn't send the repository".to_string())),
    }
    if response.content_length().is_some_and(|n| n as usize > MAX_ARCHIVE_BYTES) {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "that repository is over 50 MB".to_string()));
    }
    let bytes = response.bytes().await.map_err(|_| (StatusCode::BAD_GATEWAY, "the download was cut off".to_string()))?;
    if bytes.len() > MAX_ARCHIVE_BYTES {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "that repository is over 50 MB".to_string()));
    }
    let (files, skipped) = read_archive(&bytes).map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))?;
    Ok(Json(import_files(&state, project_id, files, skipped).await?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn zip_of(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (path, content) in files {
            writer.start_file(*path, zip::write::SimpleFileOptions::default()).unwrap();
            writer.write_all(content).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn strips_the_shared_top_folder_and_skips_what_never_belongs() {
        let bytes = zip_of(&[
            ("app-main/package.json", br#"{"devDependencies":{"@sveltejs/kit":"^2"}}"#),
            ("app-main/src/routes/+page.svelte", b"<h1>hi</h1>"),
            ("app-main/node_modules/x/index.js", b"x"),
            ("app-main/.env", b"SECRET=1"),
            ("app-main/static/logo.png", b"\x89PNG\0\0"),
        ]);
        let (files, skipped) = read_archive(&bytes).unwrap();
        let paths: Vec<_> = files.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(paths, ["package.json", "src/routes/+page.svelte"]);
        assert_eq!(skipped, 3);
        assert_eq!(detect_stack(&files), "sveltekit");
    }

    #[test]
    fn keeps_paths_when_there_is_no_single_top_folder() {
        let bytes = zip_of(&[("index.html", b"<p>a</p>"), ("css/site.css", b"p{}")]);
        let (files, _) = read_archive(&bytes).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(detect_stack(&files), "static");
        assert!(read_archive(b"not a zip").is_err());
    }

    #[test]
    fn repos_are_owner_and_name_on_github_only() {
        assert_eq!(parse_repo("triandamai/nomi-v2"), Some(("triandamai".into(), "nomi-v2".into())));
        assert_eq!(parse_repo("https://github.com/sveltejs/kit.git"), Some(("sveltejs".into(), "kit".into())));
        assert_eq!(parse_repo("https://github.com/a/b/tree/main"), Some(("a".into(), "b".into())));
        assert_eq!(parse_repo("https://gitlab.com/a/b"), None);
        assert_eq!(parse_repo("../etc"), None);
        assert_eq!(parse_repo("a"), None);
    }
}
