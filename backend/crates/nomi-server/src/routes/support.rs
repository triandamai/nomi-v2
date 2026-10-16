//! Nomi as a customer support agent for other products. A product calls
//! `POST /api/support/v1/ask` with a support key; Nomi answers from the knowledge base (and
//! nothing else). When the knowledge base doesn't cover the question, it says so politely and
//! drafts an issue for an admin: the question, how often it comes up, and a suggested article.
//!
//! Admins manage articles, drafts, keys and the question log under `/api/admin/support/*`.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_llm::{ContentBlock, LlmMessage, LlmRequest, LlmRole};

use crate::app::AppState;

type ApiError = (StatusCode, String);

const MAX_QUESTION_CHARS: usize = 2000;
const ARTICLES_PER_ANSWER: i64 = 5;
const ARTICLE_CHARS: usize = 4000;

fn internal<E: std::fmt::Display>(context: &'static str) -> impl Fn(E) -> ApiError {
    move |e| {
        tracing::error!(error = %e, "{context}");
        (StatusCode::INTERNAL_SERVER_ERROR, context.to_string())
    }
}

fn admin(claims: &nomi_auth::claims::Claims, level: &str) -> Result<(), ApiError> {
    crate::routes::admin_access::require_admin_owned(claims, "support", level)
}

fn hash_key(key: &str) -> Vec<u8> {
    Sha256::digest(key.as_bytes()).to_vec()
}

/// The words of `text` as an OR full-text query (`refund | invoice | ...`); None when nothing
/// searchable is left. Only letters and digits reach the query, so it can't be malformed.
pub fn or_query(text: &str) -> Option<String> {
    let mut words: Vec<String> = Vec::new();
    for word in text.split(|c: char| !c.is_alphanumeric()) {
        let word = word.to_lowercase();
        if word.chars().count() >= 3 && !words.contains(&word) {
            words.push(word);
        }
        if words.len() >= 24 {
            break;
        }
    }
    (!words.is_empty()).then(|| words.join(" | "))
}

#[derive(Debug, sqlx::FromRow)]
struct FoundArticle {
    id: Uuid,
    title: String,
    body: String,
}

async fn find_articles(pool: &sqlx::PgPool, question: &str) -> Result<Vec<FoundArticle>, sqlx::Error> {
    let Some(query) = or_query(question) else { return Ok(Vec::new()) };
    sqlx::query_as(
        "SELECT a.id, a.title, a.body FROM kb_articles a, to_tsquery('simple', $1) q \
         WHERE a.published AND a.search @@ q ORDER BY ts_rank(a.search, q) DESC LIMIT $2",
    )
    .bind(query)
    .bind(ARTICLES_PER_ANSWER)
    .fetch_all(pool)
    .await
}

#[derive(Deserialize)]
pub struct HistoryTurn {
    /// "customer" or "assistant".
    pub role: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct AskRequest {
    pub question: String,
    /// Who is asking, in the product's own terms (kept with the question for the admin).
    pub customer: Option<String>,
    /// The conversation so far, oldest first.
    #[serde(default)]
    pub history: Vec<HistoryTurn>,
}

#[derive(Serialize)]
pub struct Source {
    pub id: Uuid,
    pub title: String,
}

#[derive(Serialize)]
pub struct AskResponse {
    pub answer: String,
    /// False when the knowledge base didn't cover it (an admin was handed the question).
    pub answered: bool,
    pub sources: Vec<Source>,
    pub question_id: Uuid,
}

/// What the model is asked to reply with.
#[derive(Deserialize, Default)]
struct ModelReply {
    #[serde(default)]
    answered: bool,
    #[serde(default)]
    answer: String,
    #[serde(default)]
    used: Vec<usize>,
    #[serde(default)]
    draft_title: String,
    #[serde(default)]
    draft_body: String,
}

/// The JSON object in the model's text (it may wrap it in a code fence or a sentence).
fn parse_reply(text: &str) -> Option<ModelReply> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    serde_json::from_str(text.get(start..=end)?).ok()
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        format!("{}…", text.chars().take(max).collect::<String>())
    }
}

