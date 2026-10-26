-- New accounts start on the Lagoon theme. Existing rows keep the theme they have.
ALTER TABLE user_preferences ALTER COLUMN accent_color SET DEFAULT 'lagoon';
