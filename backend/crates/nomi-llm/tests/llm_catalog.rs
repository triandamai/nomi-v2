use nomi_llm::catalog::{self, AuthType, Category};
use nomi_llm::ProviderKind;

#[test]
fn popular_providers_come_first_in_the_curated_order() {
    let ids: Vec<&str> = catalog::providers().iter().take(8).map(|p| p.id.as_str()).collect();
    assert_eq!(ids, ["opencode", "opencode-go", "anthropic", "openai", "google", "openrouter", "vercel", "deepseek"]);
    assert!(catalog::providers()[..8].iter().all(|p| p.category == Category::Popular));
    assert!(catalog::providers()[8..].iter().all(|p| p.category != Category::Popular));
}

#[test]
fn opencode_and_opencode_go_are_recommended() {
    let recommended: Vec<&str> = catalog::providers().iter().filter(|p| p.recommended).map(|p| p.id.as_str()).collect();
    assert_eq!(recommended, ["opencode", "opencode-go"]);
}

#[test]
fn native_providers_keep_their_own_runtime() {
    assert_eq!(catalog::provider("anthropic").unwrap().kind, ProviderKind::Anthropic);
    assert_eq!(catalog::provider("openai").unwrap().kind, ProviderKind::OpenAi);
    assert_eq!(catalog::provider("google").unwrap().kind, ProviderKind::Gemini);
    assert_eq!(catalog::provider("openrouter").unwrap().kind, ProviderKind::OpenRouter);
    assert_eq!(catalog::provider("deepseek").unwrap().kind, ProviderKind::DeepSeek);
    let opencode = catalog::provider("opencode").unwrap();
    assert_eq!(opencode.kind, ProviderKind::OpenAiCompatible);
    assert_eq!(opencode.default_base_url.as_deref(), Some("https://opencode.ai/inference/openai/v1"));
}

#[test]
fn every_offered_provider_can_run_and_has_models() {
    for provider in catalog::providers().iter().filter(|p| p.id != "custom") {
        assert_eq!(provider.auth_type, AuthType::ApiKey, "{}", provider.id);
        assert!(!provider.models.is_empty(), "{} has no models", provider.id);
        if provider.kind == ProviderKind::OpenAiCompatible {
            let url = provider.default_base_url.as_deref().unwrap_or_default();
            assert!(url.starts_with("https://"), "{} base url {url:?}", provider.id);
        }
    }
}

#[test]
fn subscription_only_providers_are_not_offered() {
    assert!(catalog::provider("github-copilot").is_none());
    assert!(catalog::provider("amazon-bedrock").is_none());
}

#[test]
fn a_custom_openai_compatible_endpoint_comes_last() {
    let custom = catalog::providers().last().unwrap();
    assert_eq!(custom.id, "custom");
    assert_eq!(custom.category, Category::Custom);
    assert_eq!(custom.auth_type, AuthType::CustomEndpoint);
    assert_eq!(custom.kind, ProviderKind::OpenAiCompatible);
    assert!(custom.default_base_url.is_none());
}

#[test]
fn models_carry_prices_and_inputs() {
    let haiku = catalog::provider("anthropic").unwrap().models.iter().find(|m| m.id == "claude-haiku-4-5").unwrap();
    assert!(haiku.cost.input.unwrap() > 0.0);
    assert!(haiku.input.iter().any(|i| i == "image"));
    assert!(haiku.context.unwrap() >= 200_000);
}

#[test]
fn provider_kinds_round_trip_through_their_stored_names() {
    for kind in [
        ProviderKind::Anthropic,
        ProviderKind::OpenAi,
        ProviderKind::OpenRouter,
        ProviderKind::Gemini,
        ProviderKind::DeepSeek,
        ProviderKind::OpenAiCompatible,
        ProviderKind::Fake,
    ] {
        assert_eq!(ProviderKind::parse(kind.as_str()), Some(kind));
    }
    assert_eq!(ProviderKind::OpenAiCompatible.as_str(), "openai_compatible");
    assert_eq!(ProviderKind::parse("copilot"), None);
}
