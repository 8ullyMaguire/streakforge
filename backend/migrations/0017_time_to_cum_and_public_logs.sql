-- 0017_time_to_cum_and_public_logs.sql — session duration, and confessions public by default.
--
-- Owner (2026-10-29):
--   * "time to cum is time it takes from start jerking to release" — so the number
--     is a per-log DURATION the author reports, not the gap between releases.
--   * "confessions/logs are public by default, user can choose to make them private
--     through a setting".
--
-- The second one reverses 0011's decision. That decision was right at the time: the
-- note column was being published with no opt-in, so the fix was to stop publishing
-- it. It is now an explicit publication with a visible switch (`logs_public`), which
-- is what "public by default" has to mean if it is going to be honest.

alter table public.habit_logs
  add column if not exists time_to_cum_secs integer
  check (time_to_cum_secs is null or (time_to_cum_secs >= 0 and time_to_cum_secs <= 86400));

alter table public.profiles
  add column if not exists logs_public boolean not null default true;

-- The public feed, rewritten. What changed:
--   * `note` and `kind` come back — a confession is the content now, by default.
--   * `kind in ('habit','denial')`: denials are the 10x event and the feed ignoring
--     them was a hole, not a policy. Drills stay out (they are practice).
--   * `p.logs_public` and `l.hidden_at is null` are the two ways a row leaves.
drop function if exists public.activity_feed(bigint, int);

create or replace function public.activity_feed(p_cursor bigint, p_limit int default 50)
returns table (
  id bigint, user_id uuid, username text, display_name text, avatar_url text,
  kind text, note text, logged_at timestamptz
) language sql stable as $$
  select l.id, l.user_id, p.username, p.display_name, p.avatar_url,
         l.kind, l.note, l.logged_at
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where (p_cursor = 0 or l.id < p_cursor)
    and l.kind in ('habit', 'denial')
    and p.logs_public
    and l.hidden_at is null
  order by l.id desc
  limit p_limit;
$$;

-- ---- Time to cum -----------------------------------------------------------
-- Only logs where the author reported a duration, and only visible ones. A hidden
-- log is not evidence, here as in the cage metrics.

create or replace function public.time_to_cum_stats(p_user uuid)
returns table (
  samples bigint, avg_secs float8, median_secs float8,
  min_secs int, max_secs int, last_secs int
) language sql stable as $$
  with s as (
    select time_to_cum_secs, logged_at
    from public.habit_logs
    where user_id = p_user
      and kind = 'habit'
      and time_to_cum_secs is not null
      and hidden_at is null
  )
  select
    count(*)::bigint,
    coalesce(avg(time_to_cum_secs), 0)::float8,
    coalesce(percentile_cont(0.5) within group (order by time_to_cum_secs), 0)::float8,
    coalesce(min(time_to_cum_secs), 0)::int,
    coalesce(max(time_to_cum_secs), 0)::int,
    (select time_to_cum_secs from s order by logged_at desc limit 1)::int
  from s;
$$;

-- The two averages differ when one person logs far more than another: `avg_secs`
-- is pooled (every log weighs the same), `avg_of_user_avgs_secs` gives each
-- person one vote. The owner asked for both, so both are named here rather than
-- one being derived from the other in a comment.
create or replace function public.time_to_cum_aggregate()
returns table (
  samples bigint, avg_secs float8, median_secs float8,
  avg_of_user_avgs_secs float8, reporters bigint
) language sql stable as $$
  with s as (
    select user_id, time_to_cum_secs
    from public.habit_logs
    where kind = 'habit' and time_to_cum_secs is not null and hidden_at is null
  ),
  per as (
    select user_id, avg(time_to_cum_secs)::float8 as a from s group by user_id
  )
  select
    (select count(*) from s)::bigint,
    coalesce((select avg(time_to_cum_secs) from s), 0)::float8,
    coalesce((select percentile_cont(0.5) within group (order by time_to_cum_secs) from s), 0)::float8,
    coalesce((select avg(a) from per), 0)::float8,
    (select count(*) from per)::bigint;
$$;

-- MINIMUM 5 SAMPLES: one four-hour outlier must not be a ranking strategy.
create or replace function public.time_to_cum_board(p_limit int default 10)
returns table (username text, avatar_url text, samples bigint, avg_secs float8)
language sql stable as $$
  select p.username, p.avatar_url, count(*)::bigint, avg(l.time_to_cum_secs)::float8
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.kind = 'habit' and l.time_to_cum_secs is not null and l.hidden_at is null
  group by p.id, p.username, p.avatar_url
  having count(*) >= 5
  order by avg(l.time_to_cum_secs) desc, count(*) desc
  limit p_limit;
$$;

-- ---- The other boards the brainstorm asked for -----------------------------

create or replace function public.most_denied_board(p_days int default 30, p_limit int default 10)
returns table (username text, avatar_url text, denials bigint) language sql stable as $$
  select p.username, p.avatar_url, count(*)::bigint
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.kind = 'denial'
    and l.hidden_at is null
    and l.logged_at >= now() - make_interval(days => greatest(p_days, 1))
  group by p.id, p.username, p.avatar_url
  order by count(*) desc, max(l.logged_at) asc
  limit p_limit;
$$;

-- Percentage of the window's days on which the member showed up, any kind.
create or replace function public.consistency_board(p_days int default 30, p_limit int default 10)
returns table (username text, avatar_url text, days bigint, pct float8)
language sql stable as $$
  select p.username, p.avatar_url,
         count(distinct l.log_date)::bigint,
         (count(distinct l.log_date)::float8 / greatest(p_days, 1)::float8 * 100)::float8
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.hidden_at is null
    and l.logged_at >= now() - make_interval(days => greatest(p_days, 1))
  group by p.id, p.username, p.avatar_url
  order by count(distinct l.log_date) desc, p.username asc
  limit p_limit;
$$;

create or replace function public.lock_streak_board(p_limit int default 10)
returns table (username text, avatar_url text, current_streak int, longest_streak int)
language sql stable as $$
  select p.username, p.avatar_url,
         coalesce((select ls.current_streak from public.lock_streak(p.id) ls), 0)::int,
         coalesce((select ls.longest_streak from public.lock_streak(p.id) ls), 0)::int
  from public.profiles p
  where exists (select 1 from public.lock_sessions s where s.user_id = p.id)
  order by 3 desc, 4 desc
  limit p_limit;
$$;
