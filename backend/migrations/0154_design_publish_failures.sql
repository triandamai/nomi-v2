-- Why a design preview version couldn't be published (nomi_agent_core::static_publish): no
-- provider set up, or the provider refused it. Kept beside design_artifact_publications so the
-- Design Studio can say a version isn't published, and why, until a later publish succeeds.
CREATE TABLE design_artifact_publish_failures (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artifact_id  UUID NOT NULL REFERENCES design_artifacts(id) ON DELETE CASCADE,
    reason       TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX design_artifact_publish_failures_artifact_idx ON design_artifact_publish_failures (artifact_id, created_at DESC);
