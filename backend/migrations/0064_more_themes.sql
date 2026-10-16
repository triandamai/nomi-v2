-- Seven more appearance themes: Lautan, Matcha, Saffron, Lagoon, Orchid, Ruby and Graphite.
ALTER TABLE user_preferences DROP CONSTRAINT IF EXISTS user_preferences_accent_color_check;
ALTER TABLE user_preferences ADD CONSTRAINT user_preferences_accent_color_check
    CHECK (accent_color IN ('canopy', 'coral-reef', 'borneo-dusk', 'phantom', 'senja-jakarta', 'lautan', 'matcha', 'saffron', 'lagoon', 'orchid', 'ruby', 'graphite'));
