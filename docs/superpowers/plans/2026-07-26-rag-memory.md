# RAG Memory Retrieval, Writing & Reinforcement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the RAG loop described in `docs/superpowers/specs/2026-07-26-rag-memory-design.md`: a provider-agnostic `EmbeddingProvider` trait (OpenAI implementation), retrieval of relevant `memory_items` folded into the chitchat prompt, LLM-judged extraction that writes new memories after a successful reply, and a standalone `reinforce()` function for future feedback wiring.

**Architecture:** A new `backend/src/embedding/` module (mirrors `backend/src/llm/`'s shape: trait + one provider + config/build function). A new `backend/src/turn/memory.rs` owns retrieval, extraction/writing, and reinforcement SQL. `backend/src/turn/chitchat.rs` and `backend/src/turn/mod.rs` (both already merged from the turn-loop plan) are extended, not rewritten, to call into `memory.rs` and thread the new `embedding_provider`/`user_id`/`text` parameters through.

**Tech Stack:** Rust, `sqlx` (Postgres), the existing `llm` module's `LlmProvider`/`LlmRequest`/etc., `reqwest`, `async-trait`, `wiremock` (dev-dependency) — all already present. **No new Cargo.toml dependencies.** Vectors bind to the `VECTOR(1536)` column via a plain text-literal cast (`$1::vector`, e.g. `"[0.1,0.2,...]"`), verified to work against this project's real Postgres+pgvector instance — no `pgvector` crate needed.

## Global Constraints

- No new Cargo.toml dependencies. Vector binding uses the `$1::vector` text-cast pattern (helper function `to_vector_literal(&[f32]) -> String`), not the `pgvector` crate.
- `EmbeddingProvider::embed` never reads an API key from an environment variable — `api_key`/`base_url` are explicit constructor parameters, matching every existing provider in this codebase.
- Retrieval: top 5 memories (`MEMORY_RETRIEVAL_LIMIT = 5`), scored by `weight * (1 - cosine_distance)`, scoped by the sender's canonical `user_id` (not `session_id`).
- Extraction system prompt is exactly: `"Extract at most one durable fact worth remembering long-term from this exchange, or say NONE if nothing is worth storing."` — a literal `"NONE"` result (after trimming whitespace) means nothing is stored.
- Reinforcement: `ReinforcementSignal::Positive` multiplies weight by `1.2`, `Negative` by `0.8`, clamped to `[0.1, 5.0]` via `LEAST(GREATEST(weight * $factor, 0.1), 5.0)`.
- Retrieval failure (embedding call or query error) never fails the turn — chitchat proceeds with an empty memory list. Extraction/writing failure (after the reply is already persisted) never fails the turn either — both are best-effort, consistent with this project's established error-handling philosophy (e.g. the turn-loop plan's best-effort `TurnFailed` event write).
- The LLM call for extraction reuses the same `provider: &dyn LlmProvider` already passed into `run_chitchat_turn` for the chitchat reply itself — no second model/provider parameter.
- `reinforce()` is not wired into `handle_inbound_message` or any other caller in this plan — no UI or channel command exists yet to trigger it.

## File Structure

- `backend/src/embedding/types.rs` — `EmbeddingError`.
- `backend/src/embedding/mod.rs` — `EmbeddingProvider` trait, re-exports.
- `backend/src/embedding/openai.rs` — `OpenAiEmbeddingProvider`.
- `backend/src/embedding/config.rs` — `EmbeddingConfig`, `build_embedding_provider`.
- `backend/src/lib.rs` — add `pub mod embedding;`.
- `backend/migrations/0008_message_memory_usage.sql` — new table linking a reply message to the memories retrieved for it.
- `backend/src/turn/memory.rs` — `retrieve_relevant_memories`, `extract_and_store_memory`, `reinforce`/`ReinforcementSignal`/`ReinforcementError`, built up across Tasks 2, 3, and 5.
- `backend/src/turn/chitchat.rs`, `backend/src/turn/mod.rs` — extended in Task 4 (both already exist from the turn-loop plan).
- `backend/tests/support/mod.rs` — extended in Task 3 with `FakeEmbeddingProvider` and a `dummy_embedding()` helper (already exists from the turn-loop plan, holds `FakeLlmProvider`).
- New test files: `backend/tests/embedding_openai.rs`, `embedding_config.rs`, `turn_memory_retrieval.rs`, `turn_memory_writing.rs`, `turn_reinforcement.rs`.
- Modified test files (Task 4 only): `backend/tests/turn_chitchat.rs`, `backend/tests/turn_handle_inbound_message.rs`.

---

### Task 1: Embedding module (trait, OpenAI provider, config)

**Files:**
- Create: `backend/src/embedding/types.rs`
- Create: `backend/src/embedding/mod.rs`
- Create: `backend/src/embedding/openai.rs`
- Create: `backend/src/embedding/config.rs`
- Modify: `backend/src/lib.rs`
- Test: `backend/tests/embedding_openai.rs`
- Test: `backend/tests/embedding_config.rs`