fn system_prompt(product: &str, articles: &[FoundArticle]) -> String {
    let product = if product.trim().is_empty() { "this product" } else { product.trim() };
    let mut kb = String::new();
    for (i, a) in articles.iter().enumerate() {
        kb.push_str(&format!("\n[{}] {}\n{}\n", i + 1, a.title, clip(&a.body, ARTICLE_CHARS)));
    }
    if kb.is_empty() {
        kb.push_str("\n(no articles matched)\n");
    }
    format!(
        "You are the customer support assistant for {product}. Answer ONLY from the knowledge base articles below; \
         never guess, invent policies, prices, steps or links. Write in the customer's language, friendly and short.\n\n\
         If the articles answer the question, reply with answered true and the answer.\n\
         If they don't (or only partly), reply with answered false and an answer that says plainly you don't have that \
         documented yet and the team has been told, without making anything up. Then draft a knowledge base article \
         for the team: draft_title (a short how-to style title) and draft_body (an outline of what the article should \
         cover, with open questions for the team marked TODO).\n\n\
         Reply with only this JSON: {{\"answered\": bool, \"answer\": string, \"used\": [article numbers], \
         \"draft_title\": string, \"draft_body\": string}}\n\n## Knowledge base{kb}"
    )
}

async fn key_from_headers(pool: &sqlx::PgPool, headers: &HeaderMap) -> Result<(Uuid, String, Uuid), ApiError> {
    let key = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .ok_or((StatusCode::UNAUTHORIZED, "missing support key".to_string()))?;
    let row: Option<(Uuid, String, Uuid)> =
        sqlx::query_as("SELECT id, product_name, created_by FROM support_keys WHERE key_hash = $1 AND revoked_at IS NULL")
            .bind(hash_key(key))
            .fetch_optional(pool)
            .await
            .map_err(internal("failed to check the support key"))?;
    let row = row.ok_or((StatusCode::UNAUTHORIZED, "invalid support key".to_string()))?;
    let _ = sqlx::query("UPDATE support_keys SET last_used_at = now() WHERE id = $1").bind(row.0).execute(pool).await;
    Ok(row)
}

/// Adds the question to an open draft asking the same thing, or opens a new one.
async fn draft_gap(pool: &sqlx::PgPool, question: &str, title: &str, body: &str) -> Result<Uuid, sqlx::Error> {
    let normalized = question.trim().to_lowercase();
    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM kb_drafts WHERE status = 'open' AND lower(trim(question)) = $1 ORDER BY last_asked_at DESC LIMIT 1",
    )
    .bind(&normalized)
    .fetch_optional(pool)
    .await?;
    if let Some(id) = existing {
        sqlx::query("UPDATE kb_drafts SET times_asked = times_asked + 1, last_asked_at = now() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        return Ok(id);
    }
    let title = if title.trim().is_empty() { clip(question.trim(), 120) } else { clip(title.trim(), 200) };
    sqlx::query_scalar("INSERT INTO kb_drafts (question, title, body) VALUES ($1, $2, $3) RETURNING id")
        .bind(question.trim())
        .bind(title)
        .bind(body.trim())
        .fetch_one(pool)
        .await
}

