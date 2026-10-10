use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_server::routes::message_steps::{attach_steps, Source, Step};
use nomi_server::routes::sessions::MessageItem;

fn message(id: Uuid, sender: &str, minutes: i64) -> MessageItem {
    MessageItem {
        id,
        sender: sender.to_string(),
        content: String::new(),
        content_blocks: None,
        created_at: Utc::now() - Duration::minutes(60 - minutes),
        my_feedback: None,
        agent_display_name: None,
        memory_count: 0,
        steps: Vec::new(),
        sources: Vec::new(),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_reply_gets_the_steps_and_sources_that_led_to_it(pool: PgPool) {
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('A') RETURNING id").fetch_one(&pool).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', 'c') RETURNING id").bind(org).fetch_one(&pool).await.unwrap();
    let mut messages = vec![
        message(Uuid::new_v4(), "user", 0),
        message(Uuid::new_v4(), "assistant", 10),
        message(Uuid::new_v4(), "user", 20),
        message(Uuid::new_v4(), "assistant", 30),
    ];
    let at = |minutes: i64| messages[0].created_at + Duration::minutes(minutes);
    let events = [
        (at(2), json!({"tool_name": "web_search", "input": {"query": "cafes in Bandung"}, "result": "Results\n\n1. Kopi Nako\n   https://kopinako.id\n", "is_error": false})),
        (at(4), json!({"tool_name": "update_todos", "input": {}, "result": "", "is_error": false})),
        (at(6), json!({"tool_name": "read_web_page", "input": {"url": "https://kopinako.id"}, "result": "text", "is_error": false})),
        (at(25), json!({"tool_name": "run_command", "input": {"command": "npm run check"}, "result": "exit code 1", "is_error": true})),
    ];
    for (created_at, payload) in events {
        sqlx::query("INSERT INTO agent_events (session_id, event_type, payload, created_at) VALUES ($1, 'ToolCalled', $2, $3)")
            .bind(session)
            .bind(payload)
            .bind(created_at)
            .execute(&pool)
            .await
            .unwrap();
    }

    attach_steps(&pool, session, &mut messages).await.unwrap();

    assert_eq!(
        messages[1].steps,
        vec![
            Step { tool: "web_search".into(), detail: Some("cafes in Bandung".into()), ok: true },
            Step { tool: "read_web_page".into(), detail: Some("https://kopinako.id".into()), ok: true },
        ]
    );
    // Found by the search and read again: listed once.
    assert_eq!(messages[1].sources, vec![Source { title: Some("Kopi Nako".into()), url: "https://kopinako.id".into() }]);
    assert_eq!(messages[3].steps, vec![Step { tool: "run_command".into(), detail: Some("npm run check".into()), ok: false }]);
    assert!(messages[3].sources.is_empty() && messages[0].steps.is_empty() && messages[2].steps.is_empty());
}
