-- When the user last looked at Home, for its "While you were out" card. `previous_seen_at` is
-- what the card reports since; it only moves forward once a visit is more than 30 minutes old,
-- so browsing back to Home within one sitting doesn't empty the card. See routes/home.rs.
CREATE TABLE user_home_visits (
    user_id           UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    seen_at           TIMESTAMPTZ NOT NULL,
    previous_seen_at  TIMESTAMPTZ NOT NULL
);
