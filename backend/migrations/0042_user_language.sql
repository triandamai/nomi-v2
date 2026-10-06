-- The language Nomi speaks to each person: the app's text, and the crew's replies.
ALTER TABLE user_preferences ADD COLUMN language TEXT NOT NULL DEFAULT 'en' CHECK (language IN ('en', 'id'));
