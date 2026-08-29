ALTER TABLE memory_items
    ADD COLUMN embedding_provider TEXT NOT NULL DEFAULT 'openai',
    ADD COLUMN embedding_model TEXT NOT NULL DEFAULT '';
