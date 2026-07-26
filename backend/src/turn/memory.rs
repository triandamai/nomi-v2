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
