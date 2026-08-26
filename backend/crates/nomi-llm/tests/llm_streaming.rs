mod support;

use futures_util::stream;
use nomi_llm::{
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
