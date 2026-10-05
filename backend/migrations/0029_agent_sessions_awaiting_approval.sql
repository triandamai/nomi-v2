-- A turn with no agent_sessions row of its own (the default agent, or a delegated turn) that
-- pauses for tool approval parks its paused state in a dedicated row with this status — not
-- 'active', so it never collides with agent_sessions_one_active_per_speaker. See
-- resolve_tool_batch in nomi-agent-core's engine.rs.
ALTER TABLE agent_sessions DROP CONSTRAINT agent_sessions_status_check;
ALTER TABLE agent_sessions ADD CONSTRAINT agent_sessions_status_check
    CHECK (status IN ('active', 'completed', 'cancelled', 'expired', 'awaiting_approval'));
