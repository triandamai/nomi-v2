-- Sign in or create with an email code, no password (`POST /api/auth/start`). A code for an
-- address without an account has no user yet: it keeps the address (and the invite and language
-- it was asked with), and the account is made when the code is entered.
ALTER TABLE sign_in_challenges ALTER COLUMN user_id DROP NOT NULL;
ALTER TABLE sign_in_challenges ADD COLUMN email TEXT;
ALTER TABLE sign_in_challenges ADD COLUMN invite_code TEXT;
ALTER TABLE sign_in_challenges ADD COLUMN language TEXT;
ALTER TABLE sign_in_challenges ADD CONSTRAINT sign_in_challenges_for_someone CHECK (user_id IS NOT NULL OR email IS NOT NULL);
CREATE INDEX sign_in_challenges_new_address_recent ON sign_in_challenges (email, created_at DESC) WHERE user_id IS NULL;
