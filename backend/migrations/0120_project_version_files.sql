-- A project's versions (migration 0107), each with the files it held, so any one can be restored.
-- File bytes are kept once per content in project storage (`versions/<project>/<sha256>`); a
-- version is the list of paths and the content each pointed at.

-- The content the file holds now. Null for files saved before this, until the project's next change
-- keeps them (nomi_agent_coding::files::keep_blobs).
ALTER TABLE project_files ADD COLUMN blob_sha TEXT;

CREATE TABLE project_versions (
    project_id    UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    version       BIGINT NOT NULL,
    -- The crew member whose reply made it (agent type), or null for a person's own change.
    agent         TEXT,
    -- The person whose change it was (the code editor, an import, a restore), when known.
    user_id       UUID REFERENCES users(id) ON DELETE SET NULL,
    -- The version this one put back, when it was a restore.
    restored_from BIGINT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, version)
);

CREATE TABLE project_version_files (
    project_id   UUID NOT NULL,
    version      BIGINT NOT NULL,
    path         TEXT NOT NULL,
    blob_sha     TEXT,
    content_type TEXT NOT NULL,
    size_bytes   INTEGER NOT NULL,
    PRIMARY KEY (project_id, version, path),
    FOREIGN KEY (project_id, version) REFERENCES project_versions (project_id, version) ON DELETE CASCADE
);
