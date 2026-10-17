-- The address a showcased app is shown at, as Shipyard reports its production environment when
-- the owner turns the showcase on (not the address recorded when a publish started).
ALTER TABLE projects ADD COLUMN showcase_url TEXT;
