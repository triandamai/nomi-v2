-- The chat a call was made in, when there was one, so a project's usage and spend can be read:
-- everything from its own chat and from the chats filed into it. Calls from before this have no
-- chat; a project counts from the first call that has one. A deleted chat keeps its calls (they
-- still count toward the person's month) and just stops counting toward any project.
ALTER TABLE llm_usage ADD COLUMN session_id UUID REFERENCES sessions(id) ON DELETE SET NULL;

CREATE INDEX llm_usage_session_time ON llm_usage (session_id, created_at) WHERE session_id IS NOT NULL;
-- When calls began to carry their chat ("counted since").
CREATE INDEX llm_usage_with_session_time ON llm_usage (created_at) WHERE session_id IS NOT NULL;
