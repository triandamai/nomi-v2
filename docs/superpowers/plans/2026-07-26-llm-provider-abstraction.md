# Multi-Provider LLM Abstraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the `LlmProvider` trait and three concrete HTTP-backed implementations (Anthropic, OpenAI, Gemini) designed in `docs/superpowers/specs/2026-07-26-llm-provider-abstraction-design.md`, plus the model-selection factory that lets any future agent/turn pick whichever provider+model fits.

**Architecture:** A pure library addition to the existing `backend/` crate — no database dependency at all. Shared content-block types live in `backend/src/llm/types.rs`; each provider is its own file implementing the shared `LlmProvider` trait via direct HTTP calls (no official Rust SDK exists for any of these three providers). Every test runs against a local mock HTTP server (`wiremock`) — no real API keys, no network calls, no cost.

**Tech Stack:** `reqwest` (HTTP client, rustls to match the crate's existing TLS backend), `async-trait` (for `dyn LlmProvider` trait objects — native async-fn-in-traits isn't object-safe), `wiremock` (dev-dependency, HTTP mocking), building on the existing `serde`/`serde_json`/`thiserror` already in the crate.

## Global Constraints

- No API key is ever read from environment inside this plan's code — every provider constructor takes `api_key: String` as an explicit parameter, and every provider takes a `base_url: String` parameter (production code will pass the real API host; tests pass a `wiremock` server's URL). This mirrors the auth-claims-layer plan's "secret is always an explicit parameter" convention and is what makes these providers testable without live keys.
- `reqwest` is added with `default-features = false, features = ["json", "rustls-tls"]` — the crate already uses `sqlx`'s `runtime-tokio-rustls` feature; using `rustls` for `reqwest` too avoids linking two different TLS stacks.
- Every provider's wire-format mapping (request and response) is tested against `wiremock`, never a live provider — no test in this plan makes a real network call to `api.anthropic.com`, `api.openai.com`, or `generativelanguage.googleapis.com`.
- `ContentBlock`, `LlmMessage`, `LlmRequest`, `LlmResponse`, `StopReason`, `LlmError`, `LlmProvider` are the exact names and shapes from the design doc — every task after Task 1 depends on them unchanged.

---

## File Structure

- `backend/src/llm/types.rs` — `LlmRole`, `ContentBlock`, `LlmMessage`, `ToolDefinition`, `LlmRequest`, `StopReason`, `LlmResponse`, `LlmError`.
- `backend/src/llm/mod.rs` — the `LlmProvider` trait; re-exports the submodules and `types::*`.
- `backend/src/llm/anthropic.rs` — `AnthropicProvider`.
- `backend/src/llm/openai.rs` — `OpenAiProvider`.
- `backend/src/llm/gemini.rs` — `GeminiProvider`.
- `backend/src/llm/config.rs` — `ProviderKind`, `ModelConfig`, `build_provider`.
- `backend/tests/llm_anthropic.rs`, `backend/tests/llm_openai.rs`, `backend/tests/llm_gemini.rs`, `backend/tests/llm_config.rs` — one test file per task, all `wiremock`-based.

---

### Task 1: Shared Types & Trait Definition

**Files:**
- Create: `backend/src/llm/types.rs`
- Create: `backend/src/llm/mod.rs`
- Modify: `backend/Cargo.toml` (add `async-trait`, `reqwest`)
- Modify: `backend/src/lib.rs` (add `pub mod llm;`)

**Interfaces:**
- Consumes: nothing new.
- Produces: `LlmRole { User, Assistant }`, `ContentBlock { Text, ToolUse, ToolResult }`, `LlmMessage { role, content }`, `ToolDefinition { name, description, input_schema }`, `LlmRequest { system, messages, tools, max_tokens }`, `StopReason { EndTurn, ToolUse, MaxTokens, Other(String) }`, `LlmResponse { content, stop_reason, input_tokens, output_tokens }`, `LlmError { Http, ProviderError, ParseError }`, and the `LlmProvider` trait (`async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>`). Every later task implements or consumes these exact types.

- [ ] **Step 1: Add dependencies**

Add to `backend/Cargo.toml` under `[dependencies]`:

