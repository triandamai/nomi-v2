//! `/storage/*` on a project's host: the files `nomi.storage` uploads, downloads, lists, removes
//! and signs, each checked against its bucket's rules in `backend.json`. Every byte goes through
//! here; the S3 bucket behind it stays private.
//!
//! - `PUT|GET /storage/object/<bucket>/<path>`, `DELETE /storage/object/<bucket>` `{ paths }`
//! - `GET /storage/list/<bucket>?prefix&limit&offset`
//! - `POST /storage/sign/<bucket>/<path>` `{ expires_in }`, then `GET` the answer without a key
//! - `GET /storage/public/<bucket>/<path>`, without a key, for a public bucket

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{to_bytes, Body};
use axum::extract::Request;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use object_store::{Attribute, Attributes, ObjectStore, PutMode, PutOptions, PutPayload};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use relay_control::files::{bucket_prefix, object_path};
use relay_control::storage;
use relay_core::backend::{allows, clean_path, clean_prefix, BucketConfig, Rule, Who};
use relay_core::keys::{file_token, file_token_opens};
use relay_core::Env;
use relay_gateway::auth::Kind;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::gateway::{refuse, Gateway, Target, MAX_BODY_BYTES};

pub const STORAGE_PREFIX: &str = "/storage/";
/// A signed URL lasts an hour unless asked otherwise, and a week at most.
const DEFAULT_SIGNED_SECONDS: u64 = 3600;
const MAX_SIGNED_SECONDS: u64 = 7 * 24 * 3600;
const DEFAULT_LIST_LIMIT: usize = 100;
const MAX_LIST_LIMIT: usize = 1000;
/// Path segments in a signed URL are encoded like `encodeURIComponent` does.
const SEGMENT: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'_').remove(b'.').remove(b'!').remove(b'~').remove(b'*').remove(b'\'').remove(b'(').remove(b')');

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
}

fn decode(segment: &str) -> Option<String> {
    percent_decode_str(segment).decode_utf8().ok().map(|s| s.into_owned())
}

/// `<kind>/<bucket>/<path…>`, decoded: the path's segments joined back with `/`.
fn parse(rest: &str) -> Option<(String, String, String)> {
    let mut parts = rest.splitn(3, '/');
    let kind = parts.next()?.to_string();
    let bucket = decode(parts.next()?)?;
    let path = parts.next().unwrap_or_default().split('/').map(decode).collect::<Option<Vec<_>>>()?.join("/");
    Some((kind, bucket, path))
}

fn query(request: &Request, name: &str) -> Option<String> {
    request.uri().query()?.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then(|| decode(&value.replace('+', " "))).flatten()
    })
}

fn who(target: &Target) -> Who {
    Who { service: target.caller.kind == Kind::Service, user: target.caller.sub }
}

fn forbidden(rule: Rule) -> Response {
    let message = match rule {
        Rule::Anyone => "Not allowed.",
        Rule::SignedIn => "Sign in first: this bucket is for signed-in users.",
        Rule::Owner => "This file is in someone else's folder: a user's files go under their own id.",
        Rule::Service => "Only the app's server (the service key) can do this in this bucket.",
    };
    refuse(StatusCode::FORBIDDEN, "RELAY_FORBIDDEN", message)
}

fn bad_path() -> Response {
    refuse(StatusCode::BAD_REQUEST, "RELAY_BAD_PATH", "That isn't a file path in the bucket: no empty, . or .. folders.")
}

fn not_found() -> Response {
    refuse(StatusCode::NOT_FOUND, "RELAY_NOT_FOUND", "No file at that path.")
}

fn too_big(bucket: &BucketConfig) -> Response {
    refuse(StatusCode::PAYLOAD_TOO_LARGE, "RELAY_TOO_LARGE", &format!("Files in {} can be up to {} MB.", bucket.name, bucket.max_size_mb))
}

