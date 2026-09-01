-- 0006_affirmations_on_leaderboard.sql
--
-- Affirmations now count toward the leaderboard: 3 affirmation reps = 1 point
-- (equivalent to a wasted load). Denial stays 10x a waste. This redefines the
-- weighted leaderboard functions (Postgres cannot ALTER an OUT-parameter return
-- type, so we DROP + recreate with the extra affirmation_count column).
--
-- Also the user-of-the-day query in the app now includes affirmations; the SQL
-- here only updates the shared leaderboard functions.

drop function if exists public.daily_leaderboard_weighted(int);
drop function if exists public.weekly_leaderboard_weighted(int);
drop function if exists public.alltime_leaderboard_weighted(int);

create or replace function public.daily_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  points bigint, waste_count bigint, denial_count bigint, affirmation_count bigint, last_log_at timestamptz
) language sql stable as $$
  select p.id, p.username, p.display_name, p.avatar_url,
         (
           count(*) filter (where l.kind = 'denial') * 10
           + count(*) filter (where l.kind = 'habit')
           + floor(count(*) filter (where l.kind = 'affirmation') / 3)
         )::bigint as points,
         count(*) filter (where l.kind = 'habit')::bigint as waste_count,
         count(*) filter (where l.kind = 'denial')::bigint as denial_count,
         count(*) filter (where l.kind = 'affirmation')::bigint as affirmation_count,
         max(l.logged_at) as last_log_at
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.log_date = (now() at time zone 'utc')::date
    and l.kind in ('habit', 'denial', 'affirmation')
  group by p.id
  order by points desc, min(l.logged_at) asc
  limit p_limit;
$$;

create or replace function public.weekly_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  points bigint, waste_count bigint, denial_count bigint, affirmation_count bigint, last_log_at timestamptz
) language sql stable as $$
  select p.id, p.username, p.display_name, p.avatar_url,
         (
           count(*) filter (where l.kind = 'denial') * 10
           + count(*) filter (where l.kind = 'habit')
           + floor(count(*) filter (where l.kind = 'affirmation') / 3)
         )::bigint as points,
         count(*) filter (where l.kind = 'habit')::bigint as waste_count,
         count(*) filter (where l.kind = 'denial')::bigint as denial_count,
         count(*) filter (where l.kind = 'affirmation')::bigint as affirmation_count,
         max(l.logged_at) as last_log_at
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.logged_at >= date_trunc('week', now() at time zone 'utc')
    and l.kind in ('habit', 'denial', 'affirmation')
  group by p.id
  order by points desc, min(l.logged_at) asc
  limit p_limit;
$$;

create or replace function public.alltime_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  points bigint, waste_count bigint, denial_count bigint, affirmation_count bigint, last_log_at timestamptz
) language sql stable as $$
  select p.id, p.username, p.display_name, p.avatar_url,
         (
           count(*) filter (where l.kind = 'denial') * 10
           + count(*) filter (where l.kind = 'habit')
           + floor(count(*) filter (where l.kind = 'affirmation') / 3)
         )::bigint as points,
         count(*) filter (where l.kind = 'habit')::bigint as waste_count,
         count(*) filter (where l.kind = 'denial')::bigint as denial_count,
         count(*) filter (where l.kind = 'affirmation')::bigint as affirmation_count,
         max(l.logged_at) as last_log_at
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.kind in ('habit', 'denial', 'affirmation')
  group by p.id
  order by points desc, min(l.logged_at) asc
  limit p_limit;
$$;
