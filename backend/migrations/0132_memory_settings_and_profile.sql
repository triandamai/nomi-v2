-- What a person lets Nomi do with memory (no row: everything on, default budgets), the profile
-- Nomi keeps of them, and which crew member learned each memory.
CREATE TABLE memory_settings (
    user_id            UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    learn_from_chats   BOOLEAN NOT NULL DEFAULT TRUE,
    keep_profile       BOOLEAN NOT NULL DEFAULT TRUE,
    search_past_chats  BOOLEAN NOT NULL DEFAULT TRUE,
    -- Tokens (about 4 characters each) of live memories kept, and of the profile.
    memory_budget      INT NOT NULL,
    profile_budget     INT NOT NULL,
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The short summary of the person every agent that uses memory reads. The memory worker
-- rewrites it from their memories; the person can edit it (edited_at).
CREATE TABLE memory_profiles (
    user_id     UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    text        TEXT NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    edited_at   TIMESTAMPTZ
);

-- The agent type whose reply the memory was learned from (NULL for older memories).
ALTER TABLE memory_items ADD COLUMN learned_by TEXT;
ALTER TABLE memory_jobs ADD COLUMN agent_type TEXT;
