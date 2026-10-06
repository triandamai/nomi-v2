-- Appearance themes replace the six plain accent colours. Each old pick moves to the closest
-- new theme: green is Canopy (the same palette), the rest by hue.
ALTER TABLE user_preferences DROP CONSTRAINT IF EXISTS user_preferences_accent_color_check;

UPDATE user_preferences SET accent_color = CASE accent_color
    WHEN 'green' THEN 'canopy'
    WHEN 'teal' THEN 'coral-reef'
    WHEN 'orange' THEN 'borneo-dusk'
    WHEN 'blue' THEN 'phantom'
    WHEN 'purple' THEN 'phantom'
    WHEN 'pink' THEN 'senja-jakarta'
    ELSE 'canopy'
END;

ALTER TABLE user_preferences
    ALTER COLUMN accent_color SET DEFAULT 'canopy',
    ADD CONSTRAINT user_preferences_accent_color_check
        CHECK (accent_color IN ('canopy', 'coral-reef', 'borneo-dusk', 'phantom', 'senja-jakarta'));
