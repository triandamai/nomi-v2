//! The Health agent (Raga): the person's training in Hevy (workouts, routines, exercises,
//! progress, body measurements) through their own Hevy API key, connected on Connections.
//! Health monitors (heart rate, oxygen, steps, runs from Samsung Health, Apple Health, ...) join
//! as further connections later.
//!
//! Storage (docs/agents/storage.md): `health_connections`.

pub mod connection;
pub mod format;
pub mod hevy;

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde_json::{json, Map, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::content_block::{ContentBlock, ToolOutcome};
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

use hevy::{Hevy, HevyError};

pub const HEALTH_AGENT_TYPE: &str = "health";

const SYSTEM_PROMPT: &str = "You are Raga, the crew member for the person's health and training. \
You work in their own Hevy account (workouts, routines, routine folders, exercises, exercise \
history, body measurements). You only ever see the account of the person you are helping.

How you work:
- Exercises are referenced by their Hevy exercise template id. Find ids with hevy_find_exercises \
before logging a workout or building a routine; never invent ids. If an exercise doesn't exist, \
offer to create a custom one.
- Logging a workout: ask only for what's missing (which exercises, sets, reps, weight), with \
ask_user, one question at a time. Times \
are ISO 8601 with the person's offset; if they say \"today\" or \"this morning\", work it out \
from the current time. Weights are in kg (convert from lb when they use lb).
- Updating a workout or routine replaces it whole: read it first, change what was asked, send \
everything back.
- Progress questions (\"how is my bench going?\"): read the exercise history and answer with \
the trend in a sentence or two (best set, volume, how often), then the numbers.
- Show workouts, routines or measurements with `show` when there are more than a few: a list or a table, \
a chart for progress over time.
- Keep advice practical and safe. You aren't a doctor: for pain, injury or medical questions, \
suggest seeing a professional.
- Heart rate, blood oxygen, steps, sleep and runs from Samsung Health or Apple Health aren't \
connected yet; say so if asked.
- If a tool says Hevy isn't connected or the key stopped working, a card to set it up is \
already shown to the person: tell them in one short sentence and stop.";

/// One shared HTTP client for every registry this process builds.
fn http_client() -> reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new).clone()
}

pub struct HealthAgent {
    http: reqwest::Client,
    key: [u8; 32],
    hevy_base_url: String,
}

impl HealthAgent {
    pub fn new(http: reqwest::Client, key: [u8; 32]) -> Self {
        Self { http, key, hevy_base_url: hevy::base_url() }
    }

    /// Talks to Hevy at `base_url` instead (a stub in tests).
    pub fn with_hevy_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.hevy_base_url = base_url.into();
        self
    }

    /// The encryption key from the environment (SETTINGS_ENCRYPTION_KEY).
    pub fn from_env() -> Self {
        let key = std::env::var("SETTINGS_ENCRYPTION_KEY").ok().and_then(|k| nomi_settings::crypto::parse_key(&k)).unwrap_or([0u8; 32]);
        Self::new(http_client(), key)
    }
}

fn tool(name: &str, description: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.to_string(), description: description.to_string(), input_schema: schema }
}

fn str_arg<'a>(input: &'a Value, key: &str) -> Option<&'a str> {
    input.get(key).and_then(|v| v.as_str()).map(str::trim).filter(|v| !v.is_empty())
}

fn required<'a>(input: &'a Value, key: &str) -> Result<&'a str, String> {
    str_arg(input, key).ok_or_else(|| format!("missing {key}"))
}

fn page_arg(input: &Value) -> u32 {
    input.get("page").and_then(|v| v.as_u64()).map(|p| p.clamp(1, 1000) as u32).unwrap_or(1)
}

