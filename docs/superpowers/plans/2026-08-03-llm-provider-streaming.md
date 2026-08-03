# LLM Provider Streaming Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace `LlmProvider::complete()` (request/response only) with `complete_stream()` (token-by-token via SSE) across all three real providers, with a shared `collect_stream()` helper so non-incremental consumers see no behavior change.

**Architecture:** A new `complete_stream()` trait method gets a *default* implementation (delegates to the still-present `complete()`) so every task except the last keeps the whole crate compiling and every existing test green. Tasks 2-4 give Anthropic/OpenAI/Gemini real SSE-based overrides, purely additive. Task 5 is the atomic cutover: fake providers get real (non-delegating) overrides, `complete()` is deleted from the trait and all five implementors, and the four turn-loop call sites migrate to a new `llm::complete()` free-function helper built on `complete_stream()` + `collect_stream()`.

**Tech Stack:** Rust/axum backend (existing), `eventsource-stream` (SSE parsing), `async-stream` (`try_stream!` macro for readable stateful stream generators), `futures-core`/`futures-util` (new), reqwest's `"stream"` feature (newly enabled).

## Global Constraints

- Every task must leave `cargo build` and `cargo test` green — no task may leave the crate in a non-compiling state.
- Tasks 2-4 temporarily duplicate nothing: each provider's request-body-building logic (messages/tools/system JSON mapping) is extracted into a private helper shared by both `complete()` and `complete_stream()`, not copy-pasted. This is a real, immediate DRY improvement, not deferred cleanup.
- `StreamEvent` index values are assigned by each provider in the order content blocks first appear; every block that gets a `ContentBlockStart` MUST get a matching `ContentBlockDone` before the stream's `Done` event (providers are responsible for this invariant — `collect_stream()` relies on it and does not force-close open blocks).
- A non-2xx HTTP response is still returned as `Err` directly from `complete_stream()` itself (checked before the body is wrapped as a stream) — never as a stream that yields a single `Err` item. A stream is only returned once headers indicate success.
- Gemini's `ToolUse.id` is synthesized from the function name (matching the existing `complete()` convention in `gemini.rs` — `id: name.clone()`), not a random UUID.

---

## Task 1: Shared streaming types, trait default, `collect_stream()`, and dependencies

**Files:**
- Modify: `backend/Cargo.toml`
- Modify: `backend/src/llm/types.rs`
- Modify: `backend/src/llm/mod.rs`
- Test: `backend/tests/llm_streaming.rs` (new)

