-- Whether the Relay's secrets on Shipyard have been made. They're made once and never again: the
-- encryption key seals what Relay keeps, so making it twice would lock Relay out of its own state.
ALTER TABLE project_relays ADD COLUMN secrets_made BOOLEAN NOT NULL DEFAULT false;
