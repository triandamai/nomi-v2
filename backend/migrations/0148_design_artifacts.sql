-- The previews Sora builds (nomi_agent_core::design_artifact): static pages for one direction.
-- Each save is a new version of the chat's previews; a version and its files are never updated.
-- `manifest` is the preview's pages and the elements feedback can point at; the files' contents
-- are kept apart so listing versions doesn't read them.
CREATE TABLE design_artifacts (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id          UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    agent_session_id    UUID NOT NULL,
    user_id             UUID NOT NULL REFERENCES users(id),
    direction_id        TEXT NOT NULL,
    directions_version  INT,
    version             INT NOT NULL,
    manifest            JSONB NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (session_id, version)
);

CREATE TABLE design_artifact_files (
    artifact_id   UUID NOT NULL REFERENCES design_artifacts(id) ON DELETE CASCADE,
    path          TEXT NOT NULL,
    content_type  TEXT NOT NULL,
    content       TEXT NOT NULL,
    PRIMARY KEY (artifact_id, path)
);

-- Where a version was published (nomi_agent_core::static_publish), by whichever provider is set
-- up. A version can be published again; the newest row is its address.
CREATE TABLE design_artifact_publications (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artifact_id  UUID NOT NULL REFERENCES design_artifacts(id) ON DELETE CASCADE,
    provider     TEXT NOT NULL,
    url          TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX design_artifact_publications_artifact_idx ON design_artifact_publications (artifact_id, created_at DESC);
