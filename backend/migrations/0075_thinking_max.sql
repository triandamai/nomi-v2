-- The composer's thinking presets are Default (medium), Low, High and Max; thinking can no
-- longer be switched off, so chats that had it off start at the lightest level.
ALTER TABLE sessions DROP CONSTRAINT IF EXISTS sessions_thinking_level_check;
UPDATE sessions SET thinking_level = 'low' WHERE thinking_level = 'off';
ALTER TABLE sessions ADD CONSTRAINT sessions_thinking_level_check CHECK (thinking_level IN ('low', 'medium', 'high', 'max'));
