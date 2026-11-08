//! Copies a project's files from Nomi's storage into its sandbox. What the sandbox already holds
//! is remembered per sandbox (path -> updated_at) so a later pass sends only what changed; the
//! memory is emptied whenever the sandbox is recreated.

use std::collections::BTreeMap;
use std::sync::Arc;

use futures_util::future::BoxFuture;
use futures_util::stream::{self, StreamExt, TryStreamExt};
use nomi_agent_coding::{project_file_key, validate_path};
use nomi_sandbox::plan::{build_archive, plan_sync, ArchiveError, ManifestEntry, SyncAction};
use nomi_sandbox::{SandboxError, SandboxId};
use uuid::Uuid;

use crate::routes::project_runtime::load_manifest;
use crate::sandbox_steps::ServerSteps;

/// Where a project's files live inside the sandbox.
pub const PROJECT_DIR: &str = "/home/user/project";
/// More changed files than this are sent as one archive instead of one by one.
const ARCHIVE_THRESHOLD: usize = 50;
/// A file larger than this many bytes is sent inside an archive, never by itself.
const MAX_DIRECT_FILE_BYTES: i64 = 5 * 1024 * 1024;
/// Files read from storage at once.
const READ_CONCURRENCY: usize = 16;
/// Most passes one sync makes while files keep changing under it.
const MAX_PASSES: u32 = 4;

