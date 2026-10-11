//! Nomi's voice: the `speak` tool turns words into speech with ElevenLabs and posts the
//! recording in the chat. An engine tool like the web tools: any agent gets it while it's on in
//! Admin → Tools and has a key, and only uses it when the person asks to hear something.
//!
//! The admin picks the voice, the model and the voice settings ElevenLabs offers (stability,
//! similarity, style, speaker boost, speed) and the audio format. Recordings are kept where chat
//! files go (`voice/<user>/<file>`) and served by Nomi at `/voice/<user>/<file>`.

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::content_block::ContentBlock;
use crate::tools;

pub const SPEAK_TOOL_NAME: &str = "speak";
pub const ENV_API_KEY: &str = "ELEVENLABS_API_KEY";
const SECRET_NAME: &str = "api_key";
pub const BASE_URL: &str = "https://api.elevenlabs.io";
/// Points Nomi at another ElevenLabs-compatible host (a proxy, or a stub in development).
pub const ENV_BASE_URL: &str = "ELEVENLABS_BASE_URL";

fn base_url() -> String {
    std::env::var(ENV_BASE_URL).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| BASE_URL.to_string())
}

/// ElevenLabs' "Rachel": there until the admin picks another.
pub const DEFAULT_VOICE_ID: &str = "21m00Tcm4TlvDq8ikWAM";
pub const DEFAULT_MODEL_ID: &str = "eleven_multilingual_v2";

/// Formats a browser plays everywhere (MP3), from small to best.
pub const OUTPUT_FORMATS: [&str; 5] = ["mp3_22050_32", "mp3_44100_64", "mp3_44100_96", "mp3_44100_128", "mp3_44100_192"];

/// The models ElevenLabs offers for speech, shown until the admin's key lists the account's own.
pub const KNOWN_MODELS: [(&str, &str); 4] = [
    ("eleven_multilingual_v2", "Multilingual v2"),
    ("eleven_turbo_v2_5", "Turbo v2.5"),
    ("eleven_flash_v2_5", "Flash v2.5"),
    ("eleven_v3", "Eleven v3"),
];

pub const MAX_CHARS_LIMIT: u32 = 5_000;

/// `speak`'s settings as saved (the key is kept apart, encrypted).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct VoiceSettings {
    pub voice_id: String,
    /// The voice's name, for the chat and the admin page.
    pub voice_name: Option<String>,
    pub model_id: String,
    /// 0..1: lower is more expressive, higher more even.
    pub stability: f64,
    /// 0..1: how closely it keeps to the original voice.
    pub similarity_boost: f64,
    /// 0..1: style exaggeration (slower and costlier above 0).
    pub style: f64,
    pub use_speaker_boost: bool,
    /// 0.7..1.2
    pub speed: f64,
    pub output_format: String,
    /// The most characters one `speak` call may say.
    pub max_chars: u32,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            voice_id: DEFAULT_VOICE_ID.to_string(),
            voice_name: Some("Rachel".to_string()),
            model_id: DEFAULT_MODEL_ID.to_string(),
            stability: 0.5,
            similarity_boost: 0.75,
            style: 0.0,
            use_speaker_boost: true,
            speed: 1.0,
            output_format: "mp3_44100_128".to_string(),
            max_chars: 2_500,
        }
    }
}

impl VoiceSettings {
    /// The same settings with every number in the range ElevenLabs accepts.
    pub fn clamped(mut self) -> Self {
        self.stability = self.stability.clamp(0.0, 1.0);
        self.similarity_boost = self.similarity_boost.clamp(0.0, 1.0);
        self.style = self.style.clamp(0.0, 1.0);
        self.speed = self.speed.clamp(0.7, 1.2);
        self.max_chars = self.max_chars.clamp(100, MAX_CHARS_LIMIT);
        if !OUTPUT_FORMATS.contains(&self.output_format.as_str()) {
            self.output_format = Self::default().output_format;
        }
        self
    }
}

/// Where the key comes from, for Admin → Tools.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KeySource {
    Saved,
    Env,
    Missing,
}