**Interfaces:**
- Produces: `pub enum PartialBlock { Text, ToolUse { id: String, name: String } }` (derives `Debug, Clone, PartialEq`).
- Produces: `pub enum StreamEvent { ContentBlockStart { index: usize, block: PartialBlock }, TextDelta { index: usize, text: String }, ToolInputDelta { index: usize, partial_json: String }, ContentBlockDone { index: usize }, Done { stop_reason: StopReason, input_tokens: u32, output_tokens: u32 } }` (derives `Debug, PartialEq`).
- Produces: `pub type LlmEventStream = Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>`.
- Produces: `LlmProvider::complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError>` — new trait method with a default body delegating to `complete()`. `complete()` itself is unchanged.
- Produces: `pub fn response_to_stream(response: LlmResponse) -> LlmEventStream` (fully `pub` — used by the trait default here, and later by Task 5's fake-provider overrides, including from the separate `tests/` integration-test crate).
- Produces: `pub async fn collect_stream(stream: LlmEventStream) -> Result<LlmResponse, LlmError>`.
- Produces: `pub async fn complete(provider: &dyn LlmProvider, request: LlmRequest) -> Result<LlmResponse, LlmError>` (free function — `provider.complete_stream(request).await` then `collect_stream`).

- [ ] **Step 1: Add dependencies**

```bash
cd backend
cargo add futures-core
cargo add futures-util
cargo add eventsource-stream
cargo add async-stream
cargo add reqwest --features stream
```

Confirm `backend/Cargo.toml`'s `reqwest` line now reads:
```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }
```

- [ ] **Step 2: Write the failing tests**

Create `backend/tests/llm_streaming.rs`:

```rust
mod support;

use futures_util::stream;
use nomi_orchestrator::llm::{
    collect_stream, complete, ContentBlock, LlmError, LlmEventStream, LlmMessage, LlmRequest, LlmRole,
    LlmResponse, PartialBlock, StopReason, StreamEvent,
};
use support::FakeLlmProvider;

fn stream_of(events: Vec<Result<StreamEvent, LlmError>>) -> LlmEventStream {
    Box::pin(stream::iter(events))
}

#[tokio::test]
async fn collect_stream_assembles_a_single_text_block() {
    let events = stream_of(vec![
        Ok(StreamEvent::ContentBlockStart { index: 0, block: PartialBlock::Text }),
        Ok(StreamEvent::TextDelta { index: 0, text: "Hel".to_string() }),
        Ok(StreamEvent::TextDelta { index: 0, text: "lo".to_string() }),
        Ok(StreamEvent::ContentBlockDone { index: 0 }),
        Ok(StreamEvent::Done { stop_reason: StopReason::EndTurn, input_tokens: 3, output_tokens: 2 }),
    ]);

    let response = collect_stream(events).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "Hello".to_string() }]);
    assert_eq!(response.stop_reason, StopReason::EndTurn);
    assert_eq!(response.input_tokens, 3);
    assert_eq!(response.output_tokens, 2);
}

#[tokio::test]
async fn collect_stream_assembles_interleaved_text_and_tool_use_blocks_in_index_order() {
    let events = stream_of(vec![
        Ok(StreamEvent::ContentBlockStart { index: 0, block: PartialBlock::Text }),
        Ok(StreamEvent::TextDelta { index: 0, text: "Let me check. ".to_string() }),
        Ok(StreamEvent::ContentBlockDone { index: 0 }),
        Ok(StreamEvent::ContentBlockStart {
            index: 1,
            block: PartialBlock::ToolUse { id: "toolu_1".to_string(), name: "get_weather".to_string() },
        }),
        Ok(StreamEvent::ToolInputDelta { index: 1, partial_json: "{\"city\":".to_string() }),
        Ok(StreamEvent::ToolInputDelta { index: 1, partial_json: "\"Paris\"}".to_string() }),
        Ok(StreamEvent::ContentBlockDone { index: 1 }),
        Ok(StreamEvent::Done { stop_reason: StopReason::ToolUse, input_tokens: 10, output_tokens: 4 }),
    ]);

    let response = collect_stream(events).await.unwrap();

    assert_eq!(
        response.content,
        vec![
            ContentBlock::Text { text: "Let me check. ".to_string() },
            ContentBlock::ToolUse {
                id: "toolu_1".to_string(),
                name: "get_weather".to_string(),
                input: serde_json::json!({"city": "Paris"}),
            },
        ]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

#[tokio::test]
async fn collect_stream_rejects_invalid_tool_input_json() {
    let events = stream_of(vec![
        Ok(StreamEvent::ContentBlockStart {
            index: 0,
            block: PartialBlock::ToolUse { id: "t1".to_string(), name: "x".to_string() },
        }),
        Ok(StreamEvent::ToolInputDelta { index: 0, partial_json: "not json".to_string() }),
        Ok(StreamEvent::ContentBlockDone { index: 0 }),
        Ok(StreamEvent::Done { stop_reason: StopReason::ToolUse, input_tokens: 1, output_tokens: 1 }),
    ]);

    let result = collect_stream(events).await;
    assert!(matches!(result, Err(LlmError::ParseError(_))));
}

#[tokio::test]
async fn collect_stream_errors_if_the_stream_ends_without_a_done_event() {
    let events = stream_of(vec![
        Ok(StreamEvent::ContentBlockStart { index: 0, block: PartialBlock::Text }),
        Ok(StreamEvent::TextDelta { index: 0, text: "partial".to_string() }),
    ]);

    let result = collect_stream(events).await;
    assert!(matches!(result, Err(LlmError::ParseError(_))));
}

#[tokio::test]
async fn collect_stream_propagates_a_mid_stream_error() {
    let events = stream_of(vec![
        Ok(StreamEvent::ContentBlockStart { index: 0, block: PartialBlock::Text }),
        Err(LlmError::ProviderError("dropped connection".to_string())),
    ]);

    let result = collect_stream(events).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}

#[tokio::test]
async fn the_default_complete_stream_impl_wraps_a_non_streaming_providers_complete_call() {
    let provider = FakeLlmProvider::success(LlmResponse {
        content: vec![ContentBlock::Text { text: "hi".to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    });

    let request = LlmRequest {
        system: None,
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: "hey".to_string() }] }],
        tools: vec![],
        max_tokens: 10,
    };

    let response = complete(&provider, request).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi".to_string() }]);
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cd backend && cargo test --test llm_streaming`
Expected: FAIL to compile — `collect_stream`, `complete`, `PartialBlock`, `StreamEvent`, `LlmEventStream` don't exist yet.

- [ ] **Step 4: Implement**

In `backend/src/llm/types.rs`, add at the top (after `use serde_json::Value;`):

```rust
use std::pin::Pin;
use futures_core::Stream;
```

And add these new items (after the existing `LlmError` enum, before the `#[cfg(test)]` module):

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum PartialBlock {
    Text,
    ToolUse { id: String, name: String },
}

#[derive(Debug, PartialEq)]
pub enum StreamEvent {
    ContentBlockStart { index: usize, block: PartialBlock },
    TextDelta { index: usize, text: String },
    ToolInputDelta { index: usize, partial_json: String },
    ContentBlockDone { index: usize },
    Done { stop_reason: StopReason, input_tokens: u32, output_tokens: u32 },
}

pub type LlmEventStream = Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>;
```

Replace `backend/src/llm/mod.rs`:

```rust
pub mod types;
pub mod anthropic;
pub mod openai;
pub mod gemini;
pub mod config;
pub mod fake;

pub use types::*;
pub use config::{build_provider, ModelConfig, ProviderKind};

use async_trait::async_trait;
use futures_util::StreamExt;
use std::collections::BTreeMap;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;

    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let response = self.complete(request).await?;
        Ok(response_to_stream(response))
    }
}

