CREATE TABLE memory_items (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id),
    content     TEXT NOT NULL,
    embedding   VECTOR(1536) NOT NULL,
    weight      DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX memory_items_embedding_idx
    ON memory_items USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

CREATE TABLE agent_events (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id                  UUID REFERENCES sessions(id),
    sender_channel_identity_id  UUID REFERENCES channel_identities(id),
    agent_session_id            UUID REFERENCES agent_sessions(id),
    agent_type                  TEXT,
    event_type                  TEXT NOT NULL,
    payload                     JSONB NOT NULL DEFAULT '{}',
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);
