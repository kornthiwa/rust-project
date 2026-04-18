CREATE TABLE IF NOT EXISTS public.users (
    id SERIAL PRIMARY KEY,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    username VARCHAR(100) NOT NULL UNIQUE,
    password TEXT NOT NULL,
    first_name VARCHAR(120) NOT NULL,
    last_name VARCHAR(120) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_users_active_deleted
    ON public.users (active, deleted_at);

CREATE INDEX IF NOT EXISTS idx_users_username
    ON public.users (username);