/// Wraps a complete (non-streaming) LlmResponse as a single-shot LlmEventStream — the trait's
/// default complete_stream body, and reused directly by fake providers that have no real
/// network round trip to chunk.
pub fn response_to_stream(response: LlmResponse) -> LlmEventStream {
    let mut events: Vec<Result<StreamEvent, LlmError>> = Vec::new();
    for (index, block) in response.content.into_iter().enumerate() {
        match block {
            ContentBlock::Text { text } => {
                events.push(Ok(StreamEvent::ContentBlockStart { index, block: PartialBlock::Text }));
                events.push(Ok(StreamEvent::TextDelta { index, text }));
                events.push(Ok(StreamEvent::ContentBlockDone { index }));
            }
            ContentBlock::ToolUse { id, name, input } => {
                events.push(Ok(StreamEvent::ContentBlockStart { index, block: PartialBlock::ToolUse { id, name } }));
                events.push(Ok(StreamEvent::ToolInputDelta { index, partial_json: input.to_string() }));
                events.push(Ok(StreamEvent::ContentBlockDone { index }));
            }
            ContentBlock::ToolResult { .. } => {
                // A provider's own response never contains a ToolResult block (that's only ever
                // something we send as part of a request) — nothing to emit.
            }
        }
    }
    events.push(Ok(StreamEvent::Done {
        stop_reason: response.stop_reason,
        input_tokens: response.input_tokens,
        output_tokens: response.output_tokens,
    }));
    Box::pin(futures_util::stream::iter(events))
}

pub async fn complete(provider: &dyn LlmProvider, request: LlmRequest) -> Result<LlmResponse, LlmError> {
    let stream = provider.complete_stream(request).await?;
    collect_stream(stream).await
}

enum PendingBlock {
    Text(String),
    ToolUse { id: String, name: String, input_json: String },
}

pub async fn collect_stream(mut stream: LlmEventStream) -> Result<LlmResponse, LlmError> {
    let mut pending: BTreeMap<usize, PendingBlock> = BTreeMap::new();
    let mut finished: BTreeMap<usize, ContentBlock> = BTreeMap::new();

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ContentBlockStart { index, block } => {
                let pending_block = match block {
                    PartialBlock::Text => PendingBlock::Text(String::new()),
                    PartialBlock::ToolUse { id, name } => {
                        PendingBlock::ToolUse { id, name, input_json: String::new() }
                    }
                };
                pending.insert(index, pending_block);
            }
            StreamEvent::TextDelta { index, text } => {
                if let Some(PendingBlock::Text(buffer)) = pending.get_mut(&index) {
                    buffer.push_str(&text);
                }
            }
            StreamEvent::ToolInputDelta { index, partial_json } => {
                if let Some(PendingBlock::ToolUse { input_json, .. }) = pending.get_mut(&index) {
                    input_json.push_str(&partial_json);
                }
            }
            StreamEvent::ContentBlockDone { index } => {
                if let Some(block) = pending.remove(&index) {
                    let content_block = match block {
                        PendingBlock::Text(text) => ContentBlock::Text { text },
                        PendingBlock::ToolUse { id, name, input_json } => {
                            let input = if input_json.is_empty() {
                                serde_json::json!({})
                            } else {
                                serde_json::from_str(&input_json)
                                    .map_err(|e| LlmError::ParseError(format!("invalid tool input json: {e}")))?
                            };
                            ContentBlock::ToolUse { id, name, input }
                        }
                    };
                    finished.insert(index, content_block);
                }
            }
            StreamEvent::Done { stop_reason, input_tokens, output_tokens } => {
                let content = finished.into_values().collect();
                return Ok(LlmResponse { content, stop_reason, input_tokens, output_tokens });
            }
        }
    }

    Err(LlmError::ParseError("stream ended without a Done event".to_string()))
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd backend && cargo test --test llm_streaming`
Expected: all 6 tests pass.

- [ ] **Step 6: Run the full backend test suite to confirm nothing else broke**

Run: `cd backend && cargo test`
Expected: all existing tests still pass (the trait's default `complete_stream` body means no existing `LlmProvider` implementor needs any change yet).

- [ ] **Step 7: Commit**

```bash
git add backend/Cargo.toml backend/Cargo.lock backend/src/llm/types.rs backend/src/llm/mod.rs backend/tests/llm_streaming.rs
git commit -m "feat: add complete_stream trait method (default impl) and collect_stream helper"
```

---

## Task 2: Anthropic streaming implementation

**Files:**
- Modify: `backend/src/llm/anthropic.rs`
- Test: `backend/tests/llm_anthropic.rs`

**Interfaces:**
- Consumes: `StreamEvent`, `PartialBlock`, `LlmEventStream`, `collect_stream` (Task 1).
- Produces: `AnthropicProvider::complete_stream` — a real SSE-based override. `AnthropicProvider::complete` is unchanged.

- [ ] **Step 1: Write the failing tests**

Append to `backend/tests/llm_anthropic.rs` (keep all existing tests and imports; add these new imports to the existing `use` lines and these new test functions):

```rust
use nomi_orchestrator::llm::{collect_stream, PartialBlock, StreamEvent};
```

```rust
#[tokio::test]
async fn streamed_text_reply_collects_into_the_same_text_block() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":10}}}\n\n",
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"hi \"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"there\"}}\n\n",
        "event: content_block_stop\n",
        "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
        "event: message_delta\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":5}}\n\n",
        "event: message_stop\n",
        "data: {\"type\":\"message_stop\"}\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(body_partial_json(json!({"stream": true})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(sse_body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let stream = provider.complete_stream(text_request()).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi there".to_string() }]);
    assert_eq!(response.stop_reason, StopReason::EndTurn);
    assert_eq!(response.input_tokens, 10);
    assert_eq!(response.output_tokens, 5);
}

