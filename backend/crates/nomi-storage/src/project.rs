//! Where project code lives: the S3 bucket under `projects/` when one is configured
//! (`S3_BUCKET`, shared with avatars and attachments), otherwise the server's disk under
//! `PROJECT_FILES_DIR` (default `./data/projects`). With a bucket, a file missing from it is
//! still read from that disk folder, so projects written before the bucket was set up keep
//! working; it moves to the bucket the next time it's written.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::blob::{BlobError, BlobStore};
use crate::S3Config;

const S3_PREFIX: &str = "projects/";

#[derive(Clone)]
pub struct ProjectStore {
    inner: Arc<RwLock<ProjectStoreInner>>,
}

#[derive(Clone)]
struct ProjectStoreInner {
    blob: BlobStore,
    /// The disk folder older files may still be in, when `blob` is the bucket.
    legacy_disk: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectStoreError {
    #[error(transparent)]
    Blob(#[from] BlobError),
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("file content was not valid utf-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

fn disk_dir() -> PathBuf {
    std::env::var("PROJECT_FILES_DIR").unwrap_or_else(|_| "./data/projects".to_string()).into()
}

/// The bucket when one is configured, else disk.
pub fn build_project_store(s3: Option<S3Config>) -> ProjectStore {
    let inner = match s3 {
        Some(s3) => ProjectStoreInner { blob: BlobStore::S3(s3), legacy_disk: Some(disk_dir()) },
        None => ProjectStoreInner { blob: BlobStore::Disk(disk_dir()), legacy_disk: None },
    };
    ProjectStore { inner: Arc::new(RwLock::new(inner)) }
}

impl ProjectStore {
    /// On disk under `root` (tests use a temp directory).
    pub fn at(root: impl Into<PathBuf>) -> Self {
        ProjectStore {
            inner: Arc::new(RwLock::new(ProjectStoreInner {
                blob: BlobStore::Disk(root.into()),
                legacy_disk: None,
            })),
        }
    }

    /// Dynamically switches or updates the storage target across all clones of this store.
    pub fn reload(&self, s3: Option<S3Config>) {
        let mut inner = self.inner.write().unwrap_or_else(|e| e.into_inner());
        match s3 {
            Some(s3) => {
                inner.blob = BlobStore::S3(s3);
                inner.legacy_disk = Some(disk_dir());
            }
            None => {
                inner.blob = BlobStore::Disk(disk_dir());
                inner.legacy_disk = None;
            }
        }
    }

    /// Whether files go to the bucket.
    pub fn is_s3(&self) -> bool {
        let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
        matches!(inner.blob, BlobStore::S3(_))
    }

    /// `key` is `{project_id}/{path}`; the caller validates the path.
    pub async fn put_object(&self, key: &str, content: &str, content_type: &str) -> Result<(), ProjectStoreError> {
        self.put_bytes(key, content.as_bytes().to_vec(), content_type).await
    }

    pub async fn put_bytes(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> Result<(), ProjectStoreError> {
        let (blob, full_key) = {
            let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
            let full_key = match &inner.blob {
                BlobStore::S3(_) => format!("{S3_PREFIX}{key}"),
                BlobStore::Disk(_) => key.to_string(),
            };
            (inner.blob.clone(), full_key)
        };
        blob.put_bytes(&full_key, bytes, content_type).await?;
        Ok(())
    }

    /// `Ok(None)` when there's no such file.
    pub async fn get_object(&self, key: &str) -> Result<Option<String>, ProjectStoreError> {
        match self.get_bytes(key).await? {
            Some(bytes) => Ok(Some(String::from_utf8(bytes)?)),
            None => Ok(None),
        }
    }

    pub async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, ProjectStoreError> {
        let (blob, legacy_disk, full_key) = {
            let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
            let full_key = match &inner.blob {
                BlobStore::S3(_) => format!("{S3_PREFIX}{key}"),
                BlobStore::Disk(_) => key.to_string(),
            };
            (inner.blob.clone(), inner.legacy_disk.clone(), full_key)
        };
        if let Some(bytes) = blob.get_bytes(&full_key).await? {
            return Ok(Some(bytes));
        }
        match &legacy_disk {
            Some(root) => read_if_exists(&root.join(key)).await,
            None => Ok(None),
        }
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), ProjectStoreError> {
        let (blob, legacy_disk, full_key) = {
            let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
            let full_key = match &inner.blob {
                BlobStore::S3(_) => format!("{S3_PREFIX}{key}"),
                BlobStore::Disk(_) => key.to_string(),
            };
            (inner.blob.clone(), inner.legacy_disk.clone(), full_key)
        };
        blob.delete(&full_key).await?;
        if let Some(root) = &legacy_disk {
            remove_if_exists(&root.join(key)).await?;
        }
        Ok(())
    }

    /// Deletes everything under `prefix` (a whole project, keyed by its id).
    pub async fn delete_prefix(&self, prefix: &str) -> Result<(), ProjectStoreError> {
        let (blob, legacy_disk, full_key) = {
            let inner = self.inner.read().unwrap_or_else(|e| e.into_inner());
            let full_key = match &inner.blob {
                BlobStore::S3(_) => format!("{S3_PREFIX}{prefix}"),
                BlobStore::Disk(_) => prefix.to_string(),
            };
            (inner.blob.clone(), inner.legacy_disk.clone(), full_key)
        };
        match &blob {
            BlobStore::S3(s3) => delete_s3_prefix(s3, &full_key).await?,
            BlobStore::Disk(root) => remove_dir_if_exists(&root.join(prefix)).await?,
        }
        if let Some(root) = &legacy_disk {
            remove_dir_if_exists(&root.join(prefix)).await?;
        }
        Ok(())
    }
}

async fn read_if_exists(path: &Path) -> Result<Option<Vec<u8>>, ProjectStoreError> {
    match tokio::fs::read(path).await {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

async fn remove_if_exists(path: &Path) -> Result<(), ProjectStoreError> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

async fn remove_dir_if_exists(path: &Path) -> Result<(), ProjectStoreError> {
    match tokio::fs::remove_dir_all(path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

async fn delete_s3_prefix(s3: &S3Config, prefix: &str) -> Result<(), ProjectStoreError> {
    use aws_sdk_s3::types::{Delete, ObjectIdentifier};
    let prefix = if prefix.ends_with('/') { prefix.to_string() } else { format!("{prefix}/") };
    let s3_err = |e: String| ProjectStoreError::Blob(BlobError::S3(e));
    let mut token: Option<String> = None;
    loop {
        let page = s3
            .client
            .list_objects_v2()
            .bucket(&s3.bucket)
            .prefix(&prefix)
            .set_continuation_token(token.take())
            .send()
            .await
            .map_err(|e| s3_err(e.to_string()))?;
        let ids: Vec<ObjectIdentifier> =
            page.contents().iter().filter_map(|o| o.key()).filter_map(|k| ObjectIdentifier::builder().key(k).build().ok()).collect();
        if !ids.is_empty() {
            let delete = Delete::builder().set_objects(Some(ids)).quiet(true).build().map_err(|e| s3_err(e.to_string()))?;
            s3.client.delete_objects().bucket(&s3.bucket).delete(delete).send().await.map_err(|e| s3_err(e.to_string()))?;
        }
        match page.next_continuation_token() {
            Some(next) if page.is_truncated().unwrap_or(false) => token = Some(next.to_string()),
            _ => return Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!("nomi-project-store-{}", uuid::Uuid::new_v4()))
    }

    #[tokio::test]
    async fn deleting_a_prefix_removes_the_whole_project() {
        let store = ProjectStore::at(temp_root());
        store.put_object("p1/src/a.ts", "a", "text/plain").await.unwrap();
        store.put_object("p1/b.ts", "b", "text/plain").await.unwrap();
        store.put_object("p2/c.ts", "c", "text/plain").await.unwrap();
        store.delete_prefix("p1").await.unwrap();
        assert_eq!(store.get_object("p1/src/a.ts").await.unwrap(), None);
        assert_eq!(store.get_object("p2/c.ts").await.unwrap().as_deref(), Some("c"));
    }

    #[tokio::test]
    async fn files_left_on_disk_are_still_read() {
        let legacy = temp_root();
        tokio::fs::create_dir_all(legacy.join("p1")).await.unwrap();
        tokio::fs::write(legacy.join("p1/old.html"), "<h1>old</h1>").await.unwrap();
        let store = ProjectStore {
            inner: Arc::new(RwLock::new(ProjectStoreInner {
                blob: BlobStore::Disk(temp_root()),
                legacy_disk: Some(legacy),
            })),
        };
        assert_eq!(store.get_object("p1/old.html").await.unwrap().as_deref(), Some("<h1>old</h1>"));
        store.delete_object("p1/old.html").await.unwrap();
        assert_eq!(store.get_object("p1/old.html").await.unwrap(), None);
    }

    #[tokio::test]
    async fn reload_updates_shared_clones() {
        let store1 = ProjectStore::at(temp_root());
        let store2 = store1.clone();
        assert!(!store1.is_s3());
        assert!(!store2.is_s3());

        store1.reload(None);
        assert!(!store1.is_s3());
        assert!(!store2.is_s3());
    }
}
