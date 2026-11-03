-- Where in a project each person is looking (the preview, the code and which file, the
-- terminal), so teammates can follow them. Overwritten with each heartbeat, like last_seen.
ALTER TABLE project_presence ADD COLUMN view TEXT, ADD COLUMN path TEXT;
