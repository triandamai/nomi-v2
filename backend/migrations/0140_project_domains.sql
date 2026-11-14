-- A project's own domains: hostnames its admins own (shop.example.com), the records to set at
-- their registrar (A or CNAME to the app's address, a TXT proving it's theirs), and what the
-- last DNS look found. One project per hostname across Nomi.
CREATE TABLE project_domains (
    project_id   UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    hostname     TEXT NOT NULL PRIMARY KEY,
    -- The value the TXT record at `_nomi-verify.<hostname>` must carry.
    token        TEXT NOT NULL,
    -- [{type, name, value, status: found | waiting}] as of the last look.
    records      JSONB NOT NULL DEFAULT '[]'::jsonb,
    -- When every record was first found; NULL while any is still waiting.
    verified_at  TIMESTAMPTZ,
    checked_at   TIMESTAMPTZ,
    is_primary   BOOLEAN NOT NULL DEFAULT false,
    -- Whether the hosting serves it over HTTPS yet.
    https        TEXT NOT NULL DEFAULT 'pending' CHECK (https IN ('pending', 'issued')),
    created_by   UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX project_domains_project ON project_domains (project_id, created_at);
-- The periodic look takes the oldest first.
CREATE INDEX project_domains_checked ON project_domains (checked_at NULLS FIRST);
