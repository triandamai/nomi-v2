-- What a published site reports about itself: each page load with where it came from (the
-- referrer's host; NULL for a direct visit) and each form sent. Grouped into sources on read.
CREATE TABLE site_hits (
    id             BIGSERIAL PRIMARY KEY,
    project_id     UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    kind           TEXT NOT NULL CHECK (kind IN ('visit', 'form')),
    referrer_host  TEXT,
    at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX site_hits_project_at ON site_hits (project_id, at);