**Interfaces:**
- Produces: `EmbeddingError { Http(#[from] reqwest::Error), ProviderError(String), ParseError(String) }`, `trait EmbeddingProvider { async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>; }`, `OpenAiEmbeddingProvider::new(client, api_key, model, base_url)` + `default_base_url() -> String`, `EmbeddingConfig { model_id: String, api_key: String, base_url: Option<String> }`, `build_embedding_provider(config: EmbeddingConfig, http_client: reqwest::Client) -> Box<dyn EmbeddingProvider>`. Later tasks import `crate::embedding::{EmbeddingProvider, EmbeddingError}`.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/embedding_openai.rs
use nomi_orchestrator::embedding::openai::OpenAiEmbeddingProvider;
use nomi_orchestrator::embedding::{EmbeddingError, EmbeddingProvider};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn embed_returns_the_vector_from_the_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .and(body_partial_json(json!({
            "model": "text-embedding-3-small",
            "input": "hello world"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"embedding": [0.1, 0.2, 0.3], "index": 0}],
            "model": "text-embedding-3-small",
            "usage": {"prompt_tokens": 2, "total_tokens": 2}
        })))
        .mount(&server)
        .await;

    let provider = OpenAiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "text-embedding-3-small".to_string(),
        server.uri(),
    );

    let embedding = provider.embed("hello world").await.unwrap();
    assert_eq!(embedding, vec![0.1f32, 0.2, 0.3]);
}

#[tokio::test]
async fn non_success_status_becomes_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let provider = OpenAiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "text-embedding-3-small".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ProviderError(_))));
}

#[tokio::test]
async fn malformed_response_becomes_a_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .mount(&server)
        .await;

    let provider = OpenAiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "text-embedding-3-small".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ParseError(_))));
}
```

```rust
// backend/tests/embedding_config.rs
use nomi_orchestrator::embedding::{build_embedding_provider, EmbeddingConfig, EmbeddingProvider};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn build_embedding_provider_calls_the_openai_embeddings_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"embedding": [0.1, 0.2], "index": 0}],
            "model": "text-embedding-3-small",
            "usage": {"prompt_tokens": 1, "total_tokens": 1}
        })))
        .mount(&server)
        .await;

    let config = EmbeddingConfig {
        model_id: "text-embedding-3-small".to_string(),
        api_key: "test-key".to_string(),
        base_url: Some(server.uri()),
    };

    let provider = build_embedding_provider(config, reqwest::Client::new());
    let embedding = provider.embed("hello").await.unwrap();
    assert_eq!(embedding, vec![0.1f32, 0.2]);
}