/// `POST /api/support/v1/ask` (support key in `Authorization: Bearer nsk_...`).
pub async fn ask(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<AskRequest>) -> Result<Json<AskResponse>, ApiError> {
    let (key_id, product, owner) = key_from_headers(&state.pool, &headers).await?;
    let question = req.question.trim();
    if question.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "question is empty".to_string()));
    }
    if question.chars().count() > MAX_QUESTION_CHARS {
        return Err((StatusCode::BAD_REQUEST, format!("questions are at most {MAX_QUESTION_CHARS} characters")));
    }

    let articles = find_articles(&state.pool, question).await.map_err(internal("failed to search the knowledge base"))?;

    let mut messages: Vec<LlmMessage> = req
        .history
        .iter()
        .rev()
        .take(10)
        .rev()
        .filter(|t| !t.text.trim().is_empty())
        .map(|t| LlmMessage {
            role: if t.role == "assistant" { LlmRole::Assistant } else { LlmRole::User },
            content: vec![ContentBlock::Text { text: clip(&t.text, MAX_QUESTION_CHARS) }],
        })
        .collect();
    // The conversation must start with the customer.
    while messages.first().is_some_and(|m| m.role == LlmRole::Assistant) {
        messages.remove(0);
    }
    messages.push(LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: question.to_string() }] });

    let provider = crate::bootstrap::build_llm_provider_for(
        &state.pool,
        owner,
        &state.settings_key,
        state.http_client.clone(),
        crate::bootstrap::ModelPurpose::Chat,
    )
    .await;
    let request = LlmRequest {
        system: Some(system_prompt(&product, &articles)),
        messages,
        tools: Vec::new(),
        max_tokens: 1200,
        enable_reasoning: false,
        reasoning_effort: nomi_llm::ReasoningEffort::Low,
    };
    let response = nomi_llm::complete(provider.as_ref(), request).await.map_err(|e| {
        tracing::warn!(error = %e, "support answer failed");
        (StatusCode::BAD_GATEWAY, "the assistant couldn't answer right now; try again shortly".to_string())
    })?;
    let text: String = response
        .content
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();

    // Without the JSON the model was asked for, only a reply grounded in found articles counts.
    let reply = parse_reply(&text).unwrap_or_else(|| ModelReply {
        answered: !articles.is_empty(),
        answer: text.trim().to_string(),
        used: (1..=articles.len()).collect(),
        ..ModelReply::default()
    });
    let answered = reply.answered && !articles.is_empty();
    let answer = if reply.answer.trim().is_empty() {
        "I don't have that documented yet, so I've passed your question to the team.".to_string()
    } else {
        reply.answer.trim().to_string()
    };
    let used: Vec<&FoundArticle> = reply.used.iter().filter_map(|n| n.checked_sub(1).and_then(|i| articles.get(i))).collect();
    let sources: Vec<Source> = if answered { used.iter().map(|a| Source { id: a.id, title: a.title.clone() }).collect() } else { Vec::new() };

    let question_id: Uuid = sqlx::query_scalar(
        "INSERT INTO support_questions (key_id, question, answer, answered, article_ids, customer) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(key_id)
    .bind(question)
    .bind(&answer)
    .bind(answered)
    .bind(sources.iter().map(|s| s.id).collect::<Vec<_>>())
    .bind(req.customer.as_deref().map(|c| clip(c, 200)))
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to record the question"))?;

    if !answered {
        draft_gap(&state.pool, question, &reply.draft_title, &reply.draft_body)
            .await
            .map_err(internal("failed to draft the knowledge base issue"))?;
    }

    Ok(Json(AskResponse { answer, answered, sources, question_id }))
}

// ---- Admin: articles ----

#[derive(Serialize, sqlx::FromRow)]
pub struct Article {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub q: Option<String>,
}

/// `GET /api/admin/support/articles?q=`
pub async fn list_articles(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Query(query): Query<ListQuery>) -> Result<Json<Vec<Article>>, ApiError> {
    admin(&claims, "view")?;
    let articles = match query.q.as_deref().and_then(or_query) {
        Some(q) => sqlx::query_as(
            "SELECT a.id, a.title, a.body, a.tags, a.published, a.created_at, a.updated_at FROM kb_articles a, to_tsquery('simple', $1) q \
             WHERE a.search @@ q ORDER BY ts_rank(a.search, q) DESC LIMIT 200",
        )
        .bind(q)
        .fetch_all(&state.pool)
        .await,
        None => sqlx::query_as("SELECT id, title, body, tags, published, created_at, updated_at FROM kb_articles ORDER BY updated_at DESC LIMIT 200")
            .fetch_all(&state.pool)
            .await,
    }
    .map_err(internal("failed to list articles"))?;
    Ok(Json(articles))
}

#[derive(Deserialize)]
pub struct ArticleInput {
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_true")]
    pub published: bool,
}