pub fn key_source(secrets: &HashMap<String, String>) -> KeySource {
    if secrets.get(SECRET_NAME).is_some_and(|k| !k.trim().is_empty()) {
        KeySource::Saved
    } else if std::env::var(ENV_API_KEY).is_ok_and(|v| !v.trim().is_empty()) {
        KeySource::Env
    } else {
        KeySource::Missing
    }
}

/// The saved key, else the server's `ELEVENLABS_API_KEY`.
pub fn api_key(secrets: &HashMap<String, String>) -> Option<String> {
    secrets
        .get(SECRET_NAME)
        .cloned()
        .filter(|k| !k.trim().is_empty())
        .or_else(|| std::env::var(ENV_API_KEY).ok().filter(|v| !v.trim().is_empty()))
}

pub fn secret_name() -> &'static str {
    SECRET_NAME
}

pub async fn voice_settings(conn: &mut PgConnection) -> (VoiceSettings, HashMap<String, String>) {
    let row = tools::load_row(conn, SPEAK_TOOL_NAME).await.unwrap_or_default();
    let settings: VoiceSettings = serde_json::from_value(row.config).unwrap_or_default();
    (settings.clamped(), tools::decrypt_secrets(row.secrets_encrypted.as_deref()))
}

pub async fn voice_settings_with(conn: &mut PgConnection, key: &[u8; 32]) -> Result<(VoiceSettings, HashMap<String, String>), sqlx::Error> {
    let row = tools::load_row(conn, SPEAK_TOOL_NAME).await?;
    let settings: VoiceSettings = serde_json::from_value(row.config).unwrap_or_default();
    Ok((settings.clamped(), tools::decrypt_secrets_with(key, row.secrets_encrypted.as_deref())))
}

/// Whether `speak` can work: it has a key.
pub async fn is_ready(conn: &mut PgConnection) -> bool {
    let (_, secrets) = voice_settings(conn).await;
    api_key(&secrets).is_some()
}

pub fn speak_tool_definition() -> nomi_llm::ToolDefinition {
    nomi_llm::ToolDefinition {
        name: SPEAK_TOOL_NAME.to_string(),
        description: crate::prompts::SPEAK_TOOL_DESCRIPTION.to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "The words to say, as they should be spoken." }
            },
            "required": ["text"]
        }),
    }
}

fn http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| reqwest::Client::builder().timeout(Duration::from_secs(90)).build().expect("reqwest client"))
}

/// Speech for `text` from ElevenLabs, as MP3 bytes.
pub async fn synthesize(settings: &VoiceSettings, api_key: &str, text: &str) -> Result<Vec<u8>, VoiceError> {
    synthesize_at(&base_url(), settings, api_key, text).await
}

pub async fn synthesize_at(base_url: &str, settings: &VoiceSettings, api_key: &str, text: &str) -> Result<Vec<u8>, VoiceError> {
    let url = format!(
        "{}/v1/text-to-speech/{}?output_format={}",
        base_url.trim_end_matches('/'),
        urlencoding_path(&settings.voice_id),
        settings.output_format
    );
    let body = json!({
        "text": text,
        "model_id": settings.model_id,
        "voice_settings": {
            "stability": settings.stability,
            "similarity_boost": settings.similarity_boost,
            "style": settings.style,
            "use_speaker_boost": settings.use_speaker_boost,
            "speed": settings.speed,
        }
    });
    let response = http()
        .post(url)
        .header("xi-api-key", api_key)
        .header("accept", "audio/mpeg")
        .json(&body)
        .send()
        .await
        .map_err(|e| VoiceError::new(VoiceErrorCode::Unreachable, e))?;
    let status = response.status();
    if !status.is_success() {
        let detail = response.text().await.unwrap_or_default();
        return Err(describe_failure(status.as_u16(), &detail));
    }
    let bytes = response.bytes().await.map_err(|e| VoiceError::new(VoiceErrorCode::BadResponse, e))?;
    if bytes.is_empty() {
        return Err(VoiceError { code: VoiceErrorCode::EmptyRecording, detail: None });
    }
    Ok(bytes.to_vec())
}

