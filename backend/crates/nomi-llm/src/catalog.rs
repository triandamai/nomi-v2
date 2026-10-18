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
                models: p.models,
            })
            .collect()
    })
}

pub fn provider(id: &str) -> Option<&'static ProviderDefinition> {
    providers().iter().find(|p| p.id == id)
}
