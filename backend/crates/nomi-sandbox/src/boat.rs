//! `BoatProvider`: the boat.dev REST API behind the `SandboxProvider` trait.
//! Paths, field names and error codes follow docs/superpowers/specs/2026-11-08-boat-api-contract.md.

use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use base64::Engine;
use reqwest::{Method, RequestBuilder, Response, StatusCode};
use serde_json::{json, Value};

use crate::{
    CreateSpec, ExecOutput, PreviewUrl, ProcessHandle, ProcessStatus, ResumeSpec, SandboxError,
    SandboxId, SandboxInfo, SandboxProvider, SandboxState,
};

/// Longest command timeout the API accepts, in seconds.
pub const MAX_EXEC_TIMEOUT_SECS: u64 = 600;
/// Shortest command timeout the API accepts, in seconds.
pub const MIN_EXEC_TIMEOUT_SECS: u64 = 1;
/// Largest file the write endpoint takes (5 MiB, decoded).
pub const MAX_WRITE_BYTES: usize = 5 * 1024 * 1024;
/// How long to wait before retrying a throttled call when the API gives no hint.
pub const DEFAULT_RATE_LIMIT_RETRY: Duration = Duration::from_secs(60);
/// How long to wait before retrying when the provider is out of capacity.
pub const OUT_OF_CAPACITY_RETRY: Duration = Duration::from_secs(120);
/// How long to wait before retrying when no machine is ready yet.
pub const NO_READY_MACHINE_RETRY: Duration = Duration::from_secs(5);
/// Per-request network timeout for ordinary API calls.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// Extra network time on top of a command's own timeout, so the API answers first.
pub const EXEC_NETWORK_SLACK: Duration = Duration::from_secs(15);
/// Network timeout for sending an archive to its signed upload URL.
pub const UPLOAD_TIMEOUT: Duration = Duration::from_secs(300);
/// How long unpacking an uploaded archive may take, in seconds.
pub const EXTRACT_TIMEOUT: Duration = Duration::from_secs(120);
/// Page size used when listing sandboxes (the API maximum).
pub const LIST_PAGE_SIZE: u32 = 200;
/// Where an uploaded archive waits inside the sandbox until it is unpacked.
pub const ARCHIVE_STAGING_PATH: &str = "/tmp/nomi-upload.tar.gz";
/// How much of an unreadable error body to keep in an error message.
const ERROR_BODY_PREVIEW_CHARS: usize = 200;
/// First HTTP status that means the provider failed on its side.
const SERVER_ERROR_MIN_STATUS: u16 = 500;
/// First HTTP status that means we asked for something the provider refused.
const CLIENT_ERROR_MIN_STATUS: u16 = 400;
/// Header the delete endpoint needs, set to the sandbox id.
const CONFIRM_DELETE_HEADER: &str = "X-Ascii-Confirm-Delete";

/// Where boat is and how to sign in. The key is never printed.
#[derive(Clone)]
pub struct BoatConfig {
    /// e.g. `https://boat.dev/api/v1`
    pub base_url: String,
    pub api_key: String,
}

impl fmt::Debug for BoatConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoatConfig")
            .field("base_url", &self.base_url)
            .field("api_key", &"[redacted]")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct BoatProvider {
    config: BoatConfig,
    client: reqwest::Client,
}

impl BoatProvider {
    pub fn new(config: BoatConfig, client: reqwest::Client) -> Self {
        Self {
            config: BoatConfig {
                base_url: config.base_url.trim_end_matches('/').to_string(),
                api_key: config.api_key,
            },
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url, path)
    }

    fn req(&self, method: Method, path: &str) -> RequestBuilder {
        self.client
            .request(method, self.url(path))
            .bearer_auth(&self.config.api_key)
            .timeout(REQUEST_TIMEOUT)
    }

    /// Send a request, turn failures into `SandboxError`, return the JSON body on success.
    async fn send(&self, rb: RequestBuilder) -> Result<Value, SandboxError> {
        let resp = rb.send().await.map_err(transport_error)?;
        if resp.status().is_success() {
            let bytes = resp.bytes().await.map_err(transport_error)?;
            if bytes.is_empty() {
                return Ok(Value::Null);
            }
            return serde_json::from_slice(&bytes)
                .map_err(|e| SandboxError::Other(format!("unreadable response: {e}")));
        }
        Err(error_from_response(resp).await)
    }

    async fn sandbox_call(&self, rb: RequestBuilder) -> Result<SandboxInfo, SandboxError> {
        let body = self.send(rb).await?;
        parse_sandbox(&body["sandbox"])
    }
}

/// A network failure; the URL is dropped so nothing sensitive can ride along.
fn transport_error(e: reqwest::Error) -> SandboxError {
    SandboxError::Unavailable(e.without_url().to_string())
}

