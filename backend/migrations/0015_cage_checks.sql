-- 0015_cage_checks.sql — the cage check: a claim, a seal, and peers who vouch.
--
-- Owner (2026-10-29): "add cage check verified by users themselves; total verified
-- cage hours and similar metrics".
--
-- WHY IT LOOKS LIKE THIS: this server never receives an image. A check is a
-- timestamped claim carrying a SEAL CODE the site issued for that check, and the
-- hours only count once other people confirm it. "Verified by users themselves"
-- is therefore literal: the verification is attestations from peers, not a scan,
-- and the seal code is what makes a forwarded or stale claim detectable by eye.
--
-- The 72h cap on claimed_secs is load-bearing: without it, a user who does not
-- check in for a week could report a week of "cage hours" the moment they come
-- back, which would make the metric a measure of silence rather than of being
-- caged.

create table if not exists public.cage_checks (
  id bigint generated always as identity primary key,
  user_id uuid not null references public.profiles(id) on delete cascade,
  lock_id bigint references public.lock_sessions(id) on delete set null,
  checked_at timestamptz not null default now(),
  -- One check per user per UTC day: same clock as every other daily rule here.
  check_date date generated always as ((checked_at at time zone 'utc')::date) stored,
  seal_code text not null check (seal_code ~ '^[0-9a-f]{8}$'),
  note text check (note is null or char_length(note) <= 140),
  claimed_secs integer not null default 0
    check (claimed_secs >= 0 and claimed_secs <= 259200),
  status text not null default 'pending'
    check (status in ('pending', 'verified', 'disputed')),
  verified_at timestamptz,
  hidden_at timestamptz,
  created_at timestamptz not null default now(),
  unique (user_id, check_date)
);

create index if not exists cage_checks_status_idx
  on public.cage_checks(status, checked_at desc);
create index if not exists cage_checks_user_idx
  on public.cage_checks(user_id, checked_at desc);

-- One verdict per attester per check. A changed mind is an UPDATE of the verdict,
-- not a second voice — otherwise a single enthusiastic peer could reach quorum
-- alone by confirming twice.
create table if not exists public.cage_attestations (
  id bigint generated always as identity primary key,
  check_id bigint not null references public.cage_checks(id) on delete cascade,
  attester_id uuid not null references public.profiles(id) on delete cascade,
  verdict text not null check (verdict in ('confirm', 'dispute')),
  created_at timestamptz not null default now(),
  unique (check_id, attester_id)
);

create index if not exists cage_attestations_check_idx
  on public.cage_attestations(check_id);

-- Verified hours and the metrics that make them mean something. Hidden checks
-- (moderation) never contribute: a hidden row is not evidence a peer confirmed.
create or replace function public.cage_check_hours(p_user uuid)
returns table (
  checks bigint,
  claimed_secs bigint,
  verified_secs bigint,
  pending_secs bigint,
  disputed_secs bigint,
  verification_rate float8,
  avg_secs float8,
  current_verified_day_streak int,
  longest_verified_day_streak int,
  last_check_at timestamptz
) language sql stable as $$
  with visible as (
    select * from public.cage_checks
    where user_id = p_user and hidden_at is null
  ),
  verified_days as (
    select distinct check_date from visible where status = 'verified'
  ),
  ordered as (
    select check_date,
           check_date - (row_number() over (order by check_date))::int as grp
    from verified_days
  ),
  runs as (
    select grp, count(*) as len, max(check_date) as end_d
    from ordered group by grp
  ),
  agg as (
    select
      count(*)::bigint as checks,
      coalesce(sum(claimed_secs), 0)::bigint as claimed_secs,
      coalesce(sum(claimed_secs) filter (where status = 'verified'), 0)::bigint as verified_secs,
      coalesce(sum(claimed_secs) filter (where status = 'pending'), 0)::bigint as pending_secs,
      coalesce(sum(claimed_secs) filter (where status = 'disputed'), 0)::bigint as disputed_secs,
      coalesce(avg(claimed_secs), 0)::float8 as avg_secs,
      max(checked_at) as last_check_at
    from visible
  )
  select
    a.checks,
    a.claimed_secs,
    a.verified_secs,
    a.pending_secs,
    a.disputed_secs,
    case when a.claimed_secs > 0
         then a.verified_secs::float8 / a.claimed_secs::float8
         else 0 end as verification_rate,
    a.avg_secs,
    coalesce((
      select len from runs
      where end_d >= (now() at time zone 'utc')::date - 1
      order by end_d desc limit 1
    ), 0)::int as current_verified_day_streak,
    coalesce((select max(len) from runs), 0)::int as longest_verified_day_streak,
    a.last_check_at
  from agg a;
$$;

-- The site-wide number the owner asked for, with the same exclusions.
create or replace function public.total_verified_cage_hours()
returns bigint language sql stable as $$
  select coalesce(sum(claimed_secs), 0)::bigint
  from public.cage_checks
  where status = 'verified' and hidden_at is null;
$$;
