//! Both binaries start the loops through `spawn_all`, so the standalone worker can't skip one again.

use nomi_worker::spawn::{loops, WorkerContext, LOOPS};

/// Never connected to: building the loops polls nothing.
const UNUSED_DATABASE_URL: &str = "postgres://localhost/unused";
/// No broker listens here; publishers only connect once polled.
const UNUSED_MQTT_PORT: u16 = 1;

fn context() -> WorkerContext {
    WorkerContext {
        pool: sqlx::postgres::PgPoolOptions::new().connect_lazy(UNUSED_DATABASE_URL).unwrap(),
        mqtt_host: "localhost".into(),
        mqtt_port: UNUSED_MQTT_PORT,
        client_id_prefix: "nomi-worker-test",
        s3: None,
        settings_key: [0; 32],
        http_client: reqwest::Client::new(),
        database_url: UNUSED_DATABASE_URL.into(),
        project_storage: nomi_test_support::test_project_storage(),
    }
}

#[tokio::test]
async fn every_loop_is_spawned() {
    let names: Vec<&str> = loops(context()).into_iter().map(|(name, _)| name).collect();
    assert_eq!(names, LOOPS);
}

/// Each module with a `run` loop, and the name `spawn_all` starts it under.
const MODULE_LOOPS: [(&str, &str); 10] = [
    ("worker", "turns"),
    ("delegation_worker", "delegation"),
    ("background_worker", "background"),
    ("scheduler_worker", "scheduler"),
    ("reminders_worker", "reminders"),
    ("memory_worker", "memory"),
    ("issue_worker", "issues"),
    ("billing_worker", "billing"),
    ("sandbox_worker", "sandboxes"),
    ("attachment_worker", "attachments"),
];

#[test]
fn every_worker_module_is_a_loop() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut with_run: Vec<String> = std::fs::read_dir(&src)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).ok()?;
            text.contains("pub async fn run(").then(|| path.file_stem().unwrap().to_string_lossy().into_owned())
        })
        .collect();
    with_run.sort();
    let mut mapped: Vec<String> = MODULE_LOOPS.iter().map(|(module, _)| module.to_string()).collect();
    mapped.sort();
    assert_eq!(with_run, mapped, "a module with a run loop that spawn_all doesn't start (or the reverse)");
    for (module, name) in MODULE_LOOPS {
        assert!(LOOPS.contains(&name), "{module} maps to {name}, which spawn_all doesn't start");
    }
}
