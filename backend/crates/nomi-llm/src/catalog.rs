//! The providers people can connect (Providers, for admins and for themselves) and their models:
//! catalog/providers.json, generated from models.dev by scripts/gen-provider-catalog.mjs.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::ProviderKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Popular,
    Other,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    ApiKey,
    /// Signing in with an account. No provider uses it yet; the shape is ready for one.
    Oauth,
    /// A custom OpenAI-compatible endpoint: base URL, optional key and headers.
    CustomEndpoint,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelCost {
    /// USD per million tokens.
    pub input: Option<f64>,
    pub output: Option<f64>,
    pub cache_read: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,
    pub name: String,
    /// What the model reads: "text", "image", "pdf", "audio", "video".
    pub input: Vec<String>,
    pub cost: ModelCost,
    pub context: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: Category,
    pub recommended: bool,
    pub auth_type: AuthType,
    /// The client Nomi talks to it with.
    pub kind: ProviderKind,
    pub default_base_url: Option<String>,
    pub doc_url: Option<String>,
    /// Whether frontend/static/provider-logos has its logo.
    pub has_logo: bool,
    pub models: Vec<ModelMetadata>,
}

#[derive(Deserialize)]
struct RawCatalog {
    providers: Vec<RawProvider>,
}

#[derive(Deserialize)]
struct RawProvider {
    id: String,
    name: String,
    description: String,
    category: Category,
    recommended: bool,
    auth_type: AuthType,
    kind: String,
    default_base_url: Option<String>,
    doc_url: Option<String>,
    #[serde(default)]
    has_logo: bool,
    models: Vec<ModelMetadata>,
}

/// Every provider, popular ones first in their curated order, then A–Z, the custom endpoint last.
pub fn providers() -> &'static [ProviderDefinition] {
    static CATALOG: OnceLock<Vec<ProviderDefinition>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let raw: RawCatalog = serde_json::from_str(include_str!("../catalog/providers.json")).expect("catalog/providers.json is valid");
        raw.providers
            .into_iter()
            .map(|p| ProviderDefinition {
                kind: ProviderKind::parse(&p.kind).unwrap_or_else(|| panic!("catalog provider {} has unknown kind {}", p.id, p.kind)),
                id: p.id,
                name: p.name,
                description: p.description,
                category: p.category,
                recommended: p.recommended,
                auth_type: p.auth_type,
                default_base_url: p.default_base_url,
                doc_url: p.doc_url,
                has_logo: p.has_logo,
                models: p.models,
            })
            .collect()
    })
}

pub fn provider(id: &str) -> Option<&'static ProviderDefinition> {
    providers().iter().find(|p| p.id == id)
}

/// What a model is good at, as the model lists tag it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Fast,
    Reasoning,
    Vision,
    Code,
}

/// Name parts that mark a small, quick model.
const FAST_PARTS: &[&str] = &["flash", "mini", "nano", "lite", "haiku", "fast", "turbo", "instant", "small", "air", "spark"];
/// Name parts that mark a model that thinks before it answers.
const REASONING_PARTS: &[&str] = &["o1", "o3", "o4", "r1", "reasoner", "reasoning", "thinking", "think", "qwq", "opus", "pro"];
/// Model families that reason by default (matched on the start of the name).
const REASONING_FAMILIES: &[&str] = &["gpt-5", "gpt-6", "claude-sonnet", "claude-opus", "claude-fable", "gemini-2-5", "gemini-3", "gemini-4", "deepseek-v4"];
/// Name parts and families that mark a model made or tuned for code.
const CODE_PARTS: &[&str] = &["code", "coder", "codex", "codestral", "devstral"];
const CODE_FAMILIES: &[&str] = &["claude-sonnet", "claude-opus", "claude-fable", "gpt-5-3-codex"];

/// The models people should start with when they connect a provider, by the end of their id
/// (dots read as dashes, so "anthropic/claude-sonnet-5.5" matches "claude-sonnet-5-5"). Each is a
/// current model that answers well for its price; a provider none of them comes through
/// recommends nothing. Kept by hand next to catalog/providers.json.
const RECOMMENDED_MODELS: &[&str] = &[
    "claude-sonnet-5-5",
    "claude-haiku-5-5",
    "gpt-5-5",
    "gpt-5-4-mini",
    "gemini-3-8-flash",
    "gemini-3-1-pro-preview",
    "deepseek-v4-flash",
    "deepseek-v4-pro",
];