fn set_schema(for_routine: bool) -> Value {
    let mut properties = json!({
        "type": {"type": "string", "enum": ["warmup", "normal", "failure", "dropset"]},
        "weight_kg": {"type": "number", "nullable": true},
        "reps": {"type": "integer", "nullable": true},
        "distance_meters": {"type": "integer", "nullable": true},
        "duration_seconds": {"type": "integer", "nullable": true},
        "custom_metric": {"type": "number", "nullable": true}
    });
    if for_routine {
        properties["rep_range"] = json!({"type": "object", "nullable": true, "properties": {"start": {"type": "number"}, "end": {"type": "number"}}, "description": "A target range like 8-12 instead of fixed reps."});
    } else {
        properties["rpe"] = json!({"type": "number", "nullable": true, "description": "Rate of perceived exertion from 6 to 10 (e.g. 7.5, 8, 8.5, 9)."});
    }
    json!({"type": "object", "properties": properties, "required": ["type"]})
}

fn exercises_schema(for_routine: bool) -> Value {
    let mut properties = json!({
        "exercise_template_id": {"type": "string", "description": "From hevy_find_exercises."},
        "superset_id": {"type": "integer", "nullable": true},
        "notes": {"type": "string", "nullable": true},
        "sets": {"type": "array", "items": set_schema(for_routine)}
    });
    if for_routine {
        properties["rest_seconds"] = json!({"type": "integer", "nullable": true});
    }
    json!({"type": "array", "items": {"type": "object", "properties": properties, "required": ["exercise_template_id", "sets"]}})
}

fn workout_schema(with_id: bool) -> Value {
    let mut properties = json!({
        "title": {"type": "string"},
        "description": {"type": "string", "nullable": true},
        "start_time": {"type": "string", "description": "ISO 8601, e.g. 2026-10-12T07:30:00+07:00"},
        "end_time": {"type": "string"},
        "is_private": {"type": "boolean"},
        "exercises": exercises_schema(false)
    });
    let mut required = vec!["title", "start_time", "end_time", "exercises"];
    if with_id {
        properties["workout_id"] = json!({"type": "string"});
        required.insert(0, "workout_id");
    }
    json!({"type": "object", "properties": properties, "required": required})
}

fn routine_schema(update: bool) -> Value {
    let mut properties = json!({
        "title": {"type": "string"},
        "notes": {"type": "string", "nullable": true},
        "exercises": exercises_schema(true)
    });
    let mut required = vec!["title", "exercises"];
    if update {
        properties["routine_id"] = json!({"type": "string"});
        required.insert(0, "routine_id");
    } else {
        properties["folder_id"] = json!({"type": "integer", "nullable": true, "description": "From hevy_routine_folders; empty puts it in My Routines."});
    }
    json!({"type": "object", "properties": properties, "required": required})
}

const MEASUREMENTS: [&str; 17] = [
    "weight_kg", "lean_mass_kg", "fat_percent", "neck_cm", "shoulder_cm", "chest_cm", "left_bicep_cm", "right_bicep_cm", "left_forearm_cm",
    "right_forearm_cm", "abdomen", "waist", "hips", "left_thigh", "right_thigh", "left_calf", "right_calf",
];

/// What the model passes for a workout or routine, as Hevy takes it (only the known fields).
fn pick(input: &Value, keys: &[&str]) -> Value {
    let mut out = Map::new();
    for key in keys {
        if let Some(value) = input.get(*key) {
            out.insert(key.to_string(), value.clone());
        }
    }
    Value::Object(out)
}

fn connect_card(reason: &str, text: &str) -> ToolOutcome {
    ToolOutcome { display_text: text.to_string(), block: Some(ContentBlock::ConnectorConnect { connector: connection::HEVY.to_string(), reason: reason.to_string() }) }
}

/// Every person's exercise templates, kept for a few hours: the list rarely changes and is a
/// dozen pages long.
fn exercise_cache() -> &'static Mutex<HashMap<Uuid, (Instant, Vec<Value>)>> {
    static CACHE: OnceLock<Mutex<HashMap<Uuid, (Instant, Vec<Value>)>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

const KEEP_EXERCISES: Duration = Duration::from_secs(6 * 60 * 60);

