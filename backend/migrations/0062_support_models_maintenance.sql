-- Nomi as a customer support agent for other products: a knowledge base it answers from, keys
-- other apps call it with, every question it was asked, and a draft for each one it couldn't
-- answer so an admin can fill the gap.

CREATE TABLE kb_articles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title       TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 200),
    body        TEXT NOT NULL DEFAULT '',
    tags        TEXT[] NOT NULL DEFAULT '{}',
    published   BOOLEAN NOT NULL DEFAULT true,
    created_by  UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Title and tags weigh more than the body (kept by the trigger below).
    search      TSVECTOR NOT NULL DEFAULT ''::tsvector
);
CREATE INDEX kb_articles_search_idx ON kb_articles USING GIN (search);

CREATE FUNCTION kb_articles_search() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    NEW.search :=
        setweight(to_tsvector('simple', coalesce(NEW.title, '')), 'A') ||
        setweight(to_tsvector('simple', array_to_string(NEW.tags, ' ')), 'A') ||
        setweight(to_tsvector('simple', coalesce(NEW.body, '')), 'B');
    RETURN NEW;
END $$;
CREATE TRIGGER kb_articles_search BEFORE INSERT OR UPDATE ON kb_articles FOR EACH ROW EXECUTE FUNCTION kb_articles_search();

CREATE TABLE support_keys (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          TEXT NOT NULL,
    -- SHA-256 of the key; the key itself is shown once.
    key_hash      BYTEA NOT NULL UNIQUE,
    prefix        TEXT NOT NULL,
    -- The product's name, used in answers ("I'm the support assistant for <product>").
    product_name  TEXT NOT NULL DEFAULT '',
    created_by    UUID NOT NULL REFERENCES users(id),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at  TIMESTAMPTZ,
    revoked_at    TIMESTAMPTZ
);

CREATE TABLE support_questions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id       UUID REFERENCES support_keys(id) ON DELETE SET NULL,
    question     TEXT NOT NULL,
    answer       TEXT NOT NULL,
    answered     BOOLEAN NOT NULL,
    article_ids  UUID[] NOT NULL DEFAULT '{}',
    -- Who asked, in the calling product's own terms (an email, a user id).
    customer     TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX support_questions_created_idx ON support_questions (created_at DESC);

-- A question the knowledge base couldn't answer, drafted as an issue for an admin: what was
-- asked (and how often), and a suggested article to start from.
CREATE TABLE kb_drafts (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question       TEXT NOT NULL,
    title          TEXT NOT NULL,
    body           TEXT NOT NULL DEFAULT '',
    status         TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'published', 'dismissed')),
    times_asked    INTEGER NOT NULL DEFAULT 1,
    last_asked_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    article_id     UUID REFERENCES kb_articles(id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    search         TSVECTOR GENERATED ALWAYS AS (to_tsvector('simple', coalesce(question, ''))) STORED
);
CREATE INDEX kb_drafts_open_idx ON kb_drafts (status, last_asked_at DESC);
CREATE INDEX kb_drafts_search_idx ON kb_drafts USING GIN (search);

-- Admins can switch a model off entirely: nobody can pick it and nothing is routed to it.
ALTER TABLE admin_llm_models ADD COLUMN enabled BOOLEAN NOT NULL DEFAULT true;

-- Maintenance mode: while on, people see the maintenance page instead of the app.
CREATE TABLE maintenance (
    singleton   BOOLEAN PRIMARY KEY DEFAULT true CHECK (singleton),
    enabled     BOOLEAN NOT NULL DEFAULT false,
    message     TEXT NOT NULL DEFAULT '',
    -- When people can expect it back, shown on the page.
    until       TIMESTAMPTZ,
    updated_by  UUID REFERENCES users(id) ON DELETE SET NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
INSERT INTO maintenance (singleton) VALUES (true);
