CREATE TABLE IF NOT EXISTS user_models (
    id BIGSERIAL PRIMARY KEY,
    public_id TEXT NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS user_models_public_id_key ON user_models (public_id);
CREATE UNIQUE INDEX IF NOT EXISTS user_models_email_key ON user_models (email);
