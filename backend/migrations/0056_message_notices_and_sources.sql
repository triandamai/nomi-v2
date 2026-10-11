-- A failure notice ("Gemini is having trouble right now...") is shown in the chat but was never
-- said by the crew: fed back to the model as its own earlier reply, the model copies it and
-- every later answer comes back as the same "error". Notices are marked and left out of the
-- history agents read (working_memory::SAID_IN_CHAT).
ALTER TABLE messages ADD COLUMN is_notice BOOLEAN NOT NULL DEFAULT false;

-- Notices posted before this column existed: the fixed sentences of TurnError::user_message_in,
-- in English and Indonesian.
UPDATE messages SET is_notice = true
WHERE sender_channel_identity_id IS NULL AND agent_display_name IS NULL AND (
       content LIKE '% is having trouble right now. Try again in a moment.'
    OR content LIKE '% is limiting how many requests I can make right now.%'
    OR content LIKE 'I couldn''t reply because % rejected the API key.%'
    OR content LIKE 'I couldn''t reply because % is out of credits for this request.%'
    OR content LIKE '% couldn''t find the selected model.%'
    OR content LIKE 'Something went wrong talking to %. Send your message again.'
    OR content LIKE 'I couldn''t reach the AI provider.%'
    OR content LIKE 'The AI provider sent back a reply I couldn''t read.%'
    OR content LIKE 'This chat has grown too long for the model.%'
    OR content LIKE 'You''ve used this month''s allowance on Nomi''s models.%'
    OR content LIKE 'I went around in circles on that one%'
    OR content LIKE 'That took too long, so I stopped.%'
    OR content LIKE 'Something went wrong on my side.%'
    OR content LIKE '% sedang mengalami gangguan.%'
    OR content LIKE '% sedang membatasi jumlah permintaanku.%'
    OR content LIKE 'Maaf, aku belum bisa membalas karena %'
    OR content LIKE '% tidak menemukan model yang dipilih.%'
    OR content LIKE 'Maaf, ada kendala saat menghubungi %'
    OR content LIKE 'Maaf, aku belum bisa menghubungi penyedia AI.%'
    OR content LIKE 'Maaf, penyedia AI mengirim balasan yang tidak bisa aku baca.%'
    OR content LIKE 'Obrolan ini sudah terlalu panjang untuk modelnya.%'
    OR content LIKE 'Jatah bulan ini untuk model Nomi sudah habis.%'
    OR content LIKE 'Sepertinya aku berputar-putar di situ%'
    OR content LIKE 'Yang tadi kelamaan, jadi aku hentikan dulu.%'
    OR content LIKE 'Maaf, ada yang kurang beres di sisiku.%'
);

-- What a message led to: the person's message that started the turn in which the crew logged
-- a transaction, set a reminder, drafted a plan or scheduled a task. Deleting that message can
-- then offer to delete them too.
ALTER TABLE money_transactions ADD COLUMN source_message_id UUID REFERENCES messages(id) ON DELETE SET NULL;
ALTER TABLE reminders ADD COLUMN source_message_id UUID REFERENCES messages(id) ON DELETE SET NULL;
ALTER TABLE agent_plans ADD COLUMN source_message_id UUID REFERENCES messages(id) ON DELETE SET NULL;
ALTER TABLE scheduled_jobs ADD COLUMN source_message_id UUID REFERENCES messages(id) ON DELETE SET NULL;
CREATE INDEX money_transactions_source_message_idx ON money_transactions (source_message_id) WHERE source_message_id IS NOT NULL;
CREATE INDEX reminders_source_message_idx ON reminders (source_message_id) WHERE source_message_id IS NOT NULL;
CREATE INDEX agent_plans_source_message_idx ON agent_plans (source_message_id) WHERE source_message_id IS NOT NULL;
CREATE INDEX scheduled_jobs_source_message_idx ON scheduled_jobs (source_message_id) WHERE source_message_id IS NOT NULL;

-- Rows the crew writes during a turn point at the chat's latest message from a person: the one
-- the turn is answering. Reminders someone adds on the Reminders page aren't from a message.
CREATE FUNCTION stamp_source_message() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.source_message_id IS NULL THEN
        NEW.source_message_id := (
            SELECT id FROM messages
            WHERE session_id = NEW.session_id AND sender_channel_identity_id IS NOT NULL
            ORDER BY created_at DESC, id DESC LIMIT 1
        );
    END IF;
    RETURN NEW;
END $$;
CREATE FUNCTION stamp_reminder_source_message() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.created_by = 'agent' AND NEW.source_message_id IS NULL THEN
        NEW.source_message_id := (
            SELECT id FROM messages
            WHERE session_id = NEW.session_id AND sender_channel_identity_id IS NOT NULL
            ORDER BY created_at DESC, id DESC LIMIT 1
        );
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER reminders_source_message BEFORE INSERT ON reminders FOR EACH ROW EXECUTE FUNCTION stamp_reminder_source_message();
CREATE TRIGGER agent_plans_source_message BEFORE INSERT ON agent_plans FOR EACH ROW EXECUTE FUNCTION stamp_source_message();
CREATE TRIGGER scheduled_jobs_source_message BEFORE INSERT ON scheduled_jobs FOR EACH ROW EXECUTE FUNCTION stamp_source_message();