/// A voice id is letters and digits; anything else is escaped for the URL path.
fn urlencoding_path(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// What went wrong talking to ElevenLabs. `code` is stable for the admin screens to show in the
/// admin's language; `Display` says it in English for the agent and the logs. `detail` is
/// ElevenLabs' own explanation, when it gave one.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VoiceError {
    pub code: VoiceErrorCode,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceErrorCode {
    Unreachable,
    BadResponse,
    EmptyRecording,
    KeyRejected,
    MissingPermission,
    OutOfCredits,
    UnknownVoice,
    BadSettings,
    RateLimited,
    ProviderDown,
    Other,
}

impl VoiceError {
    fn new(code: VoiceErrorCode, detail: impl std::fmt::Display) -> Self {
        Self { code, detail: Some(detail.to_string()).filter(|d| !d.is_empty()) }
    }
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reason = match self.code {
            VoiceErrorCode::Unreachable => "couldn't reach ElevenLabs",
            VoiceErrorCode::BadResponse => "couldn't read what ElevenLabs sent back",
            VoiceErrorCode::EmptyRecording => "ElevenLabs sent back an empty recording",
            VoiceErrorCode::KeyRejected => "ElevenLabs rejected the API key",
            VoiceErrorCode::MissingPermission => "the ElevenLabs API key isn't allowed to do this; give it the permission in ElevenLabs → API keys",
            VoiceErrorCode::OutOfCredits => "the ElevenLabs account is out of credits",
            VoiceErrorCode::UnknownVoice => "ElevenLabs doesn't know that voice or model",
            VoiceErrorCode::BadSettings => "ElevenLabs couldn't use these settings",
            VoiceErrorCode::RateLimited => "ElevenLabs is limiting requests right now; try again in a moment",
            VoiceErrorCode::ProviderDown => "ElevenLabs is having trouble right now",
            VoiceErrorCode::Other => "ElevenLabs couldn't do it",
        };
        match &self.detail {
            Some(detail) => write!(f, "{reason} ({detail})"),
            None => f.write_str(reason),
        }
    }
}

/// Reads an ElevenLabs error response.
pub fn describe_failure(status: u16, body: &str) -> VoiceError {
    let parsed = serde_json::from_str::<Value>(body).ok();
    let detail = parsed.as_ref().and_then(|v| {
        v.pointer("/detail/message")
            .or_else(|| v.pointer("/detail"))
            .and_then(|d| d.as_str().map(str::to_string))
    });
    // A key restricted to some endpoints: valid, just not allowed this one.
    let missing_permission = parsed.as_ref().and_then(|v| v.pointer("/detail/status")).and_then(|s| s.as_str()) == Some("missing_permissions");
    let code = match status {
        401 if missing_permission => VoiceErrorCode::MissingPermission,
        401 => VoiceErrorCode::KeyRejected,
        402 => VoiceErrorCode::OutOfCredits,
        404 => VoiceErrorCode::UnknownVoice,
        422 => VoiceErrorCode::BadSettings,
        429 => VoiceErrorCode::RateLimited,
        s if s >= 500 => VoiceErrorCode::ProviderDown,
        _ => VoiceErrorCode::Other,
    };
    let detail = detail.filter(|d| !d.is_empty()).or_else(|| (code == VoiceErrorCode::Other).then(|| format!("HTTP {status}")));
    VoiceError { code, detail }
}

/// The app path a recording is served at.
pub fn voice_url(user_id: Uuid, file: &str) -> String {
    format!("/voice/{user_id}/{file}")
}

pub fn voice_key(user_id: Uuid, file: &str) -> String {
    format!("voice/{user_id}/{file}")
}

