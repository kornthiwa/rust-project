CREATE TABLE IF NOT EXISTS public.accounts (
    id SERIAL PRIMARY KEY,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    username VARCHAR(100) NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_accounts_active_deleted
    ON public.accounts (active, deleted_at);

CREATE INDEX IF NOT EXISTS idx_accounts_username
    ON public.accounts (username);