/// A model id as compared above: its last path segment, lowercased, dots and colons as dashes,
/// without a leading `~` (OpenRouter's "latest" aliases).
fn normalized(model_id: &str) -> String {
    let tail = model_id.rsplit('/').next().unwrap_or(model_id).trim_start_matches('~');
    tail.to_lowercase().replace(['.', ':', '_'], "-")
}

/// What `model_id` is good at: vision when it reads images (from the catalog's `input`), the
/// rest from what its name says.
pub fn capabilities(model_id: &str, input: &[String]) -> Vec<Capability> {
    let name = normalized(model_id);
    let parts: Vec<&str> = name.split('-').collect();
    let has_part = |list: &[&str]| parts.iter().any(|p| list.contains(p));
    let in_family = |list: &[&str]| list.iter().any(|f| name.starts_with(f));
    let mut out = Vec::new();
    if has_part(FAST_PARTS) {
        out.push(Capability::Fast);
    }
    if has_part(REASONING_PARTS) || in_family(REASONING_FAMILIES) {
        out.push(Capability::Reasoning);
    }
    if input.iter().any(|i| i == "image") {
        out.push(Capability::Vision);
    }
    if has_part(CODE_PARTS) || in_family(CODE_FAMILIES) {
        out.push(Capability::Code);
    }
    out
}

/// Whether `model_id` is one to start with (see [`RECOMMENDED_MODELS`]).
pub fn recommended(model_id: &str) -> bool {
    RECOMMENDED_MODELS.contains(&normalized(model_id).as_str())
}

/// A model's catalog entry under provider `provider_id`, when the catalog has it.
pub fn model(provider_id: &str, model_id: &str) -> Option<&'static ModelMetadata> {
    provider(provider_id)?.models.iter().find(|m| m.id == model_id)
}

/// What a model list shows about a model: how much it reads, what it's good at, and whether it's
/// one to start with. From the catalog entry under `provider_id` when there is one, else from
/// the id alone (no context).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelTraits {
    pub context: Option<u64>,
    pub capabilities: Vec<Capability>,
    pub recommended: bool,
}

pub fn traits(provider_id: &str, model_id: &str) -> ModelTraits {
    let entry = model(provider_id, model_id);
    let input = entry.map(|m| m.input.as_slice()).unwrap_or_default();
    ModelTraits { context: entry.and_then(|m| m.context), capabilities: capabilities(model_id, input), recommended: recommended(model_id) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text() -> Vec<String> {
        vec!["text".to_string()]
    }

    #[test]
    fn reads_what_a_model_is_good_at_from_its_name() {
        use Capability::*;
        assert_eq!(capabilities("gemini-3.8-flash", &["text".into(), "image".into()]), vec![Fast, Reasoning, Vision]);
        assert_eq!(capabilities("anthropic/claude-sonnet-5.5", &text()), vec![Reasoning, Code]);
        assert_eq!(capabilities("claude-haiku-5-5", &text()), vec![Fast]);
        assert_eq!(capabilities("o4-mini", &text()), vec![Fast, Reasoning]);
        assert_eq!(capabilities("mistralai/devstral-2512", &text()), vec![Code]);
        assert_eq!(capabilities("deepseek/deepseek-r1", &text()), vec![Reasoning]);
        assert_eq!(capabilities("gpt-4o", &text()), Vec::<Capability>::new());
    }

    #[test]
    fn recommends_the_listed_models_wherever_they_come_through() {
        assert!(recommended("claude-sonnet-5-5"));
        assert!(recommended("anthropic/claude-sonnet-5.5"));
        assert!(recommended("~deepseek/deepseek-v4-flash"));
        assert!(!recommended("claude-sonnet-4-5"));
        assert!(!recommended("gpt-5.5-pro"));
    }

    #[test]
    fn takes_context_and_inputs_from_the_catalog() {
        let first = &providers().iter().find(|p| p.id == "anthropic").expect("anthropic in the catalog").models[0];
        let found = traits("anthropic", &first.id);
        assert_eq!(found.context, first.context);
        let unknown = traits("nowhere", "gpt-4o-mini");
        assert_eq!(unknown.context, None);
        assert_eq!(unknown.capabilities, vec![Capability::Fast]);
    }
}
