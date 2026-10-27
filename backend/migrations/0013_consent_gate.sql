-- 0013_consent_gate.sql — GS-3 cookie consent, server-authoritative.
--
-- The gate itself reads the session (a signed cookie over a server-side row), so
-- it works for a visitor with no account. These two columns are the half that
-- survives the browser: a returning user signs in on a new device and is not
-- asked again, because the record is stamped on the identity, not on the client.
--
-- `audit_log` exists for the same reason the consent record does: "when did this
-- person consent, and to which version" has to be answerable after the fact, and
-- a log that lives only in tracing output is not answerable. GS-1's MAU count
-- reads this table too (distinct profiles with an action in the window), which is
-- why `action` is indexed alongside `at`.

alter table public.profiles
  add column if not exists consented_at timestamptz,
  add column if not exists consent_version integer;

create table if not exists public.audit_log (
  id bigserial primary key,
  -- `set null`, not cascade: deleting an account must not delete the record that
  -- the account once consented — that is exactly the record a deletion request
  -- is answered from.
  profile_id uuid references public.profiles(id) on delete set null,
  action text not null,
  detail jsonb not null default '{}'::jsonb,
  at timestamptz not null default now()
);

create index if not exists audit_log_at_idx on public.audit_log (at desc);
create index if not exists audit_log_action_at_idx on public.audit_log (action, at desc);
create index if not exists audit_log_profile_idx on public.audit_log (profile_id, at desc);
