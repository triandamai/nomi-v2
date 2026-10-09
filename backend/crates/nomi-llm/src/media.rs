//! Files a model looks at itself (images, PDFs, audio, video): which kinds each model takes, and
//! [`MediaRouter`], which sends a request carrying files the person's own model can't open to the
//! files model an admin picked (Admin → Models) instead.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use crate::{ContentBlock, LlmError, LlmEventStream, LlmProvider, LlmRequest, ProviderKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Pdf,
    Audio,
    Video,
}

impl MediaKind {
    pub fn of(media_type: &str) -> Option<MediaKind> {
        let media_type = media_type.to_ascii_lowercase();
        if media_type.starts_with("image/") {
            Some(MediaKind::Image)
        } else if media_type == "application/pdf" {
            Some(MediaKind::Pdf)
        } else if media_type.starts_with("audio/") {
            Some(MediaKind::Audio)
        } else if media_type.starts_with("video/") {
            Some(MediaKind::Video)
        } else {
            None
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            MediaKind::Image => "image",
            MediaKind::Pdf => "pdf",
            MediaKind::Audio => "audio",
            MediaKind::Video => "video",
        }
    }

    pub fn parse(s: &str) -> Option<MediaKind> {
        match s {
            "image" => Some(MediaKind::Image),
            "pdf" => Some(MediaKind::Pdf),
            "audio" => Some(MediaKind::Audio),
            "video" => Some(MediaKind::Video),
            _ => None,
        }
    }
}

/// What one model can take in besides text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MediaSupport {
    pub image: bool,
    pub pdf: bool,
    pub audio: bool,
    pub video: bool,
    /// OpenAI-style APIs only take WAV and MP3 audio; Gemini takes any common format.
    pub audio_wav_mp3_only: bool,
}

impl MediaSupport {
    pub const NONE: MediaSupport = MediaSupport { image: false, pdf: false, audio: false, video: false, audio_wav_mp3_only: false };
    pub const ALL: MediaSupport = MediaSupport { image: true, pdf: true, audio: true, video: true, audio_wav_mp3_only: false };

    /// An admin's explicit list ("image", "pdf", "audio", "video") for a model on `provider`.
    pub fn from_list(provider: &ProviderKind, kinds: &[String]) -> MediaSupport {
        let has = |k: MediaKind| kinds.iter().any(|s| MediaKind::parse(s) == Some(k));
        MediaSupport {
            image: has(MediaKind::Image),
            pdf: has(MediaKind::Pdf),
            audio: has(MediaKind::Audio),
            video: has(MediaKind::Video),
            audio_wav_mp3_only: matches!(provider, ProviderKind::OpenAi | ProviderKind::OpenRouter),
        }
    }

    /// A best guess from the provider and model id, for models nobody listed explicitly.
    pub fn infer(provider: &ProviderKind, model_id: &str) -> MediaSupport {
        let id = model_id.to_ascii_lowercase();
        match provider {
            ProviderKind::Gemini | ProviderKind::Fake => MediaSupport::ALL,
            ProviderKind::Anthropic => anthropic_support(&id),
            ProviderKind::OpenAi => openai_support(&id),
            ProviderKind::OpenRouter => {
                let (vendor, model) = id.split_once('/').unwrap_or(("", id.as_str()));
                let mut support = match vendor {
                    "google" if model.starts_with("gemini") => MediaSupport::ALL,
                    "anthropic" => anthropic_support(model),
                    "openai" => openai_support(model),
                    _ if ["vision", "-vl", "pixtral", "llava", "gemma-3"].iter().any(|m| model.contains(m)) => {
                        MediaSupport { image: true, ..MediaSupport::NONE }
                    }
                    _ => MediaSupport::NONE,
                };
                // OpenRouter reads PDFs for any model (it turns them into text when the model can't).
                support.pdf = true;
                support.audio_wav_mp3_only = true;
                support
            }
            ProviderKind::DeepSeek => MediaSupport::NONE,
        }
    }

