-- The chat's thinking level, picked in the composer. 'off' turns the model's reasoning off for
-- the chat's turns; low/medium/high map to each provider's own budget or effort setting (see
-- ReasoningEffort in nomi-llm). Medium is what every turn used before this existed.
ALTER TABLE sessions ADD COLUMN thinking_level TEXT NOT NULL DEFAULT 'medium'
    CHECK (thinking_level IN ('off', 'low', 'medium', 'high'));
