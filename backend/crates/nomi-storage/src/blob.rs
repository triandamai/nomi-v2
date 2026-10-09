//! Where files people attach in chat are kept: the S3 bucket when one is configured
//! (`S3_BUCKET`, the same settings avatars use), otherwise the server's disk under
//! `ATTACHMENTS_DIR` (default `./data/attachments`). Files are private either way: the bucket's
//! objects are only ever read back by the server, never linked to directly.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use aws_sdk_s3::primitives::ByteStream;

use crate::S3Config;

#[derive(Clone)]
pub enum BlobStore {
    S3(S3Config),
    Disk(PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub enum BlobError {
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("storage error: {0}")]
    S3(String),
}

static ATTACHMENTS: OnceLock<BlobStore> = OnceLock::new();

/// Picks where attachments go, once at startup. Without a call (tests, tools) they go to disk.
pub fn init_attachment_store(s3: Option<S3Config>) {
    let store = match s3 {
        Some(s3) => BlobStore::S3(s3),
        None => BlobStore::Disk(default_dir()),
    };
    let _ = ATTACHMENTS.set(store);
}

pub fn attachment_store() -> &'static BlobStore {
    ATTACHMENTS.get_or_init(|| BlobStore::Disk(default_dir()))
}

fn default_dir() -> PathBuf {
    std::env::var("ATTACHMENTS_DIR").unwrap_or_else(|_| "./data/attachments".to_string()).into()
}

impl BlobStore {
    /// Moves (disk) or uploads (S3) the file at `path` to `key`.
    pub async fn put_file(&self, key: &str, path: &Path, content_type: &str) -> Result<(), BlobError> {
        match self {
            BlobStore::S3(s3) => {
                let body = ByteStream::from_path(path).await.map_err(|e| BlobError::S3(e.to_string()))?;
                s3.client
                    .put_object()
                    .bucket(&s3.bucket)
                    .key(key)
                    .content_type(content_type)
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| BlobError::S3(e.to_string()))?;
                Ok(())
            }
            BlobStore::Disk(root) => {
                let target = root.join(key);
                if let Some(parent) = target.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                // A rename fails across filesystems (temp dir on another mount); copy then.
                if tokio::fs::rename(path, &target).await.is_err() {
                    tokio::fs::copy(path, &target).await?;
                }
                Ok(())
            }
        }
    }

    pub async fn put_bytes(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> Result<(), BlobError> {
        match self {
            BlobStore::S3(s3) => {
                s3.client
                    .put_object()
                    .bucket(&s3.bucket)
                    .key(key)
                    .content_type(content_type)
                    .body(ByteStream::from(bytes))
                    .send()
                    .await
                    .map_err(|e| BlobError::S3(e.to_string()))?;
                Ok(())
            }
            BlobStore::Disk(root) => {
                let target = root.join(key);
                if let Some(parent) = target.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::write(target, bytes).await?;
                Ok(())
            }
        }
    }

    /// `Ok(None)` when nothing is stored under `key`.
    pub async fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, BlobError> {
        match self {
            BlobStore::S3(s3) => {
                let result = s3.client.get_object().bucket(&s3.bucket).key(key).send().await;
                let output = match result {
                    Ok(output) => output,
                    Err(aws_sdk_s3::error::SdkError::ServiceError(e)) if e.err().is_no_such_key() => return Ok(None),
                    Err(e) => return Err(BlobError::S3(e.to_string())),
                };
                let bytes = output.body.collect().await.map_err(|e| BlobError::S3(e.to_string()))?.into_bytes();
                Ok(Some(bytes.to_vec()))
            }
            BlobStore::Disk(root) => match tokio::fs::read(root.join(key)).await {
                Ok(bytes) => Ok(Some(bytes)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e.into()),
            },
        }
    }

    pub async fn delete(&self, key: &str) -> Result<(), BlobError> {
        match self {
            BlobStore::S3(s3) => {
                s3.client.delete_object().bucket(&s3.bucket).key(key).send().await.map_err(|e| BlobError::S3(e.to_string()))?;
                Ok(())
            }
            BlobStore::Disk(root) => match tokio::fs::remove_file(root.join(key)).await {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e.into()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn disk_store_round_trips_files_and_bytes() {
        let root = std::env::temp_dir().join(format!("nomi-blob-test-{}", std::process::id()));
        let store = BlobStore::Disk(root.clone());
        let source = root.join("incoming.bin");
        tokio::fs::create_dir_all(&root).await.unwrap();
        tokio::fs::write(&source, b"hello").await.unwrap();

        store.put_file("u/a.bin", &source, "application/octet-stream").await.unwrap();
        assert_eq!(store.get_bytes("u/a.bin").await.unwrap(), Some(b"hello".to_vec()));
        assert!(!source.exists(), "the upload's temp file is moved, not copied");

        store.put_bytes("u/b.bin", b"x".to_vec(), "text/plain").await.unwrap();
        store.delete("u/b.bin").await.unwrap();
        store.delete("u/b.bin").await.unwrap();
        assert_eq!(store.get_bytes("u/b.bin").await.unwrap(), None);
        let _ = tokio::fs::remove_dir_all(root).await;
    }
}
