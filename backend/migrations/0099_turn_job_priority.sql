-- Asks waiting in a chat can be moved up (Build together → Asks in line): a higher priority is
-- answered first, then the oldest.
ALTER TABLE turn_jobs ADD COLUMN priority INTEGER NOT NULL DEFAULT 0;