#[tokio::test]
async fn base_url_none_falls_through_to_the_real_default_without_panicking() {
    let config = EmbeddingConfig {
        model_id: "text-embedding-3-small".to_string(),
        api_key: "test-key".to_string(),
        base_url: None,
    };

    // Construction only — no network call is made.
    let _provider = build_embedding_provider(config, reqwest::Client::new());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test embedding_openai --test embedding_config`
Expected: FAIL to compile — `nomi_orchestrator::embedding` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/embedding/types.rs
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("provider returned an error response: {0}")]
    ProviderError(String),
    #[error("failed to parse provider response: {0}")]
    ParseError(String),
}
```

```rust
// backend/src/embedding/mod.rs
pub mod config;
pub mod openai;
pub mod types;

pub use config::{build_embedding_provider, EmbeddingConfig};
pub use types::EmbeddingError;

use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
}
```

```rust
// backend/src/embedding/openai.rs
use async_trait::async_trait;
use serde_json::json;

use super::types::EmbeddingError;
use super::EmbeddingProvider;

pub struct OpenAiEmbeddingProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiEmbeddingProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://api.openai.com".to_string()
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAiEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let body = json!({
            "model": self.model,
            "input": text,
        });

        let response = self
            .client
            .post(format!("{}/v1/embeddings", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ProviderError(format!("openai returned {status}: {text}")));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ParseError(e.to_string()))?;

        let embedding = body
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|d| d.first())
            .and_then(|e| e.get("embedding"))
            .and_then(|e| e.as_array())
            .ok_or_else(|| EmbeddingError::ParseError("missing data[0].embedding".to_string()))?;

        embedding
            .iter()
            .map(|v| {
                v.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| EmbeddingError::ParseError("embedding element is not a number".to_string()))
            })
            .collect()
    }
}
```

```rust
// backend/src/embedding/config.rs
use super::openai::OpenAiEmbeddingProvider;
use super::EmbeddingProvider;

#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub model_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

pub fn build_embedding_provider(config: EmbeddingConfig, http_client: reqwest::Client) -> Box<dyn EmbeddingProvider> {
    let base_url = config.base_url.unwrap_or_else(OpenAiEmbeddingProvider::default_base_url);
    Box::new(OpenAiEmbeddingProvider::new(http_client, config.api_key, config.model_id, base_url))
}
```

```rust
// backend/src/lib.rs — add this line among the existing pub mod declarations
pub mod embedding;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test embedding_openai --test embedding_config`
Expected: PASS (3 + 2 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/embedding backend/src/lib.rs backend/tests/embedding_openai.rs backend/tests/embedding_config.rs
git commit -m "feat: add EmbeddingProvider trait, OpenAI implementation, and config"
```

---

### Task 2: Migration + memory retrieval

**Files:**
- Create: `backend/migrations/0008_message_memory_usage.sql`
- Create: `backend/src/turn/memory.rs`
- Modify: `backend/src/turn/mod.rs` (add `pub mod memory;`)
- Test: `backend/tests/turn_memory_retrieval.rs`

**Interfaces:**
- Produces: `RetrievedMemory { id: Uuid, content: String }`, `async fn retrieve_relevant_memories(conn: &mut PoolConnection<Postgres>, user_id: Uuid, query_embedding: &[f32], limit: i64) -> Result<Vec<RetrievedMemory>, TurnError>`, private `to_vector_literal(&[f32]) -> String` helper (reused by Task 3). Task 4 calls `retrieve_relevant_memories`.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/turn_memory_retrieval.rs
use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::turn::memory::retrieve_relevant_memories;

fn make_embedding(first: f32, second: f32) -> Vec<f32> {
    let mut v = vec![0.0f32; 1536];
    v[0] = first;
    v[1] = second;
    v
}

fn to_vector_literal(embedding: &[f32]) -> String {
    format!("[{}]", embedding.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

async fn seed_memory(pool: &PgPool, user_id: Uuid, content: &str, embedding: &[f32], weight: f64) {
    sqlx::query("INSERT INTO memory_items (user_id, content, embedding, weight) VALUES ($1, $2, $3::vector, $4)")
        .bind(user_id)
        .bind(content)
        .bind(to_vector_literal(embedding))
        .bind(weight)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test]
async fn retrieves_memories_ordered_by_weighted_similarity(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    seed_memory(&pool, user_id, "close match", &make_embedding(1.0, 0.0), 1.0).await;
    seed_memory(&pool, user_id, "far match", &make_embedding(0.0, 1.0), 1.0).await;

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(0.9, 0.1);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, 5).await.unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].content, "close match");
    assert_eq!(results[1].content, "far match");
}

#[sqlx::test]
async fn a_high_weight_can_outrank_a_higher_raw_similarity(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    seed_memory(&pool, user_id, "closer but low weight", &make_embedding(1.0, 0.0), 0.5).await;
    seed_memory(&pool, user_id, "farther but high weight", &make_embedding(0.7, 0.3), 5.0).await;

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, 5).await.unwrap();

    assert_eq!(results[0].content, "farther but high weight");
}

#[sqlx::test]
async fn respects_the_limit_parameter(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    for i in 0..3 {
        seed_memory(&pool, user_id, &format!("memory-{i}"), &make_embedding(1.0 - (i as f32 * 0.1), 0.0), 1.0).await;
    }

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, 2).await.unwrap();

    assert_eq!(results.len(), 2);
}

#[sqlx::test]
async fn only_returns_memories_for_the_queried_user(pool: PgPool) {
    let user_a = seed_user(&pool).await;
    let user_b = seed_user(&pool).await;
    seed_memory(&pool, user_a, "user a's memory", &make_embedding(1.0, 0.0), 1.0).await;
    seed_memory(&pool, user_b, "user b's memory", &make_embedding(1.0, 0.0), 1.0).await;

    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_a, &query, 5).await.unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].content, "user a's memory");
}

#[sqlx::test]
async fn returns_empty_when_the_user_has_no_memories(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let query = make_embedding(1.0, 0.0);

    let results = retrieve_relevant_memories(&mut conn, user_id, &query, 5).await.unwrap();

    assert!(results.is_empty());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_memory_retrieval`
Expected: FAIL — migration table missing and/or `nomi_orchestrator::turn::memory` doesn't exist yet.

- [ ] **Step 3: Write the implementation**

```sql
-- backend/migrations/0008_message_memory_usage.sql
CREATE TABLE message_memory_usage (
    message_id  UUID NOT NULL REFERENCES messages(id),
    memory_id   UUID NOT NULL REFERENCES memory_items(id),
    PRIMARY KEY (message_id, memory_id)
);
```

```rust
// backend/src/turn/memory.rs
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use super::types::TurnError;

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievedMemory {
    pub id: Uuid,
    pub content: String,
}

fn to_vector_literal(embedding: &[f32]) -> String {
    format!("[{}]", embedding.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}

pub async fn retrieve_relevant_memories(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    query_embedding: &[f32],
    limit: i64,
) -> Result<Vec<RetrievedMemory>, TurnError> {
    let literal = to_vector_literal(query_embedding);
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, content FROM memory_items \
         WHERE user_id = $1 \
         ORDER BY weight * (1 - (embedding <=> $2::vector)) DESC \
         LIMIT $3",
    )
    .bind(user_id)
    .bind(&literal)
    .bind(limit)
    .fetch_all(&mut **conn)
    .await?;

    Ok(rows.into_iter().map(|(id, content)| RetrievedMemory { id, content }).collect())
}
```

```rust
// backend/src/turn/mod.rs — add this line among the existing pub mod declarations
pub mod memory;
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_memory_retrieval`
Expected: PASS (5 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/migrations/0008_message_memory_usage.sql backend/src/turn/memory.rs backend/src/turn/mod.rs backend/tests/turn_memory_retrieval.rs
git commit -m "feat: add message_memory_usage table and weighted-similarity memory retrieval"
```

---

### Task 3: Memory writing (LLM-judged extraction)

**Files:**
- Modify: `backend/src/turn/memory.rs` (add `extract_and_store_memory`)
- Modify: `backend/tests/support/mod.rs` (add `FakeEmbeddingProvider`, `dummy_embedding()`)
- Test: `backend/tests/turn_memory_writing.rs`

**Interfaces:**
- Consumes: `to_vector_literal` (private, Task 2, same file); `LlmProvider`/`LlmRequest`/`LlmMessage`/`LlmRole`/`ContentBlock` from `crate::llm`; `EmbeddingProvider` from `crate::embedding`.
- Produces: `async fn extract_and_store_memory(conn: &mut PoolConnection<Postgres>, provider: &dyn LlmProvider, embedding_provider: &dyn EmbeddingProvider, user_id: Uuid, user_text: &str, assistant_text: &str)` (returns `()`, never fails the caller). Also produces `FakeEmbeddingProvider` (`success(Vec<f32>)`, `failure(impl Into<String>)`) and `dummy_embedding() -> Vec<f32>` (a 1536-length all-zero vector) in `tests/support/mod.rs`, both reused by Task 4.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/support/mod.rs — FULL FILE (existing content plus the additions below)
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use nomi_orchestrator::embedding::{EmbeddingError, EmbeddingProvider};
use nomi_orchestrator::llm::{LlmError, LlmProvider, LlmRequest, LlmResponse};

enum FakeOutcome {
    Success(LlmResponse),
    Failure(String),
}

pub struct FakeLlmProvider {
    outcome: FakeOutcome,
    delay: Option<Duration>,
    pub received_requests: Mutex<Vec<LlmRequest>>,
}

impl FakeLlmProvider {
    pub fn success(response: LlmResponse) -> Self {
        Self { outcome: FakeOutcome::Success(response), delay: None, received_requests: Mutex::new(Vec::new()) }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self { outcome: FakeOutcome::Failure(message.into()), delay: None, received_requests: Mutex::new(Vec::new()) }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}

#[async_trait]
impl LlmProvider for FakeLlmProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        self.received_requests.lock().unwrap().push(request);
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }
        match &self.outcome {
            FakeOutcome::Success(response) => Ok(response.clone()),
            FakeOutcome::Failure(message) => Err(LlmError::ProviderError(message.clone())),
        }
    }
}

enum FakeEmbeddingOutcome {
    Success(Vec<f32>),
    Failure(String),
}

pub struct FakeEmbeddingProvider {
    outcome: FakeEmbeddingOutcome,
}

impl FakeEmbeddingProvider {
    pub fn success(vector: Vec<f32>) -> Self {
        Self { outcome: FakeEmbeddingOutcome::Success(vector) }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self { outcome: FakeEmbeddingOutcome::Failure(message.into()) }
    }
}

#[async_trait]
impl EmbeddingProvider for FakeEmbeddingProvider {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
        match &self.outcome {
            FakeEmbeddingOutcome::Success(vector) => Ok(vector.clone()),
            FakeEmbeddingOutcome::Failure(message) => Err(EmbeddingError::ProviderError(message.clone())),
        }
    }
}

pub fn dummy_embedding() -> Vec<f32> {
    vec![0.0; 1536]
}
```

```rust
// backend/tests/turn_memory_writing.rs
mod support;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason};
use nomi_orchestrator::turn::memory::extract_and_store_memory;

use support::{FakeEmbeddingProvider, FakeLlmProvider};

fn extraction_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 5,
        output_tokens: 3,
    }
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

#[sqlx::test]
async fn none_extraction_stores_nothing(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(extraction_response("NONE"));
    let embedder = FakeEmbeddingProvider::success(vec![0.1; 1536]);

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "hi", "hello").await;

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM memory_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test]
async fn a_real_extracted_fact_is_embedded_and_stored(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(extraction_response("User is vegetarian"));
    let embedder = FakeEmbeddingProvider::success(vec![0.2; 1536]);

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "I don't eat meat", "Noted!").await;

    let content: String = sqlx::query_scalar("SELECT content FROM memory_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(content, "User is vegetarian");
}

#[sqlx::test]
async fn extraction_llm_failure_stores_nothing(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::failure("provider down");
    let embedder = FakeEmbeddingProvider::success(vec![0.1; 1536]);

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "hi", "hello").await;

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM memory_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[sqlx::test]
async fn embedding_failure_after_a_good_extraction_stores_nothing(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let llm = FakeLlmProvider::success(extraction_response("User is vegetarian"));
    let embedder = FakeEmbeddingProvider::failure("embeddings unavailable");

    extract_and_store_memory(&mut conn, &llm, &embedder, user_id, "I don't eat meat", "Noted!").await;

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM memory_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_memory_writing`
Expected: FAIL to compile — `extract_and_store_memory`/`FakeEmbeddingProvider` don't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/turn/memory.rs — ADD these imports at the top and this function at the end of the file
use crate::embedding::EmbeddingProvider;
use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

const EXTRACTION_SYSTEM_PROMPT: &str =
    "Extract at most one durable fact worth remembering long-term from this exchange, or say NONE if nothing is worth storing.";
const EXTRACTION_MAX_TOKENS: u32 = 128;

pub async fn extract_and_store_memory(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    user_id: Uuid,
    user_text: &str,
    assistant_text: &str,
) {
    let request = LlmRequest {
        system: Some(EXTRACTION_SYSTEM_PROMPT.to_string()),
        messages: vec![
            LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: user_text.to_string() }] },
            LlmMessage { role: LlmRole::Assistant, content: vec![ContentBlock::Text { text: assistant_text.to_string() }] },
        ],
        tools: vec![],
        max_tokens: EXTRACTION_MAX_TOKENS,
    };

    let response = match provider.complete(request).await {
        Ok(r) => r,
        Err(_) => return,
    };

    let extracted = response.content.into_iter().find_map(|block| match block {
        ContentBlock::Text { text } => Some(text),
        _ => None,
    });

    let fact = match extracted {
        Some(f) if f.trim() != "NONE" && !f.trim().is_empty() => f.trim().to_string(),
        _ => return,
    };

    let embedding = match embedding_provider.embed(&fact).await {
        Ok(e) => e,
        Err(_) => return,
    };

    let literal = to_vector_literal(&embedding);
    let _ = sqlx::query("INSERT INTO memory_items (user_id, content, embedding) VALUES ($1, $2, $3::vector)")
        .bind(user_id)
        .bind(&fact)
        .bind(&literal)
        .execute(&mut **conn)
        .await;
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_memory_writing`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/turn/memory.rs backend/tests/support/mod.rs backend/tests/turn_memory_writing.rs
git commit -m "feat: add LLM-judged memory extraction and writing"
```

---

### Task 4: Wire retrieval + writing into the chitchat turn

**Files:**
- Modify: `backend/src/turn/chitchat.rs`
- Modify: `backend/src/turn/mod.rs` (update `handle_inbound_message`)
- Modify: `backend/tests/turn_chitchat.rs`
- Modify: `backend/tests/turn_handle_inbound_message.rs`

**Interfaces:**
- Consumes: `memory::{retrieve_relevant_memories, extract_and_store_memory, RetrievedMemory}` (Tasks 2–3), `EmbeddingProvider` (Task 1), `FakeEmbeddingProvider`/`dummy_embedding` (Task 3).
- Produces: `run_chitchat_turn(conn, provider, embedding_provider: &dyn EmbeddingProvider, session_id, user_id: Uuid, text: &str) -> Result<String, TurnError>` (two new parameters: `embedding_provider`, `user_id`, `text` — `session_id` unchanged); `handle_inbound_message(pool, provider, embedding_provider: &dyn EmbeddingProvider, channel, chat_type, chat_id, sender_channel_user_id, text) -> Result<TurnOutcome, TurnError>` (one new parameter: `embedding_provider`, inserted right after `provider`).

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/turn_chitchat.rs — FULL FILE REPLACEMENT
mod support;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, LlmRole, StopReason};
use nomi_orchestrator::turn::chitchat::run_chitchat_turn;