```toml
async-trait = "0.1"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

`reqwest` is needed now (not deferred to Task 2) because `LlmError::Http(#[from] reqwest::Error)` references `reqwest::Error` directly — this task won't compile without it, even though no provider uses `reqwest::Client` yet.

- [ ] **Step 2: Write the failing tests**

```rust
// backend/src/llm/types.rs (bottom of file, #[cfg(test)] mod tests)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_request_holds_the_fields_it_was_constructed_with() {
        let request = LlmRequest {
            system: Some("be helpful".to_string()),
            messages: vec![LlmMessage {
                role: LlmRole::User,
                content: vec![ContentBlock::Text { text: "hi".to_string() }],
            }],
            tools: vec![],
            max_tokens: 1024,
        };

        assert_eq!(request.system, Some("be helpful".to_string()));
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, LlmRole::User);
        assert_eq!(
            request.messages[0].content[0],
            ContentBlock::Text { text: "hi".to_string() }
        );
        assert_eq!(request.max_tokens, 1024);
    }

    #[test]
    fn stop_reason_variants_are_distinguishable() {
        assert_ne!(StopReason::EndTurn, StopReason::ToolUse);
        assert_eq!(
            StopReason::Other("weird".to_string()),
            StopReason::Other("weird".to_string())
        );
    }

    #[test]
    fn content_block_variants_carry_their_fields() {
        let tool_use = ContentBlock::ToolUse {
            id: "toolu_1".to_string(),
            name: "get_weather".to_string(),
            input: serde_json::json!({"city": "Paris"}),
        };
        match tool_use {
            ContentBlock::ToolUse { id, name, input } => {
                assert_eq!(id, "toolu_1");
                assert_eq!(name, "get_weather");
                assert_eq!(input, serde_json::json!({"city": "Paris"}));
            }
            _ => panic!("expected ToolUse variant"),
        }
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cd backend && cargo test --lib llm::types`
Expected: FAIL to compile — none of these types exist yet.

- [ ] **Step 4: Write the implementation**

```rust
// backend/src/llm/types.rs (above the tests module)
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub input_schema: Value,
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system: Option<String>,
    pub messages: Vec<LlmMessage>,
    pub tools: Vec<ToolDefinition>,
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

```rust
// backend/src/llm/mod.rs
pub mod types;

pub use types::*;

use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;
}
```

Update `backend/src/lib.rs` to add:

```rust
pub mod llm;
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cd backend && cargo test --lib llm::types`
Expected: PASS (3 tests)

- [ ] **Step 6: Commit**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/src/lib.rs backend/src/llm/mod.rs backend/src/llm/types.rs
git commit -m "feat: add shared LLM content-block types and LlmProvider trait"
```

---

### Task 2: Anthropic Provider

**Files:**
- Modify: `backend/Cargo.toml` (add `wiremock` dev-dependency — `reqwest` was already added in Task 1)
- Create: `backend/src/llm/anthropic.rs`
- Modify: `backend/src/llm/mod.rs` (add `pub mod anthropic;`)
- Test: `backend/tests/llm_anthropic.rs`

**Interfaces:**
- Consumes: `LlmProvider`, `LlmRequest`, `LlmResponse`, `ContentBlock`, `LlmMessage`, `LlmRole`, `StopReason`, `LlmError`, `ToolDefinition` (Task 1).
- Produces: `pub struct AnthropicProvider`, `AnthropicProvider::new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self`, `AnthropicProvider::default_base_url() -> String`. Task 5 (`build_provider`) constructs this type directly.

- [ ] **Step 1: Add dev-dependency**

Add to `backend/Cargo.toml` under `[dev-dependencies]`:

```toml
wiremock = "0.6"
```

- [ ] **Step 2: Write the failing tests**

