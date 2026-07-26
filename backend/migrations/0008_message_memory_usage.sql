CREATE TABLE message_memory_usage (
    message_id  UUID NOT NULL REFERENCES messages(id),
    memory_id   UUID NOT NULL REFERENCES memory_items(id),
    PRIMARY KEY (message_id, memory_id)
);