fn store_failed(e: object_store::Error) -> Response {
    tracing::error!(error = %e, "file storage failed");
    refuse(StatusCode::BAD_GATEWAY, "RELAY_STORAGE_UNAVAILABLE", "File storage didn't answer: try again in a moment.")
}

#[derive(Deserialize)]
struct RemoveRequest {
    paths: Vec<String>,
}

#[derive(Deserialize, Default)]
struct SignRequest {
    #[serde(default)]
    expires_in: Option<u64>,
}

pub async fn handle(gateway: &Gateway, slug: &str, env: Env, request: Request) -> Response {
    let Some(store) = gateway.relay.store() else {
        return refuse(StatusCode::SERVICE_UNAVAILABLE, "RELAY_NO_STORAGE", "File storage isn't set up on this Relay.");
    };
    let Some((kind, bucket_name, path)) = request.uri().path().strip_prefix(STORAGE_PREFIX).and_then(parse) else {
        return bad_path();
    };
    let method = request.method().clone();
    let keyless = matches!(kind.as_str(), "public") || (kind == "sign" && matches!(method, Method::GET | Method::HEAD));
    let (project, signing_secret, caller) = if keyless {
        match gateway.locate(slug, env).await {
            Ok((project, row)) => (project, row.signing_secret, None),
            Err(response) => return response,
        }
    } else {
        match gateway.target(slug, env, request.headers()).await {
            Ok(target) => (target.project, target.environment.signing_secret.clone(), Some(who(&target))),
            Err(response) => return response,
        }
    };
    let config = match storage::load(&gateway.relay, project, env).await {
        Ok(config) => config,
        Err(e) => return gateway.failed(e.to_string()),
    };
    let Some(bucket) = config.bucket(&bucket_name).cloned() else {
        return refuse(StatusCode::NOT_FOUND, "RELAY_NO_BUCKET", &format!("No bucket called {bucket_name}: add it to nomi/backend.json's storage."));
    };
    let files = Files { store, project, env, bucket };

    match (method, kind.as_str(), caller) {
        (Method::PUT, "object", Some(who)) => files.upload(who, &path, request).await,
        (Method::GET | Method::HEAD, "object", Some(who)) => files.read(who, &path).await,
        (Method::DELETE, "object", Some(who)) if path.is_empty() => files.remove(who, request).await,
        (Method::GET, "list", Some(who)) => {
            let prefix = query(&request, "prefix").unwrap_or_default();
            let limit = query(&request, "limit").and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_LIST_LIMIT).min(MAX_LIST_LIMIT);
            let offset = query(&request, "offset").and_then(|v| v.parse().ok()).unwrap_or(0);
            files.list(who, &prefix, limit, offset).await
        }
        (Method::POST, "sign", Some(who)) => files.sign(who, &path, &signing_secret, request).await,
        (Method::GET | Method::HEAD, "sign", None) => {
            let token = query(&request, "token").unwrap_or_default();
            match clean_path(&path) {
                Some(path) if file_token_opens(&signing_secret, &token, project, env, &files.bucket.name, &path, now()) => files.serve(&path).await,
                Some(_) => refuse(StatusCode::FORBIDDEN, "RELAY_BAD_TOKEN", "This link has expired or isn't for this file."),
                None => bad_path(),
            }
        }
        (Method::GET | Method::HEAD, "public", None) if files.bucket.public => match clean_path(&path) {
            Some(path) => files.serve(&path).await,
            None => bad_path(),
        },
        (Method::GET | Method::HEAD, "public", None) => refuse(StatusCode::NOT_FOUND, "RELAY_NOT_FOUND", "No public file at that path."),
        _ => refuse(StatusCode::NOT_FOUND, "RELAY_NOT_FOUND", "Nothing here: see nomi.storage for what storage answers."),
    }
}

/// One bucket of one project environment.
struct Files {
    store: Arc<dyn ObjectStore>,
    project: Uuid,
    env: Env,
    bucket: BucketConfig,
}

