# Multi-Provider LLM Abstraction

Date: 2026-07-26
Status: Approved (pending user review of this doc)
Parent plan: `plans/initial.md` (implements the "multi-model layer" described there)
Related: `docs/superpowers/specs/2026-07-26-orchestrator-turn-loop-design.md` (consumes this trait for the chitchat agent)

## Purpose

The parent plan calls for "a provider trait so any agent/turn can pick the model that fits (cheap/local for chitchat and intent classification, a stronger hosted model for financial reasoning or multi-step planning)." This document designs that trait and three concrete provider implementations (Anthropic, OpenAI, Gemini), built to support multi-turn conversation and tool calling from the start — even though the first consumer (chitchat) doesn't use tools — so a future tool-using sub-agent needs no trait changes.

None of these providers have an official Rust SDK, so every implementation is a direct HTTP client (`reqwest`) against the provider's own wire format, normalized to one shared shape.

## 1. Shared Types

```rust
// backend/src/llm/types.rs
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum LlmRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
    ToolResult { tool_use_id: String, content: String, is_error: bool },
}

#[derive(Debug, Clone)]
pub struct LlmMessage {
    pub role: LlmRole,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value, // JSON schema
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system: Option<String>,
    pub messages: Vec<LlmMessage>,
    pub tools: Vec<ToolDefinition>, // empty for chitchat
    pub max_tokens: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: Vec<ContentBlock>,
    pub stop_reason: StopReason,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("provider returned an error response: {0}")]
    ProviderError(String),
    #[error("failed to parse provider response: {0}")]
    ParseError(String),
}
```

## 2. The Trait

```rust
// backend/src/llm/mod.rs
use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;
}
```

## 3. Provider Implementations

Each provider struct holds its own `reqwest::Client`, API key, and model ID (all read from environment variables at construction time — `ANTHROPIC_API_KEY`/`OPENAI_API_KEY`/`GEMINI_API_KEY` — never hardcoded, matching the auth-claims-layer's existing convention). Each implements `LlmProvider::complete` by: (1) mapping `LlmRequest` to the provider's wire format, (2) issuing the HTTP request, (3) mapping the provider's response back to `LlmResponse`.

### Anthropic (`backend/src/llm/anthropic.rs`)

- `POST https://api.anthropic.com/v1/messages`, headers `x-api-key`, `anthropic-version: 2023-06-01`, `content-type: application/json`.
- Near 1:1 mapping: `system` maps directly; `messages` map to Anthropic's native `role`/`content` blocks (`text`, `tool_use`, `tool_result` — Anthropic's content-block model is what `ContentBlock` was modeled after); `tools` map to `[{name, description, input_schema}]`.
- Response: `content` (array of blocks, filter to `text`/`tool_use` types), `stop_reason` (`"end_turn"` → `EndTurn`, `"tool_use"` → `ToolUse`, `"max_tokens"` → `MaxTokens`, anything else → `Other`), `usage.input_tokens`/`usage.output_tokens`.

### OpenAI (`backend/src/llm/openai.rs`)

- `POST https://api.openai.com/v1/chat/completions`, header `Authorization: Bearer <key>`.
- `system` becomes a `{role: "system", content}` entry prepended to `messages`. `ToolUse` blocks on an assistant message map to that message's `tool_calls` array (`{id, type: "function", function: {name, arguments}}` — `arguments` is a **JSON string**, not an object, so `ContentBlock::ToolUse.input` must be serialized to a string here). `ToolResult` blocks become separate `{role: "tool", tool_call_id, content}` messages (OpenAI has no inline tool-result content block — it's a distinct message in the array, unlike Anthropic/Gemini).
- Response: `choices[0].message.content` (→ a `Text` block if non-null) and `choices[0].message.tool_calls` (→ `ToolUse` blocks, parsing each `function.arguments` JSON string back into a `Value`). `finish_reason` (`"stop"` → `EndTurn`, `"tool_calls"` → `ToolUse`, `"length"` → `MaxTokens`). `usage.prompt_tokens`/`usage.completion_tokens`.

### Gemini (`backend/src/llm/gemini.rs`)

- `POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key=<key>`.
- `system` maps to a separate `systemInstruction: {parts: [{text}]}` field (not part of `contents`). Roles are `user`/`model`, not `user`/`assistant` — `LlmRole::Assistant` maps to `"model"`. Content blocks map to `parts` entries: `Text` → `{text}`, `ToolUse` → `{functionCall: {name, args}}`, `ToolResult` → `{functionResponse: {name, response}}`. `tools` maps to `[{functionDeclarations: [{name, description, parameters}]}]`.
- Response: `candidates[0].content.parts` (mapped back per the same rules), `candidates[0].finishReason` (`"STOP"` → `EndTurn`, presence of a `functionCall` part → `ToolUse`, `"MAX_TOKENS"` → `MaxTokens`), `usageMetadata.promptTokenCount`/`usageMetadata.candidatesTokenCount`.

## 4. Model Selection

```rust
pub enum ProviderKind { Anthropic, OpenAi, Gemini }

pub struct ModelConfig {
    pub provider: ProviderKind,
    pub model_id: String,
}

pub fn build_provider(config: &ModelConfig, http_client: reqwest::Client) -> Box<dyn LlmProvider> {
    match config.provider {
        ProviderKind::Anthropic => Box::new(AnthropicProvider::new(http_client, config.model_id.clone())),
        ProviderKind::OpenAi => Box::new(OpenAiProvider::new(http_client, config.model_id.clone())),
        ProviderKind::Gemini => Box::new(GeminiProvider::new(http_client, config.model_id.clone())),
    }
}
```

Each agent (chitchat now; money/booking later) is configured with its own `ModelConfig`, resolved once at startup — this is the entire mechanism behind "any agent/turn can pick the model that fits." Swapping chitchat from Haiku to a different provider/model is a config change, not a code change.

## 5. Testing Approach

- **No real API calls, no API keys needed for tests.** Each provider's request-mapping and response-parsing logic is tested against a local mock HTTP server (`wiremock` crate): assert the exact request body shape sent for a given `LlmRequest` (including the tool-call and tool-result mapping cases), and assert a canned JSON response parses into the expected `LlmResponse`.
- Cover per provider: a plain text-only turn, a turn that returns a tool-call (`stop_reason: ToolUse`, one `ToolUse` block, correctly parsed `input`), a turn where the *request* includes a `ToolResult` block (proving the mapping into that provider's wire format, e.g. OpenAI's separate `role: "tool"` message), and an error response (non-2xx) mapping to `LlmError::ProviderError`.

## Out of Scope

- Streaming responses (all three providers support it; not needed until a chat UI wants token-by-token display).
- Prompt caching, extended thinking, or any other provider-specific advanced feature — this is a plain single-turn completion abstraction.
- Retry/backoff policy for transient provider errors — deferred to whichever plan first needs production hardening (noted as a parallel concern to the auth-claims-layer's own deferred hardening items).
