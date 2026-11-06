//! The S3 paths of nomi-storage against an in-memory S3 (tests/support): config building, the
//! connection probe and its errors, objects, attachments (BlobStore) and project files.

mod support;

use nomi_storage::blob::{BlobError, BlobStore};
use nomi_storage::{build_from, build_project_store, ProjectStoreError, S3Error};
use support::{FakeS3, BUCKET};

fn fields(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
    move |field| pairs.iter().find(|(k, _)| *k == field).map(|(_, v)| v.to_string())
}

#[tokio::test]
async fn no_bucket_or_half_set_keys_mean_no_s3() {
    assert!(build_from(fields(&[])).await.is_none());
    assert!(build_from(fields(&[("bucket", "  ")])).await.is_none());
    assert!(build_from(fields(&[("bucket", "b"), ("access_key_id", "k")])).await.is_none(), "a bucket without the secret is not S3");
    assert!(build_from(fields(&[("bucket", "b"), ("access_key_id", " "), ("secret_access_key", "s")])).await.is_none());
}

#[tokio::test]
async fn public_urls_follow_the_endpoint_or_aws() {
    let aws = build_from(fields(&[("bucket", " shots "), ("access_key_id", "k"), ("secret_access_key", "s"), ("region", "ap-southeast-3")])).await.unwrap();
    assert_eq!(aws.bucket(), "shots");
    assert_eq!(aws.public_url_base(), "https://shots.s3.ap-southeast-3.amazonaws.com");

    let default_region = build_from(fields(&[("bucket", "b"), ("access_key_id", "k"), ("secret_access_key", "s"), ("region", "")])).await.unwrap();
    assert_eq!(default_region.public_url_base(), "https://b.s3.us-east-1.amazonaws.com");

    // A bare host gets https, and a trailing slash is dropped.
    let custom = build_from(fields(&[("bucket", "b"), ("access_key_id", "k"), ("secret_access_key", "s"), ("endpoint_url", "storage.example.com/")])).await.unwrap();
    assert_eq!(custom.public_url_base(), "https://storage.example.com/b");
    assert_eq!(custom.public_url("avatars/u1.png"), "https://storage.example.com/b/avatars/u1.png");

    let cdn = build_from(fields(&[("bucket", "b"), ("access_key_id", "k"), ("secret_access_key", "s"), ("public_url_base", "https://cdn.example.com/")])).await.unwrap();
    assert_eq!(cdn.public_url("a.png"), "https://cdn.example.com/a.png");
}

#[tokio::test]
async fn presigned_uploads_point_at_the_bucket_and_the_public_url() {
    let s3 = FakeS3::start().await;
    let config = s3.config().await;
    let upload = config.presign_put("avatars/u1/x.png", "image/png").await.unwrap();
    assert!(upload.upload_url.starts_with(&format!("{}/{BUCKET}/avatars/u1/x.png?", s3.uri())), "{}", upload.upload_url);
    assert!(upload.upload_url.contains("X-Amz-Signature"));
    assert_eq!(upload.public_url, format!("{}/{BUCKET}/avatars/u1/x.png", s3.uri()));
}

#[tokio::test]
async fn the_connection_probe_puts_reads_and_removes_its_key() {
    let s3 = FakeS3::start().await;
    let config = s3.config().await;
    config.test_connection().await.unwrap();
    assert!(s3.keys().is_empty(), "the probe cleans up after itself");
    let requests = s3.state.lock().unwrap().requests.clone();
    assert!(requests.iter().any(|r| r.starts_with("PUT")) && requests.iter().any(|r| r.starts_with("GET")) && requests.iter().any(|r| r.starts_with("DELETE")));
}