    pub fn as_list(&self) -> Vec<&'static str> {
        let mut list = Vec::new();
        for (on, kind) in [(self.image, MediaKind::Image), (self.pdf, MediaKind::Pdf), (self.audio, MediaKind::Audio), (self.video, MediaKind::Video)] {
            if on {
                list.push(kind.as_str());
            }
        }
        list
    }

    pub fn allows(&self, media_type: &str) -> bool {
        match MediaKind::of(media_type) {
            Some(MediaKind::Image) => self.image,
            Some(MediaKind::Pdf) => self.pdf,
            Some(MediaKind::Audio) => self.audio && (!self.audio_wav_mp3_only || openai_audio_format(media_type).is_some()),
            Some(MediaKind::Video) => self.video,
            None => false,
        }
    }

    fn allowed_count(&self, request: &LlmRequest) -> usize {
        media_types(request).filter(|t| self.allows(t)).count()
    }
}

fn anthropic_support(id: &str) -> MediaSupport {
    let old = id.contains("claude-2") || id.contains("instant");
    MediaSupport { image: !old, pdf: !old, ..MediaSupport::NONE }
}

fn openai_support(id: &str) -> MediaSupport {
    let vision = ["gpt-4o", "chatgpt-4o", "gpt-4.1", "gpt-4-turbo", "gpt-5", "o1", "o3", "o4"].iter().any(|p| id.starts_with(p))
        && !id.starts_with("o1-mini")
        && !id.starts_with("o3-mini");
    MediaSupport { image: vision, pdf: vision, audio: id.contains("audio"), video: false, audio_wav_mp3_only: true }
}

/// The `format` OpenAI-style `input_audio` parts take, when `media_type` is one of them.
pub fn openai_audio_format(media_type: &str) -> Option<&'static str> {
    match media_type.to_ascii_lowercase().as_str() {
        "audio/wav" | "audio/x-wav" | "audio/wave" => Some("wav"),
        "audio/mpeg" | "audio/mp3" => Some("mp3"),
        _ => None,
    }
}

/// What a model that can't open a file is told instead of seeing it.
pub fn unreadable_note(name: &str, media_type: &str) -> String {
    format!("[{name} ({media_type}) is attached, but this model can't open this kind of file]")
}

/// The part an OpenAI-style chat API (OpenAI, OpenRouter) takes for a file, or `None` when it
/// takes none for this kind.
pub fn openai_style_part(media_type: &str, data: &str, name: &str) -> Option<serde_json::Value> {
    match MediaKind::of(media_type)? {
        MediaKind::Image => Some(json!({ "type": "image_url", "image_url": { "url": format!("data:{media_type};base64,{data}") } })),
        MediaKind::Pdf => Some(json!({ "type": "file", "file": { "filename": name, "file_data": format!("data:application/pdf;base64,{data}") } })),
        MediaKind::Audio => openai_audio_format(media_type).map(|format| json!({ "type": "input_audio", "input_audio": { "data": data, "format": format } })),
        MediaKind::Video => None,
    }
}

fn media_types(request: &LlmRequest) -> impl Iterator<Item = &str> {
    request.messages.iter().flat_map(|m| m.content.iter()).filter_map(|b| match b {
        ContentBlock::Media { media_type, .. } => Some(media_type.as_str()),
        _ => None,
    })
}

/// Replaces every file `support` can't take with a note saying it's there.
pub fn strip_unsupported(mut request: LlmRequest, support: &MediaSupport) -> LlmRequest {
    for message in &mut request.messages {
        for block in &mut message.content {
            if let ContentBlock::Media { media_type, name, .. } = block {
                if !support.allows(media_type) {
                    *block = ContentBlock::Text { text: unreadable_note(name, media_type) };
                }
            }
        }
    }
    request
}

