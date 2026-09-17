-- backend/migrations/0025_dynamic_agents_and_phase.sql

CREATE TABLE dynamic_agents (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                TEXT NOT NULL,
    system_prompt       TEXT NOT NULL,
    intent_label        TEXT NOT NULL UNIQUE,
    intent_description  TEXT NOT NULL,
    granted_tools       TEXT[] NOT NULL DEFAULT '{}',
    supports_todos      BOOLEAN NOT NULL DEFAULT false,
    supports_plans      BOOLEAN NOT NULL DEFAULT false,
    can_delegate        BOOLEAN NOT NULL DEFAULT false,
    is_active           BOOLEAN NOT NULL DEFAULT true,
    created_by          UUID NOT NULL REFERENCES users(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_dynamic_agents_active ON dynamic_agents (is_active);

ALTER TABLE agent_sessions ADD COLUMN current_phase TEXT NOT NULL DEFAULT 'waiting';
ALTER TABLE agent_sessions ADD COLUMN current_phase_detail TEXT;
