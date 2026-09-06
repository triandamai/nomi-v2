use std::time::Duration;

use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

const PRESIGNED_UPLOAD_TTL: Duration = Duration::from_secs(300);

#[derive(Clone)]
pub struct S3Config {
    client: Client,
    bucket: String,
    public_url_base: String,
}

/// Optional infrastructure — a fresh install with no S3 credentials configured should boot and
/// run everything else normally, not panic at startup. Returns `None` (not an error) whenever
/// `S3_BUCKET` is unset; every other required var missing while `S3_BUCKET` IS set is treated as
/// a real misconfiguration and does panic, since that means someone intended to enable this and
/// got it wrong.
pub async fn build_from_env() -> Option<S3Config> {
    let bucket = std::env::var("S3_BUCKET").ok()?;
    let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let access_key = std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID must be set when S3_BUCKET is set");
    let secret_key =
        std::env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY must be set when S3_BUCKET is set");
    let endpoint_url = std::env::var("S3_ENDPOINT_URL").ok();
    let public_url_base = std::env::var("S3_PUBLIC_URL_BASE")
        .unwrap_or_else(|_| format!("https://{bucket}.s3.{region}.amazonaws.com"));

    let credentials = Credentials::new(access_key, secret_key, None, None, "env");
    let sdk_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(Region::new(region))
        .credentials_provider(credentials)
        .load()
        .await;

    let mut s3_builder = S3ConfigBuilder::from(&sdk_config);
    if let Some(endpoint) = endpoint_url {
        // S3-compatible services (Shipyard, R2, MinIO, ...) are usually addressed with
        // path-style URLs (host/bucket/key) rather than AWS's virtual-hosted-style
        // (bucket.host/key) — force it whenever a custom endpoint is in play.
        s3_builder = s3_builder.endpoint_url(endpoint).force_path_style(true);
    }

    Some(S3Config { client: Client::from_conf(s3_builder.build()), bucket, public_url_base })
}

#[derive(Debug, thiserror::Error)]
pub enum PresignError {
    #[error("failed to presign upload url: {0}")]
    Presign(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::put_object::PutObjectError, aws_sdk_s3::config::http::HttpResponse>),
    #[error(transparent)]
    Config(#[from] aws_sdk_s3::presigning::PresigningConfigError),
}

#[derive(Debug, thiserror::Error)]
pub enum S3Error {
    #[error("failed to put object: {0}")]
    Put(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::put_object::PutObjectError, aws_sdk_s3::config::http::HttpResponse>),
    #[error("failed to get object: {0}")]
    Get(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError, aws_sdk_s3::config::http::HttpResponse>),
    #[error("failed to delete object: {0}")]
    Delete(#[from] aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::delete_object::DeleteObjectError, aws_sdk_s3::config::http::HttpResponse>),
    #[error("failed to read object body: {0}")]
    Body(#[from] aws_sdk_s3::primitives::ByteStreamError),
    #[error("object content was not valid utf-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub struct PresignedUpload {
    pub upload_url: String,
    pub public_url: String,
}

impl S3Config {
    /// `key` is the full object key (path within the bucket) to presign a PUT for — callers
    /// build it (e.g. `avatars/{user_id}/{uuid}.{ext}`), this just talks to S3.
    pub async fn presign_put(&self, key: &str, content_type: &str) -> Result<PresignedUpload, PresignError> {
        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(PresigningConfig::expires_in(PRESIGNED_UPLOAD_TTL)?)
            .await?;

        Ok(PresignedUpload {
            upload_url: presigned.uri().to_string(),
            public_url: format!("{}/{key}", self.public_url_base.trim_end_matches('/')),
        })
    }

    /// Writes `content` directly from the server — for writers that already hold the bytes
    /// in-process (an agent tool call, a browser-submitted body proxied through our own API),
    /// as opposed to `presign_put`'s browser-uploads-directly-to-S3 shape.
    pub async fn put_object(&self, key: &str, content: &str, content_type: &str) -> Result<(), S3Error> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(content.as_bytes().to_vec()))
            .send()
            .await?;
        Ok(())
    }

