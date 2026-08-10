-- 0002_leaderboards.sql — leaderboard + user-of-the-day queries

-- Daily leaderboard: logs on current UTC date, ordered by count desc,
-- tie-break earliest max log (first to reach the count).
create or replace function public.daily_leaderboard(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  count bigint, last_log_at timestamptz
) language sql stable as $$
  select p.id, p.username, p.display_name, p.avatar_url,
         count(l.id) as count,
         max(l.logged_at) as last_log_at
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.log_date = (now() at time zone 'utc')::date
  group by p.id
  order by count desc, min(l.logged_at) asc
  limit p_limit;
$$;

-- Weekly leaderboard: since Monday 00:00 UTC
create or replace function public.weekly_leaderboard(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  count bigint, last_log_at timestamptz
) language sql stable as $$
  select p.id, p.username, p.display_name, p.avatar_url,
         count(l.id) as count,
         max(l.logged_at) as last_log_at
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.logged_at >= date_trunc('week', now() at time zone 'utc')
  group by p.id
  order by count desc, min(l.logged_at) asc
  limit p_limit;
$$;

-- All-time leaderboard
create or replace function public.alltime_leaderboard(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  count bigint, last_log_at timestamptz
) language sql stable as $$
  select p.id, p.username, p.display_name, p.avatar_url,
         count(l.id) as count,
         max(l.logged_at) as last_log_at
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  group by p.id
  order by count desc, min(l.logged_at) asc
  limit p_limit;
$$;

-- User of the day: top daily count, tie-break earliest first log, then all-time.
create or replace function public.user_of_the_day()
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  count bigint, first_log_at timestamptz, alltime_count bigint
) language sql stable as $$
  select p.id, p.username, p.display_name, p.avatar_url,
         count(l.id) as count,
         min(l.logged_at) as first_log_at,
         (select count(*) from public.habit_logs a where a.user_id = p.id) as alltime_count
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.log_date = (now() at time zone 'utc')::date
  group by p.id
  order by count desc, min(l.logged_at) asc
  limit 1;
$$;

-- Total completions (for the big hero counter)
create or replace function public.total_logs()
returns bigint language sql stable as $$
  select count(*) from public.habit_logs;
$$;

-- Feed: recent logs with profile join
create or replace function public.activity_feed(p_cursor bigint, p_limit int default 50)
returns table (
  id bigint, user_id uuid, username text, display_name text, avatar_url text,
  logged_at timestamptz, note text
) language sql stable as $$
  select l.id, l.user_id, p.username, p.display_name, p.avatar_url,
         l.logged_at, l.note
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where (p_cursor = 0 or l.id < p_cursor)
  order by l.id desc
  limit p_limit;
$$;

-- Profile stats for a given user
create or replace function public.profile_stats(p_user uuid)
returns table (
  streak int, longest_streak int, today_count bigint, week_count bigint, alltime_count bigint
) language sql stable as $$
  select
    coalesce((select current_streak from public.user_streak(p_user)), 0),
    coalesce((select longest_streak from public.user_streak(p_user)), 0),
    (select count(*) from public.habit_logs where user_id = p_user and log_date = (now() at time zone 'utc')::date),
    (select count(*) from public.habit_logs where user_id = p_user and logged_at >= date_trunc('week', now() at time zone 'utc')),
    (select count(*) from public.habit_logs where user_id = p_user)
$$;
