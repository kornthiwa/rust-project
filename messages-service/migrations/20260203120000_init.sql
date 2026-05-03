CREATE TABLE IF NOT EXISTS message_models (
    id BIGSERIAL PRIMARY KEY,
    public_id TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    author_subject TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS message_models_public_id_key ON message_models (public_id);
CREATE INDEX IF NOT EXISTS message_models_conversation_id_idx ON message_models (conversation_id);
