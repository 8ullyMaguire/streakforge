-- 0007_rolling_windows.sql
--
-- Leaderboard period semantics change:
--   - DAILY   = rolling 24 hours  (was: current UTC calendar date)
--   - WEEKLY  = rolling 7 days    (was: current UTC calendar week, Monday start)
--   - ALLTIME = unchanged
--
-- Why: the UTC-calendar-day filter made the DAILY board empty between UTC
-- midnight and the first log of the new day, even when there was plenty of
-- activity in the last 24 hours (the site's peak is US evening = 00:00–06:00
-- UTC). Same reasoning applied to weekly for consistency.
--
-- NOTE: only the *_weighted functions the API actually uses are redefined here.
-- The legacy unweighted functions (0001/0002) and user_of_the_day() are left
-- untouched — they are not exposed via the API.

drop function if exists public.daily_leaderboard_weighted(int);
drop function if exists public.weekly_leaderboard_weighted(int);

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
  where l.logged_at >= now() - interval '24 hours'
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
  where l.logged_at >= now() - interval '7 days'
    and l.kind in ('habit', 'denial', 'affirmation')
  group by p.id
  order by points desc, min(l.logged_at) asc
  limit p_limit;
$$;
