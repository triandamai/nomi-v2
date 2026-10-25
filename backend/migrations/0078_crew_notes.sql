-- The crew's shared notes for one chat: facts and decisions an agent working there wants the
-- others to know ("they want it in Indonesian", "budget is 5 million"). Every agent in the chat
-- reads the latest ones in its prompt.
CREATE TABLE crew_notes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    agent_type TEXT NOT NULL,
    note TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX crew_notes_session ON crew_notes (session_id, created_at DESC);