impl Files {
    fn location(&self, path: &str) -> object_store::path::Path {
        object_path(self.project, self.env, &self.bucket.name, path)
    }

    async fn upload(&self, who: Who, path: &str, request: Request) -> Response {
        let Some(path) = clean_path(path) else { return bad_path() };
        if !allows(self.bucket.write, who, &path) {
            return forbidden(self.bucket.write);
        }
        let headers = request.headers();
        let text = |name: header::HeaderName| headers.get(name).and_then(|v| v.to_str().ok()).map(str::to_string);
        let content_type = text(header::CONTENT_TYPE).unwrap_or_else(|| "application/octet-stream".into());
        if !self.bucket.accepts_type(&content_type) {
            let allowed = self.bucket.types.join(", ");
            return refuse(StatusCode::UNSUPPORTED_MEDIA_TYPE, "RELAY_BAD_TYPE", &format!("{} takes {allowed}, not {content_type}.", self.bucket.name));
        }
        let max = usize::try_from(self.bucket.max_bytes()).unwrap_or(usize::MAX);
        if text(header::CONTENT_LENGTH).and_then(|v| v.parse::<usize>().ok()).is_some_and(|n| n > max) {
            return too_big(&self.bucket);
        }
        let upsert = headers.get("x-nomi-upsert").and_then(|v| v.to_str().ok()) == Some("true");
        let cache_control = text(header::CACHE_CONTROL);
        let Ok(bytes) = to_bytes(request.into_body(), max).await else { return too_big(&self.bucket) };

        let mut attributes = Attributes::new();
        attributes.insert(Attribute::ContentType, content_type.into());
        if let Some(cache_control) = cache_control {
            attributes.insert(Attribute::CacheControl, cache_control.into());
        }
        let location = self.location(&path);
        let mode = if upsert { PutMode::Overwrite } else { PutMode::Create };
        let put = |mode: PutMode| self.store.put_opts(&location, PutPayload::from(bytes.clone()), PutOptions { mode, attributes: attributes.clone(), ..PutOptions::default() });
        let result = match put(mode.clone()).await {
            // A store without conditional writes: look first, then write.
            Err(object_store::Error::NotImplemented) if mode == PutMode::Create => match self.store.head(&location).await {
                Ok(_) => Err(object_store::Error::AlreadyExists { path: location.to_string(), source: "exists".into() }),
                Err(object_store::Error::NotFound { .. }) => put(PutMode::Overwrite).await,
                Err(e) => Err(e),
            },
            other => other,
        };
        match result {
            Ok(_) => Json(json!({ "path": path })).into_response(),
            Err(object_store::Error::AlreadyExists { .. }) => {
                refuse(StatusCode::CONFLICT, "RELAY_EXISTS", "A file is already at this path: upload with upsert to replace it.")
            }
            Err(e) => store_failed(e),
        }
    }

    async fn read(&self, who: Who, path: &str) -> Response {
        let Some(path) = clean_path(path) else { return bad_path() };
        if !allows(self.bucket.read, who, &path) {
            return forbidden(self.bucket.read);
        }
        self.serve(&path).await
    }

