create extension if not exists pgcrypto;

create table providers (
    id uuid primary key default gen_random_uuid(),
    stellar_public_key text not null unique,
    display_name text not null,
    email text unique,
    specialty text,
    license_number text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table auth_challenges (
    id uuid primary key default gen_random_uuid(),
    stellar_public_key text not null,
    nonce text not null unique,
    expires_at timestamptz not null,
    consumed_at timestamptz,
    created_at timestamptz not null default now()
);

create index idx_auth_challenges_public_key on auth_challenges (stellar_public_key);
create index idx_auth_challenges_expires_at on auth_challenges (expires_at);
