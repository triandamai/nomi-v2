-- The showcase: published apps their owners chose to show on Nomi's public showcase page.
ALTER TABLE projects
    ADD COLUMN showcase BOOLEAN NOT NULL DEFAULT false,
    -- One line about the app, shown on its card.
    ADD COLUMN showcase_blurb TEXT,
    ADD COLUMN showcased_at TIMESTAMPTZ;

CREATE INDEX projects_showcase_idx ON projects (showcased_at DESC) WHERE showcase;
