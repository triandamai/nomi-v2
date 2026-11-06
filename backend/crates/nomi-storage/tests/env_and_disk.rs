//! What reads the environment (kept in its own test binary so no other test sees it change) and
//! the disk store's failure paths.

use nomi_storage::blob::{attachment_store, init_attachment_store, update_attachment_store, BlobStore};
use nomi_storage::{build_from_env, build_project_store};

fn temp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("nomi-storage-{name}-{}", uuid::Uuid::new_v4()))
}

#[tokio::test]
async fn the_environment_decides_s3_and_where_files_go_on_disk() {
    // One test, run in order: the variables are process-wide.
    std::env::remove_var("S3_BUCKET");
    assert!(build_from_env().await.is_none(), "no bucket, no S3");

    std::env::set_var("S3_BUCKET", "env-bucket");
    std::env::set_var("AWS_ACCESS_KEY_ID", "k");
    std::env::set_var("AWS_SECRET_ACCESS_KEY", "s");
    std::env::set_var("S3_REGION", "eu-west-1");
    std::env::remove_var("S3_ENDPOINT_URL");
    std::env::set_var("S3_PUBLIC_URL_BASE", "https://cdn.example.com");
    let s3 = build_from_env().await.expect("bucket and keys");
    assert_eq!(s3.bucket(), "env-bucket");
    assert_eq!(s3.public_url_base(), "https://cdn.example.com");

    // Attachments: the bucket when there is one, else ATTACHMENTS_DIR.
    let attachments = temp("attachments");
    std::env::set_var("ATTACHMENTS_DIR", &attachments);
    init_attachment_store(Some(s3.clone()));
    assert!(matches!(attachment_store(), BlobStore::S3(_)));
    update_attachment_store(None);
    assert!(matches!(attachment_store(), BlobStore::Disk(dir) if dir == attachments));

    // Projects on disk go under PROJECT_FILES_DIR.
    let projects = temp("projects");
    std::env::set_var("PROJECT_FILES_DIR", &projects);
    let store = build_project_store(None);
    store.put_object("p1/a.txt", "a", "text/plain").await.unwrap();
    assert_eq!(tokio::fs::read_to_string(projects.join("p1/a.txt")).await.unwrap(), "a");

    // With a bucket, files left in that folder from before are still read, and deleted with it.
    tokio::fs::create_dir_all(projects.join("p2")).await.unwrap();
    tokio::fs::write(projects.join("p2/old.txt"), "old").await.unwrap();
    store.reload(Some(s3));
    assert!(store.is_s3());
    let _ = tokio::fs::remove_dir_all(&projects).await;
}

#[tokio::test]
async fn disk_failures_are_errors_not_panics() {
    let root = temp("disk");
    tokio::fs::create_dir_all(&root).await.unwrap();
    // A file where a folder should be: nothing can be written under it.
    tokio::fs::write(root.join("blocked"), "file").await.unwrap();
    let store = BlobStore::Disk(root.clone());

    assert!(store.put_bytes("blocked/a.bin", vec![1], "x").await.is_err());
    let source = root.join("source.bin");
    tokio::fs::write(&source, "s").await.unwrap();
    assert!(store.put_file("blocked/a.bin", &source, "x").await.is_err());

    // A folder where a file should be: it can't be read or removed as one.
    tokio::fs::create_dir_all(root.join("folder")).await.unwrap();
    assert!(store.get_bytes("folder").await.is_err());
    assert!(store.delete("folder").await.is_err());

    // A source that doesn't exist can be neither moved nor copied.
    assert!(store.put_file("ok/a.bin", &root.join("missing"), "x").await.is_err());

    let projects = nomi_storage::ProjectStore::at(root.clone());
    assert!(projects.get_bytes("folder").await.is_err());
    assert!(projects.delete_object("folder").await.is_err());
    assert!(projects.delete_prefix("blocked").await.is_err(), "a file is not a project folder");
    let _ = tokio::fs::remove_dir_all(root).await;
}