```rust
// backend/tests/llm_anthropic.rs
use nomi_orchestrator::llm::anthropic::AnthropicProvider;
use nomi_orchestrator::llm::{ContentBlock, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn text_request() -> LlmRequest {
    LlmRequest {
        system: Some("be helpful".to_string()),
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::Text { text: "hello".to_string() }],
        }],
        tools: vec![],
        max_tokens: 100,
    }
}

#[tokio::test]
async fn text_only_reply_parses_into_a_text_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "test-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": [{"type": "text", "text": "hi there"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        })))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let response = provider.complete(text_request()).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi there".to_string() }]);
    assert_eq!(response.stop_reason, StopReason::EndTurn);
    assert_eq!(response.input_tokens, 10);
    assert_eq!(response.output_tokens, 5);
}

#[tokio::test]
async fn tool_use_reply_parses_into_a_tool_use_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": [{"type": "tool_use", "id": "toolu_1", "name": "get_weather", "input": {"city": "Paris"}}],
            "stop_reason": "tool_use",
            "usage": {"input_tokens": 20, "output_tokens": 8}
        })))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let mut request = text_request();
    request.tools = vec![ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get the weather".to_string(),
        input_schema: json!({"type": "object", "properties": {"city": {"type": "string"}}}),
    }];

    let response = provider.complete(request).await.unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::ToolUse {
            id: "toolu_1".to_string(),
            name: "get_weather".to_string(),
            input: json!({"city": "Paris"}),
        }]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

#[tokio::test]
async fn tool_result_in_request_is_mapped_into_anthropic_wire_format() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(body_partial_json(json!({
            "messages": [
                {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "toolu_1", "content": "sunny", "is_error": false}]}
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": [{"type": "text", "text": "It's sunny in Paris."}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 5, "output_tokens": 5}
        })))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let request = LlmRequest {
        system: None,
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: "toolu_1".to_string(),
                content: "sunny".to_string(),
                is_error: false,
            }],
        }],
        tools: vec![],
        max_tokens: 100,
    };

    let response = provider.complete(request).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "It's sunny in Paris.".to_string() }]);
}

#[tokio::test]
async fn non_success_status_becomes_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad request"))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let result = provider.complete(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cd backend && cargo test --test llm_anthropic`
Expected: FAIL to compile — `nomi_orchestrator::llm::anthropic` doesn't exist yet.

- [ ] **Step 4: Write the implementation**

```rust
// backend/src/llm/anthropic.rs
use async_trait::async_trait;
use serde_json::json;

use super::types::{ContentBlock, LlmError, LlmRequest, LlmResponse, LlmRole, StopReason};
use super::LlmProvider;

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://api.anthropic.com".to_string()
    }
}

fn role_to_str(role: &LlmRole) -> &'static str {
    match role {
        LlmRole::User => "user",
        LlmRole::Assistant => "assistant",
    }
}

fn content_block_to_json(block: &ContentBlock) -> serde_json::Value {
    match block {
        ContentBlock::Text { text } => json!({ "type": "text", "text": text }),
        ContentBlock::ToolUse { id, name, input } => {
            json!({ "type": "tool_use", "id": id, "name": name, "input": input })
        }
        ContentBlock::ToolResult { tool_use_id, content, is_error } => {
            json!({ "type": "tool_result", "tool_use_id": tool_use_id, "content": content, "is_error": is_error })
        }
    }
}

fn json_to_content_block(value: &serde_json::Value) -> Option<ContentBlock> {
    match value.get("type").and_then(|t| t.as_str())? {
        "text" => Some(ContentBlock::Text {
            text: value.get("text")?.as_str()?.to_string(),
        }),
        "tool_use" => Some(ContentBlock::ToolUse {
            id: value.get("id")?.as_str()?.to_string(),
            name: value.get("name")?.as_str()?.to_string(),
            input: value.get("input")?.clone(),
        }),
        _ => None,
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| {
                json!({
                    "role": role_to_str(&m.role),
                    "content": m.content.iter().map(content_block_to_json).collect::<Vec<_>>(),
                })
            })
            .collect();

        let tools: Vec<serde_json::Value> = request
            .tools
            .iter()
            .map(|t| json!({ "name": t.name, "description": t.description, "input_schema": t.input_schema }))
            .collect();

        let mut body = json!({
            "model": self.model,
            "max_tokens": request.max_tokens,
            "messages": messages,
        });
        if let Some(system) = &request.system {
            body["system"] = json!(system);
        }
        if !tools.is_empty() {
            body["tools"] = json!(tools);
        }

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!("anthropic returned {status}: {text}")));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let content: Vec<ContentBlock> = body
            .get("content")
            .and_then(|c| c.as_array())
            .ok_or_else(|| LlmError::ParseError("missing content array".to_string()))?
            .iter()
            .filter_map(json_to_content_block)
            .collect();

        let stop_reason = match body.get("stop_reason").and_then(|s| s.as_str()) {
            Some("end_turn") => StopReason::EndTurn,
            Some("tool_use") => StopReason::ToolUse,
            Some("max_tokens") => StopReason::MaxTokens,
            Some(other) => StopReason::Other(other.to_string()),
            None => StopReason::Other("unknown".to_string()),
        };

        let input_tokens = body
            .get("usage")
            .and_then(|u| u.get("input_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let output_tokens = body
            .get("usage")
            .and_then(|u| u.get("output_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        Ok(LlmResponse { content, stop_reason, input_tokens, output_tokens })
    }
}
```

