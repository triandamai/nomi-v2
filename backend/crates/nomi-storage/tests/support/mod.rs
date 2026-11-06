#![allow(dead_code)]
//! A small in-memory S3 for tests: enough of PutObject, GetObject, DeleteObject, ListObjectsV2 and
//! DeleteObjects (path-style, as Nomi talks to S3-compatible services) to exercise the real SDK
//! calls, plus a switch to make any operation answer with an S3 error code.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use nomi_storage::S3Config;
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

pub const BUCKET: &str = "nomi-test";

#[derive(Default)]
pub struct State {
    pub objects: BTreeMap<String, Vec<u8>>,
    /// `(method, code, status)`: every request with that method answers this S3 error.
    pub failures: Vec<(String, String, u16)>,
    /// List pages this many keys at a time.
    pub page_size: usize,
    pub requests: Vec<String>,
}

#[derive(Clone)]
pub struct FakeS3 {
    pub state: Arc<Mutex<State>>,
    pub server: Arc<MockServer>,
}

fn error(code: &str, status: u16) -> ResponseTemplate {
    ResponseTemplate::new(status)
        .insert_header("content-type", "application/xml")
        .set_body_string(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><Error><Code>{code}</Code><Message>{code} from the fake</Message><RequestId>r1</RequestId></Error>"))
}

/// Undoes `aws-chunked` framing (`<hex size>[;ext]\r\n<data>\r\n ... 0\r\n<trailers>`).
fn unchunk(body: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut rest = body;
    loop {
        let Some(line_end) = rest.windows(2).position(|w| w == b"\r\n") else { break };
        let header = std::str::from_utf8(&rest[..line_end]).unwrap_or("0");
        let size = usize::from_str_radix(header.split(';').next().unwrap_or("0").trim(), 16).unwrap_or(0);
        if size == 0 {
            break;
        }
        let start = line_end + 2;
        out.extend_from_slice(&rest[start..start + size]);
        rest = &rest[(start + size + 2).min(rest.len())..];
    }
    out
}

struct Responder(Arc<Mutex<State>>);

impl Respond for Responder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let mut state = self.0.lock().unwrap();
        let method = request.method.to_string();
        let path = request.url.path().trim_start_matches('/').to_string();
        state.requests.push(format!("{method} {path}?{}", request.url.query().unwrap_or("")));
        if let Some((_, code, status)) = state.failures.iter().find(|(m, _, _)| *m == method).cloned() {
            return error(&code, status);
        }
        let key = path.strip_prefix(&format!("{BUCKET}/")).unwrap_or("").to_string();
        let query: Vec<(String, String)> = request.url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
        let param = |name: &str| query.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone());
        match method.as_str() {
            "PUT" => {
                let chunked = request
                    .headers
                    .get("content-encoding")
                    .map(|v| v.to_str().unwrap_or("").contains("aws-chunked"))
                    .unwrap_or(false)
                    || request.headers.get("x-amz-decoded-content-length").is_some();
                let body = if chunked { unchunk(&request.body) } else { request.body.clone() };
                state.objects.insert(key, body);
                ResponseTemplate::new(200).insert_header("etag", "\"e\"")
            }
            "GET" if param("list-type").is_some() => {
                let prefix = param("prefix").unwrap_or_default();
                let after = param("continuation-token");
                let keys: Vec<&String> = state
                    .objects
                    .keys()
                    .filter(|k| k.starts_with(&prefix))
                    .filter(|k| after.as_ref().map(|a| k.as_str() > a.as_str()).unwrap_or(true))
                    .collect();
                let size = if state.page_size == 0 { usize::MAX } else { state.page_size };
                let page: Vec<&String> = keys.iter().take(size).copied().collect();
                let truncated = keys.len() > page.len();
                let contents: String = page.iter().map(|k| format!("<Contents><Key>{k}</Key><Size>1</Size></Contents>")).collect();
                let next = if truncated { format!("<NextContinuationToken>{}</NextContinuationToken>", page.last().unwrap()) } else { String::new() };
                ResponseTemplate::new(200).insert_header("content-type", "application/xml").set_body_string(format!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ListBucketResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"><Name>{BUCKET}</Name><Prefix>{prefix}</Prefix><KeyCount>{}</KeyCount><MaxKeys>1000</MaxKeys><IsTruncated>{truncated}</IsTruncated>{contents}{next}</ListBucketResult>",
                    page.len()
                ))
            }
            "GET" => match state.objects.get(&key) {
                Some(body) => ResponseTemplate::new(200).set_body_bytes(body.clone()),
                None => error("NoSuchKey", 404),
            },
            "DELETE" => {
                state.objects.remove(&key);
                ResponseTemplate::new(204)
            }
            "POST" if param("delete").is_some() => {
                let body = String::from_utf8_lossy(&request.body).to_string();
                for part in body.split("<Key>").skip(1) {
                    if let Some(k) = part.split("</Key>").next() {
                        state.objects.remove(k);
                    }
                }
                ResponseTemplate::new(200)
                    .insert_header("content-type", "application/xml")
                    .set_body_string("<?xml version=\"1.0\" encoding=\"UTF-8\"?><DeleteResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"></DeleteResult>")
            }
            _ => error("NotImplemented", 501),
        }
    }
}

impl FakeS3 {
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        let state = Arc::new(Mutex::new(State::default()));
        Mock::given(wiremock::matchers::any()).respond_with(Responder(state.clone())).mount(&server).await;
        FakeS3 { state, server: Arc::new(server) }
    }

    pub fn uri(&self) -> String {
        self.server.uri()
    }

    pub async fn config(&self) -> S3Config {
        let uri = self.uri();
        nomi_storage::build_from(move |field| match field {
            "bucket" => Some(BUCKET.to_string()),
            "access_key_id" => Some("AKIDTEST".to_string()),
            "secret_access_key" => Some("secret".to_string()),
            "endpoint_url" => Some(uri.clone()),
            "region" => Some("auto".to_string()),
            _ => None,
        })
        .await
        .expect("a full config builds")
    }

    pub fn fail(&self, method: &str, code: &str, status: u16) {
        self.state.lock().unwrap().failures.push((method.to_string(), code.to_string(), status));
    }

    pub fn clear_failures(&self) {
        self.state.lock().unwrap().failures.clear();
    }

    pub fn object(&self, key: &str) -> Option<Vec<u8>> {
        self.state.lock().unwrap().objects.get(key).cloned()
    }

    pub fn insert(&self, key: &str, body: &[u8]) {
        self.state.lock().unwrap().objects.insert(key.to_string(), body.to_vec());
    }

    pub fn keys(&self) -> Vec<String> {
        self.state.lock().unwrap().objects.keys().cloned().collect()
    }
}
