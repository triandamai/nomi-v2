-- Push for reminders, and quiet hours (Inbox › How it reaches you). A reminder due inside the
-- person's quiet hours (read in their time zone) is held until they end: `held_until` keeps the
-- reminders worker from claiming it again before then, while `due_at` keeps the schedule.
ALTER TABLE user_preferences ADD COLUMN push_reminders BOOLEAN NOT NULL DEFAULT true;
ALTER TABLE user_preferences ADD COLUMN quiet_hours_start TIME;
ALTER TABLE user_preferences ADD COLUMN quiet_hours_end TIME;
ALTER TABLE user_preferences ADD CONSTRAINT user_preferences_quiet_hours_check CHECK (
    (quiet_hours_start IS NULL AND quiet_hours_end IS NULL)
    OR (quiet_hours_start IS NOT NULL AND quiet_hours_end IS NOT NULL AND quiet_hours_start <> quiet_hours_end)
);

ALTER TABLE reminders ADD COLUMN held_until TIMESTAMPTZ;
