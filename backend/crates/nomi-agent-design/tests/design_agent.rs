//! Sora through the engine's eyes: a tool she doesn't have is refused, and without a provider
//! nothing is connected.

use nomi_agent_core::SubAgent;
use nomi_agent_design::DesignAgent;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn a_tool_sora_does_not_have_is_refused_by_name(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let refused = DesignAgent::new().execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "publish_preview", json!({})).await;
    assert_eq!(refused.err().as_deref(), Some("unknown tool: publish_preview"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_a_provider_listing_says_nothing_is_connected(pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();
    let listed = DesignAgent::new().execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "list_connected_tools", json!({})).await.unwrap();
    assert_eq!(listed.display_text, nomi_agent_core::mcp_ops::NOTHING_CONNECTED);
    let failed = DesignAgent::new().execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "use_connected_tool", json!({"server": "Refs", "tool": "search"})).await;
    assert!(failed.unwrap_err().contains("aren't available"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn sora_starts_from_the_chats_newest_brief_when_there_is_one(pool: PgPool) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(DesignAgent::new().domain_context(&mut conn, None, session_id, user_id, &[]).await, None, "no brief yet");

    for audience in ["Neighbours", "Office workers nearby"] {
        let brief = nomi_agent_core::design_brief::DesignBrief::parse(&json!({
            "company": [{"value": "Roti Pagi", "source": "confirmed"}],
            "audience": [{"value": audience, "source": "assumed"}],
            "objectives": [{"value": "Pre-orders", "source": "confirmed"}]
        }))
        .unwrap();
        nomi_agent_core::design_brief::save(&mut conn, session_id, Uuid::new_v4(), user_id, &brief).await.unwrap();
    }
    let context = DesignAgent::new().domain_context(&mut conn, None, session_id, user_id, &[]).await.expect("a brief");
    assert!(context.contains("# Design brief (version 2)") && context.contains("Office workers nearby") && !context.contains("Neighbours"));
}

fn two_directions() -> serde_json::Value {
    let direction = |id: &str, name: &str, accent: &str, font: &str| {
        json!({
            "id": id, "name": name,
            "summary": "What it looks like.", "rationale": "Why it fits.", "trade_offs": ["What it costs"],
            "palette": [{"name": "Paper", "hex": "#F6EFE4"}, {"name": "Accent", "hex": accent}],
            "typography": {"heading": font, "body": "Inter"},
            "layout": "One long page."
        })
    };
    json!({"directions": [direction("editorial", "Editorial", "#B5532A", "Fraunces"), direction("swiss", "Swiss utility", "#1F4FFF", "Helvetica")]})
}

#[sqlx::test(migrations = "../../migrations")]
async fn proposing_saves_a_version_shows_a_choice_and_refuses_a_set_that_is_not_valid(pool: PgPool) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let sora = DesignAgent::new();

    let brief = nomi_agent_core::design_brief::DesignBrief::parse(&json!({
        "company": [{"value": "Roti Pagi", "source": "confirmed"}],
        "audience": [{"value": "Neighbours", "source": "assumed"}],
        "objectives": [{"value": "Pre-orders", "source": "confirmed"}]
    }))
    .unwrap();
    nomi_agent_core::design_brief::save(&mut conn, session_id, Uuid::new_v4(), user_id, &brief).await.unwrap();

    let proposed = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", two_directions()).await.unwrap();
    assert!(proposed.display_text.starts_with("Directions saved as version 1, from design brief version 1:"));
    let Some(nomi_agent_core::ContentBlock::View(nomi_agent_core::View::Decision(choice))) = proposed.block else { panic!("a choice for the person") };
    assert_eq!(choice.options.len(), 2);

    let one = json!({"directions": [two_directions()["directions"][0].clone()]});
    let refused = sora.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "propose_directions", one).await.unwrap_err();
    assert!(refused.starts_with("propose 2 to 4 directions"));
    let (version, brief_version, set) = nomi_agent_core::design_directions::latest(&mut conn, session_id).await.unwrap().unwrap();
    assert_eq!((version, brief_version, set.directions.len()), (1, Some(1), 2), "a refused set saves nothing");

    let context = sora.domain_context(&mut conn, None, session_id, user_id, &[]).await.unwrap();
    assert!(context.contains("# Design brief (version 1)") && context.contains("# Design directions (version 1)"));
}