fn default_true() -> bool {
    true
}

fn check_article(input: &ArticleInput) -> Result<(), ApiError> {
    let len = input.title.trim().chars().count();
    if len == 0 || len > 200 {
        return Err((StatusCode::BAD_REQUEST, "titles are 1–200 characters".to_string()));
    }
    Ok(())
}

/// `POST /api/admin/support/articles`
pub async fn create_article(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(input): Json<ArticleInput>) -> Result<(StatusCode, Json<Article>), ApiError> {
    admin(&claims, "write")?;
    check_article(&input)?;
    let article = sqlx::query_as(
        "INSERT INTO kb_articles (title, body, tags, published, created_by) VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, title, body, tags, published, created_at, updated_at",
    )
    .bind(input.title.trim())
    .bind(&input.body)
    .bind(&input.tags)
    .bind(input.published)
    .bind(claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to save the article"))?;
    Ok((StatusCode::CREATED, Json(article)))
}

/// `PUT /api/admin/support/articles/:id`
pub async fn update_article(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
    Json(input): Json<ArticleInput>,
) -> Result<Json<Article>, ApiError> {
    admin(&claims, "write")?;
    check_article(&input)?;
    let article = sqlx::query_as(
        "UPDATE kb_articles SET title = $2, body = $3, tags = $4, published = $5, updated_at = now() WHERE id = $1 \
         RETURNING id, title, body, tags, published, created_at, updated_at",
    )
    .bind(id)
    .bind(input.title.trim())
    .bind(&input.body)
    .bind(&input.tags)
    .bind(input.published)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to save the article"))?
    .ok_or((StatusCode::NOT_FOUND, "article not found".to_string()))?;
    Ok(Json(article))
}

/// `DELETE /api/admin/support/articles/:id`
pub async fn delete_article(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    admin(&claims, "manage")?;
    sqlx::query("DELETE FROM kb_articles WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to delete the article"))?;
    Ok(StatusCode::NO_CONTENT)
}

// ---- Admin: drafts ----

#[derive(Serialize, sqlx::FromRow)]
pub struct Draft {
    pub id: Uuid,
    pub question: String,
    pub title: String,
    pub body: String,
    pub status: String,
    pub times_asked: i32,
    pub last_asked_at: DateTime<Utc>,
    pub article_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct DraftsQuery {
    pub status: Option<String>,
}

/// `GET /api/admin/support/drafts?status=open`
pub async fn list_drafts(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Query(query): Query<DraftsQuery>) -> Result<Json<Vec<Draft>>, ApiError> {
    admin(&claims, "view")?;
    let status = query.status.unwrap_or_else(|| "open".to_string());
    let drafts = sqlx::query_as(
        "SELECT id, question, title, body, status, times_asked, last_asked_at, article_id, created_at FROM kb_drafts \
         WHERE status = $1 ORDER BY times_asked DESC, last_asked_at DESC LIMIT 200",
    )
    .bind(status)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list drafts"))?;
    Ok(Json(drafts))
}

/// `POST /api/admin/support/drafts/:id/publish`: turns the draft into an article (with the
/// admin's edits) and closes it.
pub async fn publish_draft(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
    Json(input): Json<ArticleInput>,
) -> Result<Json<Article>, ApiError> {
    admin(&claims, "write")?;
    check_article(&input)?;
    let mut tx = state.pool.begin().await.map_err(internal("failed to publish the draft"))?;
    let article: Article = sqlx::query_as(
        "INSERT INTO kb_articles (title, body, tags, published, created_by) VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, title, body, tags, published, created_at, updated_at",
    )
    .bind(input.title.trim())
    .bind(&input.body)
    .bind(&input.tags)
    .bind(input.published)
    .bind(claims.sub)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal("failed to publish the draft"))?;
    let updated = sqlx::query("UPDATE kb_drafts SET status = 'published', article_id = $2 WHERE id = $1 AND status = 'open'")
        .bind(id)
        .bind(article.id)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to publish the draft"))?;
    if updated.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "draft not found or already handled".to_string()));
    }
    tx.commit().await.map_err(internal("failed to publish the draft"))?;
    Ok(Json(article))
}

