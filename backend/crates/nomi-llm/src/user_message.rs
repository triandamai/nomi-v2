//! Plain-language explanations of provider failures, for the person in the chat. The raw error
//! (status, provider JSON) stays in the logs; the user gets what happened and what to do.

use crate::LlmError;

/// The provider's display name and HTTP status, from a `"<provider> returned <status>: <body>"`
/// error (the format every provider module uses).
fn provider_and_status(text: &str) -> (Option<&'static str>, Option<u16>) {
    let (name, rest) = match text.split_once(" returned ") {
        Some(parts) => parts,
        None => return (None, None),
    };
    let provider = match name.trim() {
        "openrouter" => Some("OpenRouter"),
        "openai" => Some("OpenAI"),
        "anthropic" => Some("Anthropic"),
        "gemini" => Some("Gemini"),
        "deepseek" => Some("DeepSeek"),
        _ => None,
    };
    let status = rest.split_whitespace().next().and_then(|code| code.trim_end_matches(':').parse().ok());
    (provider, status)
}

impl LlmError {
    /// What went wrong and what the user can do about it, in a sentence or two.
    pub fn user_message(&self) -> String {
        let text = match self {
            LlmError::Http(_) => {
                return "I couldn't reach the AI provider. Check the server's connection, then send your message again.".to_string()
            }
            LlmError::ParseError(_) => {
                return "The AI provider sent back a reply I couldn't read. Send your message again; if it keeps happening, try another model in Settings.".to_string()
            }
            LlmError::ProviderError(text) => text,
        };
        let (provider, status) = provider_and_status(text);
        let who = provider.unwrap_or("The AI provider");
        let account = provider.map(|p| format!("your {p} account")).unwrap_or_else(|| "your AI provider account".to_string());
        let lower = text.to_lowercase();

        if status == Some(402) || lower.contains("insufficient_quota") || lower.contains("more credits") || lower.contains("billing") {
            return format!(
                "I couldn't reply because {account} is out of credits for this request. Add credits or raise the API key's limit with {}, then send your message again.",
                provider.unwrap_or("your provider")
            );
        }
        if matches!(status, Some(401) | Some(403)) || lower.contains("invalid api key") || lower.contains("invalid_api_key") {
            return format!("I couldn't reply because {who} rejected the API key. Check the key in Settings, then try again.");
        }
        if status == Some(429) {
            return format!("{who} is limiting how many requests I can make right now. Wait a minute, then try again.");
        }
        if lower.contains("context length") || lower.contains("context_length") || lower.contains("too many tokens") || lower.contains("prompt is too long") {
            return "This chat has grown too long for the model. Start a new chat to keep going.".to_string();
        }
        if status == Some(404) && lower.contains("model") {
            return format!("{who} couldn't find the selected model. Pick another model in Settings.");
        }
        if status.is_some_and(|s| s >= 500) || lower.contains("overloaded") {
            return format!("{who} is having trouble right now. Try again in a moment.");
        }
        format!("Something went wrong talking to {}. Send your message again.", provider.unwrap_or("the AI provider"))
    }
}

#[cfg(test)]
mod tests {
    use crate::LlmError;

    fn provider(text: &str) -> LlmError {
        LlmError::ProviderError(text.to_string())
    }

    #[test]
    fn out_of_credits_says_so_and_names_the_provider() {
        let err = provider(r#"openrouter returned 402 Payment Required: {"error":{"message":"This request requires more credits, or fewer max_tokens."}}"#);
        let message = err.user_message();
        assert!(message.contains("out of credits"), "{message}");
        assert!(message.contains("OpenRouter"), "{message}");
    }

    #[test]
    fn openai_quota_errors_count_as_out_of_credits() {
        let err = provider(r#"openai returned 429 Too Many Requests: {"error":{"code":"insufficient_quota"}}"#);
        assert!(err.user_message().contains("out of credits"));
    }

    #[test]
    fn each_common_failure_gets_its_own_explanation() {
        assert!(provider("anthropic returned 401 Unauthorized: invalid x-api-key").user_message().contains("rejected the API key"));
        assert!(provider("gemini returned 429 Too Many Requests: slow down").user_message().contains("Wait a minute"));
        assert!(provider("anthropic returned 529 <unknown status code>: overloaded").user_message().contains("having trouble"));
        assert!(provider("openai returned 400 Bad Request: maximum context length is 128000 tokens").user_message().contains("too long"));
        assert!(provider("deepseek returned 418 I'm a teapot: ?").user_message().contains("DeepSeek"));
    }
}
