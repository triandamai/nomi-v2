//! `BoatProvider` against a mocked boat.dev API; no live calls.

use std::collections::BTreeMap;
use std::time::Duration;

use nomi_sandbox::boat::{
    BoatConfig, BoatProvider, DEFAULT_RATE_LIMIT_RETRY, NO_READY_MACHINE_RETRY,
    OUT_OF_CAPACITY_RETRY,
};
use nomi_sandbox::{CreateSpec, ProcessHandle, SandboxError, SandboxId, SandboxProvider, Size};
use serde_json::json;
use wiremock::matchers::{body_json, header, header_exists, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "boat_secretkey123";
/// Bytes just over the 5 MiB write limit.
const OVER_LIMIT_BYTES: usize = 5 * 1024 * 1024 + 1;
/// A timeout far above the API maximum.
const HUGE_TIMEOUT_SECS: u64 = 9999;

fn provider(server: &MockServer) -> BoatProvider {
    BoatProvider::new(
        BoatConfig {
            base_url: server.uri(),
            api_key: KEY.to_string(),
        },
        reqwest::Client::new(),
    )
}

fn sid() -> SandboxId {
    SandboxId("bx_7k2m9p4q".to_string())
}

fn spec() -> CreateSpec {
    CreateSpec {
        size: Size::Default,
        ttl_seconds: Some(1800),
        from_snapshot: None,
        idempotency_key: "idem-1".to_string(),
        labels: BTreeMap::from([("nomi-project".to_string(), "p1".to_string())]),
    }
}

fn error_body(status: u16, code: &str) -> serde_json::Value {
    json!({"ok": false, "type": "sandbox.error", "status": status, "code": code,
           "message": "nope", "error": {"code": code, "message": "nope", "status": status}})
}

#[tokio::test]
async fn create_sends_bearer_and_idempotency_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sandboxes"))
        .and(header("authorization", format!("Bearer {KEY}").as_str()))
        .and(header("idempotency-key", "idem-1"))
        .and(body_json(json!({
            "type": "default", "ttlSeconds": 1800, "labels": {"nomi-project": "p1"}
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "ok": true, "type": "sandbox.created", "status": "provisioning",
            "sandbox": {"id": "bx_7k2m9p4q", "state": "provisioning"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    let info = provider(&server).create(spec()).await.unwrap();
    assert_eq!(info.id, sid());
    assert_eq!(info.state, nomi_sandbox::SandboxState::Starting);
}

#[tokio::test]
async fn rate_limited_maps_retry_after() {
    let server = MockServer::start().await;
    let mut body = error_body(429, "rate_limited");
    body["retryAfterSeconds"] = json!(7);
    Mock::given(method("POST"))
        .and(path("/sandboxes"))
        .respond_with(ResponseTemplate::new(429).set_body_json(body))
        .mount(&server)
        .await;
    let err = provider(&server).create(spec()).await.unwrap_err();
    assert_eq!(
        err,
        SandboxError::RateLimited {
            retry_after: Duration::from_secs(7)
        }
    );

    // No hint in the body: falls back to the default.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429).set_body_json(error_body(429, "limit_reached")))
        .mount(&server)
        .await;
    let err = provider(&server).create(spec()).await.unwrap_err();
    assert_eq!(err.retry_after(), Some(DEFAULT_RATE_LIMIT_RETRY));

    // The IP limit carries a Retry-After header.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "3")
                .set_body_json(error_body(429, "IP_RATE_LIMIT_EXCEEDED")),
        )
        .mount(&server)
        .await;
    let err = provider(&server).create(spec()).await.unwrap_err();
    assert_eq!(err.retry_after(), Some(Duration::from_secs(3)));
}

#[tokio::test]
async fn out_of_capacity_maps_120s() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503).set_body_json(error_body(503, "out_of_capacity")))
        .mount(&server)
        .await;
    let err = provider(&server).create(spec()).await.unwrap_err();
    assert_eq!(
        err,
        SandboxError::OutOfCapacity {
            retry_after: Duration::from_secs(120)
        }
    );
    assert_eq!(OUT_OF_CAPACITY_RETRY, Duration::from_secs(120));

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503).set_body_json(error_body(503, "no_ready_machine")))
        .mount(&server)
        .await;
    let err = provider(&server).create(spec()).await.unwrap_err();
    assert_eq!(err.retry_after(), Some(NO_READY_MACHINE_RETRY));
    assert_eq!(NO_READY_MACHINE_RETRY, Duration::from_secs(5));
}