#[tokio::test]
async fn streamed_tool_use_reply_reassembles_the_fragmented_input_json() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":20}}}\n\n",
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"get_weather\"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"city\\\":\"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"Paris\\\"}\"}}\n\n",
        "event: content_block_stop\n",
        "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
        "event: message_delta\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":8}}\n\n",
        "event: message_stop\n",
        "data: {\"type\":\"message_stop\"}\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
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

    let stream = provider.complete_stream(request).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

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
async fn a_mid_stream_error_event_surfaces_as_an_err() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
        "event: error\n",
        "data: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"overloaded\"}}\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let stream = provider.complete_stream(text_request()).await.unwrap();
    let result = collect_stream(stream).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}

#[tokio::test]
async fn non_success_status_is_returned_before_any_stream_is_produced() {
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

    let result = provider.complete_stream(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && cargo test --test llm_anthropic`
Expected: FAIL to compile — `complete_stream` isn't overridden yet on `AnthropicProvider` (it inherits the default, which calls `complete()` — that would actually still compile and might even pass some of these tests by accident since `complete()` still works! But the mocked SSE body wouldn't match what `complete()` expects (`set_body_json`, not raw SSE with `stream:true` in the request) — the `body_partial_json(json!({"stream": true}))` matcher in the first test won't match `complete()`'s request body (no `stream` field), so wiremock returns a 404-equivalent unmatched-request error, causing these new tests to fail. This confirms they're actually exercising a real streaming code path once implemented.

- [ ] **Step 3: Implement**

Replace `backend/src/llm/anthropic.rs`:

```rust
use async_stream::try_stream;
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde_json::json;

use super::types::{ContentBlock, LlmError, LlmEventStream, LlmRequest, LlmResponse, LlmRole, PartialBlock, StopReason, StreamEvent};
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

    fn build_body(&self, request: &LlmRequest, stream: bool) -> serde_json::Value {
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
            "stream": stream,
        });
        if let Some(system) = &request.system {
            body["system"] = json!(system);
        }
        if !tools.is_empty() {
            body["tools"] = json!(tools);
        }
        body
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
        let body = self.build_body(&request, false);

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

    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let body = self.build_body(&request, true);

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

        let mut events = response.bytes_stream().eventsource();

        let stream = try_stream! {
            let mut input_tokens: u32 = 0;

            loop {
                let event = match events.next().await {
                    Some(event) => event.map_err(|e| LlmError::ParseError(format!("sse stream error: {e}")))?,
                    None => Err(LlmError::ParseError("stream ended without a Done event".to_string()))?,
                };

                match event.event.as_str() {
                    "message_start" => {
                        let data: serde_json::Value = serde_json::from_str(&event.data)
                            .map_err(|e| LlmError::ParseError(format!("invalid message_start json: {e}")))?;
                        input_tokens = data
                            .get("message")
                            .and_then(|m| m.get("usage"))
                            .and_then(|u| u.get("input_tokens"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32;
                    }
                    "content_block_start" => {
                        let data: serde_json::Value = serde_json::from_str(&event.data)
                            .map_err(|e| LlmError::ParseError(format!("invalid content_block_start json: {e}")))?;
                        let index = data.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let block = data
                            .get("content_block")
                            .ok_or_else(|| LlmError::ParseError("missing content_block".to_string()))?;
                        let partial = match block.get("type").and_then(|t| t.as_str()) {
                            Some("text") => PartialBlock::Text,
                            Some("tool_use") => PartialBlock::ToolUse {
                                id: block.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                                name: block.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                            },
                            other => Err(LlmError::ParseError(format!("unknown content_block type: {other:?}")))?,
                        };
                        yield StreamEvent::ContentBlockStart { index, block: partial };
                    }
                    "content_block_delta" => {
                        let data: serde_json::Value = serde_json::from_str(&event.data)
                            .map_err(|e| LlmError::ParseError(format!("invalid content_block_delta json: {e}")))?;
                        let index = data.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let delta = data
                            .get("delta")
                            .ok_or_else(|| LlmError::ParseError("missing delta".to_string()))?;
                        match delta.get("type").and_then(|t| t.as_str()) {
                            Some("text_delta") => {
                                let text = delta.get("text").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                yield StreamEvent::TextDelta { index, text };
                            }
                            Some("input_json_delta") => {
                                let partial_json =
                                    delta.get("partial_json").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                yield StreamEvent::ToolInputDelta { index, partial_json };
                            }
                            _ => {}
                        }
                    }
                    "content_block_stop" => {
                        let data: serde_json::Value = serde_json::from_str(&event.data)
                            .map_err(|e| LlmError::ParseError(format!("invalid content_block_stop json: {e}")))?;
                        let index = data.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        yield StreamEvent::ContentBlockDone { index };
                    }
                    "message_delta" => {
                        let data: serde_json::Value = serde_json::from_str(&event.data)
                            .map_err(|e| LlmError::ParseError(format!("invalid message_delta json: {e}")))?;
                        let stop_reason = match data.get("delta").and_then(|d| d.get("stop_reason")).and_then(|s| s.as_str()) {
                            Some("end_turn") => StopReason::EndTurn,
                            Some("tool_use") => StopReason::ToolUse,
                            Some("max_tokens") => StopReason::MaxTokens,
                            Some(other) => StopReason::Other(other.to_string()),
                            None => StopReason::Other("unknown".to_string()),
                        };
                        let output_tokens =
                            data.get("usage").and_then(|u| u.get("output_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        yield StreamEvent::Done { stop_reason, input_tokens, output_tokens };
                        return;
                    }
                    "error" => {
                        let data: serde_json::Value =
                            serde_json::from_str(&event.data).unwrap_or(serde_json::Value::Null);
                        Err(LlmError::ProviderError(format!("anthropic stream error: {data}")))?;
                    }
                    _ => {}
                }
            }
        };

        Ok(Box::pin(stream))
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test --test llm_anthropic`
Expected: all 8 tests pass (4 existing `complete()` tests, unchanged and still passing, plus 4 new streaming tests).

- [ ] **Step 5: Run the full backend test suite**

Run: `cd backend && cargo test`
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add backend/src/llm/anthropic.rs backend/tests/llm_anthropic.rs
git commit -m "feat: add real SSE-based complete_stream to AnthropicProvider"
```

---

## Task 3: OpenAI streaming implementation

**Files:**
- Modify: `backend/src/llm/openai.rs`
- Test: `backend/tests/llm_openai.rs`

**Interfaces:**
- Consumes: same as Task 2.
- Produces: `OpenAiProvider::complete_stream`. `OpenAiProvider::complete` is unchanged.

- [ ] **Step 1: Write the failing tests**

Append to `backend/tests/llm_openai.rs` (add to the existing imports and add these test functions):

```rust
use nomi_orchestrator::llm::{collect_stream, PartialBlock, StreamEvent};
```

```rust
#[tokio::test]
async fn streamed_text_reply_collects_into_the_same_text_block() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"hi \"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\"there\"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":5}}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(json!({"stream": true, "stream_options": {"include_usage": true}})))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
        server.uri(),
    );

    let stream = provider.complete_stream(text_request()).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi there".to_string() }]);
    assert_eq!(response.stop_reason, StopReason::EndTurn);
    assert_eq!(response.input_tokens, 10);
    assert_eq!(response.output_tokens, 5);
}

#[tokio::test]
async fn streamed_tool_call_reassembles_fragmented_arguments_by_index() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"function\":{\"name\":\"get_weather\",\"arguments\":\"\"}}]},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"{\\\"city\\\":\"}}]},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"\\\"Paris\\\"}\"}}]},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n",
        "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":20,\"completion_tokens\":8}}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
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

    let stream = provider.complete_stream(request).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

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
async fn non_success_status_is_returned_before_any_stream_is_produced() {
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

    let result = provider.complete_stream(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && cargo test --test llm_openai`
Expected: FAIL — SSE-shaped mock bodies don't match what the default (`complete()`-delegating) `complete_stream` sends (no `stream`/`stream_options` fields in its request), so wiremock's request matcher fails.

- [ ] **Step 3: Implement**

Replace `backend/src/llm/openai.rs`:

```rust
use async_stream::try_stream;
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde_json::json;
use std::collections::BTreeMap;

use super::types::{ContentBlock, LlmError, LlmEventStream, LlmRequest, LlmResponse, LlmRole, PartialBlock, StopReason, StreamEvent};
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

    fn build_body(&self, request: &LlmRequest, stream: bool) -> serde_json::Value {
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
        if stream {
            body["stream"] = json!(true);
            body["stream_options"] = json!({ "include_usage": true });
        }
        body
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
        let body = self.build_body(&request, false);

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

    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let body = self.build_body(&request, true);

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

        let mut events = response.bytes_stream().eventsource();

        let stream = try_stream! {
            let mut next_index: usize = 0;
            let mut text_index: Option<usize> = None;
            let mut tool_index_map: BTreeMap<u64, usize> = BTreeMap::new();
            let mut open_indices: Vec<usize> = Vec::new();
            let mut stop_reason = StopReason::Other("unknown".to_string());
            let mut input_tokens: u32 = 0;
            let mut output_tokens: u32 = 0;

            loop {
                let event = match events.next().await {
                    Some(event) => event.map_err(|e| LlmError::ParseError(format!("sse stream error: {e}")))?,
                    None => Err(LlmError::ParseError("stream ended without a Done event".to_string()))?,
                };

                if event.data == "[DONE]" {
                    yield StreamEvent::Done { stop_reason, input_tokens, output_tokens };
                    return;
                }

                let data: serde_json::Value = serde_json::from_str(&event.data)
                    .map_err(|e| LlmError::ParseError(format!("invalid stream chunk json: {e}")))?;

                if let Some(usage) = data.get("usage") {
                    if !usage.is_null() {
                        input_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        output_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    }
                }

                let choice = data.get("choices").and_then(|c| c.as_array()).and_then(|c| c.first());
                let Some(choice) = choice else {
                    continue;
                };

                if let Some(text) = choice.get("delta").and_then(|d| d.get("content")).and_then(|c| c.as_str()) {
                    let is_first = text_index.is_none();
                    let index = *text_index.get_or_insert_with(|| {
                        let idx = next_index;
                        next_index += 1;
                        open_indices.push(idx);
                        idx
                    });
                    if is_first {
                        yield StreamEvent::ContentBlockStart { index, block: PartialBlock::Text };
                    }
                    yield StreamEvent::TextDelta { index, text: text.to_string() };
                }

                if let Some(tool_calls) = choice.get("delta").and_then(|d| d.get("tool_calls")).and_then(|t| t.as_array()) {
                    for tc in tool_calls {
                        let raw_index = tc.get("index").and_then(|v| v.as_u64()).unwrap_or(0);
                        let is_new = !tool_index_map.contains_key(&raw_index);
                        let index = *tool_index_map.entry(raw_index).or_insert_with(|| {
                            let idx = next_index;
                            next_index += 1;
                            idx
                        });
                        if is_new {
                            let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                            let name = tc
                                .get("function")
                                .and_then(|f| f.get("name"))
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            yield StreamEvent::ContentBlockStart { index, block: PartialBlock::ToolUse { id, name } };
                            open_indices.push(index);
                        }
                        if let Some(args) = tc.get("function").and_then(|f| f.get("arguments")).and_then(|v| v.as_str()) {
                            yield StreamEvent::ToolInputDelta { index, partial_json: args.to_string() };
                        }
                    }
                }

                if let Some(finish_reason) = choice.get("finish_reason").and_then(|f| f.as_str()) {
                    stop_reason = match finish_reason {
                        "stop" => StopReason::EndTurn,
                        "tool_calls" => StopReason::ToolUse,
                        "length" => StopReason::MaxTokens,
                        other => StopReason::Other(other.to_string()),
                    };
                    for index in open_indices.drain(..) {
                        yield StreamEvent::ContentBlockDone { index };
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test --test llm_openai`
Expected: all 7 tests pass (4 existing + 3 new).

- [ ] **Step 5: Run the full backend test suite**

Run: `cd backend && cargo test`
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add backend/src/llm/openai.rs backend/tests/llm_openai.rs
git commit -m "feat: add real SSE-based complete_stream to OpenAiProvider"
```

---

## Task 4: Gemini streaming implementation

**Files:**
- Modify: `backend/src/llm/gemini.rs`
- Test: `backend/tests/llm_gemini.rs`

**Interfaces:**
- Consumes: same as Task 2.
- Produces: `GeminiProvider::complete_stream`. `GeminiProvider::complete` is unchanged.

- [ ] **Step 1: Write the failing tests**

Append to `backend/tests/llm_gemini.rs` (add to existing imports and add these test functions):

```rust
use nomi_orchestrator::llm::{collect_stream, PartialBlock, StreamEvent};
```

```rust
#[tokio::test]
async fn streamed_text_reply_collects_into_the_same_text_block() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"hi \"}]}}]}\n\n",
        "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"there\"}]},\"finishReason\":\"STOP\"}],\"usageMetadata\":{\"promptTokenCount\":10,\"candidatesTokenCount\":5}}\n\n",
    );
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:streamGenerateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let stream = provider.complete_stream(text_request()).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi there".to_string() }]);
    assert_eq!(response.stop_reason, StopReason::EndTurn);
    assert_eq!(response.input_tokens, 10);
    assert_eq!(response.output_tokens, 5);
}

#[tokio::test]
async fn streamed_function_call_arrives_whole_in_one_chunk() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"candidates\":[{\"content\":{\"parts\":[{\"functionCall\":{\"name\":\"get_weather\",\"args\":{\"city\":\"Paris\"}}}]},\"finishReason\":\"STOP\"}],\"usageMetadata\":{\"promptTokenCount\":20,\"candidatesTokenCount\":8}}\n\n",
    );
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:streamGenerateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
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

    let stream = provider.complete_stream(request).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

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
async fn non_success_status_is_returned_before_any_stream_is_produced() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:streamGenerateContent$"))
        .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let result = provider.complete_stream(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend && cargo test --test llm_gemini`
Expected: FAIL — the default `complete_stream` posts to `:generateContent` (non-streaming path via `complete()`), not `:streamGenerateContent`, so the mocked path regex never matches.

- [ ] **Step 3: Implement**

Replace `backend/src/llm/gemini.rs`:

```rust
use async_stream::try_stream;
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde_json::json;

use super::types::{ContentBlock, LlmError, LlmEventStream, LlmRequest, LlmResponse, LlmRole, PartialBlock, StopReason, StreamEvent};
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

    fn build_body(&self, request: &LlmRequest) -> serde_json::Value {
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
        body
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
        ContentBlock::ToolResult { tool_use_id, content, .. } => {
            json!({ "functionResponse": { "name": tool_use_id, "response": { "content": content } } })
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let body = self.build_body(&request);
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

    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let body = self.build_body(&request);
        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            self.base_url, self.model, self.api_key
        );

        let response = self.client.post(url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!("gemini returned {status}: {text}")));
        }

        let mut events = response.bytes_stream().eventsource();

        let stream = try_stream! {
            let mut next_index: usize = 0;
            let mut text_index: Option<usize> = None;
            let mut saw_function_call = false;
            let mut input_tokens: u32 = 0;
            let mut output_tokens: u32 = 0;
            let mut finish_reason: Option<String> = None;

            loop {
                let event = match events.next().await {
                    Some(event) => event.map_err(|e| LlmError::ParseError(format!("sse stream error: {e}")))?,
                    None => break,
                };

                let data: serde_json::Value = serde_json::from_str(&event.data)
                    .map_err(|e| LlmError::ParseError(format!("invalid stream chunk json: {e}")))?;

                if let Some(usage) = data.get("usageMetadata") {
                    input_tokens = usage.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    output_tokens = usage.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                }

                let candidate = data.get("candidates").and_then(|c| c.as_array()).and_then(|c| c.first());
                let Some(candidate) = candidate else {
                    continue;
                };

                if let Some(fr) = candidate.get("finishReason").and_then(|f| f.as_str()) {
                    finish_reason = Some(fr.to_string());
                }

                let parts = candidate.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array());
                if let Some(parts) = parts {
                    for part in parts {
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            let is_first = text_index.is_none();
                            let index = *text_index.get_or_insert_with(|| {
                                let idx = next_index;
                                next_index += 1;
                                idx
                            });
                            if is_first {
                                yield StreamEvent::ContentBlockStart { index, block: PartialBlock::Text };
                            }
                            yield StreamEvent::TextDelta { index, text: text.to_string() };
                        } else if let Some(fc) = part.get("functionCall") {
                            saw_function_call = true;
                            let name = fc.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                            let args = fc.get("args").cloned().unwrap_or(serde_json::Value::Null);
                            let index = next_index;
                            next_index += 1;
                            // Gemini has no tool-call id; synthesize one from the function name,
                            // matching complete()'s convention.
                            yield StreamEvent::ContentBlockStart {
                                index,
                                block: PartialBlock::ToolUse { id: name.clone(), name },
                            };
                            yield StreamEvent::ToolInputDelta { index, partial_json: args.to_string() };
                            yield StreamEvent::ContentBlockDone { index };
                        }
                    }
                }
            }

            if let Some(index) = text_index {
                yield StreamEvent::ContentBlockDone { index };
            }

            let stop_reason = if saw_function_call {
                StopReason::ToolUse
            } else {
                match finish_reason.as_deref() {
                    Some("STOP") => StopReason::EndTurn,
                    Some("MAX_TOKENS") => StopReason::MaxTokens,
                    Some(other) => StopReason::Other(other.to_string()),
                    None => StopReason::Other("unknown".to_string()),
                }
            };

            yield StreamEvent::Done { stop_reason, input_tokens, output_tokens };
        };

        Ok(Box::pin(stream))
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend && cargo test --test llm_gemini`
Expected: all 7 tests pass (4 existing + 3 new).

- [ ] **Step 5: Run the full backend test suite**

Run: `cd backend && cargo test`
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add backend/src/llm/gemini.rs backend/tests/llm_gemini.rs
git commit -m "feat: add real SSE-based complete_stream to GeminiProvider"
```

