CREATE TABLE IF NOT EXISTS account_models (
    id BIGSERIAL PRIMARY KEY,
    public_id TEXT NOT NULL,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    failed_login_attempts INTEGER NOT NULL,
    locked_until TEXT,
    last_login_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS account_models_public_id_key ON account_models (public_id);
CREATE UNIQUE INDEX IF NOT EXISTS account_models_username_key ON account_models (username);