#[tokio::test]
async fn exec_uses_timeout_seconds_clamped_1_to_600() {
    let server = MockServer::start().await;
    for (secs, want) in [(0u64, 1u64), (45, 45), (HUGE_TIMEOUT_SECS, 600)] {
        server.reset().await;
        Mock::given(method("POST"))
            .and(path("/sandboxes/bx_7k2m9p4q/commands"))
            .and(body_json(
                json!({"command": "echo hi", "timeoutSeconds": want}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "ok": true, "type": "command.finished", "success": true, "exitCode": 0,
                "stdout": "hi\n", "stderr": "", "timedOut": false
            })))
            .expect(1)
            .mount(&server)
            .await;
        let out = provider(&server)
            .exec(&sid(), "echo hi", Duration::from_secs(secs))
            .await
            .unwrap();
        assert_eq!(out.exit_code, Some(0));
        assert_eq!(out.stdout, "hi\n");
        server.verify().await;
    }
}

#[tokio::test]
async fn write_file_sends_base64_and_rejects_over_5mib_before_the_request() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/sandboxes/bx_7k2m9p4q/files"))
        .and(body_json(json!({
            "path": "/home/user/a.txt", "content": "aGk=", "encoding": "base64"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"ok": true, "type": "file.written", "success": true})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let p = provider(&server);
    p.write_file(&sid(), "/home/user/a.txt", b"hi")
        .await
        .unwrap();

    let big = vec![0u8; OVER_LIMIT_BYTES];
    let err = p
        .write_file(&sid(), "/home/user/big", &big)
        .await
        .unwrap_err();
    assert!(matches!(err, SandboxError::Refused(_)));
    // Only the first, small write reached the server.
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn upload_archive_uses_upload_url_then_extracts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sandboxes/bx_7k2m9p4q/files/upload-url"))
        .and(header("authorization", format!("Bearer {KEY}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true, "type": "file.upload_url", "method": "PUT",
            "url": format!("{}/signed/abc", server.uri()), "path": "x", "expiresAt": "later"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/signed/abc"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/sandboxes/bx_7k2m9p4q/commands"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true, "type": "command.finished", "success": true, "exitCode": 0,
            "stdout": "", "stderr": "", "timedOut": false
        })))
        .expect(1)
        .mount(&server)
        .await;
    provider(&server)
        .upload_archive(&sid(), "/home/user/app", vec![1, 2, 3])
        .await
        .unwrap();

    let reqs = server.received_requests().await.unwrap();
    let put = reqs.iter().find(|r| r.method.as_str() == "PUT").unwrap();
    assert_eq!(put.body, vec![1, 2, 3]);
    assert!(put.headers.get("authorization").is_none());
    let exec = reqs
        .iter()
        .find(|r| r.url.path().ends_with("/commands"))
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&exec.body).unwrap();
    let cmd = body["command"].as_str().unwrap();
    assert!(
        cmd.contains("tar") && cmd.contains("/home/user/app"),
        "{cmd}"
    );
}

#[tokio::test]
async fn upload_archive_reports_a_failed_extract() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sandboxes/bx_7k2m9p4q/files/upload-url"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "url": format!("{}/signed/abc", server.uri())
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/sandboxes/bx_7k2m9p4q/commands"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": false, "exitCode": 2, "stdout": "", "stderr": "bad archive", "timedOut": false
        })))
        .mount(&server)
        .await;
    let err = provider(&server)
        .upload_archive(&sid(), "/home/user/app", vec![1])
        .await
        .unwrap_err();
    assert!(matches!(err, SandboxError::Other(m) if m.contains("bad archive")));
}

