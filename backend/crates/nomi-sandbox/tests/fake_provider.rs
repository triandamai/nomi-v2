use std::collections::BTreeMap;
use std::time::Duration;

use nomi_sandbox::{
    CreateSpec, FakeProvider, PreviewUrl, ResumeSpec, SandboxError, SandboxId, SandboxProvider,
    SandboxState, Size,
};

fn spec() -> CreateSpec {
    CreateSpec {
        size: Size::Default,
        ttl: nomi_sandbox::Ttl::ProviderDefault,
        from_snapshot: None,
        idempotency_key: "k1".into(),
        labels: BTreeMap::from([("nomi-project".to_string(), "p1".to_string())]),
    }
}

const TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn create_then_stop_then_resume_keeps_files() {
    let p = FakeProvider::new();
    let info = p.create(spec()).await.unwrap();
    p.write_file(&info.id, "/work/a.txt", b"hi").await.unwrap();
    p.stop(&info.id, false).await.unwrap();
    assert_eq!(p.get(&info.id).await.unwrap().state, SandboxState::Stopped);
    let resumed = p
        .resume(&info.id, ResumeSpec::default())
        .await
        .unwrap();
    assert_eq!(resumed.state, SandboxState::Running);
    assert_eq!(
        p.files(&info.id).get("/work/a.txt").map(Vec::as_slice),
        Some(&b"hi"[..])
    );
}

#[tokio::test]
async fn detached_process_does_not_survive_resume() {
    let p = FakeProvider::new();
    let info = p.create(spec()).await.unwrap();
    let h = p.exec_detached(&info.id, "sleep 99").await.unwrap();
    assert!(p.process_status(&info.id, &h, 0, 0).await.unwrap().running);
    p.stop(&info.id, true).await.unwrap();
    p.resume(&info.id, ResumeSpec::default())
        .await
        .unwrap();
    assert!(!p.process_status(&info.id, &h, 0, 0).await.unwrap().running);
    let _ = p.exec(&info.id, "true", TIMEOUT).await.unwrap();
}

#[tokio::test]
async fn fail_next_returns_the_queued_error_once() {
    let p = FakeProvider::new();
    p.fail_next(SandboxError::BillingRequired);
    assert!(matches!(
        p.create(spec()).await,
        Err(SandboxError::BillingRequired)
    ));
    assert!(p.create(spec()).await.is_ok());
    assert!(p.calls().iter().any(|c| c.starts_with("create")));
}

#[test]
fn retryable_errors_are_exactly_rate_limited_capacity_unavailable() {
    let d = Duration::from_secs(3);
    assert!(SandboxError::RateLimited { retry_after: d }.is_retryable());
    assert!(SandboxError::OutOfCapacity { retry_after: d }.is_retryable());
    assert!(SandboxError::Unavailable("x".into()).is_retryable());
    assert!(!SandboxError::BillingRequired.is_retryable());
    assert!(!SandboxError::SnapshotExpired.is_retryable());
    assert!(!SandboxError::NotFound.is_retryable());
    assert!(!SandboxError::Refused("x".into()).is_retryable());
    assert!(!SandboxError::Other("x".into()).is_retryable());
    assert_eq!(
        SandboxError::RateLimited { retry_after: d }.retry_after(),
        Some(d)
    );
    assert_eq!(SandboxError::NotFound.retry_after(), None);
}

#[test]
fn preview_url_debug_is_redacted() {
    let u = PreviewUrl {
        url: "https://secret.example".into(),
    };
    let s = format!("{u:?}");
    assert!(s.contains("[redacted]") && !s.contains("secret"));
    let _ = SandboxId("x".into());
}

#[tokio::test]
async fn create_after_delete_with_same_key_makes_a_new_sandbox() {
    let p = FakeProvider::new();
    let first = p.create(spec()).await.unwrap();
    p.delete(&first.id).await.unwrap();
    let second = p.create(spec()).await.unwrap();
    assert_ne!(first.id, second.id);
    assert_eq!(second.state, SandboxState::Running);
}

#[tokio::test]
async fn extending_the_auto_stop_is_recorded() {
    let p = FakeProvider::new();
    let info = p.create(spec()).await.unwrap();
    assert_eq!(p.extend_auto_stop(&info.id, nomi_sandbox::Ttl::Seconds(60)).await.unwrap(), None);
    p.report_auto_stop("2026-06-01T16:00:00Z");
    assert_eq!(p.extend_auto_stop(&info.id, nomi_sandbox::Ttl::Seconds(60)).await.unwrap().as_deref(), Some("2026-06-01T16:00:00Z"));
    p.fail_extensions(Some(SandboxError::Refused("sandbox_running_stop_first".into())));
    assert!(p.extend_auto_stop(&info.id, nomi_sandbox::Ttl::Seconds(60)).await.is_err());
    assert!(p.get(&info.id).await.is_ok(), "only extensions fail");
    p.fail_extensions(None);
    assert_eq!(p.extensions().len(), 3, "every call is recorded, failed ones too");
    assert!(p.calls().contains(&format!("extend_auto_stop {}", info.id.0)));
    assert!(matches!(p.extend_auto_stop(&SandboxId("missing".into()), nomi_sandbox::Ttl::Never).await, Err(SandboxError::NotFound)));
}
