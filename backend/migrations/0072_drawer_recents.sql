-- Recent chats listed in the navigation drawer (Preferences → Navigation drawer). NULL means the
-- defaults: shown, ten at a time.
ALTER TABLE user_preferences ADD COLUMN drawer_recents BOOLEAN;
ALTER TABLE user_preferences ADD COLUMN drawer_recents_limit SMALLINT
    CHECK (drawer_recents_limit IN (5, 10, 15, 20));
