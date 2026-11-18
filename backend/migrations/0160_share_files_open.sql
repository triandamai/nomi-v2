-- A link may also open the chat's files (never together with `mask`: masking can't reach inside
-- a file). Off for every link made before this.
ALTER TABLE chat_shares ADD COLUMN files_open BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE chat_shares ADD CONSTRAINT chat_shares_mask_or_files CHECK (NOT (mask AND files_open));
