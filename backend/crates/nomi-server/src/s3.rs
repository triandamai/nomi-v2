use std::time::Duration;

use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;

const PRESIGNED_UPLOAD_TTL: Duration = Duration::from_secs(300);

#[derive(Clone)]
pub struct S3Config {
    client: Client,
    bucket: String,
    public_url_base: String,
}

/// Avatar upload is optional infrastructure — a fresh install with no S3 credentials configured
/// should boot and run everything else normally, not panic at startup. Returns `None` (not an
/// error) whenever `S3_BUCKET` is unset; every other required var missing while `S3_BUCKET` IS
/// set is treated as a real misconfiguration and does panic, since that means someone intended
/// to enable this and got it wrong.
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
}