    /// `Ok(None)` when the object doesn't exist (S3's `NoSuchKey`) — callers that want "not
    /// found" to be a normal, expected outcome rather than an error path get that for free.
    pub async fn get_object(&self, key: &str) -> Result<Option<String>, S3Error> {
        let result = self.client.get_object().bucket(&self.bucket).key(key).send().await;
        let output = match result {
            Ok(output) => output,
            Err(err) => {
                if let aws_sdk_s3::error::SdkError::ServiceError(service_err) = &err {
                    if service_err.err().is_no_such_key() {
                        return Ok(None);
                    }
                }
                return Err(S3Error::Get(err));
            }
        };
        let bytes = output.body.collect().await?.into_bytes();
        Ok(Some(String::from_utf8(bytes.to_vec())?))
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), S3Error> {
        self.client.delete_object().bucket(&self.bucket).key(key).send().await?;
        Ok(())
    }
}

/// Project file storage backed by the server's local disk — always available (no credentials, no
/// bucket, nothing to misconfigure), unlike S3Config above which stays reserved for avatar
/// uploads. Rooted at `PROJECT_FILES_DIR` (default `./data/projects`), created on first write.
#[derive(Clone)]
pub struct LocalFsStore {
    root: std::path::PathBuf,
}

pub fn build_local_fs_store() -> LocalFsStore {
    let root = std::env::var("PROJECT_FILES_DIR").unwrap_or_else(|_| "./data/projects".to_string());
    LocalFsStore::at(root)
}

impl LocalFsStore {
    /// Explicit-root constructor — used by build_local_fs_store above and by other crates' tests
    /// that need an isolated temp directory rather than the env-var-driven default.
    pub fn at(root: impl Into<std::path::PathBuf>) -> Self {
        LocalFsStore { root: root.into() }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LocalFsError {
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("file content was not valid utf-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl LocalFsStore {
    /// `key` is a relative path (e.g. `{project_id}/src/index.html`) — the caller owns building
    /// it and validating it, same division of responsibility as S3Config above.
    pub async fn put_object(&self, key: &str, content: &str, _content_type: &str) -> Result<(), LocalFsError> {
        let path = self.root.join(key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&path, content.as_bytes()).await?;
        Ok(())
    }

    /// `Ok(None)` when the file doesn't exist — mirrors S3Config::get_object's NoSuchKey handling
    /// so callers written against either backend don't need to care which one is active.
    pub async fn get_object(&self, key: &str) -> Result<Option<String>, LocalFsError> {
        let path = self.root.join(key);
        match tokio::fs::read(&path).await {
            Ok(bytes) => Ok(Some(String::from_utf8(bytes)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), LocalFsError> {
        let path = self.root.join(key);
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Deletes an entire directory under root (e.g. a whole project's files at once, keyed by
    /// project id) — the bulk counterpart to delete_object, for callers deleting the owning
    /// record rather than one file within it.
    pub async fn delete_prefix(&self, prefix: &str) -> Result<(), LocalFsError> {
        let path = self.root.join(prefix);
        match tokio::fs::remove_dir_all(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod local_fs_tests {
    use super::*;

    fn store_in(dir: &std::path::Path) -> LocalFsStore {
        LocalFsStore::at(dir)
    }

    #[tokio::test]
    async fn round_trips_a_file() {
        let dir = tempfile_dir();
        let store = store_in(&dir);
        store.put_object("proj/index.html", "<h1>hi</h1>", "text/html").await.unwrap();
        assert_eq!(store.get_object("proj/index.html").await.unwrap(), Some("<h1>hi</h1>".to_string()));
    }

    #[tokio::test]
    async fn missing_file_is_none_not_error() {
        let dir = tempfile_dir();
        let store = store_in(&dir);
        assert_eq!(store.get_object("proj/nope.html").await.unwrap(), None);
    }

    #[tokio::test]
    async fn delete_is_idempotent() {
        let dir = tempfile_dir();
        let store = store_in(&dir);
        store.put_object("proj/a.txt", "x", "text/plain").await.unwrap();
        store.delete_object("proj/a.txt").await.unwrap();
        store.delete_object("proj/a.txt").await.unwrap();
        assert_eq!(store.get_object("proj/a.txt").await.unwrap(), None);
    }

    fn tempfile_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("nomi-storage-test-{}", uuid_like()));
        dir
    }

    fn uuid_like() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        format!("{}-{:?}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(), std::thread::current().id())
    }
}