/// `POST /api/admin/support/drafts/:id/dismiss`
pub async fn dismiss_draft(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    admin(&claims, "write")?;
    sqlx::query("UPDATE kb_drafts SET status = 'dismissed' WHERE id = $1 AND status = 'open'")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to dismiss the draft"))?;
    Ok(StatusCode::NO_CONTENT)
}

// ---- Admin: keys and questions ----

#[derive(Serialize, sqlx::FromRow)]
pub struct SupportKey {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub product_name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// `GET /api/admin/support/keys`
pub async fn list_keys(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Vec<SupportKey>>, ApiError> {
    admin(&claims, "view")?;
    let keys = sqlx::query_as(
        "SELECT id, name, prefix, product_name, created_at, last_used_at, revoked_at FROM support_keys ORDER BY revoked_at NULLS FIRST, created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list keys"))?;
    Ok(Json(keys))
}

#[derive(Deserialize)]
pub struct KeyInput {
    pub name: String,
    #[serde(default)]
    pub product_name: String,
}

#[derive(Serialize)]
pub struct CreatedKey {
    #[serde(flatten)]
    pub key: SupportKey,
    /// The key itself: shown this once.
    pub secret: String,
}

/// `POST /api/admin/support/keys`: a new key for a product (shown once).
pub async fn create_key(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(input): Json<KeyInput>) -> Result<(StatusCode, Json<CreatedKey>), ApiError> {
    admin(&claims, "manage")?;
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err((StatusCode::BAD_REQUEST, "names are 1–80 characters".to_string()));
    }
    let mut bytes = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut bytes);
    let secret = format!("nsk_{}", hex::encode(bytes));
    let prefix: String = secret.chars().take(10).collect();
    let key: SupportKey = sqlx::query_as(
        "INSERT INTO support_keys (name, key_hash, prefix, product_name, created_by) VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, name, prefix, product_name, created_at, last_used_at, revoked_at",
    )
    .bind(name)
    .bind(hash_key(&secret))
    .bind(prefix)
    .bind(input.product_name.trim())
    .bind(claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to create the key"))?;
    Ok((StatusCode::CREATED, Json(CreatedKey { key, secret })))
}

/// `DELETE /api/admin/support/keys/:id`: revokes it.
pub async fn revoke_key(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    admin(&claims, "manage")?;
    sqlx::query("UPDATE support_keys SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to revoke the key"))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, sqlx::FromRow)]
pub struct AskedQuestion {
    pub id: Uuid,
    pub question: String,
    pub answer: String,
    pub answered: bool,
    pub customer: Option<String>,
    pub key_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// `GET /api/admin/support/questions`: the newest questions and how they were answered.
pub async fn list_questions(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Vec<AskedQuestion>>, ApiError> {
    admin(&claims, "view")?;
    let questions = sqlx::query_as(
        "SELECT q.id, q.question, q.answer, q.answered, q.customer, k.name AS key_name, q.created_at \
         FROM support_questions q LEFT JOIN support_keys k ON k.id = q.key_id ORDER BY q.created_at DESC LIMIT 200",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list questions"))?;
    Ok(Json(questions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn or_queries_keep_only_words() {
        assert_eq!(or_query("How do I get a refund?! (refund)").as_deref(), Some("how | get | refund"));
        assert_eq!(or_query("a & b | !!"), None);
        assert_eq!(or_query("Bagaimana cara reset kata sandi").as_deref(), Some("bagaimana | cara | reset | kata | sandi"));
    }

    #[test]
    fn replies_are_read_from_wrapped_json() {
        let reply = parse_reply("Sure:\n```json\n{\"answered\": true, \"answer\": \"Go to Settings.\", \"used\": [1]}\n```").unwrap();
        assert!(reply.answered);
        assert_eq!(reply.answer, "Go to Settings.");
        assert_eq!(reply.used, vec![1]);
        assert!(parse_reply("no json here").is_none());
    }
}
