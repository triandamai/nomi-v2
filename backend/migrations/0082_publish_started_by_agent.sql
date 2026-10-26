-- Which crew member started a publish through publish_project ("Koda", "Rena"); null when a
-- person pressed Publish (or rolled back) themselves. created_by stays the person either way:
-- the agent publishes for them.
ALTER TABLE project_publishes ADD COLUMN started_by_agent TEXT;
