CREATE TABLE user_personality_versions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id),
    version     INT NOT NULL,
    description TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, version)
);

ALTER TABLE user_personality ADD COLUMN current_version INT NOT NULL DEFAULT 1;

INSERT INTO user_personality_versions (user_id, version, description, created_at)
SELECT user_id, 1, description, updated_at FROM user_personality;
