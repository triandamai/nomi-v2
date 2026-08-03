# LLM Provider Streaming

Date: 2026-08-03
Status: Approved (pending user review of this doc)
Related: `docs/superpowers/specs/2026-07-26-llm-provider-abstraction-design.md` (the `LlmProvider` trait and three providers this design converts to streaming). Sub-project 1 of 5 in the realtime chat effort — see the other four (MQTT/pub-sub infra, turn-loop decoupling, WebSocket bridge, frontend realtime rendering), specced separately.

## Purpose

The current `LlmProvider::complete()` is request/response only — a provider call blocks until the model has finished generating the entire reply, then returns it in one shot. That's the reason the chat UI has no "typing" feel: even once realtime delivery exists (later sub-projects), there's nothing to deliver incrementally. This design adds streaming to the trait and to all three real providers (Anthropic, OpenAI, Gemini), replacing `complete()` outright rather than adding a second method next to it.

All four current call sites (`turn/chitchat.rs`, `turn/memory.rs`, `turn/routing.rs`, `turn/tools.rs`) migrate to the new streaming primitive. Three of them (`memory`, `routing`, `tools`) don't need incremental delivery — nothing about intent classification, memory extraction, or the money-agent's tool loop is shown to a user token-by-token — so they drain the stream through a shared `collect_stream()` helper back into the same `LlmResponse` shape they use today, with no behavior change. `chitchat.rs` also uses `collect_stream()` in this sub-project (there's nothing to publish incremental deltas to yet — that wiring lands in sub-project 3, the turn-loop decoupling). The point of doing all four now is that `collect_stream()` becomes the *one* place tool-call argument reassembly happens, instead of four near-duplicate implementations, and every provider's streaming path gets proven correct by real tests immediately rather than only when the first streaming consumer arrives.

## 1. Shared Types

```rust
// backend/src/llm/types.rs — additions

use std::pin::Pin;
use futures_core::Stream;

#[derive(Debug, Clone, PartialEq)]
pub enum PartialBlock {
    Text,
    ToolUse { id: String, name: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    ContentBlockStart { index: usize, block: PartialBlock },
    TextDelta { index: usize, text: String },
    ToolInputDelta { index: usize, partial_json: String },
    ContentBlockDone { index: usize },
    Done { stop_reason: StopReason, input_tokens: u32, output_tokens: u32 },
}

pub type LlmEventStream = Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>;
```

`index` matches the provider's own content-block indexing (Anthropic emits this natively; OpenAI's `tool_calls[].index` is reused directly; for Gemini, which doesn't index blocks explicitly, the provider assigns indices in the order blocks first appear in `parts[]`).

`LlmResponse`, `ContentBlock`, `LlmRequest`, `StopReason`, `LlmError` are unchanged from the existing design.

## 2. The Trait

```rust
// backend/src/llm/mod.rs
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError>;
}
```

`complete()` is deleted. A non-2xx HTTP response (checked before the body is wrapped as a stream, same as today's upfront status check) still returns `Err` directly from `complete_stream()` itself — the caller never gets a stream at all in that case. An error that arrives *mid*-stream (Anthropic's `event: error`, or the connection dropping) is yielded as an `Err` item from the stream instead.

## 3. `collect_stream()` — the shared drain helper

```rust
// backend/src/llm/mod.rs
pub async fn collect_stream(mut stream: LlmEventStream) -> Result<LlmResponse, LlmError> {
    // internally: BTreeMap<usize, PendingBlock> keyed by index, where PendingBlock is
    // Text(String) or ToolUse { id, name, input_json: String }. On ContentBlockDone,
    // finalize that index into a ContentBlock (parsing input_json for ToolUse — a parse
    // failure becomes LlmError::ParseError, matching today's tool_calls-arguments handling
    // in openai.rs). On Done, assemble content in index order into LlmResponse.
}
```

This is the one place tool-call argument reassembly lives — `memory.rs`, `routing.rs`, `tools.rs`, and (for now) `chitchat.rs` all call `provider.complete_stream(request).await.and_then(collect_stream)` and get back exactly the `LlmResponse` they use today. No call site's surrounding logic changes beyond that one call.

## 4. Provider Implementations

### Anthropic (`backend/src/llm/anthropic.rs`)

Add `"stream": true` to the request body. Response is `text/event-stream` with Anthropic's native block-structured events:

- `content_block_start` (`data.content_block: {type:"text"}` or `{type:"tool_use", id, name}`) → `StreamEvent::ContentBlockStart`
- `content_block_delta` (`data.delta: {type:"text_delta", text}` or `{type:"input_json_delta", partial_json}`) → `TextDelta` / `ToolInputDelta`
- `content_block_stop` → `ContentBlockDone`
- `message_delta` (carries `delta.stop_reason` and `usage.output_tokens`) + `message_stop` → `Done` (input token count comes from the earlier `message_start` event's `usage.input_tokens`, buffered until `Done` is emitted)
- `ping` events are ignored; `event: error` maps to `Err(LlmError::ProviderError(...))`

This provider's mapping is close to 1:1 — the shared `StreamEvent` model was designed after Anthropic's own event shape.

### OpenAI (`backend/src/llm/openai.rs`)

Add `"stream": true, "stream_options": {"include_usage": true}` (the `stream_options` flag is required to still get a usage/token-count chunk, matching today's non-streaming response). Response is SSE with unnamed `data: {...}` lines, terminated by a literal `data: [DONE]`.

Each chunk's `choices[0].delta`:
- `content` (a string, when present) → `TextDelta` at index 0 (OpenAI has one text stream, index is always 0 for the text block)
- `tool_calls[]`, each entry has an `index`; the first chunk for a given index carries `id` + `function.name` (→ `ContentBlockStart`), subsequent chunks for that index carry `function.arguments` fragments (→ `ToolInputDelta`)
- `choices[0].finish_reason` (non-null on the chunk that ends generation) triggers `ContentBlockDone` for whichever indices are still open, then the final `data:` chunk's `usage.{prompt_tokens,completion_tokens}` produces `Done`

### Gemini (`backend/src/llm/gemini.rs`)

Use `POST /v1beta/models/{model}:streamGenerateContent?alt=sse&key=...` — the `alt=sse` param is what makes Gemini emit proper `data:` SSE lines instead of its default chunked-JSON-array format, so it can share the same SSE-parsing code path as the other two providers.

Each `data:` chunk is a partial `GenerateContentResponse`. `candidates[0].content.parts[]`:
- a part with `text` → `TextDelta` (Gemini doesn't pre-announce a block start for text the way Anthropic does; the provider emits a synthetic `ContentBlockStart{index: 0, block: Text}` on the first text-bearing chunk, then `TextDelta` for every chunk after)
- a part with `functionCall` → Gemini does not fragment function-call arguments the way OpenAI/Anthropic do; they arrive whole in one part. The provider emits `ContentBlockStart` (with `id` synthesized, since Gemini doesn't provide one — a UUID) immediately followed by one `ToolInputDelta` carrying the complete arguments JSON and `ContentBlockDone`, all from that single chunk.
- the final chunk's `candidates[0].finishReason` + `usageMetadata.{promptTokenCount,candidatesTokenCount}` → `Done`

### Fake (`backend/src/llm/fake.rs`)

`FakeLlmProvider::complete_stream` returns a stream that yields a single `ContentBlockStart{index:0, block:Text}`, one `TextDelta` with the canned reply text, `ContentBlockDone{index:0}`, then `Done`. The existing `SIMULATE_FAILURE_SENTINEL` check runs before the stream is constructed and returns `Err` directly from `complete_stream()` (matching today's `complete()` behavior, where the failure is returned before any content — this preserves the frontend e2e test that relies on a 502 from the whole request).

## 5. New Dependency

`eventsource-stream` — parses Server-Sent Events out of a byte stream (handles multi-line `data:` fields, comments, and chunk boundaries split mid-event, which hand-rolled newline-splitting tends to get wrong). Wraps `reqwest::Response::bytes_stream()`, which requires enabling reqwest's `"stream"` feature (currently disabled — `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }`).

## 6. Testing Approach

Same `wiremock`-based approach as the existing provider tests, with `ResponseTemplate` bodies set to raw SSE text instead of a single JSON object. Per provider (Anthropic/OpenAI/Gemini):
- a plain streamed text reply — collect the stream and assert the joined text matches
- a streamed tool call — assert `collect_stream()` reconstructs the exact `ToolUse{id, name, input}` from the deltas
- a mid-stream error event — assert it surfaces as `Err`

Plus `collect_stream()` gets its own unit tests independent of any provider, feeding it hand-built `StreamEvent` sequences (interleaved text and tool-call deltas across multiple indices) to verify assembly order and multi-block handling without needing a real SSE wire format at all.

The four call sites' existing tests (`turn_chitchat.rs`, `turn_memory_retrieval.rs`, `turn_intent_classification.rs`, `turn_money_agent.rs` etc.) continue to use `tests/support/mod.rs`'s `FakeLlmProvider` — that test double needs its own `complete_stream` implementation added (mirroring the real `FakeLlmProvider`'s behavior: wrap the configured `Success`/`Failure`/`Sequence` response as a single-chunk stream), so none of those existing tests change their assertions, only the plumbing underneath compiles against the new trait.

## Out of Scope

- Actually consuming the stream incrementally anywhere (that's sub-project 3, turn-loop decoupling — this sub-project only proves streaming works and collects it back to the old shape everywhere except where sub-project 3 later changes `chitchat.rs`).
- MQTT, WebSockets, or any frontend change (sub-projects 2, 4, 5).
- Retry/backoff on a dropped stream connection mid-generation — a dropped connection surfaces as an `Err` item and the caller (today, `collect_stream()`'s users) fails the turn exactly as a non-streaming HTTP failure would today. Reconnection/resume is a future hardening concern, not needed for this slice.
