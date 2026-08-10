-- 0004_local_auth.sql — StreakForge local username/password auth
-- Replaces X OAuth + dev-login with normal registration/login.
-- Adds password_hash + social_url; relaxes the provider constraint.

-- password_hash for local accounts (argon2 PHC string). NULL for any
-- legacy/other-provider rows (none should remain in prod after this migration).
alter table public.profiles
  add column if not exists password_hash text;

-- social_url: one external link (e.g. X/Tumblr/portfolio) shown on the
-- public profile when someone clicks the user's name.
alter table public.profiles
  add column if not exists social_url text
  check (social_url is null or social_url ~ '^https?://' or social_url = '');

-- provider check: allow 'local' (the new default); keep old values valid
-- during transition, then default to 'local'.
alter table public.profiles
  drop constraint if exists profiles_provider_check;

alter table public.profiles
  add constraint profiles_provider_check
  check (provider in ('local', 'x', 'dev'));

alter table public.profiles
  alter column provider set default 'local';

-- username uniqueness is already enforced by the unique constraint.
-- provider_id is now optional (local accounts key off username), but keep
-- the unique (provider, provider_id) constraint valid for legacy rows:
-- local rows will have provider_id = NULL and the constraint allows
-- multiple NULLs in Postgres, so it stays harmless.