use support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

async fn seed_session_and_user(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    (session_id, user_id)
}

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

fn make_embedding(first: f32) -> Vec<f32> {
    let mut v = vec![0.0f32; 1536];
    v[0] = first;
    v
}

fn make_embedding_literal(first: f32) -> String {
    make_embedding(first).iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

#[sqlx::test]
async fn persists_reply_and_chitchat_reply_event_on_success(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("Hello there!"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let reply = run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "hi")
        .await
        .unwrap();
    assert_eq!(reply, "Hello there!");

    let (sender, content): (Option<Uuid>, String) = sqlx::query_as(
        "SELECT sender_channel_identity_id, content FROM messages WHERE session_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(sender, None);
    assert_eq!(content, "Hello there!");

    let payload: serde_json::Value = sqlx::query_scalar(
        "SELECT payload FROM agent_events WHERE session_id = $1 AND event_type = 'ChitchatReply'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payload, serde_json::json!({"input_tokens": 10, "output_tokens": 5}));
}

#[sqlx::test]
async fn persists_nothing_when_the_provider_call_fails(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::failure("provider unavailable");
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let result = run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "hi").await;
    assert!(result.is_err());

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 0);

    let event_count: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_events WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(event_count, 0);
}

#[sqlx::test]
async fn keeps_only_the_last_20_messages_ordered_oldest_first(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let base = chrono::Utc::now();
    for i in 0..25 {
        sqlx::query(
            "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(session_id)
        .bind(identity_id)
        .bind(format!("seq-{i}"))
        .bind(base + chrono::Duration::milliseconds(i))
        .execute(&pool)
        .await
        .unwrap();
    }

    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("ok"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "latest").await.unwrap();

    let requests = provider.received_requests.lock().unwrap();
    let sent = &requests[0];
    assert_eq!(sent.messages.len(), 20);
    for (offset, message) in sent.messages.iter().enumerate() {
        let expected_seq = 5 + offset;
        match &message.content[0] {
            ContentBlock::Text { text } => assert_eq!(text, &format!("seq-{expected_seq}")),
            _ => panic!("expected a text block"),
        }
    }
}

#[sqlx::test]
async fn maps_sender_presence_to_role_correctly(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let base = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, 'hi', $3)",
    )
    .bind(session_id)
    .bind(identity_id)
    .bind(base)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, NULL, 'hello back', $2)",
    )
    .bind(session_id)
    .bind(base + chrono::Duration::milliseconds(1))
    .execute(&pool)
    .await
    .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("ok"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "latest").await.unwrap();

    let requests = provider.received_requests.lock().unwrap();
    let sent = &requests[0];
    assert_eq!(sent.messages[0].role, LlmRole::User);
    assert_eq!(sent.messages[1].role, LlmRole::Assistant);
}

