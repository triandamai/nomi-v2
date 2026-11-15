-- How many times a shared link was opened by someone other than its owner (the owner's banner
-- says "3 views so far"), and what viewers flagged on a shared page (`POST /api/shared/:token/report`,
-- open without a sign-in). `reporter` is the signed-in person's id, or a hash of the address a
-- report came from; it's only used to rate-limit reports, so admins never see it.
ALTER TABLE chat_shares ADD COLUMN views BIGINT NOT NULL DEFAULT 0;

CREATE TABLE shared_page_reports (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    share_id    UUID NOT NULL REFERENCES chat_shares(id) ON DELETE CASCADE,
    reason      TEXT NOT NULL CHECK (reason IN ('spam', 'abuse', 'personal_info', 'illegal', 'other')),
    note        TEXT,
    reporter    TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX shared_page_reports_created_at_idx ON shared_page_reports (created_at DESC);
CREATE INDEX shared_page_reports_reporter_idx ON shared_page_reports (reporter, created_at DESC);
