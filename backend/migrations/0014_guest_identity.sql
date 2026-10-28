-- 0014_guest_identity.sql — GS-1 (guest identity) and GS-2 (the fold).
--
-- A guest is a **real** `profiles` row with an empty credential, not a shadow
-- table: every authorship column in this schema already points at `profiles(id)`,
-- so a parallel guests table would mean two sources of truth for "who did this".
-- `password_hash` is already nullable (0004), which is what makes the row
-- unloggable without a placeholder hash that some future login path might accept.
--
-- `merged_into` is the tombstone: after a guest registers, the guest row is
-- anonymised rather than deleted, because `audit_log` rows point at it and a
-- deletion request is answered *from* those rows. Nulling the username out is not
-- possible (it is `not null` and carries the format check), so the fold renames it
-- to a fixed opaque form and records where the history went.

alter table public.profiles
  add column if not exists is_guest boolean not null default false,
  -- The address a guest identity was minted from, hashed with the same secret the
  -- sessions use. Stored because the standard's anti-abuse rule is "one identity
  -- per cookie + IP fingerprint", and a rule with nothing recorded cannot be
  -- enforced or audited later.
  add column if not exists guest_fingerprint text,
  add column if not exists merged_into uuid references public.profiles(id) on delete set null;

create index if not exists profiles_guest_idx
  on public.profiles(is_guest) where is_guest;

create index if not exists profiles_merged_into_idx
  on public.profiles(merged_into) where merged_into is not null;

-- MAU, GS-1's definition kept verbatim: distinct *accounts* with at least one
-- recorded action in the window. This site's actions live in two places — a habit
-- log and the audit trail — so the count is the union of both, as a function rather
-- than a Rust-only query so the number can be read while diagnosing. The window is
-- a parameter because `guest.mau_window_days` is configurable and a hardcoded 30
-- would make the setting a lie.
create or replace function public.monthly_active_accounts(p_window_days int default 30)
returns bigint language sql stable as $$
  select count(distinct actor)::bigint
  from (
    select user_id as actor
    from public.habit_logs
    where logged_at >= now() - make_interval(days => greatest(p_window_days, 1))
    union
    select profile_id as actor
    from public.audit_log
    where profile_id is not null
      and at >= now() - make_interval(days => greatest(p_window_days, 1))
  ) active
  where actor is not null;
$$;
