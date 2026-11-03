//! Everything Koda built is the person's, on every plan (Build → Export): what's in it (the
//! files, the database it uses, the names of its environment variables, never their values) and
//! the files as a zip. Pushing to GitHub is `routes::github::export`.

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use super::deploy::{database_kind, database_name, internal, project_files, project_role, ApiError};
use crate::app::AppState;

#[derive(Serialize)]
pub struct ExportSummary {
    pub name: String,
    pub files: usize,
    pub bytes: usize,
    /// postgres, sqlite or none.
    pub database: &'static str,
    /// The environment variables it needs, by name only.
    pub env: Vec<String>,
}

/// `GET /api/projects/:id/export`: what an export holds.
pub async fn summary(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<ExportSummary>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let name: String = sqlx::query_scalar("SELECT name FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to load project"))?;
    let files = project_files(&state, project_id).await?;
    let env: Vec<String> = sqlx::query_scalar("SELECT name FROM project_secrets WHERE project_id = $1 ORDER BY name")
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal("failed to list environment variables"))?;
    Ok(Json(ExportSummary {
        name,
        files: files.len(),
        bytes: files.iter().map(|(_, bytes)| bytes.len()).sum(),
        database: database_name(database_kind(&state, project_id).await?),
        env,
    }))
}

/// A file name made from the project's name: letters, digits and dashes.
pub fn archive_name(project: &str) -> String {
    let slug: String = project
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    format!("{}.zip", if slug.is_empty() { "project".to_string() } else { slug })
}

/// The project's files as a zip, each at its path.
pub fn zip_files(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, zip::result::ZipError> {
    use std::io::Write;
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (path, bytes) in files {
        writer.start_file(path.as_str(), options)?;
        writer.write_all(bytes)?;
    }
    Ok(writer.finish()?.into_inner())
}

/// `GET /api/projects/:id/export/zip`: the project's files, to download.
pub async fn download(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Response, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let name: String = sqlx::query_scalar("SELECT name FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to load project"))?;
    let files = project_files(&state, project_id).await?;
    let bytes = zip_files(&files).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("failed to zip the project: {e}")))?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", archive_name(&name)))
        .body(Body::from(bytes))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_archive_is_named_after_the_project_and_holds_every_file() {
        assert_eq!(archive_name("Warung Bu Sri!"), "warung-bu-sri.zip");
        assert_eq!(archive_name("  "), "project.zip");
        let bytes = zip_files(&[("index.html".into(), b"<h1>hi</h1>".to_vec()), ("src/app.ts".into(), b"x".to_vec())]).unwrap();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 2);
        assert_eq!(archive.by_name("src/app.ts").unwrap().size(), 1);
    }
}
