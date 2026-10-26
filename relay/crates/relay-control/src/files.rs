//! The S3 bucket every project's files go in: each file at `<project>/<env>/<bucket>/<path>`.

use std::sync::Arc;

use object_store::aws::{AmazonS3Builder, S3ConditionalPut};
use object_store::path::Path;
use object_store::ObjectStore;
use relay_core::settings::StorageSettings;
use relay_core::Env;
use uuid::Uuid;

/// A store for these settings. Makes no request.
pub fn store_for(settings: &StorageSettings) -> Result<Arc<dyn ObjectStore>, String> {
    let mut builder = AmazonS3Builder::new()
        .with_region(&settings.region)
        .with_bucket_name(&settings.bucket)
        .with_access_key_id(&settings.access_key_id)
        .with_secret_access_key(&settings.secret_access_key)
        .with_virtual_hosted_style_request(!settings.path_style)
        // Uploads that mustn't overwrite use If-None-Match (AWS, R2 and MinIO all honour it).
        .with_conditional_put(S3ConditionalPut::ETagMatch);
    if !settings.endpoint.is_empty() {
        builder = builder.with_endpoint(&settings.endpoint).with_allow_http(settings.endpoint.starts_with("http://"));
    }
    builder.build().map(|store| Arc::new(store) as Arc<dyn ObjectStore>).map_err(|e| e.to_string())
}

/// Lists a page of the bucket: the credentials work and the bucket is there.
pub async fn check(store: &dyn ObjectStore) -> Result<(), String> {
    store.list_with_delimiter(Some(&Path::from("relay-check"))).await.map(|_| ()).map_err(|e| e.to_string())
}

/// Where `env`'s `bucket` of `project` keeps its files.
pub fn bucket_prefix(project: Uuid, env: Env, bucket: &str) -> Path {
    Path::from(format!("{project}/{}/{bucket}", env.as_str()))
}

/// Where one file is; `path` must already be clean (`relay_core::backend::clean_path`).
pub fn object_path(project: Uuid, env: Env, bucket: &str, path: &str) -> Path {
    Path::from(format!("{project}/{}/{bucket}/{path}", env.as_str()))
}
