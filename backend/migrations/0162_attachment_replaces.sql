-- An upload sent as a replacement for another of the person's files (X-Replaces on the upload)
-- remembers which one, so its chip can say "replaces <name>". Removing the old file clears it.
ALTER TABLE attachments ADD COLUMN replaces_attachment_id UUID REFERENCES attachments(id) ON DELETE SET NULL;