Update `backend/src/llm/mod.rs` to add:

```rust
pub mod anthropic;
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cd backend && cargo test --test llm_anthropic`
Expected: PASS (4 tests)

- [ ] **Step 6: Commit**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/src/llm/mod.rs backend/src/llm/anthropic.rs backend/tests/llm_anthropic.rs
git commit -m "feat: add AnthropicProvider implementing LlmProvider"
```

---

### Task 3: OpenAI Provider

**Files:**
- Create: `backend/src/llm/openai.rs`
- Modify: `backend/src/llm/mod.rs` (add `pub mod openai;`)
- Test: `backend/tests/llm_openai.rs`

**Interfaces:**
- Consumes: `LlmProvider`, `LlmRequest`, `LlmResponse`, `ContentBlock`, `LlmMessage`, `LlmRole`, `StopReason`, `LlmError`, `ToolDefinition` (Task 1).
- Produces: `pub struct OpenAiProvider`, `OpenAiProvider::new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self`, `OpenAiProvider::default_base_url() -> String`. Task 5 constructs this type directly.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/llm_openai.rs
use nomi_orchestrator::llm::openai::OpenAiProvider;
use nomi_orchestrator::llm::{ContentBlock, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn text_request() -> LlmRequest {
    LlmRequest {
        system: Some("be helpful".to_string()),
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::Text { text: "hello".to_string() }],
        }],
        tools: vec![],
        max_tokens: 100,
    }
}

#[tokio::test]
async fn text_only_reply_parses_into_a_text_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(json!({
            "messages": [
                {"role": "system", "content": "be helpful"},
                {"role": "user", "content": "hello"}
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "hi there"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5}
        })))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
        server.uri(),
    );

    let response = provider.complete(text_request()).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi there".to_string() }]);
    assert_eq!(response.stop_reason, StopReason::EndTurn);
    assert_eq!(response.input_tokens, 10);
    assert_eq!(response.output_tokens, 5);
}

#[tokio::test]
async fn tool_call_reply_parses_into_a_tool_use_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{
                "message": {
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {"name": "get_weather", "arguments": "{\"city\":\"Paris\"}"}
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {"prompt_tokens": 20, "completion_tokens": 8}
        })))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
        server.uri(),
    );

    let mut request = text_request();
    request.tools = vec![ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get the weather".to_string(),
        input_schema: json!({"type": "object", "properties": {"city": {"type": "string"}}}),
    }];

    let response = provider.complete(request).await.unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::ToolUse {
            id: "call_1".to_string(),
            name: "get_weather".to_string(),
            input: json!({"city": "Paris"}),
        }]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

#[tokio::test]
async fn tool_result_in_request_becomes_a_separate_tool_role_message() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(json!({
            "messages": [
                {"role": "tool", "tool_call_id": "call_1", "content": "sunny"}
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "It's sunny in Paris."}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 5, "completion_tokens": 5}
        })))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
        server.uri(),
    );

    let request = LlmRequest {
        system: None,
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: "call_1".to_string(),
                content: "sunny".to_string(),
                is_error: false,
            }],
        }],
        tools: vec![],
        max_tokens: 100,
    };

    let response = provider.complete(request).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "It's sunny in Paris.".to_string() }]);
}

#[tokio::test]
async fn non_success_status_becomes_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
        server.uri(),
    );

    let result = provider.complete(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test llm_openai`
Expected: FAIL to compile — `nomi_orchestrator::llm::openai` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/llm/openai.rs
use async_trait::async_trait;
use serde_json::json;

use super::types::{ContentBlock, LlmError, LlmRequest, LlmResponse, LlmRole, StopReason};
use super::LlmProvider;

pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://api.openai.com".to_string()
    }
}

