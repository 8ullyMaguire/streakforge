-- 0005_denial_lock.sql — StreakForge denial & chastity-lock subsystem
--
-- The core habit tracker is re-themed as a denial/whiteboi accountability tool:
--   - habit_logs.kind gains 'denial'  (a reported denial: edged-and-held,
--     ruined, leaked while plapping/pegging, etc.)
--   - a new lock_sessions table tracks chastity lock periods (self-lock or
--     keyheld). The current/longest lock streak is derived from consecutive
--     lock periods with no more than a short gap.
--   - weighted leaderboards: denial counts 10x a waste (points).
--
-- The 'habit' kind remains the main counter (the "loads wasted" global). Denial
-- logs are a separate, heavier-weighted kind.

-- 1) Extend the kind check to allow 'denial'.
alter table public.habit_logs
  drop constraint if exists habit_logs_kind_check;

alter table public.habit_logs
  add constraint habit_logs_kind_check
  check (kind in ('habit', 'affirmation', 'denial'));

-- 2) Chastity lock sessions.
--    A row starts when the user locks (locked_at) and is closed when they
--    unlock (unlocked_at). A NULL unlocked_at = currently locked.
--    `reason` is a free-form short label (self, keyholder, pegging, plapping,
--    ruined, edge-marathon...) shown on the profile.
create table if not exists public.lock_sessions (
  id bigint generated always as identity primary key,
  user_id uuid not null references public.profiles(id) on delete cascade,
  locked_at timestamptz not null default now(),
  unlocked_at timestamptz,
  reason text check (reason is null or char_length(reason) <= 60),
  check (unlocked_at is null or unlocked_at >= locked_at)
);

create index if not exists lock_sessions_user_locked_idx
  on public.lock_sessions(user_id, locked_at desc);

-- 3) Current lock info for a user: whether locked, when the current lock
--    started, total duration of the current lock, all-time total time locked
--    (sum of closed + current), and the longest single lock.
create or replace function public.lock_info(p_user uuid)
returns table (
  locked bool,
  locked_at timestamptz,
  current_duration interval,
  total_locked interval,
  longest_lock interval
) language sql stable as $$
  select
    (ls.id is not null) as locked,
    ls.locked_at,
    case when ls.id is not null then now() - ls.locked_at else null end as current_duration,
    coalesce((
      select sum(coalesce(unlocked_at, now()) - locked_at)
      from public.lock_sessions where user_id = p_user
    ), interval '0') as total_locked,
    coalesce((
      select max(coalesce(unlocked_at, now()) - locked_at)
      from public.lock_sessions where user_id = p_user
    ), interval '0') as longest_lock
  from (select 1) x
  left join public.lock_sessions ls
    on ls.user_id = p_user and ls.unlocked_at is null
  order by ls.locked_at desc
  limit 1;
$$;

-- 4) Lock streak: consecutive lock periods, allowing a short unlock gap of up
--    to 24h (a whiteboi still gets credit if they unlock briefly to clean,
--    per the good-whiteboy guide). The streak counts whole lock periods whose
--    combined span is consecutive.
create or replace function public.lock_streak(p_user uuid)
returns table (current_streak int, longest_streak int, last_locked_at timestamptz) language sql stable as $$
  with spans as (
    select
      locked_at,
      coalesce(unlocked_at, now()) as unlocked_at
    from public.lock_sessions
    where user_id = p_user
  ),
  merged as (
    select
      min(locked_at) as start_d,
      max(unlocked_at) as end_d,
      count(*) as periods
    from (
      select
        locked_at, unlocked_at,
        sum(flag) over (order by locked_at) as grp
      from (
        select
          locked_at, unlocked_at,
          case
            when locked_at > lag(unlocked_at) over (order by locked_at) + interval '24 hours'
            then 1 else 0
          end as flag
        from spans
      ) t
    ) u
    group by grp
  )
  select
    coalesce((
      select periods from merged
      where end_d >= now() - interval '24 hours'
      order by end_d desc limit 1
    ), 0) as current_streak,
    coalesce(max(periods), 0) as longest_streak,
    (select max(end_d) from merged) as last_locked_at
  from merged;
$$;

-- 5) Denial streak: consecutive UTC days with at least one 'denial' log.
create or replace function public.user_streak_kind(p_user uuid, p_kind text)
returns table (current_streak int, longest_streak int) language sql stable as $$
  with days as (
    select distinct log_date
    from public.habit_logs
    where user_id = p_user and kind = p_kind
  ),
  ordered as (
    select log_date,
           log_date - (row_number() over (order by log_date))::int as grp
    from days
  ),
  groups as (
    select grp, min(log_date) as start_d, max(log_date) as end_d, count(*) as len
    from ordered group by grp
  )
  select
    case
      when g.end_d = t.d or g.end_d = t.d - 1 then g.len
      else 0
    end as current_streak,
    coalesce(g2.longest, 0) as longest
  from (select (now() at time zone 'utc')::date as d) t
  left join groups g on g.end_d = (select max(end_d) from groups)
  cross join (select max(len) as longest from groups) g2;
$$;

-- 6) Total denied loads for the public counter (only denial kind).
create or replace function public.total_denied()
returns bigint language sql stable as $$
  select count(*) from public.habit_logs where kind = 'denial';
$$;

-- 7) Weighted leaderboards: denial counts 10x a waste (1 pt per habit log,
--    10 pts per denial log; affirmations are 0). A user ranks by points, and
--    we also surface the raw waste/denial breakdown for the UI.
create or replace function public.daily_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  points bigint, waste_count bigint, denial_count bigint, last_log_at timestamptz
) language sql stable as $$
  select p.id, p.username, p.display_name, p.avatar_url,
         (count(*) filter (where l.kind = 'denial') * 10 + count(*) filter (where l.kind = 'habit'))::bigint as points,
         count(*) filter (where l.kind = 'habit')::bigint as waste_count,
         count(*) filter (where l.kind = 'denial')::bigint as denial_count,
         max(l.logged_at) as last_log_at
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.log_date = (now() at time zone 'utc')::date
    and l.kind in ('habit', 'denial')
  group by p.id
  order by points desc, min(l.logged_at) asc
  limit p_limit;
$$;

create or replace function public.weekly_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  points bigint, waste_count bigint, denial_count bigint, last_log_at timestamptz
) language sql stable as $$
  select p.id, p.username, p.display_name, p.avatar_url,
         (count(*) filter (where l.kind = 'denial') * 10 + count(*) filter (where l.kind = 'habit'))::bigint as points,
         count(*) filter (where l.kind = 'habit')::bigint as waste_count,
         count(*) filter (where l.kind = 'denial')::bigint as denial_count,
         max(l.logged_at) as last_log_at
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.logged_at >= date_trunc('week', now() at time zone 'utc')
    and l.kind in ('habit', 'denial')
  group by p.id
  order by points desc, min(l.logged_at) asc
  limit p_limit;
$$;

create or replace function public.alltime_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  points bigint, waste_count bigint, denial_count bigint, last_log_at timestamptz
) language sql stable as $$
  select p.id, p.username, p.display_name, p.avatar_url,
         (count(*) filter (where l.kind = 'denial') * 10 + count(*) filter (where l.kind = 'habit'))::bigint as points,
         count(*) filter (where l.kind = 'habit')::bigint as waste_count,
         count(*) filter (where l.kind = 'denial')::bigint as denial_count,
         max(l.logged_at) as last_log_at
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.kind in ('habit', 'denial')
  group by p.id
  order by points desc, min(l.logged_at) asc
  limit p_limit;
$$;
