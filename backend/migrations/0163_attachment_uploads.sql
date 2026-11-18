-- A file being sent in pieces (POST /api/attachments/uploads, then PATCH with Content-Range).
-- The bytes received so far are staged on the server's disk; this row says how many, what the
-- file is called and until when the upload may be picked up again. The attachment row is only
-- made when the last piece lands, by the same path a single-request upload takes.
CREATE TABLE attachment_uploads (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    claimed_mime    TEXT NOT NULL DEFAULT '',
    voice           BOOLEAN NOT NULL DEFAULT false,
    transcript      TEXT,
    replaces_attachment_id UUID REFERENCES attachments(id) ON DELETE SET NULL,
    -- Known from the first piece's Content-Range (or X-Upload-Length when the upload is created).
    total_bytes     BIGINT,
    received_bytes  BIGINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL
);

CREATE INDEX attachment_uploads_expiry ON attachment_uploads (expires_at);
