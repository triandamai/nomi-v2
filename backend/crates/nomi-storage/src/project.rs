//! Where project code lives: the S3 bucket under `projects/` when one is configured
//! (`S3_BUCKET`, shared with avatars and attachments), otherwise the server's disk under
//! `PROJECT_FILES_DIR` (default `./data/projects`). With a bucket, a file missing from it is
//! still read from that disk folder, so projects written before the bucket was set up keep
//! working; it moves to the bucket the next time it's written.

use std::path::{Path, PathBuf};

use crate::blob::{BlobError, BlobStore};
use crate::S3Config;

const S3_PREFIX: &str = "projects/";

#[derive(Clone)]
pub struct ProjectStore {
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
    match s3 {
        Some(s3) => ProjectStore { blob: BlobStore::S3(s3), legacy_disk: Some(disk_dir()) },
        None => ProjectStore::at(disk_dir()),
    }
}

impl ProjectStore {
    /// On disk under `root` (tests use a temp directory).
    pub fn at(root: impl Into<PathBuf>) -> Self {
        ProjectStore { blob: BlobStore::Disk(root.into()), legacy_disk: None }
    }

    /// Whether files go to the bucket.
    pub fn is_s3(&self) -> bool {
        matches!(self.blob, BlobStore::S3(_))
    }

    fn key(&self, key: &str) -> String {
        match self.blob {
            BlobStore::S3(_) => format!("{S3_PREFIX}{key}"),
            BlobStore::Disk(_) => key.to_string(),
        }
    }

    /// `key` is `{project_id}/{path}`; the caller validates the path.
    pub async fn put_object(&self, key: &str, content: &str, content_type: &str) -> Result<(), ProjectStoreError> {
        self.put_bytes(key, content.as_bytes().to_vec(), content_type).await
    }

    pub async fn put_bytes(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> Result<(), ProjectStoreError> {
        self.blob.put_bytes(&self.key(key), bytes, content_type).await?;
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
        if let Some(bytes) = self.blob.get_bytes(&self.key(key)).await? {
            return Ok(Some(bytes));
        }
        match &self.legacy_disk {
            Some(root) => read_if_exists(&root.join(key)).await,
            None => Ok(None),
        }
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), ProjectStoreError> {
        self.blob.delete(&self.key(key)).await?;
        if let Some(root) = &self.legacy_disk {
            remove_if_exists(&root.join(key)).await?;
        }
        Ok(())
    }

    /// Deletes everything under `prefix` (a whole project, keyed by its id).
    pub async fn delete_prefix(&self, prefix: &str) -> Result<(), ProjectStoreError> {
        match &self.blob {
            BlobStore::S3(s3) => delete_s3_prefix(s3, &self.key(prefix)).await?,
            BlobStore::Disk(root) => remove_dir_if_exists(&root.join(prefix)).await?,
        }
        if let Some(root) = &self.legacy_disk {
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
        let store = ProjectStore { blob: BlobStore::Disk(temp_root()), legacy_disk: Some(legacy) };
        assert_eq!(store.get_object("p1/old.html").await.unwrap().as_deref(), Some("<h1>old</h1>"));
        store.delete_object("p1/old.html").await.unwrap();
        assert_eq!(store.get_object("p1/old.html").await.unwrap(), None);
    }
}