fn secs_field(v: &Value) -> Option<Duration> {
    v.as_f64()
        .filter(|s| s.is_finite() && s.is_sign_positive())
        .map(Duration::from_secs_f64)
}

/// The wait the API asked for: body `retryAfter` / `retryAfterSeconds` (top level or in
/// `error.details`), else the `Retry-After` header.
fn retry_hint(body: &Value, header: Option<Duration>) -> Option<Duration> {
    [
        &body["retryAfter"],
        &body["retryAfterSeconds"],
        &body["error"]["details"]["retryAfterSeconds"],
    ]
    .into_iter()
    .find_map(secs_field)
    .or(header)
}

async fn error_from_response(resp: Response) -> SandboxError {
    let status = resp.status();
    let header = resp
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(Duration::from_secs);
    let text = resp.text().await.unwrap_or_default();
    let body: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    map_error(status, &body, header, &text)
}

fn map_error(
    status: StatusCode,
    body: &Value,
    header: Option<Duration>,
    raw: &str,
) -> SandboxError {
    let code = body["code"]
        .as_str()
        .or_else(|| body["error"]["code"].as_str())
        .unwrap_or("");
    let message = body["message"]
        .as_str()
        .or_else(|| body["error"]["message"].as_str())
        .map(str::to_string)
        .unwrap_or_else(|| raw.chars().take(ERROR_BODY_PREVIEW_CHARS).collect());
    let detail = if code.is_empty() {
        format!("{} {message}", status.as_u16())
    } else {
        format!("{} {code}: {message}", status.as_u16())
    };
    match (status.as_u16(), code) {
        (429, "rate_limited" | "limit_reached" | "IP_RATE_LIMIT_EXCEEDED") => {
            SandboxError::RateLimited {
                retry_after: retry_hint(body, header).unwrap_or(DEFAULT_RATE_LIMIT_RETRY),
            }
        }
        (503, "out_of_capacity") => SandboxError::OutOfCapacity {
            retry_after: retry_hint(body, header).unwrap_or(OUT_OF_CAPACITY_RETRY),
        },
        (503, "no_ready_machine") => SandboxError::OutOfCapacity {
            retry_after: retry_hint(body, header).unwrap_or(NO_READY_MACHINE_RETRY),
        },
        (402, _) => SandboxError::BillingRequired,
        (409, "snapshot_data_expired") => SandboxError::SnapshotExpired,
        (404, _) => SandboxError::NotFound,
        // Transient 409s: the same call works once the sandbox or the first create settles.
        (
            409,
            "idempotency_in_progress" | "boat_starting" | "sandbox_starting" | "sandbox_resuming",
        ) => SandboxError::Unavailable(detail),
        (s, _) if s >= SERVER_ERROR_MIN_STATUS => SandboxError::Unavailable(detail),
        (s, _) if s >= CLIENT_ERROR_MIN_STATUS => SandboxError::Refused(detail),
        _ => SandboxError::Other(detail),
    }
}

/// UNSPECIFIED by the docs: how `stopped` relates to the state enum. We follow the enum:
/// `archiving` is stopping, `archived` is stopped, `cancelled` is gone.
fn map_state(state: &str) -> SandboxState {
    match state {
        "init" | "provisioning" | "provisioned" | "cloning" => SandboxState::Starting,
        "ready" | "idle" | "running" => SandboxState::Running,
        "archiving" => SandboxState::Stopping,
        "archived" => SandboxState::Stopped,
        "cancelled" => SandboxState::Gone,
        _ => SandboxState::Error,
    }
}

fn parse_sandbox(v: &Value) -> Result<SandboxInfo, SandboxError> {
    let id = v["id"]
        .as_str()
        .ok_or_else(|| SandboxError::Other("response had no sandbox id".to_string()))?;
    Ok(SandboxInfo {
        id: SandboxId(id.to_string()),
        state: map_state(v["state"].as_str().unwrap_or("")),
        labels: v["labels"]
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(k, val)| val.as_str().map(|val| (k.clone(), val.to_string())))
            .collect(),
    })
}

/// Seconds for the API: rounded up, then clamped into the accepted range.
fn timeout_secs(timeout: Duration) -> u64 {
    let secs = timeout.as_secs() + u64::from(timeout.subsec_nanos() > 0);
    secs.clamp(MIN_EXEC_TIMEOUT_SECS, MAX_EXEC_TIMEOUT_SECS)
}

/// Wrap `s` in single quotes for a POSIX shell.
fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

fn exit_code(v: &Value) -> Option<i32> {
    v.as_i64().and_then(|c| i32::try_from(c).ok())
}

fn text(v: &Value) -> String {
    v.as_str().unwrap_or_default().to_string()
}

