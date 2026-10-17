-- Which home page the person picked (Preferences → Home). NULL means the default one. The
-- frontend owns the list of layouts, so the column only holds a slug.
ALTER TABLE user_preferences ADD COLUMN home_layout TEXT;
-- The frosted, animated crew background behind chats and the Stage home; on unless turned off.
ALTER TABLE user_preferences ADD COLUMN animated_backdrop BOOLEAN;
