-- A user asked the supervisor to stop agents. Running agent turns poll this between steps
-- (nomi-agent-core's stop::is_stop_requested): a turn that started before a matching request
-- ends at its next step. NULL session_id = every chat of the user; NULL agent_type = every agent.
-- exempt_agent_type keeps the supervisor turn that asked for the stop from stopping itself.
CREATE TABLE agent_stop_requests (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID NOT NULL REFERENCES users(id),
    session_id    UUID REFERENCES sessions(id),
    agent_type    TEXT,
    exempt_agent_type TEXT,
    requested_at  TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE INDEX agent_stop_requests_user_idx ON agent_stop_requests (user_id, requested_at DESC);
