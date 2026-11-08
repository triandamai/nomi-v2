//! Copying a project's files from Nomi's storage into its sandbox.

mod support;

use std::io::Read;
use std::sync::Arc;

use nomi_sandbox::{CreateSpec, FakeProvider, SandboxId, SandboxProvider, Size};
use nomi_server::sandbox_steps::ServerSteps;
use nomi_server::sandbox_store as store;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

const PROVIDER: &str = "boat";
const PROJECT_DIR: &str = "/home/user/project";
/// More files than one incremental pass will write one by one.
const MANY_FILES: usize = 60;
/// A file just over the size that goes through the archive path.
const BIG_FILE_BYTES: usize = 5 * 1024 * 1024 + 1;
/// The name the fake provider files an uploaded archive under, inside the destination directory.
const ARCHIVE_KEY_SUFFIX: &str = "/.upload.tar.gz";

struct Fixture {
    pool: PgPool,
    provider: Arc<FakeProvider>,
    steps: ServerSteps,
    storage: nomi_storage::ProjectStore,
    project: Uuid,
    sandbox: SandboxId,
}

async fn fixture(pool: PgPool) -> Fixture {
    let app = App::new(pool.clone());
    let owner = app.person("owner@sync.test").await;
    let project = app.project(&owner, "sync").await.0;
    let provider = Arc::new(FakeProvider::new());
    let storage = nomi_test_support::test_project_storage();
    let sandbox = provider
        .create(CreateSpec { size: Size::Small, ttl: nomi_sandbox::Ttl::ProviderDefault, from_snapshot: None, idempotency_key: "k".into(), labels: Default::default() })
        .await
        .unwrap()
        .id;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, project, PROVIDER).await.unwrap();
    sqlx::query("UPDATE project_sandboxes SET sandbox_id = $2 WHERE project_id = $1").bind(project).bind(&sandbox.0).execute(&mut *conn).await.unwrap();
    drop(conn);
    let steps = ServerSteps::new(pool.clone(), storage.clone(), provider.clone());
    Fixture { pool, provider, steps, storage, project, sandbox }
}

impl Fixture {
    async fn save(&self, path: &str, content: &[u8]) {
        let mut conn = self.pool.acquire().await.unwrap();
        nomi_agent_coding::files::save_bytes(&mut conn, &self.storage, self.project, path, content).await.unwrap();
        nomi_agent_coding::files::bump_files_version(&mut conn, self.project).await.unwrap();
    }

    async fn remove(&self, path: &str) {
        let mut conn = self.pool.acquire().await.unwrap();
        nomi_agent_coding::files::remove_file(&mut conn, &self.storage, self.project, path).await.unwrap();
        nomi_agent_coding::files::bump_files_version(&mut conn, self.project).await.unwrap();
    }

    async fn files_version(&self) -> i64 {
        sqlx::query_scalar("SELECT files_version FROM projects WHERE id = $1").bind(self.project).fetch_one(&self.pool).await.unwrap()
    }

    async fn synced_version(&self) -> i64 {
        sqlx::query_scalar("SELECT synced_version FROM project_sandboxes WHERE project_id = $1").bind(self.project).fetch_one(&self.pool).await.unwrap()
    }

    fn calls(&self, prefix: &str) -> Vec<String> {
        self.provider.calls().into_iter().filter(|c| c.starts_with(prefix)).collect()
    }

