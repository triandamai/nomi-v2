//! Where a design check reads a built project from (`nomi_agent_core::design_review::ProjectSource`):
//! the project's own source text in project storage, for the people who can open the project.

use std::sync::Arc;

use async_trait::async_trait;
use nomi_agent_coding::files::project_file_key;
use nomi_agent_core::design_review::{ProjectSource, SharedProjectSource, SourceFile, MAX_SOURCE_FILES, MAX_SOURCE_FILE_BYTES};
use nomi_storage::ProjectStore;
use sqlx::PgPool;
use uuid::Uuid;

/// The kinds of file a design can show up in.
const SOURCE_EXTENSIONS: [&str; 12] = ["html", "css", "scss", "svelte", "vue", "astro", "ts", "tsx", "js", "jsx", "mjs", "cjs"];
/// Folders that hold what was installed or built, not what was written.
const SKIPPED_FOLDERS: [&str; 6] = ["node_modules", ".svelte-kit", "dist", "build", ".next", ".astro"];
const NOT_YOURS: &str = "there is no such project, or it isn't yours to open";

/// Whether `path` is source text the project's author wrote.
pub fn is_source(path: &str) -> bool {
    let mut segments = path.split('/');
    let name = segments.next_back().unwrap_or_default();
    if segments.any(|folder| SKIPPED_FOLDERS.contains(&folder)) {
        return false;
    }
    name.rsplit_once('.').is_some_and(|(_, extension)| SOURCE_EXTENSIONS.contains(&extension))
}

pub struct StoreProjectSource {
    pool: PgPool,
    storage: ProjectStore,
}

impl StoreProjectSource {
    pub fn new(pool: PgPool, storage: ProjectStore) -> Self {
        Self { pool, storage }
    }

    pub fn shared(pool: PgPool, storage: ProjectStore) -> SharedProjectSource {
        Arc::new(Self::new(pool, storage))
    }
}

#[async_trait]
impl ProjectSource for StoreProjectSource {
    async fn files(&self, user_id: Uuid, project_id: Uuid) -> Result<Vec<SourceFile>, String> {
        let unreadable = |e: sqlx::Error| {
            tracing::error!(error = %e, "design review: couldn't read the project");
            "the project couldn't be read right now".to_string()
        };
        let can_open: bool = sqlx::query_scalar("SELECT can_open_project($1, $2)").bind(project_id).bind(user_id).fetch_one(&self.pool).await.map_err(unreadable)?;
        if !can_open {
            return Err(NOT_YOURS.to_string());
        }
        let paths: Vec<String> = sqlx::query_scalar("SELECT path FROM project_files WHERE project_id = $1 AND size_bytes <= $2 ORDER BY path")
            .bind(project_id)
            .bind(MAX_SOURCE_FILE_BYTES)
            .fetch_all(&self.pool)
            .await
            .map_err(unreadable)?;
        let mut files = Vec::new();
        for path in paths.into_iter().filter(|path| is_source(path)).take(MAX_SOURCE_FILES) {
            match self.storage.get_object(&project_file_key(project_id, &path)).await {
                Ok(Some(content)) => files.push(SourceFile { path, content }),
                Ok(None) => {}
                Err(e) => tracing::warn!(error = %e, %path, "design review: a file couldn't be read"),
            }
        }
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_projects_own_source_text_is_read() {
        for path in ["src/app.css", "src/routes/+page.svelte", "src/lib/Hero.tsx", "index.html", "tailwind.config.ts", "src/pages/index.astro", "src/App.vue"] {
            assert!(is_source(path), "{path}");
        }
        for path in ["static/logo.png", "node_modules/x/index.js", "package-lock.json", ".svelte-kit/generated/root.js", "dist/assets/app.js", "build/index.html", "README.md", "src/data.json"] {
            assert!(!is_source(path), "{path}");
        }
    }
}