fn role_to_str(role: &LlmRole) -> &'static str {
    match role {
        LlmRole::User => "user",
        LlmRole::Assistant => "assistant",
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let mut messages: Vec<serde_json::Value> = Vec::new();
        if let Some(system) = &request.system {
            messages.push(json!({ "role": "system", "content": system }));
        }

        for m in &request.messages {
            let mut text_parts = Vec::new();
            let mut tool_calls = Vec::new();
            let mut tool_result_messages = Vec::new();

            for block in &m.content {
                match block {
                    ContentBlock::Text { text } => text_parts.push(text.clone()),
                    ContentBlock::ToolUse { id, name, input } => {
                        tool_calls.push(json!({
                            "id": id,
                            "type": "function",
                            "function": { "name": name, "arguments": input.to_string() }
                        }));
                    }
                    ContentBlock::ToolResult { tool_use_id, content, .. } => {
                        tool_result_messages.push(json!({
                            "role": "tool",
                            "tool_call_id": tool_use_id,
                            "content": content,
                        }));
                    }
                }
            }

            if !text_parts.is_empty() || !tool_calls.is_empty() {
                let mut msg = json!({ "role": role_to_str(&m.role) });
                msg["content"] = if !text_parts.is_empty() {
                    json!(text_parts.join(""))
                } else {
                    serde_json::Value::Null
                };
                if !tool_calls.is_empty() {
                    msg["tool_calls"] = json!(tool_calls);
                }
                messages.push(msg);
            }
            messages.extend(tool_result_messages);
        }

        let tools: Vec<serde_json::Value> = request
            .tools
            .iter()
            .map(|t| json!({
                "type": "function",
                "function": { "name": t.name, "description": t.description, "parameters": t.input_schema }
            }))
            .collect();

        let mut body = json!({
            "model": self.model,
            "max_tokens": request.max_tokens,
            "messages": messages,
        });
        if !tools.is_empty() {
            body["tools"] = json!(tools);
        }

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!("openai returned {status}: {text}")));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let choice = body
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .ok_or_else(|| LlmError::ParseError("missing choices[0]".to_string()))?;

        let message = choice
            .get("message")
            .ok_or_else(|| LlmError::ParseError("missing message".to_string()))?;

        let mut content = Vec::new();
        if let Some(text) = message.get("content").and_then(|c| c.as_str()) {
            content.push(ContentBlock::Text { text: text.to_string() });
        }
        if let Some(tool_calls) = message.get("tool_calls").and_then(|t| t.as_array()) {
            for tc in tool_calls {
                let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let function = tc
                    .get("function")
                    .ok_or_else(|| LlmError::ParseError("missing function".to_string()))?;
                let name = function.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let arguments_str = function.get("arguments").and_then(|v| v.as_str()).unwrap_or("{}");
                let input: serde_json::Value = serde_json::from_str(arguments_str)
                    .map_err(|e| LlmError::ParseError(format!("invalid tool_calls arguments JSON: {e}")))?;
                content.push(ContentBlock::ToolUse { id, name, input });
            }
        }

        let stop_reason = match choice.get("finish_reason").and_then(|s| s.as_str()) {
            Some("stop") => StopReason::EndTurn,
            Some("tool_calls") => StopReason::ToolUse,
            Some("length") => StopReason::MaxTokens,
            Some(other) => StopReason::Other(other.to_string()),
            None => StopReason::Other("unknown".to_string()),
        };

        let input_tokens = body.get("usage").and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let output_tokens = body.get("usage").and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        Ok(LlmResponse { content, stop_reason, input_tokens, output_tokens })
    }
}
```

Update `backend/src/llm/mod.rs` to add:

```rust
pub mod openai;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test llm_openai`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/llm/mod.rs backend/src/llm/openai.rs backend/tests/llm_openai.rs
git commit -m "feat: add OpenAiProvider implementing LlmProvider"
```

---

### Task 4: Gemini Provider

**Files:**
- Create: `backend/src/llm/gemini.rs`
- Modify: `backend/src/llm/mod.rs` (add `pub mod gemini;`)
- Test: `backend/tests/llm_gemini.rs`

**Interfaces:**
- Consumes: `LlmProvider`, `LlmRequest`, `LlmResponse`, `ContentBlock`, `LlmMessage`, `LlmRole`, `StopReason`, `LlmError`, `ToolDefinition` (Task 1).
- Produces: `pub struct GeminiProvider`, `GeminiProvider::new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self`, `GeminiProvider::default_base_url() -> String`. Task 5 constructs this type directly.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/llm_gemini.rs
use nomi_orchestrator::llm::gemini::GeminiProvider;
use nomi_orchestrator::llm::{ContentBlock, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn text_request() -> LlmRequest {
    LlmRequest {
        system: Some("be helpful".to_string()),
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::Text { text: "hello".to_string() }],
        }],
        tools: vec![],
        max_tokens: 100,
    }
}

