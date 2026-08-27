CREATE TABLE user_personality (
    user_id     UUID PRIMARY KEY REFERENCES users(id),
    description TEXT NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