    /// The paths inside the archive that was uploaded last.
    fn archive_paths(&self) -> Vec<String> {
        let files = self.provider.files(&self.sandbox);
        let bytes = files.get(&format!("{PROJECT_DIR}{ARCHIVE_KEY_SUFFIX}")).expect("an archive was uploaded");
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(bytes.as_slice()));
        let mut paths = Vec::new();
        for entry in tar.entries().unwrap() {
            let mut entry = entry.unwrap();
            let mut sink = Vec::new();
            entry.read_to_end(&mut sink).unwrap();
            paths.push(entry.path().unwrap().to_string_lossy().into_owned());
        }
        paths.sort();
        paths
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn initial_sync_uploads_one_archive_with_every_file(pool: PgPool) {
    let f = fixture(pool).await;
    f.save("package.json", b"{}").await;
    f.save("src/main.js", b"hi").await;
    let version = f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    assert_eq!(version, f.files_version().await);
    assert_eq!(f.synced_version().await, version);
    assert_eq!(f.calls("upload_archive").len(), 1);
    assert!(f.calls("write_file").is_empty());
    assert_eq!(f.archive_paths(), vec!["package.json".to_string(), "src/main.js".to_string()]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn incremental_sync_writes_only_changed_and_deletes_removed(pool: PgPool) {
    let f = fixture(pool).await;
    f.save("a.txt", b"a").await;
    f.save("b.txt", b"b").await;
    f.save("c.txt", b"c").await;
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    // The fake files an archive as one blob; seed the files the delete will remove.
    f.provider.write_file(&f.sandbox, &format!("{PROJECT_DIR}/c.txt"), b"c").await.unwrap();
    f.save("a.txt", b"a2").await;
    f.remove("c.txt").await;
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    let files = f.provider.files(&f.sandbox);
    assert_eq!(files.get(&format!("{PROJECT_DIR}/a.txt")).map(Vec::as_slice), Some(b"a2".as_slice()));
    assert!(!files.contains_key(&format!("{PROJECT_DIR}/c.txt")));
    assert_eq!(f.calls("write_file").len(), 2, "the seed and the one changed file, not b.txt");
    assert_eq!(f.calls("delete_file").len(), 1);
    assert_eq!(f.calls("upload_archive").len(), 1, "only the first sync used an archive");
}

#[sqlx::test(migrations = "../../migrations")]
async fn many_changes_go_via_archive(pool: PgPool) {
    let f = fixture(pool).await;
    f.save("a.txt", b"a").await;
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    for i in 0..MANY_FILES {
        f.save(&format!("f{i}.txt"), b"x").await;
    }
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    assert_eq!(f.calls("upload_archive").len(), 2);
    assert!(f.calls("write_file").is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn archive_above_threshold_contains_only_changed_files(pool: PgPool) {
    let f = fixture(pool).await;
    f.save("old.txt", b"old").await;
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    for i in 0..MANY_FILES {
        f.save(&format!("f{i}.txt"), b"x").await;
    }
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    let paths = f.archive_paths();
    assert_eq!(paths.len(), MANY_FILES);
    assert!(!paths.contains(&"old.txt".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn file_over_5mib_goes_via_archive(pool: PgPool) {
    let f = fixture(pool).await;
    f.save("a.txt", b"a").await;
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    f.save("assets/big.bin", &vec![7u8; BIG_FILE_BYTES]).await;
    f.save("b.txt", b"b").await;
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    assert_eq!(f.calls("upload_archive").len(), 2);
    assert_eq!(f.archive_paths(), vec!["assets/big.bin".to_string()]);
    let writes = f.calls("write_file");
    assert_eq!(writes.len(), 1);
    assert!(writes[0].ends_with("/b.txt"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn sync_is_idempotent_at_the_same_version(pool: PgPool) {
    let f = fixture(pool).await;
    f.save("a.txt", b"a").await;
    let first = f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    let calls_after_first = f.provider.calls().len();
    let second = f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    assert_eq!(first, second);
    assert_eq!(f.provider.calls().len(), calls_after_first, "nothing is sent again");
}

#[sqlx::test(migrations = "../../migrations")]
async fn files_changed_during_sync_trigger_another_pass(pool: PgPool) {
    let f = fixture(pool.clone()).await;
    f.save("a.txt", b"a").await;
    let storage = f.storage.clone();
    let project = f.project;
    let steps = ServerSteps::new(pool.clone(), f.storage.clone(), f.provider.clone()).with_pass_hook(Arc::new(move |pass| {
        let pool = pool.clone();
        let storage = storage.clone();
        Box::pin(async move {
            if pass == 0 {
                let mut conn = pool.acquire().await.unwrap();
                nomi_agent_coding::files::save_bytes(&mut conn, &storage, project, "late.txt", b"late").await.unwrap();
                nomi_agent_coding::files::bump_files_version(&mut conn, project).await.unwrap();
            }
        })
    }));
    let version = steps.sync_project(f.project, &f.sandbox).await.unwrap();
    assert_eq!(version, f.files_version().await, "the second pass caught the late file");
    assert_eq!(f.synced_version().await, version);
    let writes = f.calls("write_file");
    assert_eq!(writes.len(), 1);
    assert!(writes[0].ends_with("/late.txt"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn recreated_sandbox_starts_from_an_empty_manifest(pool: PgPool) {
    let f = fixture(pool).await;
    f.save("a.txt", b"a").await;
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    sqlx::query("UPDATE project_sandboxes SET synced_version = 0, synced_manifest = '{}'::jsonb WHERE project_id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    assert_eq!(f.calls("upload_archive").len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn remembered_paths_that_climb_out_are_never_deleted(pool: PgPool) {
    let f = fixture(pool).await;
    f.save("a.txt", b"a").await;
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    // A tampered memory names paths outside the project directory (or the directory itself).
    sqlx::query("UPDATE project_sandboxes SET synced_manifest = synced_manifest || '{\"../../etc\": \"x\", \"\": \"x\", \"/root\": \"x\"}'::jsonb WHERE project_id = $1")
        .bind(f.project)
        .execute(&f.pool)
        .await
        .unwrap();
    f.save("b.txt", b"b").await;
    f.steps.sync_project(f.project, &f.sandbox).await.unwrap();
    let deletes = f.calls("delete_file");
    assert!(deletes.is_empty(), "{deletes:?}");
}
