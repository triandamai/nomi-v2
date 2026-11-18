-- An approval decided with the person's edits and a rule they asked to remember, a deny that
-- can be taken back for a short while, and stopped work that can be discarded.

-- What the person changed on the card; the tool runs on this instead of what was proposed.
ALTER TABLE approval_resumes ADD COLUMN edited_input JSONB;
-- "Don't ask again" limited to one recipient: { scope: 'recipient', value: 'bu.sri@example.com' }.
ALTER TABLE approval_resumes ADD COLUMN remember_scope TEXT;
ALTER TABLE approval_resumes ADD COLUMN remember_value TEXT;
-- A deny waits until this moment before the turn carries on, so it can be undone meanwhile.
ALTER TABLE approval_resumes ADD COLUMN run_after TIMESTAMPTZ NOT NULL DEFAULT now();

-- A rule limited to calls whose scope (recipient) is this value; NULL covers the whole tool.
ALTER TABLE tool_permission_rules ADD COLUMN scope TEXT;
ALTER TABLE tool_permission_rules ADD COLUMN scope_value TEXT;

-- Stopped work the person discarded: its partial output is cleared and it is marked.
ALTER TABLE agent_delegations ADD COLUMN discarded_at TIMESTAMPTZ;
ALTER TABLE background_tasks ADD COLUMN discarded_at TIMESTAMPTZ;
