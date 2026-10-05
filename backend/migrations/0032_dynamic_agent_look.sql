-- How a dynamic agent looks in the app: one of the frontend's shapes, gradient tones and
-- working motions (frontend/src/lib/components/m3/shapes.ts; validated against the same lists in
-- nomi-server's routes/agents.rs). Built-in agents' looks are fixed in the frontend.
ALTER TABLE dynamic_agents
    ADD COLUMN shape  TEXT NOT NULL DEFAULT 'cookie9',
    ADD COLUMN tone   TEXT NOT NULL DEFAULT 'glow',
    ADD COLUMN motion TEXT NOT NULL DEFAULT 'spin';