---

## Task 5: Cutover — real fake-provider streams, delete `complete()`, migrate call sites

**Files:**
- Modify: `backend/src/llm/mod.rs`
- Modify: `backend/src/llm/fake.rs`
- Modify: `backend/tests/support/mod.rs`
- Modify: `backend/tests/llm_config.rs`
- Modify: `backend/src/turn/chitchat.rs`
- Modify: `backend/src/turn/memory.rs`
- Modify: `backend/src/turn/routing.rs`
- Modify: `backend/src/turn/tools.rs`
- Test: `backend/tests/llm_anthropic.rs`, `backend/tests/llm_openai.rs`, `backend/tests/llm_gemini.rs` (delete the now-obsolete `.complete()`-based tests)

**Interfaces:**
- Consumes: `response_to_stream` (Task 1, already `pub`), `complete_stream` overrides (Tasks 2-4).
- Produces: `LlmProvider` trait now has only `complete_stream` (no `complete`, no default body). `crate::llm::complete(provider, request)` is the only way to get a full `LlmResponse` from here on.

This is the one task in this plan that is not purely additive — every implementor of `LlmProvider` and every caller of `.complete(...)` must change together, so this lands as a single commit.

- [ ] **Step 1: Give the fake providers real (non-delegating) `complete_stream` implementations**

