-- What the user decided on an approval card, so an identical tool call (same tool, same input)
-- later in the same chat reuses the decision instead of asking again. Models often repeat a call
-- after it was approved, or retry one that was denied; each used to raise a fresh card.
-- See permissions::check_tool_permission_in_session.
CREATE TABLE session_tool_decisions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id  UUID NOT NULL REFERENCES sessions(id),
    user_id     UUID NOT NULL REFERENCES users(id),
    tool_name   TEXT NOT NULL,
    input       JSONB NOT NULL,
    decision    TEXT NOT NULL CHECK (decision IN ('allow', 'deny')),
    decided_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX session_tool_decisions_lookup_idx ON session_tool_decisions (session_id, tool_name, decided_at DESC);
