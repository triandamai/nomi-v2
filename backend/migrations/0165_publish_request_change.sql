-- What a request to publish holds: the ask (turn job) it follows and the files changed since the
-- live deploy, as they were when it was asked (the change it names doesn't move as work goes on).
ALTER TABLE publish_requests ADD COLUMN ask_job_id UUID REFERENCES turn_jobs(id) ON DELETE SET NULL;
ALTER TABLE publish_requests ADD COLUMN files TEXT[] NOT NULL DEFAULT '{}';