Replace `backend/src/llm/fake.rs`:

```rust
use async_trait::async_trait;

use super::{response_to_stream, ContentBlock, LlmError, LlmEventStream, LlmProvider, LlmRequest, LlmResponse, StopReason};

/// Sending a message whose text contains this exact string makes FakeLlmProvider return an
/// error instead of its canned success reply — used by the frontend's e2e suite to exercise a
/// real backend turn failure (a 502 from POST /api/sessions/:id/messages) without needing
/// browser-level network mocking, which can't intercept this app's server-side backend calls.
/// Shared contract with frontend/e2e/conversation.e2e.ts — keep both in sync if this changes.
const SIMULATE_FAILURE_SENTINEL: &str = "__SIMULATE_TURN_FAILURE__";

pub struct FakeLlmProvider;

#[async_trait]
impl LlmProvider for FakeLlmProvider {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let should_fail = request.messages.iter().any(|m| {
            m.content.iter().any(|block| match block {
                ContentBlock::Text { text } => text.contains(SIMULATE_FAILURE_SENTINEL),
                _ => false,
            })
        });

        if should_fail {
            return Err(LlmError::ProviderError("simulated failure for e2e testing".to_string()));
        }

        Ok(response_to_stream(LlmResponse {
            content: vec![ContentBlock::Text {
                text: "This is a fake response for local development and testing.".to_string(),
            }],
            stop_reason: StopReason::EndTurn,
            input_tokens: 0,
            output_tokens: 0,
        }))
    }
}
```