#[tokio::test]
async fn text_only_reply_parses_into_a_text_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:generateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{
                "content": {"parts": [{"text": "hi there"}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {"promptTokenCount": 10, "candidatesTokenCount": 5}
        })))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let response = provider.complete(text_request()).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi there".to_string() }]);
    assert_eq!(response.stop_reason, StopReason::EndTurn);
    assert_eq!(response.input_tokens, 10);
    assert_eq!(response.output_tokens, 5);
}

#[tokio::test]
async fn function_call_reply_parses_into_a_tool_use_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:generateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{
                "content": {"parts": [{"functionCall": {"name": "get_weather", "args": {"city": "Paris"}}}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {"promptTokenCount": 20, "candidatesTokenCount": 8}
        })))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let mut request = text_request();
    request.tools = vec![ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get the weather".to_string(),
        input_schema: json!({"type": "object", "properties": {"city": {"type": "string"}}}),
    }];

    let response = provider.complete(request).await.unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::ToolUse {
            id: "get_weather".to_string(),
            name: "get_weather".to_string(),
            input: json!({"city": "Paris"}),
        }]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

#[tokio::test]
async fn tool_result_in_request_is_mapped_into_a_function_response_part() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:generateContent$"))
        .and(body_partial_json(json!({
            "contents": [
                {"role": "user", "parts": [{"functionResponse": {"name": "tool_result", "response": {"content": "sunny"}}}]}
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{
                "content": {"parts": [{"text": "It's sunny in Paris."}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {"promptTokenCount": 5, "candidatesTokenCount": 5}
        })))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let request = LlmRequest {
        system: None,
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: "get_weather".to_string(),
                content: "sunny".to_string(),
                is_error: false,
            }],
        }],
        tools: vec![],
        max_tokens: 100,
    };

    let response = provider.complete(request).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "It's sunny in Paris.".to_string() }]);
}

