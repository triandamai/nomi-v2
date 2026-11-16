-- What the person says about a design preview in the Design Studio
-- (nomi_agent_core::design_feedback): about the whole design, a page, a section, a component,
-- one element, or how something behaves. `number` is its place in the chat's feedback. What was
-- said is never changed; a later preview version can only say it dealt with it.
CREATE TABLE design_feedback (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id           UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id              UUID NOT NULL REFERENCES users(id),
    number               INT NOT NULL,
    -- The preview version it was given on (design_artifacts.version of the same chat).
    artifact_version     INT NOT NULL,
    level                TEXT NOT NULL CHECK (level IN ('global', 'page', 'section', 'component', 'element', 'interaction')),
    page                 TEXT,
    element_id           TEXT,
    body                 TEXT NOT NULL,
    -- A point inside what it is about, each from 0 to 1.
    pin_x                REAL,
    pin_y                REAL,
    status               TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'resolved')),
    resolved_in_version  INT,
    resolution           TEXT,
    resolved_at          TIMESTAMPTZ,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (session_id, number)
);

CREATE INDEX design_feedback_open_idx ON design_feedback (session_id) WHERE status = 'open';
