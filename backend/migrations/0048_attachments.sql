-- Files people attach in chat. The file itself lives in the attachment store (S3 or disk, see
-- nomi_storage::blob) under `storage_key`; messages reference it by id
-- (<attachment id="…" …/>). `extracted_text` is what Nomi read out of it: the text of a document,
-- a description of an image, a transcript of audio or video. Text formats are read at upload;
-- the rest is read in the background by the files model (status pending → ready / failed).
CREATE TABLE attachments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id      UUID REFERENCES sessions(id) ON DELETE SET NULL,
    message_id      UUID REFERENCES messages(id) ON DELETE SET NULL,
    name            TEXT NOT NULL,
    mime            TEXT NOT NULL,
    kind            TEXT NOT NULL CHECK (kind IN ('text', 'pdf', 'document', 'spreadsheet', 'presentation', 'image', 'audio', 'voice', 'video', 'other')),
    size_bytes      BIGINT NOT NULL,
    storage_key     TEXT NOT NULL,
    -- A downscaled JPEG of an image: what the chat shows and what models are sent.
    preview_key     TEXT,
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'ready', 'failed')),
    extracted_text  TEXT,
    -- Pages, sheets, width/height, the browser's own transcript of a voice note, why reading failed.
    details         JSONB NOT NULL DEFAULT '{}',
    attempts        INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX attachments_user_created ON attachments (user_id, created_at DESC);
CREATE INDEX attachments_waiting ON attachments (created_at) WHERE status IN ('pending', 'processing');

-- Admin → Models: which model reads files the person's own model can't (at most one), and what
-- each model can take in. `media_inputs` NULL means "work it out from the provider and model id".
ALTER TABLE admin_llm_models ADD COLUMN is_files_model BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE admin_llm_models ADD COLUMN media_inputs TEXT[];
CREATE UNIQUE INDEX admin_llm_models_one_files_model ON admin_llm_models (is_files_model) WHERE is_files_model;
