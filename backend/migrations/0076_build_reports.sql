-- A check only says something about the files it ran on: each command remembers the version of
-- the project's files it ran against (NULL for runs from before this was recorded).
ALTER TABLE project_runs ADD COLUMN files_version BIGINT;

-- What a delegated build showed when it ended (see nomi_agent_coding::report): its changed files
-- and the checks that ran on the final files. NULL for other agents and older delegations.
ALTER TABLE agent_delegations ADD COLUMN report JSONB;
