-- A thumbs-down can give any number of reasons, a note in the person's own words, and whether the
-- chat may be read to look into it. `reason` stays for the one reason older apps send; `reasons`
-- holds every reason, that one included.
ALTER TABLE message_feedback DROP CONSTRAINT IF EXISTS message_feedback_reason_check;
ALTER TABLE message_feedback
    ADD CONSTRAINT message_feedback_reason_check CHECK (reason IN (
        'wrong_memory', 'not_relevant', 'too_long', 'other', 'inaccurate', 'ignored', 'wrong_crew', 'outdated', 'unsafe'
    )),
    ADD COLUMN reasons TEXT[] NOT NULL DEFAULT '{}' CHECK (reasons <@ ARRAY[
        'wrong_memory', 'not_relevant', 'too_long', 'other', 'inaccurate', 'ignored', 'wrong_crew', 'outdated', 'unsafe'
    ]::TEXT[]),
    ADD COLUMN note TEXT CHECK (char_length(note) <= 2000),
    ADD COLUMN include_chat BOOLEAN NOT NULL DEFAULT false;

UPDATE message_feedback SET reasons = ARRAY[reason] WHERE reason IS NOT NULL;