/// `speak`: says `input.text` and returns the tool's result and the card to post.
pub async fn speak(conn: &mut PgConnection, user_id: Uuid, input: &Value) -> Result<(String, ContentBlock), String> {
    let text = input.get("text").and_then(|v| v.as_str()).map(str::trim).filter(|t| !t.is_empty()).ok_or("text is required")?;
    let (settings, secrets) = voice_settings(conn).await;
    let key = api_key(&secrets).ok_or("speaking isn't set up: no ElevenLabs API key")?;
    if text.chars().count() > settings.max_chars as usize {
        return Err(format!("that's too long to say in one go (at most {} characters); say a shorter version", settings.max_chars));
    }
    let audio = synthesize(&settings, &key, text).await.map_err(|e| e.to_string())?;
    let file = format!("{}.mp3", Uuid::new_v4());
    nomi_storage::blob::attachment_store()
        .put_bytes(&voice_key(user_id, &file), audio, "audio/mpeg")
        .await
        .map_err(|e| format!("couldn't keep the recording: {e}"))?;
    let block = ContentBlock::Voice { url: voice_url(user_id, &file), text: text.to_string(), voice: settings.voice_name.clone() };
    Ok((format!("{} Words: {text}", crate::prompts::SPEAK_TOOL_DONE), block))
}

/// A voice in the admin's ElevenLabs account.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceOption {
    pub voice_id: String,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub preview_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelOption {
    pub model_id: String,
    pub name: String,
}

/// The voices and speech models the key's account can use.
pub async fn list_options(api_key: &str) -> Result<(Vec<VoiceOption>, Vec<ModelOption>), VoiceError> {
    list_options_at(&base_url(), api_key).await
}