fn offset(v: &Value) -> u64 {
    v.as_u64().unwrap_or(0)
}

/// UNSPECIFIED by the docs: the upload response's shape beyond `url`. We read only `url`.
fn upload_url_from(body: &Value) -> Result<String, SandboxError> {
    body["url"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| SandboxError::Other("upload-url response had no url".to_string()))
}

impl BoatProvider {
    fn commands_path(id: &SandboxId) -> String {
        format!("/sandboxes/{}/commands", id.0)
    }

    fn process_path(id: &SandboxId, handle: &ProcessHandle) -> String {
        format!("/sandboxes/{}/commands/{}", id.0, handle.process_id)
    }
}

#[async_trait]
impl SandboxProvider for BoatProvider {
    async fn create(&self, spec: CreateSpec) -> Result<SandboxInfo, SandboxError> {
        let mut body = json!({ "type": spec.size.as_api() });
        if let Some(ttl) = spec.ttl_seconds {
            body["ttlSeconds"] = json!(ttl);
        }
        if let Some(from) = &spec.from_snapshot {
            body["from"] = json!(from);
        }
        if !spec.labels.is_empty() {
            body["labels"] = json!(spec.labels);
        }
        let rb = self
            .req(Method::POST, "/sandboxes")
            .header("Idempotency-Key", &spec.idempotency_key)
            .json(&body);
        self.sandbox_call(rb).await
    }

    async fn get(&self, id: &SandboxId) -> Result<SandboxInfo, SandboxError> {
        self.sandbox_call(self.req(Method::GET, &format!("/sandboxes/{}", id.0)))
            .await
    }

    async fn resume(&self, id: &SandboxId, spec: ResumeSpec) -> Result<SandboxInfo, SandboxError> {
        let mut body = json!({});
        if let Some(ttl) = spec.ttl_seconds {
            body["ttlSeconds"] = json!(ttl);
        }
        let rb = self
            .req(Method::POST, &format!("/sandboxes/{}/resume", id.0))
            .json(&body);
        let resp = self.send(rb).await?;
        // The 202 may carry `sandbox: null`; the sandbox is then on its way up.
        if resp["sandbox"].is_object() {
            parse_sandbox(&resp["sandbox"])
        } else {
            Ok(SandboxInfo {
                id: id.clone(),
                state: SandboxState::Starting,
                labels: Default::default(),
            })
        }
    }

    async fn stop(&self, id: &SandboxId, force: bool) -> Result<(), SandboxError> {
        let rb = self
            .req(Method::POST, &format!("/sandboxes/{}/stop", id.0))
            .json(&json!({ "force": force }));
        self.send(rb).await.map(|_| ())
    }

    async fn delete(&self, id: &SandboxId) -> Result<(), SandboxError> {
        let rb = self
            .req(Method::DELETE, &format!("/sandboxes/{}", id.0))
            .header(CONFIRM_DELETE_HEADER, &id.0);
        self.send(rb).await.map(|_| ())
    }