#[tokio::test]
async fn detached_exec_returns_process_id_and_status_uses_offsets() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sandboxes/bx_7k2m9p4q/commands"))
        .and(body_json(
            json!({"command": "npm run dev", "detached": true}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true, "type": "command.started", "success": true, "processId": 42, "pid": 1337
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/sandboxes/bx_7k2m9p4q/commands/42"))
        .and(query_param("stdoutOffset", "6"))
        .and(query_param("stderrOffset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true, "type": "command.status", "status": "exited", "exitCode": 1,
            "stdout": "more\n", "stderr": "", "stdoutNextOffset": 11, "stderrNextOffset": 0
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/sandboxes/bx_7k2m9p4q/commands/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;

    let p = provider(&server);
    let handle = p.exec_detached(&sid(), "npm run dev").await.unwrap();
    assert_eq!(
        handle,
        ProcessHandle {
            process_id: "42".to_string()
        }
    );
    let st = p.process_status(&sid(), &handle, 6, 0).await.unwrap();
    assert!(!st.running);
    assert_eq!(st.exit_code, Some(1));
    assert_eq!(st.stdout, "more\n");
    assert_eq!((st.stdout_next, st.stderr_next), (11, 0));
    p.kill_process(&sid(), &handle).await.unwrap();
}

#[tokio::test]
async fn snapshot_expired_maps_409() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sandboxes/bx_7k2m9p4q/resume"))
        .respond_with(
            ResponseTemplate::new(409).set_body_json(error_body(409, "snapshot_data_expired")),
        )
        .mount(&server)
        .await;
    let err = provider(&server)
        .resume(&sid(), Default::default())
        .await
        .unwrap_err();
    assert_eq!(err, SandboxError::SnapshotExpired);
}

#[tokio::test]
async fn other_errors_map_by_status() {
    for (status, code, want) in [
        (402u16, "billing_required", SandboxError::BillingRequired),
        (404, "not_found", SandboxError::NotFound),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(status).set_body_json(error_body(status, code)))
            .mount(&server)
            .await;
        assert_eq!(provider(&server).get(&sid()).await.unwrap_err(), want);
    }
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;
    assert!(matches!(
        provider(&server).get(&sid()).await.unwrap_err(),
        SandboxError::Unavailable(_)
    ));
}

#[tokio::test]
async fn delete_sends_confirm_header_and_list_pages() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/sandboxes/bx_7k2m9p4q"))
        .and(header("x-ascii-confirm-delete", "bx_7k2m9p4q"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/sandboxes"))
        .and(query_param("cursor", "c1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sandboxes": [{"id": "bx_bbbbbbbb", "state": "archived"}],
            "pageInfo": {"nextCursor": null, "hasMore": false, "limit": 100}
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/sandboxes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sandboxes": [{"id": "bx_aaaaaaaa", "state": "ready"}],
            "pageInfo": {"nextCursor": "c1", "hasMore": true, "limit": 100}
        })))
        .mount(&server)
        .await;
    let p = provider(&server);
    p.delete(&sid()).await.unwrap();
    let all = p.list().await.unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[1].state, nomi_sandbox::SandboxState::Stopped);
}

#[tokio::test]
async fn expose_port_returns_url_and_never_prints_it() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/sandboxes/bx_7k2m9p4q/host"))
        .and(header_exists("authorization"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true, "url": "https://x-3000.on.boat.dev?_token=s3cret"
        })))
        .mount(&server)
        .await;
    let url = provider(&server).expose_port(&sid(), 3000).await.unwrap();
    assert!(url.url.contains("_token=s3cret"));
    assert!(!format!("{url:?}").contains("s3cret"));
}

