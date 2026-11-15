-- Per-plan limits ("entitlements"). NULL always means unlimited, so plans that exist today behave
-- exactly as before until an admin sets a limit.
ALTER TABLE plans
    ADD COLUMN max_projects    INT    CHECK (max_projects IS NULL OR max_projects >= 0),
    ADD COLUMN max_orgs        INT    CHECK (max_orgs IS NULL OR max_orgs >= 0),
    ADD COLUMN max_org_members INT    CHECK (max_org_members IS NULL OR max_org_members >= 0),
    ADD COLUMN max_custom_crew INT    CHECK (max_custom_crew IS NULL OR max_custom_crew >= 0),
    ADD COLUMN max_storage_mb  BIGINT CHECK (max_storage_mb IS NULL OR max_storage_mb >= 0),
    -- A project may use a domain of its own (Project → Domains), on its owner's plan.
    ADD COLUMN custom_domain   BOOLEAN NOT NULL DEFAULT true,
    -- Sandbox size for the account's projects (small, default or large); NULL is the platform default.
    ADD COLUMN sandbox_size    TEXT    CHECK (sandbox_size IS NULL OR sandbox_size IN ('small', 'default', 'large')),
    -- Every admin-listed model is available; when false, only the ones in plan_models.
    ADD COLUMN all_models      BOOLEAN NOT NULL DEFAULT true;

CREATE TABLE plan_models (
    plan_id  UUID NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    model_id UUID NOT NULL REFERENCES admin_llm_models(id) ON DELETE CASCADE,
    PRIMARY KEY (plan_id, model_id)
);
