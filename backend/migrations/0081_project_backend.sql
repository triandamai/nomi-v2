-- Which backend a project's app is built on: 'drizzle' (the app carries its own database code,
-- as every project did before Nomi Relay) or 'relay' (migrations, auth, storage and functions
-- come from Relay; see relay/README.md). relay_project_id is Relay's id for the project's
-- backend, set once Relay's control plane has made it.
ALTER TABLE projects ADD COLUMN backend TEXT NOT NULL DEFAULT 'drizzle'
    CHECK (backend IN ('drizzle', 'relay'));
ALTER TABLE projects ADD COLUMN relay_project_id UUID;