In `backend/src/llm/mod.rs`, remove `complete` from the trait definition and delete its default body — the trait becomes:

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError>;
}
```

(Everything else in `mod.rs` — `response_to_stream`, `collect_stream`, the free `complete` function — is unchanged; `response_to_stream` is still used by `fake.rs` above, just called directly instead of via a default trait method.)

- [ ] **Step 2: Update the test double**

In `backend/tests/support/mod.rs`, replace the `LlmProvider` impl block for the test-only `FakeLlmProvider`:

```rust
#[async_trait]
impl LlmProvider for FakeLlmProvider {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        self.received_requests.lock().unwrap().push(request);
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }
        let response = match &self.outcome {
            FakeOutcome::Success(response) => Ok(response.clone()),
            FakeOutcome::Failure(message) => Err(LlmError::ProviderError(message.clone())),
            FakeOutcome::Sequence(queue) => queue
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| LlmError::ProviderError("sequence exhausted".to_string())),
        }?;
        Ok(nomi_orchestrator::llm::response_to_stream(response))
    }
}
```

Add `LlmEventStream` to that file's existing `use nomi_orchestrator::llm::{...}` import list (alongside `LlmError, LlmProvider, LlmRequest, LlmResponse`).

- [ ] **Step 3: Delete the now-obsolete `.complete()`-based tests**

In `backend/tests/llm_anthropic.rs`, `backend/tests/llm_openai.rs`, and `backend/tests/llm_gemini.rs`: delete the 4 original test functions in each file that call `.complete(...)` directly (`text_only_reply_parses_into_a_text_block`, the tool-use/tool-call test, the tool-result-mapping test, and `non_success_status_becomes_a_provider_error` — the *old* non-streaming version, not the new `non_success_status_is_returned_before_any_stream_is_produced` one added in Tasks 2-4, which stays). The streaming tests added in Tasks 2-4 already cover equivalent request-mapping and error-handling behavior; these old tests would no longer compile once `complete()` is gone from the trait.

- [ ] **Step 4: Update `llm_config.rs`**

In `backend/tests/llm_config.rs`, replace every `provider.complete(...)` call with the free-function form. For example, the Anthropic test:

```rust
let response = build_provider(
    ModelConfig {
        provider: ProviderKind::Anthropic,
        model_id: "claude-haiku-4-5".to_string(),
        api_key: "key".to_string(),
        base_url: Some(server.uri()),
    },
    reqwest::Client::new(),
);
let response = nomi_orchestrator::llm::complete(response.as_ref(), simple_request()).await.unwrap();
```

Apply the same `nomi_orchestrator::llm::complete(provider.as_ref(), request).await` pattern to the OpenAI and Gemini cases in that file. Add `nomi_orchestrator::llm::complete` usage doesn't need a new `use` if called fully-qualified as above; alternatively add `use nomi_orchestrator::llm::complete;` to the top of the file and call it as `complete(response.as_ref(), simple_request()).await`. The mock SSE bodies these tests expect can stay as plain JSON bodies from `build_provider_uses_the_real_default_base_url_when_none_given` (construction-only, no call) — only the three tests that actually call `.complete(...)` need their mocked response bodies changed to match what `complete_stream` now sends. Since `build_provider_selects_anthropic`/`_openai`/`_gemini` currently mock a single JSON response body for the non-streaming endpoint, update each `Mock::given(...)` to match the streaming endpoint/body shape used in that provider's Task 2/3/4 tests (same `path`/`path_regex` matchers, `set_body_raw(..., "text/event-stream")` instead of `set_body_json`), reusing the exact SSE body strings from the corresponding provider's streaming test in Tasks 2-4.

- [ ] **Step 5: Migrate the four turn-loop call sites**

In `backend/src/turn/chitchat.rs`, change the import line to add `complete` to the existing `use crate::llm::{...}` list, and change:

```rust
let response = provider.complete(request).await.map_err(TurnError::LlmCallFailed)?;
```

to:

```rust
let response = crate::llm::complete(provider, request).await.map_err(TurnError::LlmCallFailed)?;
```

In `backend/src/turn/memory.rs`, same import change, and change:

```rust
let response = match provider.complete(request).await {
```

to:

```rust
let response = match crate::llm::complete(provider, request).await {
```

In `backend/src/turn/routing.rs`, same import change, and change:

```rust
let response = match provider.complete(request).await {
```

to:

```rust
let response = match crate::llm::complete(provider, request).await {
```

In `backend/src/turn/tools.rs`, same import change, and change:

```rust
let response = provider.complete(request).await.map_err(TurnError::LlmCallFailed)?;
```

to:

```rust
let response = crate::llm::complete(provider, request).await.map_err(TurnError::LlmCallFailed)?;
```

- [ ] **Step 6: Run the full backend test suite**

Run: `cd backend && cargo test`
Expected: all tests pass — this is the final integration checkpoint for the whole plan. If `llm_config.rs`'s updated SSE mock bodies don't match exactly, fix them against the real SSE strings used in that provider's Task 2/3/4 test file (they're testing the identical code path).

- [ ] **Step 7: Commit**

```bash
git add backend/src/llm/mod.rs backend/src/llm/fake.rs backend/tests/support/mod.rs backend/tests/llm_config.rs backend/tests/llm_anthropic.rs backend/tests/llm_openai.rs backend/tests/llm_gemini.rs backend/src/turn/chitchat.rs backend/src/turn/memory.rs backend/src/turn/routing.rs backend/src/turn/tools.rs
git commit -m "refactor: delete LlmProvider::complete(), migrate all call sites to complete_stream + collect_stream"
```