#[tokio::test]
async fn api_key_never_leaks_into_errors_or_debug() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401).set_body_json(error_body(401, "unauthorized")))
        .mount(&server)
        .await;
    let p = provider(&server);
    let err = p.get(&sid()).await.unwrap_err();
    assert!(!format!("{err} {err:?}").contains(KEY));
    assert!(!format!("{p:?}").contains(KEY));

    // A connection failure must not leak it either.
    let dead = BoatProvider::new(
        BoatConfig {
            base_url: "http://127.0.0.1:1".to_string(),
            api_key: KEY.to_string(),
        },
        reqwest::Client::new(),
    );
    let err = dead.get(&sid()).await.unwrap_err();
    assert!(matches!(err, SandboxError::Unavailable(_)));
    assert!(!format!("{err} {err:?}").contains(KEY));
}

#[tokio::test]
async fn usage_reads_the_seconds_the_sandbox_has_run() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sandboxes/bx_7k2m9p4q/usage"))
        .and(header("authorization", format!("Bearer {KEY}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "ok": true, "type": "sandbox.usage", "sandboxId": "bx_7k2m9p4q", "seconds": 5400,
            "dollars": 0.5, "secondsPerDollar": 10800, "running": true
        })))
        .mount(&server)
        .await;
    assert_eq!(provider(&server).usage(&sid()).await.unwrap(), Some(Duration::from_secs(5400)));
}

#[tokio::test]
async fn usage_of_an_unknown_sandbox_is_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sandboxes/bx_7k2m9p4q/usage"))
        .respond_with(ResponseTemplate::new(404).set_body_json(error_body(404, "not_found")))
        .mount(&server)
        .await;
    assert!(matches!(provider(&server).usage(&sid()).await, Err(SandboxError::NotFound)));
}

#[tokio::test]
async fn list_returns_each_sandboxes_labels() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/sandboxes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "sandboxes": [
                {"id": "bx_aaaaaaaa", "state": "ready", "labels": {"nomi-project": "p1", "n": 3}},
                {"id": "bx_bbbbbbbb", "state": "ready"}
            ],
            "pageInfo": {"nextCursor": null, "hasMore": false, "limit": 100}
        })))
        .mount(&server)
        .await;
    let all = provider(&server).list().await.unwrap();
    assert_eq!(all[0].labels, BTreeMap::from([("nomi-project".to_string(), "p1".to_string())]));
    assert!(all[1].labels.is_empty());
}

/// A preview address as boat returns it for a private port.
const PREVIEW_WITH_TOKEN: &str = "https://x-3000.on.boat.dev/?_token=prevsecret";

#[tokio::test]
async fn provider_error_text_never_carries_the_key_or_a_tokened_url() {
    let server = MockServer::start().await;
    let echo = format!("bad key {KEY} for {PREVIEW_WITH_TOKEN}");
    Mock::given(method("POST"))
        .and(path("/sandboxes/bx_7k2m9p4q/host"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({"code": "bad", "message": echo})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500).set_body_string(echo.clone()))
        .mount(&server)
        .await;
    let p = provider(&server);
    for err in [p.expose_port(&sid(), 3000).await.unwrap_err(), p.get(&sid()).await.unwrap_err()] {
        let text = format!("{err} {err:?}");
        assert!(!text.contains(KEY) && !text.contains("prevsecret"), "{text}");
        assert!(text.contains("x-3000.on.boat.dev"), "the host stays for debugging: {text}");
    }
}

/// Mounts an upload-url answer pointing at `url`, and a successful extract.
async fn mount_upload(server: &MockServer, url: &str) {
    Mock::given(method("POST"))
        .and(path("/sandboxes/bx_7k2m9p4q/files/upload-url"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "url": url })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/sandboxes/bx_7k2m9p4q/commands"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "exitCode": 0, "stdout": "", "stderr": "", "timedOut": false
        })))
        .mount(server)
        .await;
}

