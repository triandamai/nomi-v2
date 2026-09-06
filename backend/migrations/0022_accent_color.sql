ALTER TABLE user_preferences
    ADD COLUMN accent_color TEXT NOT NULL DEFAULT 'green'
        CHECK (accent_color IN ('green', 'blue', 'purple', 'pink', 'orange', 'teal'));
