//! A small client for the Hevy API (https://api.hevyapp.com/docs/): workouts, routines, routine
//! folders, exercise templates, exercise history and body measurements. Every call carries the
//! person's own key in the `api-key` header (Hevy Pro: hevy.com → Settings → Developer).

use std::time::Duration;

use serde_json::{json, Value};

pub const DEFAULT_BASE_URL: &str = "https://api.hevyapp.com";
/// Points Nomi at another Hevy-compatible host (a stub in development).
pub const ENV_BASE_URL: &str = "HEVY_BASE_URL";

pub fn base_url() -> String {
    std::env::var(ENV_BASE_URL).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
}

/// What went wrong talking to Hevy.
#[derive(Debug, Clone, PartialEq)]
pub enum HevyError {
    /// The key was refused (wrong, revoked, or the account isn't on Hevy Pro).
    KeyRejected,
    NotFound(String),
    /// Hevy refused the request; its own words.
    Rejected(String),
    /// A limit on the account (routines, custom exercises).
    LimitReached(String),
    /// Already there (a body measurement for that date).
    Conflict(String),
    Unreachable(String),
}

impl std::fmt::Display for HevyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HevyError::KeyRejected => f.write_str("Hevy rejected the API key (it may have been revoked, or the account isn't on Hevy Pro)"),
            HevyError::NotFound(what) => write!(f, "Hevy has no such {what}"),
            HevyError::Rejected(why) => write!(f, "Hevy refused it: {why}"),
            HevyError::LimitReached(why) => write!(f, "the Hevy account has reached a limit: {why}"),
            HevyError::Conflict(why) => write!(f, "Hevy already has that: {why}"),
            HevyError::Unreachable(why) => write!(f, "couldn't reach Hevy: {why}"),
        }
    }
}

/// The person's Hevy account, reached with their key.
pub struct Hevy<'a> {
    pub http: &'a reqwest::Client,
    pub base_url: String,
    pub api_key: &'a str,
}

/// Hevy's own `{"error": "..."}` text, or the raw body.
fn error_text(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(str::to_string))
        .unwrap_or_else(|| body.chars().take(300).collect())
}

impl<'a> Hevy<'a> {
    pub fn new(http: &'a reqwest::Client, api_key: &'a str) -> Self {
        Self { http, base_url: base_url(), api_key }
    }