    async fn serve(&self, path: &str) -> Response {
        let found = match self.store.get(&self.location(path)).await {
            Ok(found) => found,
            Err(object_store::Error::NotFound { .. }) => return not_found(),
            Err(e) => return store_failed(e),
        };
        let mut response = Response::new(Body::empty());
        let headers = response.headers_mut();
        let content_type = found.attributes.get(&Attribute::ContentType).map(|v| v.to_string()).unwrap_or_else(|| "application/octet-stream".into());
        if let Ok(value) = HeaderValue::from_str(&content_type) {
            headers.insert(header::CONTENT_TYPE, value);
        }
        if let Some(value) = found.attributes.get(&Attribute::CacheControl).and_then(|v| HeaderValue::from_str(v).ok()) {
            headers.insert(header::CACHE_CONTROL, value);
        }
        headers.insert(header::CONTENT_LENGTH, HeaderValue::from(found.meta.size));
        // Never rendered as a page on the API's own origin.
        headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
        headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("sandbox"));
        *response.body_mut() = Body::from_stream(found.into_stream());
        response
    }

    async fn remove(&self, who: Who, request: Request) -> Response {
        let Ok(bytes) = to_bytes(request.into_body(), MAX_BODY_BYTES).await else {
            return refuse(StatusCode::PAYLOAD_TOO_LARGE, "RELAY_TOO_LARGE", "The request body is too large.");
        };
        let Ok(body) = serde_json::from_slice::<RemoveRequest>(&bytes) else {
            return refuse(StatusCode::BAD_REQUEST, "RELAY_BAD_BODY", "Send { paths: [...] }.");
        };
        let mut paths = Vec::with_capacity(body.paths.len());
        for path in &body.paths {
            let Some(path) = clean_path(path) else { return bad_path() };
            if !allows(self.bucket.write, who, &path) {
                return forbidden(self.bucket.write);
            }
            paths.push(path);
        }
        for path in &paths {
            match self.store.delete(&self.location(path)).await {
                Ok(()) | Err(object_store::Error::NotFound { .. }) => {}
                Err(e) => return store_failed(e),
            }
        }
        Json(serde_json::Value::Null).into_response()
    }

    async fn list(&self, who: Who, prefix: &str, limit: usize, offset: usize) -> Response {
        let Some(prefix) = clean_prefix(prefix) else { return bad_path() };
        if !allows(self.bucket.read, who, &prefix) {
            return forbidden(self.bucket.read);
        }
        let base = bucket_prefix(self.project, self.env, &self.bucket.name);
        let folder = if prefix.is_empty() { base } else { self.location(&prefix) };
        let listing = match self.store.list_with_delimiter(Some(&folder)).await {
            Ok(listing) => listing,
            Err(e) => return store_failed(e),
        };
        let folders = listing.common_prefixes.iter().filter_map(|p| p.filename().map(|name| json!({ "name": format!("{name}/"), "size": 0, "content_type": null, "updated_at": null })));
        let mut objects: Vec<_> = listing.objects.iter().collect();
        objects.sort_by(|a, b| a.location.as_ref().cmp(b.location.as_ref()));
        let files = objects.into_iter().filter_map(|o| {
            o.location.filename().map(|name| json!({ "name": name, "size": o.size, "content_type": null, "updated_at": o.last_modified.to_rfc3339() }))
        });
        let entries: Vec<_> = folders.chain(files).skip(offset).take(limit).collect();
        Json(entries).into_response()
    }

    async fn sign(&self, who: Who, path: &str, secret: &str, request: Request) -> Response {
        let Some(path) = clean_path(path) else { return bad_path() };
        if !allows(self.bucket.read, who, &path) {
            return forbidden(self.bucket.read);
        }
        let bytes = to_bytes(request.into_body(), MAX_BODY_BYTES).await.unwrap_or_default();
        let body: SignRequest = if bytes.is_empty() { SignRequest::default() } else { serde_json::from_slice(&bytes).unwrap_or_default() };
        let seconds = body.expires_in.unwrap_or(DEFAULT_SIGNED_SECONDS).clamp(1, MAX_SIGNED_SECONDS);
        match self.store.head(&self.location(&path)).await {
            Ok(_) => {}
            Err(object_store::Error::NotFound { .. }) => return not_found(),
            Err(e) => return store_failed(e),
        }
        let token = file_token(secret, self.project, self.env, &self.bucket.name, &path, now(), seconds);
        let encoded: Vec<String> = path.split('/').map(|s| utf8_percent_encode(s, SEGMENT).to_string()).collect();
        let bucket = utf8_percent_encode(&self.bucket.name, SEGMENT);
        Json(json!({ "signed_url": format!("/storage/sign/{bucket}/{}?token={token}", encoded.join("/")) })).into_response()
    }
}
