-- The callback address Google was sent to when a flow started: the web app the person started on
-- (the app or the new app), so the answer comes back there. The code is traded with the same
-- address, as Google requires. NULL (flows started before this) means the configured address.
ALTER TABLE google_sign_in_states ADD COLUMN redirect_uri TEXT;
ALTER TABLE workspace_oauth_states ADD COLUMN redirect_uri TEXT;