#[sqlx::test]
async fn retrieved_memories_are_folded_into_the_system_prompt_and_linked_to_the_reply(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let memory_id: Uuid = sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, embedding) VALUES ($1, $2, $3::vector) RETURNING id",
    )
    .bind(user_id)
    .bind("User is vegetarian")
    .bind(format!("[{}]", make_embedding_literal(1.0)))
    .fetch_one(&pool)
    .await
    .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("Got it, no meat!"));
    let embedder = FakeEmbeddingProvider::success(make_embedding(1.0));

    run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "what should I eat?")
        .await
        .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    let system = requests[0].system.as_ref().unwrap();
    assert!(system.contains("Relevant things you know about this user"));
    assert!(system.contains("User is vegetarian"));

    let linked_memory_id: Uuid = sqlx::query_scalar(
        "SELECT memory_id FROM message_memory_usage mu \
         JOIN messages m ON mu.message_id = m.id \
         WHERE m.session_id = $1 AND m.sender_channel_identity_id IS NULL",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(linked_memory_id, memory_id);
}

#[sqlx::test]
async fn a_failing_embedding_provider_does_not_prevent_a_normal_reply(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("Still here!"));
    let embedder = FakeEmbeddingProvider::failure("embeddings unavailable");

    let reply = run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "hi")
        .await
        .unwrap();
    assert_eq!(reply, "Still here!");

    let requests = provider.received_requests.lock().unwrap();
    assert_eq!(
        requests[0].system.as_ref().unwrap(),
        "You are a helpful, friendly assistant chatting with the user. Keep replies concise."
    );
}
```

```rust
// backend/tests/turn_handle_inbound_message.rs — FULL FILE REPLACEMENT
mod support;

