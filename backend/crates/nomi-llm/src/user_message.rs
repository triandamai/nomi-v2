//! Plain-language explanations of provider failures, for the person in the chat. The raw error
//! (status, provider JSON) stays in the logs; the user gets what happened and what to do.

use crate::LlmError;
use nomi_i18n::Locale;

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

/// What a provider's answer says about the key it was called with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyHealth {
    Ok,
    /// The provider wants payment: the account's balance or credits ran out.
    BalanceEmpty,
    /// The provider refused the key.
    Rejected,
}

impl KeyHealth {
    pub fn as_str(self) -> &'static str {
        match self {
            KeyHealth::Ok => "ok",
            KeyHealth::BalanceEmpty => "balance_empty",
            KeyHealth::Rejected => "rejected",
        }
    }
}

/// HTTP 402.
const PAYMENT_REQUIRED: u16 = 402;
/// HTTP 401 and 403.
const UNAUTHORIZED: u16 = 401;
const FORBIDDEN: u16 = 403;

fn out_of_credits(status: Option<u16>, lower: &str) -> bool {
    status == Some(PAYMENT_REQUIRED) || lower.contains("insufficient_quota") || lower.contains("more credits") || lower.contains("billing")
        || lower.contains("insufficient balance") || lower.contains("credit balance is too low")
}

fn key_refused(status: Option<u16>, lower: &str) -> bool {
    matches!(status, Some(UNAUTHORIZED) | Some(FORBIDDEN)) || lower.contains("invalid api key") || lower.contains("invalid_api_key")
}

impl LlmError {
    /// What this failure says about the key the call was made with: its balance ran out, or the
    /// provider refused it. None for anything else (unreachable, rate limited, a bad request…),
    /// which says nothing about the key.
    pub fn key_health(&self) -> Option<KeyHealth> {
        let LlmError::ProviderError(text) = self else { return None };
        let (_, status) = provider_and_status(text);
        let lower = text.to_lowercase();
        if out_of_credits(status, &lower) {
            Some(KeyHealth::BalanceEmpty)
        } else if key_refused(status, &lower) {
            Some(KeyHealth::Rejected)
        } else {
            None
        }
    }

    /// What went wrong and what the user can do about it, in a sentence or two, in English.
    pub fn user_message(&self) -> String {
        self.user_message_in(Locale::En)
    }

    /// [`LlmError::user_message`] in the person's language.
    pub fn user_message_in(&self, locale: Locale) -> String {
        let text = match self {
            LlmError::Http(_) => return locale.t("error.llm_unreachable"),
            LlmError::QuotaExceeded => return locale.t("error.quota_used_up"),
            LlmError::ParseError(_) => return locale.t("error.llm_unreadable"),
            LlmError::ProviderError(text) => text,
        };
        let (provider, status) = provider_and_status(text);
        let who_capital = provider.map(str::to_string).unwrap_or_else(|| locale.t("error.llm_the_provider_capital"));
        let who = provider.map(str::to_string).unwrap_or_else(|| locale.t("error.llm_the_provider"));
        let account = match provider {
            Some(p) => locale.tf("error.llm_account", &[("provider", p)]),
            None => locale.t("error.llm_any_account"),
        };
        let lower = text.to_lowercase();

        if out_of_credits(status, &lower) {
            let your_provider = provider.map(str::to_string).unwrap_or_else(|| locale.t("error.llm_your_provider"));
            return locale.tf("error.llm_no_credits", &[("account", &account), ("provider", &your_provider)]);
        }
        if key_refused(status, &lower) {
            return locale.tf("error.llm_bad_key", &[("who", &who_capital)]);
        }
        if status == Some(429) {
            return locale.tf("error.llm_rate_limited", &[("who", &who_capital)]);
        }
        if lower.contains("context length") || lower.contains("context_length") || lower.contains("too many tokens") || lower.contains("prompt is too long") {
            return locale.t("error.llm_too_long");
        }
        if status == Some(404) && lower.contains("model") {
            return locale.tf("error.llm_no_model", &[("who", &who_capital)]);
        }
        if status.is_some_and(|s| s >= 500) || lower.contains("overloaded") {
            return locale.tf("error.llm_down", &[("who", &who_capital)]);
        }
        locale.tf("error.llm_other", &[("who", &who)])
    }
}

#[cfg(test)]
mod tests {
    use super::KeyHealth;
    use crate::LlmError;

    #[test]
    fn tells_an_empty_balance_from_a_refused_key() {
        let health = |text: &str| provider(text).key_health();
        assert_eq!(health(r#"openrouter returned 402 Payment Required: {"error":{"message":"more credits"}}"#), Some(KeyHealth::BalanceEmpty));
        assert_eq!(health(r#"openai returned 429 Too Many Requests: {"error":{"code":"insufficient_quota"}}"#), Some(KeyHealth::BalanceEmpty));
        assert_eq!(health("anthropic returned 400 Bad Request: Your credit balance is too low"), Some(KeyHealth::BalanceEmpty));
        assert_eq!(health("deepseek returned 402 Payment Required: Insufficient Balance"), Some(KeyHealth::BalanceEmpty));
        assert_eq!(health("anthropic returned 401 Unauthorized: invalid x-api-key"), Some(KeyHealth::Rejected));
        assert_eq!(health("gemini returned 403 Forbidden: denied"), Some(KeyHealth::Rejected));
        assert_eq!(health("openai returned 429 Too Many Requests: slow down"), None);
        assert_eq!(health("openai returned 503 Service Unavailable: overloaded"), None);
        assert_eq!(LlmError::QuotaExceeded.key_health(), None);
        assert_eq!(KeyHealth::BalanceEmpty.as_str(), "balance_empty");
    }

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
