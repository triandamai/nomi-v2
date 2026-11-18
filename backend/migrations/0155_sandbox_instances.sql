-- Every sandbox a project has had. A project still has one live sandbox (its project_sandboxes
-- row, which the worker drives); when the provider's sandbox behind it changes (it was deleted and
-- made again, or replaced), the old one ends here and the new one starts. Admin → Sandboxes lists
-- these, whatever their state. The trigger keeps them, so no code path can forget to.
CREATE TABLE sandbox_instances (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id  UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    provider    TEXT NOT NULL,
    sandbox_id  TEXT NOT NULL,
    -- The last state the project's sandbox row had while this was its sandbox.
    state       TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    ended_at    TIMESTAMPTZ
);

CREATE INDEX sandbox_instances_project_idx ON sandbox_instances (project_id, created_at DESC);
-- At most one live record per project, as there is one live sandbox.
CREATE UNIQUE INDEX sandbox_instances_one_open_idx ON sandbox_instances (project_id) WHERE ended_at IS NULL;

CREATE FUNCTION track_sandbox_instance() RETURNS trigger AS $$
BEGIN
    IF TG_OP = 'UPDATE' THEN
        IF OLD.sandbox_id IS NOT DISTINCT FROM NEW.sandbox_id THEN
            IF OLD.state IS DISTINCT FROM NEW.state THEN
                UPDATE sandbox_instances SET state = NEW.state, updated_at = clock_timestamp()
                 WHERE project_id = NEW.project_id AND ended_at IS NULL;
            END IF;
            RETURN NEW;
        END IF;
    END IF;
    -- The provider's sandbox changed (or the row is new): end the one before, start the new one.
    UPDATE sandbox_instances SET ended_at = clock_timestamp(), updated_at = clock_timestamp(),
           state = CASE WHEN NEW.sandbox_id IS NULL THEN NEW.state ELSE state END
     WHERE project_id = NEW.project_id AND ended_at IS NULL;
    IF NEW.sandbox_id IS NOT NULL THEN
        INSERT INTO sandbox_instances (project_id, provider, sandbox_id, state)
        VALUES (NEW.project_id, NEW.provider, NEW.sandbox_id, NEW.state);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER project_sandboxes_track_instance
    AFTER INSERT OR UPDATE OF sandbox_id, state ON project_sandboxes
    FOR EACH ROW EXECUTE FUNCTION track_sandbox_instance();

-- The sandboxes that exist now, each as its project's live record.
INSERT INTO sandbox_instances (project_id, provider, sandbox_id, state, created_at, updated_at)
SELECT project_id, provider, sandbox_id, state, created_at, updated_at FROM project_sandboxes WHERE sandbox_id IS NOT NULL;