use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason};
use nomi_orchestrator::turn::handle_inbound_message;

use support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

#[sqlx::test]
async fn new_sender_gets_bootstrapped_and_receives_a_chitchat_reply(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("Hi! How can I help?"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let outcome = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "hello")
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Hi! How can I help?");

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(outcome.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 2); // inbound + reply

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(user_count, 1);
    let org_count: i64 = sqlx::query_scalar("SELECT count(*) FROM organizations").fetch_one(&pool).await.unwrap();
    assert_eq!(org_count, 1);
}

#[sqlx::test]
async fn existing_sender_reuses_identity_and_session_across_two_calls(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("ok"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let first = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "first")
        .await
        .unwrap();
    let second = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "second")
        .await
        .unwrap();

    assert_eq!(first.session_id, second.session_id);

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(first.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 4); // 2 inbound + 2 replies

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(user_count, 1);
}

#[sqlx::test]
async fn inbound_message_is_durable_even_when_the_provider_call_fails(pool: PgPool) {
    let provider = FakeLlmProvider::failure("provider unavailable");
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let result = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "hello")
        .await;
    assert!(result.is_err());

    let (content,): (String,) = sqlx::query_as(
        "SELECT content FROM messages m JOIN sessions s ON m.session_id = s.id WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(content, "hello");

    let event_type: String = sqlx::query_scalar(
        "SELECT event_type FROM agent_events e JOIN sessions s ON e.session_id = s.id WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_type, "TurnFailed");
}

#[sqlx::test]
async fn an_active_agent_session_does_not_block_the_chitchat_fallback_in_this_slice(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("still chatting"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let first = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "hi")
        .await
        .unwrap();

    let identity_id: Uuid = sqlx::query_scalar(
        "SELECT ci.id FROM channel_identities ci WHERE ci.channel = 'telegram' AND ci.channel_user_id = 'tg-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active')",
    )
    .bind(first.session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let second = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "still there?")
        .await
        .unwrap();

    assert_eq!(second.reply, "still chatting");
}

#[sqlx::test]
async fn concurrent_messages_for_the_same_session_are_serialized(pool: PgPool) {
    let bootstrap_provider = FakeLlmProvider::success(canned_response("bootstrapped"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    handle_inbound_message(&pool, &bootstrap_provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "bootstrap")
        .await
        .unwrap();

    let provider = FakeLlmProvider::success(canned_response("ok")).with_delay(Duration::from_millis(200));

    let call1 = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "first");
    let call2 = handle_inbound_message(&pool, &provider, &embedder, "telegram", "dm", "chat-1", "tg-1", "second");
    let (result1, result2) = tokio::join!(call1, call2);
    result1.unwrap();
    result2.unwrap();

    let rows: Vec<(Option<Uuid>, String)> = sqlx::query_as(
        "SELECT m.sender_channel_identity_id, m.content FROM messages m \
         JOIN sessions s ON m.session_id = s.id \
         WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1' \
         ORDER BY m.created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    // bootstrap turn (2 rows) + two serialized turns (2 rows each) = 6, never interleaved.
    assert_eq!(rows.len(), 6);
    let (first_inbound, first_reply, second_inbound, second_reply) =
        (&rows[2], &rows[3], &rows[4], &rows[5]);
    assert!(first_inbound.0.is_some());
    assert!(first_inbound.1 == "first" || first_inbound.1 == "second");
    assert!(first_reply.0.is_none());
    assert_ne!(first_inbound.1, second_inbound.1);
    assert!(second_inbound.0.is_some());
    assert!(second_reply.0.is_none());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_chitchat --test turn_handle_inbound_message`
Expected: FAIL to compile — `run_chitchat_turn`/`handle_inbound_message` don't accept the new parameters yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/turn/chitchat.rs — FULL FILE REPLACEMENT
use sqlx::pool::PoolConnection;
use sqlx::{Acquire, Postgres};
use uuid::Uuid;

use crate::embedding::EmbeddingProvider;
use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

use super::memory::{self, RetrievedMemory};
use super::types::TurnError;

const CHITCHAT_SYSTEM_PROMPT: &str =
    "You are a helpful, friendly assistant chatting with the user. Keep replies concise.";
const CHITCHAT_HISTORY_LIMIT: i64 = 20;
const CHITCHAT_MAX_TOKENS: u32 = 1024;
const MEMORY_RETRIEVAL_LIMIT: i64 = 5;

pub async fn run_chitchat_turn(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    session_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<String, TurnError> {
    let rows: Vec<(Option<Uuid>, String)> = sqlx::query_as(
        "SELECT sender_channel_identity_id, content FROM ( \
             SELECT sender_channel_identity_id, content, created_at FROM messages \
             WHERE session_id = $1 ORDER BY created_at DESC LIMIT $2 \
         ) recent ORDER BY created_at ASC",
    )
    .bind(session_id)
    .bind(CHITCHAT_HISTORY_LIMIT)
    .fetch_all(&mut **conn)
    .await?;

    let messages: Vec<LlmMessage> = rows
        .into_iter()
        .map(|(sender, content)| LlmMessage {
            role: if sender.is_some() { LlmRole::User } else { LlmRole::Assistant },
            content: vec![ContentBlock::Text { text: content }],
        })
        .collect();

    let memories = try_retrieve_memories(conn, embedding_provider, user_id, text).await;

    let system_prompt = if memories.is_empty() {
        CHITCHAT_SYSTEM_PROMPT.to_string()
    } else {
        let mut prompt = format!("{CHITCHAT_SYSTEM_PROMPT}\n\nRelevant things you know about this user:\n");
        for m in &memories {
            prompt.push_str(&format!("- {}\n", m.content));
        }
        prompt
    };

    let request = LlmRequest {
        system: Some(system_prompt),
        messages,
        tools: vec![],
        max_tokens: CHITCHAT_MAX_TOKENS,
    };

    // The LLM call happens outside any DB transaction: holding a transaction open across a
    // slow network round trip would needlessly extend how long this connection's locks are held.
    let response = provider.complete(request).await.map_err(TurnError::LlmCallFailed)?;

    let reply_text = response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or_default();

    let mut tx = conn.begin().await?;

    let reply_message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2) RETURNING id",
    )
    .bind(session_id)
    .bind(&reply_text)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'ChitchatReply', $2)")
        .bind(session_id)
        .bind(serde_json::json!({
            "input_tokens": response.input_tokens,
            "output_tokens": response.output_tokens,
        }))
        .execute(&mut *tx)
        .await?;

    for m in &memories {
        sqlx::query("INSERT INTO message_memory_usage (message_id, memory_id) VALUES ($1, $2)")
            .bind(reply_message_id)
            .bind(m.id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    // Best-effort: extracting and storing a new memory from this exchange never affects the
    // turn's outcome — the user already has their reply by this point.
    memory::extract_and_store_memory(conn, provider, embedding_provider, user_id, text, &reply_text).await;

    Ok(reply_text)
}

async fn try_retrieve_memories(
    conn: &mut PoolConnection<Postgres>,
    embedding_provider: &dyn EmbeddingProvider,
    user_id: Uuid,
    text: &str,
) -> Vec<RetrievedMemory> {
    let embedding = match embedding_provider.embed(text).await {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    memory::retrieve_relevant_memories(conn, user_id, &embedding, MEMORY_RETRIEVAL_LIMIT)
        .await
        .unwrap_or_default()
}
```

```rust
// backend/src/turn/mod.rs — FULL FILE REPLACEMENT
pub mod bootstrap;
pub mod chitchat;
pub mod lock;
pub mod memory;
pub mod routing;
pub mod types;

pub use types::{TurnError, TurnOutcome};

use sqlx::PgPool;

use crate::embedding::EmbeddingProvider;
use crate::llm::LlmProvider;

pub async fn handle_inbound_message(
    pool: &PgPool,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
) -> Result<TurnOutcome, TurnError> {
    let bootstrap::BootstrapResult { user_id, sender_channel_identity_id, session_id, .. } =
        bootstrap::bootstrap_identity_and_session(pool, channel, chat_type, chat_id, sender_channel_user_id)
            .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    lock::insert_inbound_message(&mut conn, session_id, sender_channel_identity_id, text).await?;

    // Sub-agent routing is scaffolded but not yet implemented: nothing populates
    // agent_sessions in this plan, so every turn falls through to chitchat. See
    // docs/superpowers/specs/2026-07-26-orchestrator-turn-loop-design.md §2 step 5.
    let _active_agent_session_id =
        routing::find_active_agent_session(&mut conn, session_id, sender_channel_identity_id).await?;

    let result = chitchat::run_chitchat_turn(&mut conn, provider, embedding_provider, session_id, user_id, text).await;

    match result {
        Ok(reply) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply })
        }
        Err(err) => {
            // Best-effort: a failed event write here must never mask the original error.
            let _ = sqlx::query(
                "INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'TurnFailed', $2)",
            )
            .bind(session_id)
            .bind(serde_json::json!({"error": err.to_string()}))
            .execute(&mut *conn)
            .await;

            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

async fn release_lock_ignoring_errors(conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>, session_id: uuid::Uuid) {
    let _ = lock::release_session_lock(conn, session_id).await;
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_chitchat --test turn_handle_inbound_message`
Expected: PASS (6 + 5 tests)

- [ ] **Step 5: Commit**

```bash
git add backend/src/turn/chitchat.rs backend/src/turn/mod.rs backend/tests/turn_chitchat.rs backend/tests/turn_handle_inbound_message.rs
git commit -m "feat: wire memory retrieval and writing into the chitchat turn"
```

---

### Task 5: Reinforcement

**Files:**
- Modify: `backend/src/turn/memory.rs` (add `ReinforcementSignal`, `ReinforcementError`, `reinforce`)
- Test: `backend/tests/turn_reinforcement.rs`

**Interfaces:**
- Consumes: `message_memory_usage` table (Task 2).
- Produces: `ReinforcementSignal { Positive, Negative }`, `ReinforcementError { Db(#[from] sqlx::Error) }`, `async fn reinforce(pool: &PgPool, reply_message_id: Uuid, signal: ReinforcementSignal) -> Result<(), ReinforcementError>`. Not called by any other code in this plan.

- [ ] **Step 1: Write the failing tests**

```rust
// backend/tests/turn_reinforcement.rs
use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::turn::memory::{reinforce, ReinforcementSignal};

fn zero_embedding_literal() -> String {
    format!("[{}]", vec!["0.0"; 1536].join(","))
}

async fn seed_memory_linked_to_a_message(pool: &PgPool, weight: f64) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id).fetch_one(pool).await.unwrap();
    let message_id: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'reply') RETURNING id")
        .bind(session_id).fetch_one(pool).await.unwrap();
    let memory_id: Uuid = sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, embedding, weight) VALUES ($1, 'fact', $2::vector, $3) RETURNING id",
    )
    .bind(user_id)
    .bind(zero_embedding_literal())
    .bind(weight)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO message_memory_usage (message_id, memory_id) VALUES ($1, $2)")
        .bind(message_id)
        .bind(memory_id)
        .execute(pool)
        .await
        .unwrap();
    (message_id, memory_id)
}

#[sqlx::test]
async fn positive_signal_increases_weight(pool: PgPool) {
    let (message_id, memory_id) = seed_memory_linked_to_a_message(&pool, 1.0).await;

    reinforce(&pool, message_id, ReinforcementSignal::Positive).await.unwrap();

    let weight: f64 = sqlx::query_scalar("SELECT weight FROM memory_items WHERE id = $1")
        .bind(memory_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!((weight - 1.2).abs() < 1e-9);
}

#[sqlx::test]
async fn negative_signal_decreases_weight(pool: PgPool) {
    let (message_id, memory_id) = seed_memory_linked_to_a_message(&pool, 1.0).await;

    reinforce(&pool, message_id, ReinforcementSignal::Negative).await.unwrap();

    let weight: f64 = sqlx::query_scalar("SELECT weight FROM memory_items WHERE id = $1")
        .bind(memory_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!((weight - 0.8).abs() < 1e-9);
}

#[sqlx::test]
async fn weight_is_clamped_to_the_upper_bound_under_repeated_positive_reinforcement(pool: PgPool) {
    let (message_id, memory_id) = seed_memory_linked_to_a_message(&pool, 4.9).await;

    for _ in 0..10 {
        reinforce(&pool, message_id, ReinforcementSignal::Positive).await.unwrap();
    }

    let weight: f64 = sqlx::query_scalar("SELECT weight FROM memory_items WHERE id = $1")
        .bind(memory_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!((weight - 5.0).abs() < 1e-9);
}

#[sqlx::test]
async fn weight_is_clamped_to_the_lower_bound_under_repeated_negative_reinforcement(pool: PgPool) {
    let (message_id, memory_id) = seed_memory_linked_to_a_message(&pool, 0.15).await;

    for _ in 0..10 {
        reinforce(&pool, message_id, ReinforcementSignal::Negative).await.unwrap();
    }

    let weight: f64 = sqlx::query_scalar("SELECT weight FROM memory_items WHERE id = $1")
        .bind(memory_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!((weight - 0.1).abs() < 1e-9);
}

#[sqlx::test]
async fn a_message_with_no_linked_memories_is_a_no_op(pool: PgPool) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id).fetch_one(&pool).await.unwrap();
    let message_id: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'reply') RETURNING id")
        .bind(session_id).fetch_one(&pool).await.unwrap();

    let result = reinforce(&pool, message_id, ReinforcementSignal::Positive).await;
    assert!(result.is_ok());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd backend && cargo test --test turn_reinforcement`
Expected: FAIL to compile — `reinforce`/`ReinforcementSignal` don't exist yet.

- [ ] **Step 3: Write the implementation**

```rust
// backend/src/turn/memory.rs — ADD this import and this code at the end of the file
use sqlx::PgPool;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReinforcementSignal {
    Positive,
    Negative,
}

#[derive(Debug, thiserror::Error)]
pub enum ReinforcementError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub async fn reinforce(
    pool: &PgPool,
    reply_message_id: Uuid,
    signal: ReinforcementSignal,
) -> Result<(), ReinforcementError> {
    let factor: f64 = match signal {
        ReinforcementSignal::Positive => 1.2,
        ReinforcementSignal::Negative => 0.8,
    };

    sqlx::query(
        "UPDATE memory_items SET weight = LEAST(GREATEST(weight * $1, 0.1), 5.0) \
         WHERE id IN (SELECT memory_id FROM message_memory_usage WHERE message_id = $2)",
    )
    .bind(factor)
    .bind(reply_message_id)
    .execute(pool)
    .await?;

    Ok(())
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cd backend && cargo test --test turn_reinforcement`
Expected: PASS (5 tests)

- [ ] **Step 5: Run the full test suite**

Run: `cd backend && cargo test`
Expected: PASS, no regressions (all prior suites plus the new suites from this plan).

- [ ] **Step 6: Commit**

```bash
git add backend/src/turn/memory.rs backend/tests/turn_reinforcement.rs
git commit -m "feat: add reinforce() weight nudging for memory retrieval"
```
