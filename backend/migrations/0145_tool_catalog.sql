-- The tool catalog as rows: one per tool a crew member can be given, kept in sync with the Rust
-- catalog at startup (nomi_agent_core::tool_finder::sync_catalog_table). `embedding` lets
-- find_tools find a tool by meaning, not only by its words; it is filled in the background and
-- redone whenever the tool's text (content_hash) or the embedding model changes.
-- This table is also where tools defined without Rust would be listed later.
CREATE TABLE tool_catalog_entries (
    name            TEXT PRIMARY KEY,
    description     TEXT NOT NULL,
    tool_group      TEXT NOT NULL,
    keywords        TEXT[] NOT NULL,
    content_hash    TEXT NOT NULL,
    embedding       VECTOR(1536) NULL,
    -- "provider/model" that made `embedding`: vectors from another model aren't comparable.
    embedding_model TEXT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
