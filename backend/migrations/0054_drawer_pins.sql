-- Features the person pinned to the navigation drawer, in their order. NULL means they haven't
-- customised it yet, and the app's default pins apply.
ALTER TABLE user_preferences ADD COLUMN drawer_pins TEXT[];
