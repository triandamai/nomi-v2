-- Private storage for agents that don't ship their own tables: every dynamic agent, and any
-- future built-in that opts in (SubAgent::uses_records). Rows are scoped to one agent and one
-- user, and the engine only ever reads or writes the calling agent's own rows, so no agent can
-- break another's data. See nomi-agent-core's records.rs and docs/agents/storage.md.
CREATE TABLE agent_records (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_type  TEXT NOT NULL,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    collection  TEXT NOT NULL,
    data        JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX agent_records_owner_idx ON agent_records (agent_type, user_id, collection, updated_at DESC);
