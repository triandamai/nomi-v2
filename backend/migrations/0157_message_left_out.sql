-- Files a person sent a message without, because the chosen model couldn't read them:
-- [{ "name", "reason", "model_label" }]. Shown on the message's bubble, and told to the crew.
ALTER TABLE messages ADD COLUMN left_out JSONB;
