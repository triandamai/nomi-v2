//! A chat's design brief in storage: every save is a new version, and old versions stay as
//! they were.

use nomi_agent_core::design_brief::{self, DesignBrief};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn chat(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    (session_id, user_id)
}

fn brief(audience: &str) -> DesignBrief {
    DesignBrief::parse(&json!({
        "company": [{"value": "Roti Pagi", "source": "confirmed"}],
        "audience": [{"value": audience, "source": "assumed"}],
        "objectives": [{"value": "Pre-orders", "source": "confirmed"}]
    }))
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_save_is_a_new_version_and_the_old_ones_are_kept(pool: PgPool) {
    let (session_id, user_id) = chat(&pool).await;
    let (other_chat, _) = chat(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(design_brief::latest(&mut conn, session_id).await.unwrap(), None);

    let first = brief("Neighbours");
    let second = brief("Office workers nearby");
    assert_eq!(design_brief::save(&mut conn, session_id, Uuid::new_v4(), user_id, &first).await.unwrap(), 1);
    assert_eq!(design_brief::save(&mut conn, session_id, Uuid::new_v4(), user_id, &second).await.unwrap(), 2);
    assert_eq!(design_brief::save(&mut conn, other_chat, Uuid::new_v4(), user_id, &first).await.unwrap(), 1, "versions are per chat");

    assert_eq!(design_brief::latest(&mut conn, session_id).await.unwrap(), Some((2, second)));
    assert_eq!(design_brief::version(&mut conn, session_id, 1).await.unwrap(), Some(first));
    assert_eq!(design_brief::version(&mut conn, session_id, 3).await.unwrap(), None);
}