#[tokio::test]
async fn non_success_status_becomes_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:generateContent$"))
        .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let result = provider.complete(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test llm_gemini`
Expected: FAIL to compile — `nomi_orchestrator::llm::gemini` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/llm/gemini.rs
use async_trait::async_trait;
use serde_json::json;

use super::types::{ContentBlock, LlmError, LlmRequest, LlmResponse, LlmRole, StopReason};
use super::LlmProvider;

pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl GeminiProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://generativelanguage.googleapis.com".to_string()
    }
}

fn role_to_str(role: &LlmRole) -> &'static str {
    match role {
        LlmRole::User => "user",
        LlmRole::Assistant => "model",
    }
}

fn content_block_to_part(block: &ContentBlock) -> serde_json::Value {
    match block {
        ContentBlock::Text { text } => json!({ "text": text }),
        ContentBlock::ToolUse { name, input, .. } => {
            json!({ "functionCall": { "name": name, "args": input } })
        }
        // Gemini's functionResponse part is keyed by the function's name, not a
        // tool-call id (Gemini has no id concept for function calls) - see
        // json_to_content_block's matching synthesis of `id` from `name`.
        ContentBlock::ToolResult { content, .. } => {
            json!({ "functionResponse": { "name": "tool_result", "response": { "content": content } } })
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let contents: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| {
                json!({
                    "role": role_to_str(&m.role),
                    "parts": m.content.iter().map(content_block_to_part).collect::<Vec<_>>(),
                })
            })
            .collect();

        let mut body = json!({ "contents": contents });
        if let Some(system) = &request.system {
            body["systemInstruction"] = json!({ "parts": [{ "text": system }] });
        }
        if !request.tools.is_empty() {
            let declarations: Vec<serde_json::Value> = request
                .tools
                .iter()
                .map(|t| json!({ "name": t.name, "description": t.description, "parameters": t.input_schema }))
                .collect();
            body["tools"] = json!([{ "functionDeclarations": declarations }]);
        }
        body["generationConfig"] = json!({ "maxOutputTokens": request.max_tokens });

        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
        );

        let response = self.client.post(url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!("gemini returned {status}: {text}")));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let candidate = body
            .get("candidates")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .ok_or_else(|| LlmError::ParseError("missing candidates[0]".to_string()))?;

        let parts = candidate
            .get("content")
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array())
            .ok_or_else(|| LlmError::ParseError("missing content.parts".to_string()))?;

        let mut content = Vec::new();
        let mut saw_function_call = false;
        for part in parts {
            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                content.push(ContentBlock::Text { text: text.to_string() });
            } else if let Some(fc) = part.get("functionCall") {
                saw_function_call = true;
                let name = fc.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let args = fc.get("args").cloned().unwrap_or(serde_json::Value::Null);
                // Gemini has no tool-call id; synthesize one from the function name.
                content.push(ContentBlock::ToolUse { id: name.clone(), name, input: args });
            }
        }

        let finish_reason = candidate.get("finishReason").and_then(|s| s.as_str());
        let stop_reason = if saw_function_call {
            StopReason::ToolUse
        } else {
            match finish_reason {
                Some("STOP") => StopReason::EndTurn,
                Some("MAX_TOKENS") => StopReason::MaxTokens,
                Some(other) => StopReason::Other(other.to_string()),
                None => StopReason::Other("unknown".to_string()),
            }
        };

        let input_tokens = body.get("usageMetadata").and_then(|u| u.get("promptTokenCount")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let output_tokens = body.get("usageMetadata").and_then(|u| u.get("candidatesTokenCount")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        Ok(LlmResponse { content, stop_reason, input_tokens, output_tokens })
    }
}
```

Update `backend/src/llm/mod.rs` to add:

```rust
pub mod gemini;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test llm_gemini`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/llm/mod.rs backend/src/llm/gemini.rs backend/tests/llm_gemini.rs
git commit -m "feat: add GeminiProvider implementing LlmProvider"
```

---

### Task 5: Model Selection (`ProviderKind`, `ModelConfig`, `build_provider`)

**Files:**
- Create: `backend/src/llm/config.rs`
- Modify: `backend/src/llm/mod.rs` (add `pub mod config;` and re-export)
- Test: `backend/tests/llm_config.rs`

**Interfaces:**
- Consumes: `AnthropicProvider`, `OpenAiProvider`, `GeminiProvider` (Tasks 2-4), `LlmProvider` (Task 1).
- Produces: `pub enum ProviderKind { Anthropic, OpenAi, Gemini }`, `pub struct ModelConfig { provider: ProviderKind, model_id: String, api_key: String, base_url: Option<String> }`, `pub fn build_provider(config: ModelConfig, http_client: reqwest::Client) -> Box<dyn LlmProvider>`. This is the plan's capstone — the future turn-loop plan calls `build_provider` once per configured agent (chitchat now, money/booking later) to get the concrete provider to pass into `handle_inbound_message`.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/llm_config.rs
use nomi_orchestrator::llm::{build_provider, ContentBlock, LlmMessage, LlmRequest, LlmRole, ModelConfig, ProviderKind};
use serde_json::json;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn simple_request() -> LlmRequest {
    LlmRequest {
        system: None,
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::Text { text: "hi".to_string() }],
        }],
        tools: vec![],
        max_tokens: 50,
    }
}

#[tokio::test]
async fn build_provider_selects_anthropic() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": [{"type": "text", "text": "anthropic reply"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 1, "output_tokens": 1}
        })))
        .mount(&server)
        .await;

    let provider = build_provider(
        ModelConfig {
            provider: ProviderKind::Anthropic,
            model_id: "claude-haiku-4-5".to_string(),
            api_key: "key".to_string(),
            base_url: Some(server.uri()),
        },
        reqwest::Client::new(),
    );

    let response = provider.complete(simple_request()).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "anthropic reply".to_string() }]);
}

#[tokio::test]
async fn build_provider_selects_openai() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "openai reply"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1}
        })))
        .mount(&server)
        .await;

    let provider = build_provider(
        ModelConfig {
            provider: ProviderKind::OpenAi,
            model_id: "gpt-4o".to_string(),
            api_key: "key".to_string(),
            base_url: Some(server.uri()),
        },
        reqwest::Client::new(),
    );

    let response = provider.complete(simple_request()).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "openai reply".to_string() }]);
}

#[tokio::test]
async fn build_provider_selects_gemini() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:generateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{"content": {"parts": [{"text": "gemini reply"}]}, "finishReason": "STOP"}],
            "usageMetadata": {"promptTokenCount": 1, "candidatesTokenCount": 1}
        })))
        .mount(&server)
        .await;

    let provider = build_provider(
        ModelConfig {
            provider: ProviderKind::Gemini,
            model_id: "gemini-2.0-flash".to_string(),
            api_key: "key".to_string(),
            base_url: Some(server.uri()),
        },
        reqwest::Client::new(),
    );

    let response = provider.complete(simple_request()).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "gemini reply".to_string() }]);
}

#[tokio::test]
async fn build_provider_uses_the_real_default_base_url_when_none_given() {
    // No mock server involved — this only checks construction doesn't panic
    // and that omitting base_url falls through to each provider's own default,
    // without actually making a network call to the real API.
    let _anthropic = build_provider(
        ModelConfig {
            provider: ProviderKind::Anthropic,
            model_id: "claude-haiku-4-5".to_string(),
            api_key: "key".to_string(),
            base_url: None,
        },
        reqwest::Client::new(),
    );
    let _openai = build_provider(
        ModelConfig {
            provider: ProviderKind::OpenAi,
            model_id: "gpt-4o".to_string(),
            api_key: "key".to_string(),
            base_url: None,
        },
        reqwest::Client::new(),
    );
    let _gemini = build_provider(
        ModelConfig {
            provider: ProviderKind::Gemini,
            model_id: "gemini-2.0-flash".to_string(),
            api_key: "key".to_string(),
            base_url: None,
        },
        reqwest::Client::new(),
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test llm_config`
Expected: FAIL to compile — `nomi_orchestrator::llm::{build_provider, ModelConfig, ProviderKind}` don't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/llm/config.rs
use super::anthropic::AnthropicProvider;
use super::gemini::GeminiProvider;
use super::openai::OpenAiProvider;
use super::LlmProvider;

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderKind {
    Anthropic,
    OpenAi,
    Gemini,
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub provider: ProviderKind,
    pub model_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

pub fn build_provider(config: ModelConfig, http_client: reqwest::Client) -> Box<dyn LlmProvider> {
    match config.provider {
        ProviderKind::Anthropic => {
            let base_url = config.base_url.unwrap_or_else(AnthropicProvider::default_base_url);
            Box::new(AnthropicProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::OpenAi => {
            let base_url = config.base_url.unwrap_or_else(OpenAiProvider::default_base_url);
            Box::new(OpenAiProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::Gemini => {
            let base_url = config.base_url.unwrap_or_else(GeminiProvider::default_base_url);
            Box::new(GeminiProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
    }
}
```

Update `backend/src/llm/mod.rs` to add:

```rust
pub mod config;
pub use config::{build_provider, ModelConfig, ProviderKind};
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test llm_config`
Expected: PASS (4 tests)

- [ ] **Step 5: Run the full test suite once to confirm nothing regressed**

Run: `cd backend && cargo test`
Expected: PASS (every test from this plan and every prior plan — schema-migrations and auth-claims-layer)

- [ ] **Step 6: Commit**

```bash
git add backend/src/llm/mod.rs backend/src/llm/config.rs backend/tests/llm_config.rs
git commit -m "feat: add ProviderKind/ModelConfig/build_provider for model selection"
```

---

## Self-Review Notes

- **Spec coverage**: every section of `docs/superpowers/specs/2026-07-26-llm-provider-abstraction-design.md` has a task — shared types (§1 → Task 1), the trait (§2 → Task 1), Anthropic/OpenAI/Gemini (§3 → Tasks 2-4), model selection (§4 → Task 5), and the testing approach (§5 → every task's `wiremock`-based test file).
- **Deliberately deferred**: streaming, prompt caching/extended thinking, retry/backoff — all explicitly out of scope in the design doc, not built here.
- **Type consistency**: `LlmProvider`, `LlmRequest`, `LlmResponse`, `ContentBlock`, `StopReason`, `LlmError` are used identically (same field names, same variant names) across all four provider/config tasks that consume them from Task 1.
- **Known Gemini simplification, called out inline**: Gemini's wire format has no tool-call id, so `ToolUse.id` is synthesized from the function name (documented via a code comment in Task 4's implementation, not left implicit) — a real limitation if two identically-named tool calls occur in the same turn, acceptable for this plan's scope (chitchat uses no tools; a future tool-using sub-agent plan can revisit if it specifically targets Gemini).
