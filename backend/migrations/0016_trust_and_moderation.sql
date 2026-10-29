-- 0016_trust_and_moderation.sql — Discourse-style trust levels, quorum moderation.
--
-- Owner (2026-10-29): "all moderation through discourse-style trust levels or quorum".
--
-- The rules live in SQL for the same reason the rate limits do: enforcement, the
-- UI's progress bar and the tests all read ONE definition. A trust level computed
-- in Rust as well would be two definitions of the same number, and this project
-- has already paid for that mistake once (the budget clock).
--
-- Level 4 is the only stored level, and deliberately so: it is a grant, not a
-- score. Everything else is derived from rows the account actually produced
-- (logs, attestations) so nobody can be promoted by saying so.

alter table public.profiles
  add column if not exists is_leader boolean not null default false;

alter table public.habit_logs
  add column if not exists hidden_at timestamptz;

create table if not exists public.flags (
  id bigint generated always as identity primary key,
  -- 'profile' is deliberately absent: profile moderation is a rename, not a
  -- content hide, and it has no id space that shares the bigint targets.
  target_kind text not null check (target_kind in ('log', 'cage_check')),
  target_id bigint not null,
  reporter_id uuid not null references public.profiles(id) on delete cascade,
  reason text not null check (char_length(reason) between 1 and 140),
  created_at timestamptz not null default now(),
  unique (reporter_id, target_kind, target_id)
);

create index if not exists flags_target_idx on public.flags(target_kind, target_id);

create table if not exists public.moderation_actions (
  id bigint generated always as identity primary key,
  actor_id uuid references public.profiles(id) on delete set null,
  action text not null check (action in ('hide', 'restore', 'auto_hide')),
  target_kind text not null check (target_kind in ('log', 'cage_check')),
  target_id bigint not null,
  note text,
  created_at timestamptz not null default now()
);

create index if not exists moderation_actions_target_idx
  on public.moderation_actions(target_kind, target_id, created_at desc);

-- Trust level, computed. Windows: 30 days for levels 1–2, 90 for level 3.
-- "upheld" is measured as content of theirs that moderation actually hid, which
-- is the only signal this site records; it is not a reputation score.
create or replace function public.trust_level(p_user uuid)
returns table (
  level int,
  active_days int,
  contributions int,
  upheld_flags int,
  next_level_at int
) language sql stable as $$
  with me as (
    select id, is_leader, created_at from public.profiles where id = p_user
  ),
  activity as (
    select
      (select count(distinct log_date) from public.habit_logs
        where user_id = p_user and hidden_at is null)::int as active_days,
      (select count(*) from public.habit_logs where user_id = p_user)::int as logs,
      (select count(*) from public.cage_attestations where attester_id = p_user)::int as attestations
  ),
  upheld_30 as (
    select (
      (select count(*) from public.habit_logs
        where user_id = p_user and hidden_at > now() - interval '30 days')
      + (select count(*) from public.cage_checks
        where user_id = p_user and hidden_at > now() - interval '30 days')
    )::int as n
  ),
  upheld_90 as (
    select (
      (select count(*) from public.habit_logs
        where user_id = p_user and hidden_at > now() - interval '90 days')
      + (select count(*) from public.cage_checks
        where user_id = p_user and hidden_at > now() - interval '90 days')
    )::int as n
  ),
  lv as (
    select
      case
        when coalesce(m.is_leader, false) then 4
        when a.active_days >= 30
             and (a.logs + a.attestations) >= 40
             and u90.n = 0 then 3
        when a.active_days >= 7
             and (a.logs + a.attestations) >= 10
             and u30.n <= 1 then 2
        when m.created_at <= now() - interval '24 hours'
             and a.logs >= 3
             and u30.n = 0 then 1
        else 0
      end::int as level,
      a.active_days,
      (a.logs + a.attestations)::int as contributions,
      u30.n as upheld_30
    from activity a, upheld_30 u30, upheld_90 u90
    left join me m on true
  )
  select
    lv.level,
    coalesce(lv.active_days, 0),
    coalesce(lv.contributions, 0),
    coalesce(lv.upheld_30, 0),
    case lv.level
      when 0 then greatest(0, 3 - coalesce(lv.contributions, 0))
      when 1 then greatest(0, 10 - coalesce(lv.contributions, 0))
      when 2 then greatest(0, 40 - coalesce(lv.contributions, 0))
      else null
    end::int as next_level_at
  from lv;
$$;

-- How many independent voices a target has, and how many of them are trusted
-- enough to matter for the quorum.
create or replace function public.flag_quorum(p_kind text, p_target bigint)
returns table (flags int, from_tl2 int) language sql stable as $$
  select
    count(*)::int,
    count(*) filter (
      where (select tl.level from public.trust_level(f.reporter_id) tl) >= 2
    )::int
  from public.flags f
  where f.target_kind = p_kind and f.target_id = p_target;
$$;

create or replace function public.hide_target(p_kind text, p_target bigint)
returns void language plpgsql as $$
begin
  if p_kind = 'log' then
    update public.habit_logs set hidden_at = coalesce(hidden_at, now()) where id = p_target;
  elsif p_kind = 'cage_check' then
    update public.cage_checks set hidden_at = coalesce(hidden_at, now()) where id = p_target;
  end if;
end;
$$;

create or replace function public.restore_target(p_kind text, p_target bigint)
returns void language plpgsql as $$
begin
  if p_kind = 'log' then
    update public.habit_logs set hidden_at = null where id = p_target;
  elsif p_kind = 'cage_check' then
    update public.cage_checks set hidden_at = null where id = p_target;
  end if;
end;
$$;