/// The person's model, plus the files model for requests carrying files it can't open. A request
/// without files, or with files the person's model takes, goes to the person's model unchanged.
pub struct MediaRouter {
    primary: Arc<dyn LlmProvider>,
    primary_support: MediaSupport,
    files: Option<(Arc<dyn LlmProvider>, MediaSupport)>,
}

impl MediaRouter {
    pub fn new(primary: Arc<dyn LlmProvider>, primary_support: MediaSupport, files: Option<(Arc<dyn LlmProvider>, MediaSupport)>) -> Self {
        MediaRouter { primary, primary_support, files }
    }

    /// Which model a request goes to (`true` for the files model), and what it can open.
    fn choose(&self, request: &LlmRequest) -> (bool, MediaSupport) {
        let total = media_types(request).count();
        let primary = self.primary_support.allowed_count(request);
        if total == primary {
            return (false, self.primary_support);
        }
        match &self.files {
            Some((_, support)) if support.allowed_count(request) > primary => (true, *support),
            _ => (false, self.primary_support),
        }
    }
}

#[async_trait]
impl LlmProvider for MediaRouter {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let (use_files, support) = self.choose(&request);
        let request = strip_unsupported(request, &support);
        match (&self.files, use_files) {
            (Some((files, _)), true) => files.complete_stream(request).await,
            _ => self.primary.complete_stream(request).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::{response_to_stream, LlmMessage, LlmResponse, LlmRole, ReasoningEffort, StopReason};

    struct Recorder {
        name: &'static str,
        seen: Mutex<Vec<LlmRequest>>,
    }

    #[async_trait]
    impl LlmProvider for Recorder {
        async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
            self.seen.lock().unwrap().push(request);
            Ok(response_to_stream(LlmResponse {
                content: vec![ContentBlock::Text { text: self.name.to_string() }],
                stop_reason: StopReason::EndTurn,
                input_tokens: 0,
                output_tokens: 0,
            }))
        }
    }

    fn request_with(media_type: &str) -> LlmRequest {
        LlmRequest {
            system: None,
            messages: vec![LlmMessage {
                role: LlmRole::User,
                content: vec![
                    ContentBlock::Text { text: "what's this?".into() },
                    ContentBlock::Media { media_type: media_type.into(), data: "AAAA".into(), name: "f".into() },
                ],
            }],
            tools: vec![],
            max_tokens: 10,
            enable_reasoning: false,
            reasoning_effort: ReasoningEffort::default(),
        }
    }

    fn recorder(name: &'static str) -> Arc<Recorder> {
        Arc::new(Recorder { name, seen: Mutex::new(Vec::new()) })
    }