/// Every exercise template in the person's Hevy account, built-in and custom.
async fn all_exercises(hevy: &Hevy<'_>, user_id: Uuid) -> Result<Vec<Value>, HevyError> {
    if let Some((at, list)) = exercise_cache().lock().unwrap().get(&user_id) {
        if at.elapsed() < KEEP_EXERCISES {
            return Ok(list.clone());
        }
    }
    let mut list = Vec::new();
    let mut page = 1;
    loop {
        let body = hevy.exercise_templates(page).await?;
        list.extend(body.get("exercise_templates").and_then(|t| t.as_array()).cloned().unwrap_or_default());
        let pages = body.get("page_count").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
        if page >= pages || page >= 30 {
            break;
        }
        page += 1;
    }
    exercise_cache().lock().unwrap().insert(user_id, (Instant::now(), list.clone()));
    Ok(list)
}

/// Forgets the person's cached exercise list (after they add a custom exercise).
fn forget_exercises(user_id: Uuid) {
    exercise_cache().lock().unwrap().remove(&user_id);
}

#[async_trait]
impl SubAgent for HealthAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(HEALTH_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(SYSTEM_PROMPT)
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed("health")
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("health, fitness and training: logging or reviewing workouts, gym sessions and exercises, building or changing workout routines, progress on a lift, body weight and measurements, the person's Hevy account")
    }

    fn uses_memory(&self) -> bool {
        true
    }

    fn uses_personality(&self) -> bool {
        true
    }

    fn wants_current_time(&self) -> bool {
        true
    }

    fn max_tool_turns(&self) -> u32 {
        16
    }

    /// Everything here is the person's own training log in their own Hevy account: nothing
    /// reaches anyone else, so nothing waits for an OK.
    fn tool_needs_approval(&self, _tool_name: &str) -> bool {
        false
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            tool("hevy_workouts", "The person's Hevy workouts, newest first, 10 a page, with exercises and sets.",
                json!({"type": "object", "properties": {"page": {"type": "integer", "description": "1 for the newest."}}})),
            tool("hevy_workout", "One Hevy workout in full.", json!({"type": "object", "properties": {"workout_id": {"type": "string"}}, "required": ["workout_id"]})),
            tool("hevy_workout_count", "How many workouts the person has logged in Hevy.", json!({"type": "object", "properties": {}})),
            tool("hevy_workout_changes", "Workouts added, changed or deleted in Hevy since a time (ISO 8601).",
                json!({"type": "object", "properties": {"since": {"type": "string"}, "page": {"type": "integer"}}, "required": ["since"]})),
            tool("hevy_log_workout", "Log a finished workout in Hevy.", workout_schema(false)),
            tool("hevy_update_workout", "Replace a Hevy workout with this version (read it first and send it back whole).", workout_schema(true)),
            tool("hevy_routines", "The person's Hevy routines (workout plans), 10 a page.",
                json!({"type": "object", "properties": {"page": {"type": "integer"}}})),
            tool("hevy_routine", "One Hevy routine in full.", json!({"type": "object", "properties": {"routine_id": {"type": "string"}}, "required": ["routine_id"]})),
            tool("hevy_create_routine", "Create a routine (a workout plan) in Hevy.", routine_schema(false)),
            tool("hevy_update_routine", "Replace a Hevy routine with this version (read it first and send it back whole).", routine_schema(true)),
            tool("hevy_routine_folders", "The person's Hevy routine folders.", json!({"type": "object", "properties": {"page": {"type": "integer"}}})),
            tool("hevy_create_routine_folder", "Create a routine folder in Hevy.",
                json!({"type": "object", "properties": {"title": {"type": "string"}}, "required": ["title"]})),
            tool("hevy_find_exercises", "Find exercises (Hevy exercise templates, built-in and custom) by name and/or muscle group. Returns their ids.",
                json!({"type": "object", "properties": {
                    "query": {"type": "string", "description": "Part of the name, e.g. \"bench\" or \"squat\"."},
                    "muscle_group": {"type": "string", "description": "e.g. chest, quadriceps, lats, shoulders, biceps, hamstrings, glutes, abdominals, cardio."},
                    "custom_only": {"type": "boolean"}
                }})),
            tool("hevy_create_exercise", "Create a custom exercise in Hevy.",
                json!({"type": "object", "properties": {
                    "title": {"type": "string"},
                    "exercise_type": {"type": "string", "enum": ["weight_reps", "reps_only", "bodyweight_reps", "bodyweight_assisted_reps", "duration", "weight_duration", "distance_duration", "short_distance_weight"]},
                    "equipment_category": {"type": "string", "enum": ["none", "barbell", "dumbbell", "kettlebell", "machine", "plate", "resistance_band", "suspension", "other"]},
                    "muscle_group": {"type": "string", "enum": ["abdominals", "shoulders", "biceps", "triceps", "forearms", "quadriceps", "hamstrings", "calves", "glutes", "abductors", "adductors", "lats", "upper_back", "traps", "lower_back", "chest", "cardio", "neck", "full_body", "other"]},
                    "other_muscles": {"type": "array", "items": {"type": "string"}}
                }, "required": ["title", "exercise_type", "equipment_category", "muscle_group"]})),
            tool("hevy_exercise_history", "Every logged set of one exercise (for progress), optionally between two dates (ISO 8601).",
                json!({"type": "object", "properties": {"exercise_template_id": {"type": "string"}, "start_date": {"type": "string"}, "end_date": {"type": "string"}}, "required": ["exercise_template_id"]})),
            tool("hevy_body_measurements", "The person's body measurements in Hevy (weight, body fat, sizes), newest first, 10 a page.",
                json!({"type": "object", "properties": {"page": {"type": "integer"}}})),
            tool("hevy_log_body_measurement", "Log body measurements for a day in Hevy (an existing day is updated, keeping what isn't given).",
                json!({"type": "object", "properties": {
                    "date": {"type": "string", "description": "YYYY-MM-DD"},
                    "weight_kg": {"type": "number"}, "lean_mass_kg": {"type": "number"}, "fat_percent": {"type": "number"},
                    "neck_cm": {"type": "number"}, "shoulder_cm": {"type": "number"}, "chest_cm": {"type": "number"},
                    "left_bicep_cm": {"type": "number"}, "right_bicep_cm": {"type": "number"}, "left_forearm_cm": {"type": "number"}, "right_forearm_cm": {"type": "number"},
                    "abdomen": {"type": "number"}, "waist": {"type": "number"}, "hips": {"type": "number"},
                    "left_thigh": {"type": "number"}, "right_thigh": {"type": "number"}, "left_calf": {"type": "number"}, "right_calf": {"type": "number"}
                }, "required": ["date"]})),
        ]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<ToolOutcome, String> {
        let api_key = connection::api_key(conn, &self.key, user_id, connection::HEVY).await.map_err(|e| format!("couldn't load the Hevy connection: {e}"))?;
        let Some(api_key) = api_key else {
            return Ok(connect_card("not_connected", "The person hasn't connected Hevy. A card to connect it is shown to them."));
        };
        let hevy = Hevy { http: &self.http, base_url: self.hevy_base_url.clone(), api_key: &api_key };
        let mut saved = None;
        match run_tool(&hevy, user_id, name, &input, &mut saved).await {
            Ok(text) => {
                let block = match saved {
                    Some((key, value)) => {
                        let locale = nomi_agent_core::user_locale(conn, user_id).await;
                        Some(ContentBlock::View(format::card(&value, locale.t(key), locale)))
                    }
                    None => None,
                };
                Ok(ToolOutcome { display_text: text, block })
            }
            Err(HevyError::KeyRejected) => Ok(connect_card("key_rejected", "Hevy rejected the saved API key (revoked, or the account isn't on Hevy Pro). A card to set a new key is shown to the person.")),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Runs one Hevy tool. A workout or routine it saves goes in `saved` (with the card's heading),
/// so the chat can show it as a card.
async fn run_tool(hevy: &Hevy<'_>, user_id: Uuid, name: &str, input: &Value, saved: &mut Option<(&'static str, Value)>) -> Result<String, HevyError> {
    let arg = |key: &str| required(input, key).map_err(HevyError::Rejected);
    match name {
        "hevy_workouts" => Ok(format::workouts_page(&hevy.workouts(page_arg(input), 10).await?)),
        "hevy_workout" => Ok(format::workout(&hevy.workout(arg("workout_id")?).await?)),
        "hevy_workout_count" => Ok(format!("{} workouts logged.", hevy.workout_count().await?.get("workout_count").and_then(|v| v.as_u64()).unwrap_or(0))),
        "hevy_workout_changes" => Ok(format::workout_events(&hevy.workout_events(arg("since")?, page_arg(input)).await?)),
        "hevy_log_workout" => {
            let workout = pick(input, &["title", "description", "start_time", "end_time", "is_private", "exercises"]);
            let created = hevy.create_workout(&workout).await?;
            let text = format!("Logged.\n{}", format::workout(&created));
            *saved = Some(("health.card_logged", created));
            Ok(text)
        }
        "hevy_update_workout" => {
            let workout = pick(input, &["title", "description", "start_time", "end_time", "is_private", "exercises"]);
            let updated = hevy.update_workout(arg("workout_id")?, &workout).await?;
            let text = format!("Updated.\n{}", format::workout(&updated));
            *saved = Some(("health.card_updated", updated));
            Ok(text)
        }
        "hevy_routines" => Ok(format::routines_page(&hevy.routines(page_arg(input), 10).await?)),
        "hevy_routine" => Ok(format::routine(&hevy.routine(arg("routine_id")?).await?)),
        "hevy_create_routine" => {
            let mut routine = pick(input, &["title", "folder_id", "notes", "exercises"]);
            if routine.get("folder_id").is_none() {
                routine["folder_id"] = Value::Null;
            }
            let created = hevy.create_routine(&routine).await?;
            let text = format!("Created.\n{}", format::routine(&created));
            *saved = Some(("health.card_routine", created));
            Ok(text)
        }
        "hevy_update_routine" => {
            let routine = pick(input, &["title", "notes", "exercises"]);
            Ok(format!("Updated.\n{}", format::routine(&hevy.update_routine(arg("routine_id")?, &routine).await?)))
        }
        "hevy_routine_folders" => Ok(format::folders_page(&hevy.routine_folders(page_arg(input)).await?)),
        "hevy_create_routine_folder" => {
            let folder = hevy.create_routine_folder(arg("title")?).await?;
            Ok(format!("Created folder {} (id {}).", folder.get("title").and_then(|v| v.as_str()).unwrap_or(""), folder.get("id").map(|v| v.to_string()).unwrap_or_default()))
        }
        "hevy_find_exercises" => {
            let all = all_exercises(hevy, user_id).await?;
            Ok(format::find_exercises(&all, str_arg(input, "query"), str_arg(input, "muscle_group"), input.get("custom_only").and_then(|v| v.as_bool()).unwrap_or(false)))
        }
        "hevy_create_exercise" => {
            let exercise = pick(input, &["title", "exercise_type", "equipment_category", "muscle_group", "other_muscles"]);
            let created = hevy.create_exercise_template(&exercise).await?;
            forget_exercises(user_id);
            Ok(format!("Created custom exercise {} (id {}).", arg("title")?, created.get("id").map(|v| v.to_string().trim_matches('"').to_string()).unwrap_or_default()))
        }
        "hevy_exercise_history" => {
            let id = arg("exercise_template_id")?;
            let history = hevy.exercise_history(id, str_arg(input, "start_date"), str_arg(input, "end_date")).await?;
            Ok(format::exercise_history(&history))
        }
        "hevy_body_measurements" => Ok(format::measurements_page(&hevy.body_measurements(page_arg(input)).await?)),
        "hevy_log_body_measurement" => {
            let date = arg("date")?;
            let values: Map<String, Value> = MEASUREMENTS.iter().filter_map(|key| input.get(*key).filter(|v| v.is_number()).map(|v| (key.to_string(), v.clone()))).collect();
            if values.is_empty() {
                return Err(HevyError::Rejected("give at least one measurement".to_string()));
            }
            hevy.log_body_measurement(date, &values).await?;
            Ok(format!("Logged for {date}: {}.", format::measurement_values(&values)))
        }
        other => Err(HevyError::Rejected(format!("unknown tool {other}"))),
    }
}