    async fn send(&self, method: reqwest::Method, path: &str, query: &[(&str, String)], body: Option<&Value>, what: &str) -> Result<Value, HevyError> {
        let url = format!("{}{path}", self.base_url.trim_end_matches('/'));
        let mut request = self.http.request(method, url).header("api-key", self.api_key).header("accept", "application/json").timeout(Duration::from_secs(30));
        if !query.is_empty() {
            request = request.query(query);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().await.map_err(|e| HevyError::Unreachable(e.to_string()))?;
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        match status {
            200..=299 => Ok(if text.trim().is_empty() { json!({}) } else { serde_json::from_str(&text).unwrap_or(json!({ "raw": text })) }),
            401 => Err(HevyError::KeyRejected),
            403 if text.contains("limit") => Err(HevyError::LimitReached(error_text(&text))),
            403 => Err(HevyError::KeyRejected),
            404 => Err(HevyError::NotFound(what.to_string())),
            409 => Err(HevyError::Conflict(error_text(&text))),
            s if s >= 500 => Err(HevyError::Unreachable(format!("Hevy returned {s}"))),
            _ => Err(HevyError::Rejected(error_text(&text))),
        }
    }

    async fn get(&self, path: &str, query: &[(&str, String)], what: &str) -> Result<Value, HevyError> {
        self.send(reqwest::Method::GET, path, query, None, what).await
    }

    fn page(page: u32, size: u32) -> Vec<(&'static str, String)> {
        vec![("page", page.max(1).to_string()), ("pageSize", size.to_string())]
    }

    /// The account the key belongs to: `{id, name, url}`.
    pub async fn user_info(&self) -> Result<Value, HevyError> {
        let body = self.get("/v1/user/info", &[], "user").await?;
        Ok(body.get("data").cloned().unwrap_or(body))
    }

    pub async fn workouts(&self, page: u32, size: u32) -> Result<Value, HevyError> {
        self.get("/v1/workouts", &Self::page(page, size.clamp(1, 10)), "workouts page").await
    }

    pub async fn workout(&self, id: &str) -> Result<Value, HevyError> {
        self.get(&format!("/v1/workouts/{}", segment(id)), &[], "workout").await
    }

    pub async fn workout_count(&self) -> Result<Value, HevyError> {
        self.get("/v1/workouts/count", &[], "count").await
    }

    /// Workouts updated or deleted since `since` (ISO 8601).
    pub async fn workout_events(&self, since: &str, page: u32) -> Result<Value, HevyError> {
        let mut query = Self::page(page, 10);
        query.push(("since", since.to_string()));
        self.get("/v1/workouts/events", &query, "events page").await
    }

    pub async fn create_workout(&self, workout: &Value) -> Result<Value, HevyError> {
        self.send(reqwest::Method::POST, "/v1/workouts", &[], Some(&json!({ "workout": workout })), "workout").await
    }

    pub async fn update_workout(&self, id: &str, workout: &Value) -> Result<Value, HevyError> {
        self.send(reqwest::Method::PUT, &format!("/v1/workouts/{}", segment(id)), &[], Some(&json!({ "workout": workout })), "workout").await
    }

    pub async fn routines(&self, page: u32, size: u32) -> Result<Value, HevyError> {
        self.get("/v1/routines", &Self::page(page, size.clamp(1, 10)), "routines page").await
    }

    pub async fn routine(&self, id: &str) -> Result<Value, HevyError> {
        let body = self.get(&format!("/v1/routines/{}", segment(id)), &[], "routine").await?;
        Ok(body.get("routine").cloned().unwrap_or(body))
    }

    pub async fn create_routine(&self, routine: &Value) -> Result<Value, HevyError> {
        let body = self.send(reqwest::Method::POST, "/v1/routines", &[], Some(&json!({ "routine": routine })), "routine").await?;
        Ok(body.get("routine").cloned().unwrap_or(body))
    }

    pub async fn update_routine(&self, id: &str, routine: &Value) -> Result<Value, HevyError> {
        let body = self.send(reqwest::Method::PUT, &format!("/v1/routines/{}", segment(id)), &[], Some(&json!({ "routine": routine })), "routine").await?;
        Ok(body.get("routine").cloned().unwrap_or(body))
    }

    pub async fn routine_folders(&self, page: u32) -> Result<Value, HevyError> {
        self.get("/v1/routine_folders", &Self::page(page, 10), "folders page").await
    }

    pub async fn create_routine_folder(&self, title: &str) -> Result<Value, HevyError> {
        let body = self.send(reqwest::Method::POST, "/v1/routine_folders", &[], Some(&json!({ "routine_folder": { "title": title } })), "folder").await?;
        Ok(body.get("routine_folder").cloned().unwrap_or(body))
    }

    pub async fn exercise_templates(&self, page: u32) -> Result<Value, HevyError> {
        self.get("/v1/exercise_templates", &Self::page(page, 100), "exercises page").await
    }

    pub async fn exercise_template(&self, id: &str) -> Result<Value, HevyError> {
        self.get(&format!("/v1/exercise_templates/{}", segment(id)), &[], "exercise").await
    }

    pub async fn create_exercise_template(&self, exercise: &Value) -> Result<Value, HevyError> {
        self.send(reqwest::Method::POST, "/v1/exercise_templates", &[], Some(&json!({ "exercise": exercise })), "exercise").await
    }

    pub async fn exercise_history(&self, template_id: &str, start: Option<&str>, end: Option<&str>) -> Result<Value, HevyError> {
        let mut query = Vec::new();
        if let Some(start) = start {
            query.push(("start_date", start.to_string()));
        }
        if let Some(end) = end {
            query.push(("end_date", end.to_string()));
        }
        self.get(&format!("/v1/exercise_history/{}", segment(template_id)), &query, "exercise").await
    }

    pub async fn body_measurements(&self, page: u32) -> Result<Value, HevyError> {
        self.get("/v1/body_measurements", &Self::page(page, 10), "measurements page").await
    }

    pub async fn body_measurement(&self, date: &str) -> Result<Value, HevyError> {
        self.get(&format!("/v1/body_measurements/{}", segment(date)), &[], "measurement for that date").await
    }

    /// Logs measurements for `date`. A day that already has an entry is updated instead, keeping
    /// the values not given this time (Hevy's update clears whatever is left out).
    pub async fn log_body_measurement(&self, date: &str, values: &serde_json::Map<String, Value>) -> Result<Value, HevyError> {
        let mut entry = values.clone();
        entry.insert("date".to_string(), json!(date));
        match self.send(reqwest::Method::POST, "/v1/body_measurements", &[], Some(&Value::Object(entry)), "measurement").await {
            Err(HevyError::Conflict(_)) => {}
            other => return other.map(|_| json!({ "date": date, "logged": values })),
        }
        let mut merged = match self.body_measurement(date).await? {
            Value::Object(existing) => existing,
            _ => serde_json::Map::new(),
        };
        merged.remove("date");
        for (key, value) in values {
            merged.insert(key.clone(), value.clone());
        }
        self.send(reqwest::Method::PUT, &format!("/v1/body_measurements/{}", segment(date)), &[], Some(&Value::Object(merged.clone())), "measurement").await?;
        Ok(json!({ "date": date, "updated": merged }))
    }
}

/// An id or date as one URL path segment (nothing that could step into another path).
fn segment(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')).collect()
}