    async fn reply(router: &MediaRouter, request: LlmRequest) -> String {
        let response = crate::complete(router, request).await.unwrap();
        match &response.content[0] {
            ContentBlock::Text { text } => text.clone(),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[tokio::test]
    async fn files_the_persons_model_cant_open_go_to_the_files_model() {
        let deepseek = recorder("primary");
        let gemini = recorder("files");
        let router = MediaRouter::new(deepseek.clone(), MediaSupport::infer(&ProviderKind::DeepSeek, "deepseek-chat"), Some((gemini.clone(), MediaSupport::ALL)));

        assert_eq!(reply(&router, request_with("image/jpeg")).await, "files");
        let mut text_only = request_with("image/jpeg");
        text_only.messages[0].content.truncate(1);
        assert_eq!(reply(&router, text_only).await, "primary");
    }

    #[tokio::test]
    async fn a_model_that_can_open_the_file_keeps_the_request() {
        let claude = recorder("primary");
        let gemini = recorder("files");
        let router = MediaRouter::new(claude.clone(), MediaSupport::infer(&ProviderKind::Anthropic, "claude-sonnet-4-5"), Some((gemini, MediaSupport::ALL)));
        assert_eq!(reply(&router, request_with("image/png")).await, "primary");
        assert_eq!(reply(&router, request_with("audio/webm")).await, "files");
    }

    #[tokio::test]
    async fn without_a_files_model_unreadable_files_become_a_note() {
        let deepseek = recorder("primary");
        let router = MediaRouter::new(deepseek.clone(), MediaSupport::NONE, None);
        assert_eq!(reply(&router, request_with("video/mp4")).await, "primary");
        let seen = deepseek.seen.lock().unwrap();
        assert!(matches!(&seen[0].messages[0].content[1], ContentBlock::Text { text } if text.contains("can't open")));
    }

    #[test]
    fn guesses_what_well_known_models_take() {
        assert!(MediaSupport::infer(&ProviderKind::OpenAi, "gpt-4o-mini").image);
        assert!(!MediaSupport::infer(&ProviderKind::OpenAi, "o3-mini").image);
        assert!(!MediaSupport::infer(&ProviderKind::OpenAi, "gpt-4o").allows("audio/webm"));
        assert!(MediaSupport::infer(&ProviderKind::OpenAi, "gpt-4o-audio-preview").allows("audio/mpeg"));
        assert!(MediaSupport::infer(&ProviderKind::OpenRouter, "google/gemini-2.5-flash").video);
        assert!(MediaSupport::infer(&ProviderKind::OpenRouter, "anthropic/claude-sonnet-4").image);
        assert!(!MediaSupport::infer(&ProviderKind::DeepSeek, "deepseek-chat").image);
        let listed = MediaSupport::from_list(&ProviderKind::Gemini, &["image".into(), "video".into()]);
        assert!(listed.image && listed.video && !listed.pdf);
    }
}

#[cfg(test)]
mod encoding_tests {
    use crate::anthropic::AnthropicProvider;
    use crate::gemini::GeminiProvider;
    use crate::openai::OpenAiProvider;
    use crate::{ContentBlock, LlmMessage, LlmRequest, LlmRole, ReasoningEffort};

    fn request(media_type: &str) -> LlmRequest {
        LlmRequest {
            system: None,
            messages: vec![LlmMessage {
                role: LlmRole::User,
                content: vec![
                    ContentBlock::Text { text: "what's this?".into() },
                    ContentBlock::Media { media_type: media_type.into(), data: "QUJD".into(), name: "f.bin".into() },
                ],
            }],
            tools: vec![],
            max_tokens: 10,
            enable_reasoning: false,
            reasoning_effort: ReasoningEffort::default(),
        }
    }

    #[test]
    fn each_provider_sends_files_its_own_way() {
        let client = reqwest::Client::new();
        let claude = AnthropicProvider::new(client.clone(), "k".into(), "claude".into(), "http://x".into());
        let body = claude.body_for_test(&request("image/png"));
        assert_eq!(body["messages"][0]["content"][1]["type"], "image");
        assert_eq!(body["messages"][0]["content"][1]["source"]["data"], "QUJD");
        let body = claude.body_for_test(&request("video/mp4"));
        assert!(body["messages"][0]["content"][1]["text"].as_str().unwrap().contains("can't open"));

        let gpt = OpenAiProvider::new(client.clone(), "k".into(), "gpt-4o".into(), "http://x".into());
        let body = gpt.body_for_test(&request("image/jpeg"));
        let parts = &body["messages"][0]["content"];
        assert_eq!(parts[0]["text"], "what's this?");
        assert_eq!(parts[1]["image_url"]["url"], "data:image/jpeg;base64,QUJD");
        let body = gpt.body_for_test(&request("audio/mpeg"));
        assert_eq!(body["messages"][0]["content"][1]["input_audio"]["format"], "mp3");

        let gemini = GeminiProvider::new(client, "k".into(), "gemini-2.5-flash".into(), "http://x".into());
        let body = gemini.body_for_test(&request("video/mp4"));
        assert_eq!(body["contents"][0]["parts"][1]["inline_data"]["mime_type"], "video/mp4");
    }
}