    async fn list(&self) -> Result<Vec<SandboxInfo>, SandboxError> {
        let mut out = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let mut rb = self
                .req(Method::GET, "/sandboxes")
                .query(&[("limit", LIST_PAGE_SIZE.to_string())]);
            if let Some(c) = &cursor {
                rb = rb.query(&[("cursor", c)]);
            }
            let body = self.send(rb).await?;
            for s in body["sandboxes"].as_array().into_iter().flatten() {
                out.push(parse_sandbox(s)?);
            }
            match (
                body["pageInfo"]["hasMore"].as_bool().unwrap_or(false),
                body["pageInfo"]["nextCursor"].as_str(),
            ) {
                (true, Some(next)) => cursor = Some(next.to_string()),
                _ => return Ok(out),
            }
        }
    }

    async fn exec(
        &self,
        id: &SandboxId,
        command: &str,
        timeout: Duration,
    ) -> Result<ExecOutput, SandboxError> {
        let secs = timeout_secs(timeout);
        let rb = self
            .req(Method::POST, &Self::commands_path(id))
            .timeout(Duration::from_secs(secs) + EXEC_NETWORK_SLACK)
            .json(&json!({ "command": command, "timeoutSeconds": secs }));
        let body = self.send(rb).await?;
        let timed_out = body["timedOut"].as_bool().unwrap_or(false);
        Ok(ExecOutput {
            exit_code: if timed_out {
                None
            } else {
                exit_code(&body["exitCode"])
            },
            stdout: text(&body["stdout"]),
            stderr: text(&body["stderr"]),
        })
    }

    async fn exec_detached(
        &self,
        id: &SandboxId,
        command: &str,
    ) -> Result<ProcessHandle, SandboxError> {
        let rb = self
            .req(Method::POST, &Self::commands_path(id))
            .json(&json!({ "command": command, "detached": true }));
        let body = self.send(rb).await?;
        let pid = body["processId"]
            .as_u64()
            .ok_or_else(|| SandboxError::Other("response had no processId".to_string()))?;
        Ok(ProcessHandle {
            process_id: pid.to_string(),
        })
    }

    async fn process_status(
        &self,
        id: &SandboxId,
        handle: &ProcessHandle,
        stdout_from: u64,
        stderr_from: u64,
    ) -> Result<ProcessStatus, SandboxError> {
        let rb = self
            .req(Method::GET, &Self::process_path(id, handle))
            .query(&[("stdoutOffset", stdout_from), ("stderrOffset", stderr_from)]);
        let body = self.send(rb).await?;
        Ok(ProcessStatus {
            running: body["status"].as_str() == Some("running"),
            exit_code: exit_code(&body["exitCode"]),
            stdout: text(&body["stdout"]),
            stderr: text(&body["stderr"]),
            stdout_next: offset(&body["stdoutNextOffset"]),
            stderr_next: offset(&body["stderrNextOffset"]),
        })
    }

    async fn kill_process(
        &self,
        id: &SandboxId,
        handle: &ProcessHandle,
    ) -> Result<(), SandboxError> {
        self.send(self.req(Method::DELETE, &Self::process_path(id, handle)))
            .await
            .map(|_| ())
    }

    async fn write_file(
        &self,
        id: &SandboxId,
        path: &str,
        bytes: &[u8],
    ) -> Result<(), SandboxError> {
        if bytes.len() > MAX_WRITE_BYTES {
            return Err(SandboxError::Refused(format!(
                "file is {} bytes; the write limit is {MAX_WRITE_BYTES}",
                bytes.len()
            )));
        }
        let rb = self
            .req(Method::PUT, &format!("/sandboxes/{}/files", id.0))
            .json(&json!({
                "path": path,
                "content": base64::engine::general_purpose::STANDARD.encode(bytes),
                "encoding": "base64",
            }));
        self.send(rb).await.map(|_| ())
    }

    async fn delete_file(&self, id: &SandboxId, path: &str) -> Result<(), SandboxError> {
        // `recursive` lets a directory go too; the API refuses the home and tmp roots itself.
        let rb = self
            .req(Method::DELETE, &format!("/sandboxes/{}/files", id.0))
            .query(&[("path", path), ("recursive", "true")]);
        self.send(rb).await.map(|_| ())
    }

    async fn upload_archive(
        &self,
        id: &SandboxId,
        dest_dir: &str,
        tar_gz: Vec<u8>,
    ) -> Result<(), SandboxError> {
        let rb = self
            .req(
                Method::POST,
                &format!("/sandboxes/{}/files/upload-url", id.0),
            )
            .json(&json!({ "path": ARCHIVE_STAGING_PATH }));
        let url = upload_url_from(&self.send(rb).await?)?;

        // The signed URL authenticates itself, so no bearer token goes with it.
        let resp = self
            .client
            .put(url)
            .timeout(UPLOAD_TIMEOUT)
            .body(tar_gz)
            .send()
            .await
            .map_err(transport_error)?;
        if !resp.status().is_success() {
            return Err(error_from_response(resp).await);
        }

        let dest = shell_quote(dest_dir);
        let tmp = shell_quote(ARCHIVE_STAGING_PATH);
        let command =
            format!("mkdir -p {dest} && tar -xzf {tmp} -C {dest}; rc=$?; rm -f {tmp}; exit $rc");
        let out = self.exec(id, &command, EXTRACT_TIMEOUT).await?;
        match out.exit_code {
            Some(0) => Ok(()),
            _ => Err(SandboxError::Other(format!(
                "unpacking the archive failed: {}",
                out.stderr.trim()
            ))),
        }
    }

    async fn usage(&self, id: &SandboxId) -> Result<Option<Duration>, SandboxError> {
        let body = self
            .send(self.req(Method::GET, &format!("/sandboxes/{}/usage", id.0)))
            .await?;
        Ok(secs_field(&body["seconds"]))
    }

    async fn expose_port(&self, id: &SandboxId, port: u16) -> Result<PreviewUrl, SandboxError> {
        // Private (token in the URL) rather than public: the URL is treated as the secret.
        let rb = self
            .req(Method::POST, &format!("/sandboxes/{}/host", id.0))
            .json(&json!({ "port": port, "public": false }));
        let body = self.send(rb).await?;
        let url = body["url"]
            .as_str()
            .ok_or_else(|| SandboxError::Other("response had no url".to_string()))?;
        Ok(PreviewUrl {
            url: url.to_string(),
        })
    }
}