pub async fn list_options_at(base_url: &str, api_key: &str) -> Result<(Vec<VoiceOption>, Vec<ModelOption>), VoiceError> {
    let base = base_url.trim_end_matches('/');
    let voices = http()
        .get(format!("{base}/v1/voices"))
        .header("xi-api-key", api_key)
        .send()
        .await
        .map_err(|e| VoiceError::new(VoiceErrorCode::Unreachable, e))?;
    if !voices.status().is_success() {
        let status = voices.status().as_u16();
        return Err(describe_failure(status, &voices.text().await.unwrap_or_default()));
    }
    let body: Value = voices.json().await.map_err(|e| VoiceError::new(VoiceErrorCode::BadResponse, e))?;
    let voices: Vec<VoiceOption> = body
        .get("voices")
        .and_then(|v| v.as_array())
        .map(|list| {
            list.iter()
                .filter_map(|v| {
                    let labels = v.get("labels");
                    let description = labels
                        .and_then(|l| l.as_object())
                        .map(|l| l.values().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", "))
                        .filter(|d| !d.is_empty());
                    Some(VoiceOption {
                        voice_id: v.get("voice_id")?.as_str()?.to_string(),
                        name: v.get("name")?.as_str()?.to_string(),
                        category: v.get("category").and_then(|c| c.as_str()).map(str::to_string),
                        description,
                        preview_url: v.get("preview_url").and_then(|p| p.as_str()).map(str::to_string),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Models are a nicety: the known list stands in when they can't be read.
    let models = match http().get(format!("{base}/v1/models")).header("xi-api-key", api_key).send().await {
        Ok(response) if response.status().is_success() => response
            .json::<Value>()
            .await
            .ok()
            .and_then(|v| v.as_array().cloned())
            .map(|list| {
                list.iter()
                    .filter(|m| m.get("can_do_text_to_speech").and_then(|c| c.as_bool()).unwrap_or(true))
                    .filter_map(|m| {
                        Some(ModelOption {
                            model_id: m.get("model_id")?.as_str()?.to_string(),
                            name: m.get("name").and_then(|n| n.as_str()).unwrap_or_default().to_string(),
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|list| !list.is_empty()),
        _ => None,
    }
    .unwrap_or_else(known_models);
    Ok((voices, models))
}

pub fn known_models() -> Vec<ModelOption> {
    KNOWN_MODELS.iter().map(|(id, name)| ModelOption { model_id: id.to_string(), name: name.to_string() }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_stay_in_the_ranges_elevenlabs_accepts() {
        let wild = VoiceSettings { stability: 3.0, similarity_boost: -1.0, speed: 2.0, max_chars: 50, output_format: "wav".into(), ..Default::default() };
        let clamped = wild.clamped();
        assert_eq!((clamped.stability, clamped.similarity_boost, clamped.speed), (1.0, 0.0, 1.2));
        assert_eq!(clamped.max_chars, 100);
        assert_eq!(clamped.output_format, "mp3_44100_128");
    }

    #[test]
    fn failures_read_as_what_to_do() {
        let rejected = describe_failure(401, r#"{"detail":{"status":"invalid_api_key","message":"Invalid API key"}}"#);
        assert_eq!(rejected.code, VoiceErrorCode::KeyRejected);
        assert_eq!(rejected.to_string(), "ElevenLabs rejected the API key (Invalid API key)");
        assert_eq!(describe_failure(402, "").to_string(), "the ElevenLabs account is out of credits");
        let scoped = describe_failure(401, r#"{"detail":{"status":"missing_permissions","message":"The API key you used is missing the permission voices_read to execute this operation."}}"#);
        assert_eq!(scoped.code, VoiceErrorCode::MissingPermission);
        assert_eq!(scoped.detail.as_deref(), Some("The API key you used is missing the permission voices_read to execute this operation."));
        assert_eq!(serde_json::to_value(&scoped).unwrap()["code"], "missing_permission");
        assert_eq!(describe_failure(418, "").detail.as_deref(), Some("HTTP 418"));
    }

    /// A one-request HTTP server: answers with `status` and `body`, hands back what it was sent.
    async fn one_shot(status: &str, content_type: &str, body: &'static [u8]) -> (String, tokio::task::JoinHandle<String>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let head = format!("HTTP/1.1 {status}\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n", body.len());
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut seen = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                let n = socket.read(&mut buf).await.unwrap();
                seen.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&seen).to_string();
                if let Some(end) = text.find("\r\n\r\n") {
                    let length = text[..end]
                        .lines()
                        .find_map(|l| l.to_ascii_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap()))
                        .unwrap_or(0);
                    if seen.len() >= end + 4 + length {
                        break;
                    }
                }
                if n == 0 {
                    break;
                }
            }
            socket.write_all(head.as_bytes()).await.unwrap();
            socket.write_all(body).await.unwrap();
            String::from_utf8_lossy(&seen).to_string()
        });
        (base, handle)
    }

    #[tokio::test]
    async fn synthesizing_sends_the_chosen_voice_and_settings() {
        let (base, server) = one_shot("200 OK", "audio/mpeg", b"ID3-audio").await;
        let settings = VoiceSettings { voice_id: "voice-abc".into(), model_id: "eleven_flash_v2_5".into(), speed: 1.1, ..Default::default() };
        let audio = synthesize_at(&base, &settings, "key-123", "Hello there").await.unwrap();
        assert_eq!(audio, b"ID3-audio");

        let request = server.await.unwrap();
        assert!(request.starts_with("POST /v1/text-to-speech/voice-abc?output_format=mp3_44100_128 "), "{request}");
        assert!(request.to_ascii_lowercase().contains("xi-api-key: key-123"));
        let body: Value = serde_json::from_str(&request[request.find("\r\n\r\n").unwrap() + 4..]).unwrap();
        assert_eq!(body["text"], "Hello there");
        assert_eq!(body["model_id"], "eleven_flash_v2_5");
        assert!((body["voice_settings"]["speed"].as_f64().unwrap() - 1.1).abs() < 1e-6);
    }

    #[tokio::test]
    async fn a_rejected_key_comes_back_as_a_readable_error() {
        let (base, _server) = one_shot("401 Unauthorized", "application/json", br#"{"detail":{"message":"Invalid API key"}}"#).await;
        let error = synthesize_at(&base, &VoiceSettings::default(), "bad", "Hi").await.unwrap_err();
        assert_eq!(error.code, VoiceErrorCode::KeyRejected);
        assert!(error.to_string().contains("rejected the API key"), "{error}");
    }
}