#[tokio::test]
async fn the_connection_probe_explains_put_failures() {
    let s3 = FakeS3::start().await;
    let config = s3.config().await;
    for (code, expected) in [
        ("AccessDenied", "Access denied"),
        ("NoSuchBucket", "was not found"),
        ("InvalidAccessKeyId", "does not exist in our records"),
        ("SignatureDoesNotMatch", "Signature mismatch"),
        ("SlowDownPlease", "PutObject failed (SlowDownPlease)"),
    ] {
        s3.clear_failures();
        s3.fail("PUT", code, 403);
        let error = config.test_connection().await.unwrap_err();
        assert!(error.contains(expected), "{code}: {error}");
    }
}

#[tokio::test]
async fn the_connection_probe_reports_read_and_delete_failures() {
    let s3 = FakeS3::start().await;
    let config = s3.config().await;
    s3.fail("GET", "AccessDenied", 403);
    let error = config.test_connection().await.unwrap_err();
    assert!(error.starts_with("GetObject failed (AccessDenied)"), "{error}");
    s3.clear_failures();
    s3.fail("DELETE", "AccessDenied", 403);
    let error = config.test_connection().await.unwrap_err();
    assert!(error.starts_with("DeleteObject failed (AccessDenied)"), "{error}");
}

#[tokio::test]
async fn objects_round_trip_and_missing_ones_are_none() {
    let s3 = FakeS3::start().await;
    let config = s3.config().await;
    config.put_object("pages/a.html", "<h1>a</h1>", "text/html").await.unwrap();
    config.put_object_bytes("avatars/u1.png", vec![1, 2, 3], "image/png").await.unwrap();
    assert_eq!(s3.object("avatars/u1.png"), Some(vec![1, 2, 3]));
    assert_eq!(config.get_object("pages/a.html").await.unwrap().as_deref(), Some("<h1>a</h1>"));
    assert_eq!(config.get_object("pages/missing.html").await.unwrap(), None);
    config.delete_object("pages/a.html").await.unwrap();
    assert_eq!(config.get_object("pages/a.html").await.unwrap(), None);
}

#[tokio::test]
async fn object_errors_surface_as_errors() {
    let s3 = FakeS3::start().await;
    let config = s3.config().await;
    s3.insert("bad.txt", &[0xff, 0xfe]);
    assert!(matches!(config.get_object("bad.txt").await, Err(S3Error::Utf8(_))));

    s3.fail("PUT", "AccessDenied", 403);
    s3.fail("GET", "AccessDenied", 403);
    s3.fail("DELETE", "AccessDenied", 403);
    assert!(matches!(config.put_object("a", "b", "text/plain").await, Err(S3Error::Put(_))));
    assert!(matches!(config.put_object_bytes("a", vec![], "text/plain").await, Err(S3Error::Put(_))));
    let get = config.get_object("a").await.unwrap_err();
    assert!(matches!(get, S3Error::Get(_)));
    assert!(get.to_string().starts_with("failed to get object"));
    assert!(matches!(config.delete_object("a").await, Err(S3Error::Delete(_))));
}

#[tokio::test]
async fn attachments_in_the_bucket_round_trip() {
    let s3 = FakeS3::start().await;
    let store = BlobStore::S3(s3.config().await);
    let source = std::env::temp_dir().join(format!("nomi-blob-s3-{}", uuid::Uuid::new_v4()));
    tokio::fs::write(&source, b"file body").await.unwrap();

    store.put_file("u1/a.txt", &source, "text/plain").await.unwrap();
    store.put_bytes("u1/b.bin", vec![9, 9], "application/octet-stream").await.unwrap();
    assert_eq!(store.get_bytes("u1/a.txt").await.unwrap(), Some(b"file body".to_vec()));
    assert_eq!(store.get_bytes("u1/b.bin").await.unwrap(), Some(vec![9, 9]));
    assert_eq!(store.get_bytes("u1/none").await.unwrap(), None);
    store.delete("u1/b.bin").await.unwrap();
    assert_eq!(s3.object("u1/b.bin"), None);
    let _ = tokio::fs::remove_file(source).await;

    let missing = std::env::temp_dir().join("nomi-blob-does-not-exist");
    let error = store.put_file("u1/c", &missing, "text/plain").await.unwrap_err();
    assert!(matches!(&error, BlobError::S3(message) if message.contains("failed to read local file")), "{error}");
}