/// Called with the pass number after each pass has copied its files.
pub type PassHook = Arc<dyn Fn(u32) -> BoxFuture<'static, ()> + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("project storage: {0}")]
    Storage(String),
    #[error("sandbox: {0}")]
    Sandbox(#[from] SandboxError),
    #[error("archive: {0}")]
    Archive(#[from] ArchiveError),
}

/// A file to copy, read from storage.
type Loaded = (String, Vec<u8>);

fn sandbox_path(path: &str) -> String {
    format!("{PROJECT_DIR}/{path}")
}

impl ServerSteps {
    /// Bring the sandbox's files up to the project's current files. Returns the files version
    /// copied. Passes repeat while the files change under them (up to a limit); the last version
    /// copied is returned either way, and anything newer is picked up by the next sync.
    pub async fn sync_project(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<i64, SyncError> {
        let (mut synced_version, mut known) = self.remembered(project_id, sandbox).await?;
        for pass in 0..MAX_PASSES {
            let manifest = load_manifest(&self.pool, project_id).await?;
            let version = manifest.files_version;
            if version != synced_version || synced_version == 0 {
                let entries = self.valid_entries(&manifest.files);
                known = self.copy(project_id, sandbox, &known, &entries).await?;
                synced_version = version;
                self.remember(project_id, sandbox, version, &known).await?;
            }
            if let Some(hook) = &self.pass_hook {
                hook(pass).await;
            }
            let current: i64 = sqlx::query_scalar("SELECT files_version FROM projects WHERE id = $1").bind(project_id).fetch_one(&self.pool).await?;
            if current == synced_version {
                break;
            }
        }
        Ok(synced_version)
    }

    /// The version and file list last copied into this sandbox. A row that points at a different
    /// sandbox, or none, remembers nothing.
    async fn remembered(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<(i64, BTreeMap<String, String>), SyncError> {
        let row: Option<(i64, serde_json::Value)> =
            sqlx::query_as("SELECT synced_version, synced_manifest FROM project_sandboxes WHERE project_id = $1 AND sandbox_id = $2")
                .bind(project_id)
                .bind(&sandbox.0)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(|(v, m)| (v, serde_json::from_value(m).unwrap_or_default())).unwrap_or_default())
    }

    /// Record what the sandbox now holds, unless the row has moved on to another sandbox.
    async fn remember(&self, project_id: Uuid, sandbox: &SandboxId, version: i64, known: &BTreeMap<String, String>) -> Result<(), SyncError> {
        let manifest = serde_json::to_value(known).unwrap_or_default();
        sqlx::query("UPDATE project_sandboxes SET synced_version = $3, synced_manifest = $4 WHERE project_id = $1 AND sandbox_id = $2")
            .bind(project_id)
            .bind(&sandbox.0)
            .bind(version)
            .bind(manifest)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// The manifest as the planner sees it, leaving out any path that isn't safe to write.
    fn valid_entries(&self, files: &[crate::routes::project_runtime::ManifestEntry]) -> Vec<(ManifestEntry, i64)> {
        files
            .iter()
            .filter(|f| match validate_path(&f.path) {
                Ok(()) => true,
                Err(why) => {
                    tracing::warn!(path = %f.path, %why, "sandbox sync: skipping a file with an unusable path");
                    false
                }
            })
            .map(|f| (ManifestEntry { path: f.path.clone(), updated_at: f.updated_at.to_rfc3339_opts(chrono::SecondsFormat::Micros, true) }, i64::from(f.size_bytes)))
            .collect()
    }

    /// Carry out the plan and return what the sandbox holds afterwards.
    async fn copy(
        &self,
        project_id: Uuid,
        sandbox: &SandboxId,
        known: &BTreeMap<String, String>,
        entries: &[(ManifestEntry, i64)],
    ) -> Result<BTreeMap<String, String>, SyncError> {
        let manifest: Vec<ManifestEntry> = entries.iter().map(|(e, _)| e.clone()).collect();
        let sizes: BTreeMap<&str, i64> = entries.iter().map(|(e, size)| (e.path.as_str(), *size)).collect();
        let mut held = known.clone();
        let mut archived: Vec<String> = Vec::new();
        let mut direct: Vec<String> = Vec::new();
        let mut deleted: Vec<String> = Vec::new();
        for action in plan_sync(known, &manifest, ARCHIVE_THRESHOLD) {
            match action {
                SyncAction::Archive => {
                    archived = manifest.iter().map(|e| e.path.clone()).collect();
                    let current: std::collections::BTreeSet<&str> = archived.iter().map(String::as_str).collect();
                    deleted = known.keys().filter(|p| !current.contains(p.as_str())).cloned().collect();
                    direct.clear();
                    break;
                }
                SyncAction::Write(path) if sizes.get(path.as_str()).copied().unwrap_or(0) > MAX_DIRECT_FILE_BYTES => archived.push(path),
                SyncAction::Write(path) => direct.push(path),
                SyncAction::Delete(path) => deleted.push(path),
            }
        }
        let mut copied: std::collections::BTreeSet<String> = Default::default();
        let at: BTreeMap<&str, &str> = manifest.iter().map(|e| (e.path.as_str(), e.updated_at.as_str())).collect();
        if !archived.is_empty() {
            let files = self.load(project_id, archived).await?;
            if !files.is_empty() {
                self.provider.upload_archive(sandbox, PROJECT_DIR, build_archive(&files)?).await?;
            }
            copied.extend(files.into_iter().map(|(p, _)| p));
        }
        for (path, bytes) in self.load(project_id, direct).await? {
            self.provider.write_file(sandbox, &sandbox_path(&path), &bytes).await?;
            copied.insert(path);
        }
        for path in deleted {
            match self.provider.delete_file(sandbox, &sandbox_path(&path)).await {
                Ok(()) | Err(SandboxError::NotFound) => {}
                Err(e) => return Err(e.into()),
            }
            held.remove(&path);
        }
        // Remember a file only once it is in the sandbox. One whose bytes are missing from
        // storage is dropped from the memory, so it is tried again next time.
        for path in &copied {
            if let Some(updated_at) = at.get(path.as_str()) {
                held.insert(path.clone(), (*updated_at).to_string());
            }
        }
        let current: std::collections::BTreeSet<&str> = at.keys().copied().collect();
        held.retain(|p, _| current.contains(p.as_str()));
        Ok(held)
    }

    /// Read these files from storage. A path with no stored bytes is left out.
    async fn load(&self, project_id: Uuid, paths: Vec<String>) -> Result<Vec<Loaded>, SyncError> {
        let storage = &self.storage;
        let loaded: Vec<Option<Loaded>> = stream::iter(paths)
            .map(|path| async move {
                let bytes = storage.get_bytes(&project_file_key(project_id, &path)).await.map_err(|e| SyncError::Storage(e.to_string()))?;
                Ok::<_, SyncError>(bytes.map(|b| (path, b)))
            })
            .buffered(READ_CONCURRENCY)
            .try_collect()
            .await?;
        Ok(loaded.into_iter().flatten().collect())
    }
}