#[tokio::test]
async fn upload_url_must_be_https_on_a_public_host_or_the_api_host() {
    for bad in [
        "http://localhost:1/signed",
        "http://169.254.169.254/latest/meta-data",
        "https://127.0.0.1:1/signed",
        "https://10.0.0.5/signed",
        "https://[::1]/signed",
        "https://metadata.internal/signed",
        "https://user:pw@up.example.com/signed",
        "file:///etc/passwd",
        "not a url",
    ] {
        let server = MockServer::start().await;
        mount_upload(&server, bad).await;
        let err = provider(&server).upload_archive(&sid(), "/home/user/app", vec![1]).await.unwrap_err();
        assert!(matches!(err, SandboxError::Refused(_)), "{bad}: {err:?}");
        let reqs = server.received_requests().await.unwrap();
        assert!(!reqs.iter().any(|r| r.url.path().ends_with("/commands")), "{bad}: nothing is extracted");
    }
}

#[tokio::test]
async fn a_failed_upload_does_not_echo_the_signed_url() {
    let server = MockServer::start().await;
    let signed = format!("{}/signed/abc?X-Signature=sigsecret", server.uri());
    mount_upload(&server, &signed).await;
    Mock::given(method("PUT"))
        .respond_with(ResponseTemplate::new(403).set_body_string(format!("<Error><Url>{signed}</Url></Error>")))
        .mount(&server)
        .await;
    let err = provider(&server).upload_archive(&sid(), "/home/user/app", vec![1]).await.unwrap_err();
    assert!(!format!("{err} {err:?}").contains("sigsecret"), "{err}");
}

#[tokio::test]
async fn extraction_refuses_owners_and_permissions_from_the_archive_and_never_sees_the_key() {
    let server = MockServer::start().await;
    mount_upload(&server, &format!("{}/signed/abc", server.uri())).await;
    Mock::given(method("PUT")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    provider(&server).upload_archive(&sid(), "/home/user/app", vec![1]).await.unwrap();
    let reqs = server.received_requests().await.unwrap();
    let exec = reqs.iter().find(|r| r.url.path().ends_with("/commands")).unwrap();
    let body: serde_json::Value = serde_json::from_slice(&exec.body).unwrap();
    let cmd = body["command"].as_str().unwrap();
    assert!(cmd.contains("--no-same-owner") && cmd.contains("--no-same-permissions"), "{cmd}");
    for r in reqs.iter().filter(|r| r.method.as_str() != "GET") {
        assert!(!String::from_utf8_lossy(&r.body).contains(KEY), "the key only travels in the auth header");
    }
}

#[tokio::test]
async fn expose_port_refuses_an_address_that_is_not_https() {
    for bad in ["javascript:alert(1)", "http://x-3000.on.boat.dev/?_token=t", "data:text/html,hi"] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sandboxes/bx_7k2m9p4q/host"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "url": bad })))
            .mount(&server)
            .await;
        let err = provider(&server).expose_port(&sid(), 3000).await.unwrap_err();
        assert!(!format!("{err}").contains("_token=t"), "{err}");
    }
}

#[tokio::test]
async fn a_sandbox_id_that_is_not_a_plain_name_never_reaches_a_request_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET")).respond_with(ResponseTemplate::new(200).set_body_json(json!({}))).mount(&server).await;
    let p = provider(&server);
    for bad in ["../account", "bx_1/commands", "bx_1?x=1", "", "bx 1"] {
        let id = SandboxId(bad.to_string());
        assert!(matches!(p.get(&id).await, Err(SandboxError::Refused(_))), "{bad}");
        assert!(matches!(p.delete(&id).await, Err(SandboxError::Refused(_))), "{bad}");
        assert!(matches!(p.exec(&id, "true", Duration::from_secs(1)).await, Err(SandboxError::Refused(_))), "{bad}");
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}
