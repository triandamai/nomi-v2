use uuid::Uuid;

use nomi_llm::{PartialBlock, StopReason, StreamEvent};
use nomi_realtime::{StreamEnvelope, UserEnvelope};

#[test]
fn a_delta_envelope_round_trips_through_json() {
    let turn_job_id = Uuid::new_v4();
    let envelope = StreamEnvelope::Delta {
        turn_job_id,
        event: StreamEvent::ContentBlockStart { index: 0, block: PartialBlock::Text },
    };

    let json = serde_json::to_string(&envelope).unwrap();
    assert!(json.contains("\"kind\":\"Delta\""));

    let round_tripped: StreamEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(round_tripped, envelope);
}

#[test]
fn a_turn_completed_envelope_round_trips_through_json() {
    let envelope = StreamEnvelope::TurnCompleted { turn_job_id: Uuid::new_v4(), message_id: Uuid::new_v4() };
    let json = serde_json::to_string(&envelope).unwrap();
    let round_tripped: StreamEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(round_tripped, envelope);
}

#[test]
fn a_turn_failed_envelope_round_trips_through_json() {
    let envelope = StreamEnvelope::TurnFailed { turn_job_id: Uuid::new_v4(), error: "boom".to_string() };
    let json = serde_json::to_string(&envelope).unwrap();
    let round_tripped: StreamEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(round_tripped, envelope);
}

#[test]
fn a_text_delta_event_carries_its_text_through_json() {
    let envelope = StreamEnvelope::Delta {
        turn_job_id: Uuid::new_v4(),
        event: StreamEvent::Done { stop_reason: StopReason::EndTurn, input_tokens: 3, output_tokens: 2 },
    };
    let json = serde_json::to_string(&envelope).unwrap();
    let round_tripped: StreamEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(round_tripped, envelope);
}

#[test]
fn deletes_go_out_in_the_shape_the_apps_read() {
    let id = Uuid::nil();
    let deleted = serde_json::to_value(StreamEnvelope::MessagesDeleted { message_ids: vec![id] }).unwrap();
    assert_eq!(deleted, serde_json::json!({ "kind": "MessagesDeleted", "message_ids": [id] }));
    assert_eq!(serde_json::to_value(StreamEnvelope::SessionDeleted).unwrap(), serde_json::json!({ "kind": "SessionDeleted" }));
    let chat = serde_json::to_value(UserEnvelope::ChatDeleted { session_id: id }).unwrap();
    assert_eq!(chat, serde_json::json!({ "kind": "ChatDeleted", "session_id": id }));
    let left = serde_json::to_value(UserEnvelope::MessagesDeleted { session_id: id, message_ids: vec![id] }).unwrap();
    assert_eq!(left, serde_json::json!({ "kind": "MessagesDeleted", "session_id": id, "message_ids": [id] }));
    let edited = serde_json::to_value(UserEnvelope::MessageUpdated { session_id: id, message_id: id }).unwrap();
    assert_eq!(edited, serde_json::json!({ "kind": "MessageUpdated", "session_id": id, "message_id": id }));
}
