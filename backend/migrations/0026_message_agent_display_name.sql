-- backend/migrations/0026_message_agent_display_name.sql

ALTER TABLE messages ADD COLUMN agent_display_name TEXT;
