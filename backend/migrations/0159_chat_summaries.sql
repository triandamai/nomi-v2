-- "Summarise and continue": the messages a summary stands in for stay in the chat but are no
-- longer sent to the model (they point at the summary message that replaced them), and a chat
-- being summarised is marked so a second request can be refused while it runs.
ALTER TABLE messages ADD COLUMN summarized_by UUID REFERENCES messages(id) ON DELETE SET NULL;
ALTER TABLE sessions ADD COLUMN summarizing_since TIMESTAMPTZ;
