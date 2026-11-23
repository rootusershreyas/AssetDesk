CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    email TEXT NOT NULL CHECK (email = btrim(email) AND email <> ''),
    role TEXT NOT NULL CHECK (role IN ('employee', 'it_manager', 'it_member')),
    account_status TEXT NOT NULL DEFAULT 'active'
        CHECK (account_status IN ('active', 'inactive')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX users_email_unique_ci ON users (lower(email));