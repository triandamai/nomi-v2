use uuid::Uuid;

use nomi_llm::{PartialBlock, StopReason, StreamEvent};
use nomi_realtime::StreamEnvelope;

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
