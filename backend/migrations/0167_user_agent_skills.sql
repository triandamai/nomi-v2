-- Skills a person adds to a crew member (Settings › Crew): SKILL.md files in the Agent Skills
-- convention, typed, uploaded or imported from a link. Each belongs to one person and one member
-- (`agent_type`, as soul.md and permissions key it); the member reads them with `read_skill`.
-- `text` is the whole SKILL.md as written; name and description are copied out of its frontmatter.
CREATE TABLE user_agent_skills (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_type   TEXT NOT NULL,
    name         TEXT NOT NULL,
    description  TEXT NOT NULL,
    license      TEXT NOT NULL DEFAULT '',
    source       TEXT NOT NULL DEFAULT '',
    text         TEXT NOT NULL,
    enabled      BOOLEAN NOT NULL DEFAULT true,
    origin       TEXT NOT NULL CHECK (origin IN ('typed', 'upload', 'link')),
    source_url   TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, agent_type, name)
);

-- A skill's `references/*.md`, read one at a time.
CREATE TABLE user_agent_skill_references (
    skill_id  UUID NOT NULL REFERENCES user_agent_skills(id) ON DELETE CASCADE,
    name      TEXT NOT NULL,
    text      TEXT NOT NULL,
    PRIMARY KEY (skill_id, name)
);
