-- Settings › General: number and date formats apart from the language, who new chats start
-- with, whether Enter sends, and tokens under replies. NULL means the app's default.
ALTER TABLE user_preferences ADD COLUMN formats TEXT;
ALTER TABLE user_preferences ADD COLUMN default_agent TEXT;
ALTER TABLE user_preferences ADD COLUMN enter_sends BOOLEAN;
ALTER TABLE user_preferences ADD COLUMN show_reply_tokens BOOLEAN;

-- Moving the account to a new address, and closing it, are confirmed with an emailed code:
-- for 'change_email' the challenge's `email` is the new address the code went to.
ALTER TABLE sign_in_challenges DROP CONSTRAINT IF EXISTS sign_in_challenges_purpose_check;
ALTER TABLE sign_in_challenges ADD CONSTRAINT sign_in_challenges_purpose_check
    CHECK (purpose IN ('register', 'login', 'change_email', 'delete_account'));

-- A closed account is kept as a row (chats, projects and usage others share point at it) with
-- its sign-in, memberships and personal data removed, as an admin removing someone from a
-- workspace keeps the membership row.
ALTER TABLE users ADD COLUMN deleted_at TIMESTAMPTZ;