#[tokio::test]
async fn attachment_bucket_errors_name_the_bucket_and_key() {
    let s3 = FakeS3::start().await;
    let store = BlobStore::S3(s3.config().await);
    let source = std::env::temp_dir().join(format!("nomi-blob-s3-{}", uuid::Uuid::new_v4()));
    tokio::fs::write(&source, b"x").await.unwrap();
    s3.fail("PUT", "AccessDenied", 403);
    s3.fail("GET", "AccessDenied", 403);
    s3.fail("DELETE", "AccessDenied", 403);
    for error in [
        store.put_file("k1", &source, "text/plain").await.unwrap_err(),
        store.put_bytes("k1", vec![1], "text/plain").await.unwrap_err(),
        store.get_bytes("k1").await.unwrap_err(),
        store.delete("k1").await.unwrap_err(),
    ] {
        let message = error.to_string();
        assert!(message.contains(BUCKET) && message.contains("k1"), "{message}");
    }
    let _ = tokio::fs::remove_file(source).await;
}

#[tokio::test]
async fn project_files_go_under_projects_in_the_bucket() {
    let s3 = FakeS3::start().await;
    let store = build_project_store(Some(s3.config().await));
    assert!(store.is_s3());
    store.put_object("p1/index.html", "<p>hi</p>", "text/html").await.unwrap();
    assert_eq!(s3.object("projects/p1/index.html"), Some(b"<p>hi</p>".to_vec()));
    assert_eq!(store.get_object("p1/index.html").await.unwrap().as_deref(), Some("<p>hi</p>"));
    assert_eq!(store.get_object("p1/missing.html").await.unwrap(), None);
    store.delete_object("p1/index.html").await.unwrap();
    assert_eq!(s3.object("projects/p1/index.html"), None);

    s3.insert("projects/p1/bad.bin", &[0xff]);
    assert!(matches!(store.get_object("p1/bad.bin").await, Err(ProjectStoreError::Utf8(_))));
}

#[tokio::test]
async fn deleting_a_project_in_the_bucket_pages_through_every_key() {
    let s3 = FakeS3::start().await;
    s3.state.lock().unwrap().page_size = 2;
    let store = build_project_store(Some(s3.config().await));
    for name in ["a", "b", "c", "d", "e"] {
        store.put_object(&format!("p1/{name}.ts"), name, "text/plain").await.unwrap();
    }
    store.put_object("p10/keep.ts", "keep", "text/plain").await.unwrap();
    store.delete_prefix("p1").await.unwrap();
    assert_eq!(s3.keys(), vec!["projects/p10/keep.ts".to_string()], "p1/ only, not p10");
    let lists = s3.state.lock().unwrap().requests.iter().filter(|r| r.contains("list-type")).count();
    assert_eq!(lists, 3, "five keys two at a time");

    // An empty project deletes nothing and is fine.
    store.delete_prefix("p2/").await.unwrap();

    s3.fail("GET", "AccessDenied", 403);
    assert!(store.delete_prefix("p10").await.is_err());
}

#[tokio::test]
async fn reloading_moves_every_clone_to_the_bucket_and_back() {
    let s3 = FakeS3::start().await;
    let store = build_project_store(None);
    let clone = store.clone();
    assert!(!clone.is_s3());
    store.reload(Some(s3.config().await));
    assert!(clone.is_s3());
    clone.put_object("p1/a.txt", "a", "text/plain").await.unwrap();
    assert_eq!(s3.object("projects/p1/a.txt"), Some(b"a".to_vec()));
    store.reload(None);
    assert!(!clone.is_s3());
}
