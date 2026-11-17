-- The draft of a preview Sora is still building (nomi_agent_core::design_draft). A whole preview
-- rarely fits in one reply, so save_preview takes its files over several calls; until the last
-- one they wait here, per chat and direction, unseen by the person. The last call turns the
-- draft into a version in design_artifacts and clears it.
CREATE TABLE design_preview_drafts (
    session_id    UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    direction_id  TEXT NOT NULL,
    path          TEXT NOT NULL,
    content       TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (session_id, direction_id, path)
);
