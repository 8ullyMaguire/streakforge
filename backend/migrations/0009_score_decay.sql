-- 0009_score_decay.sql — score decay after 30 days of inactivity
--
-- The Devotion Index (0008) is a monotonic count:
--
--     Score = (WLW + WLD + Game Bonus) x Multiplier
--
-- Nothing in it could ever decrease. The multiplier resets when a streak
-- breaks, but the underlying counts are permanent, so a user could log once
-- and hold a score forever by never touching the site again. This migration
-- makes inactivity cost something.
--
-- (1) DECAY. A user who has not logged anything for 30 days loses their
--     score entirely; it stays 0 until they log again. The drain is measured
--     in 30-DAY ERAS, not in a per-second rate: each completed 30-day
--     inactivity period erases the score accumulated up to that point, so a
--     lapse costs exactly one month's earnings and no more. Between the 1st
--     and 30th day of silence there is no partial decay — the score simply
--     becomes 0 on day 31. There is no per-day drip, no rounding curve, and
--     no gradual fade.
--
--     Why eras rather than a rate: a rate needs a score column, a last-decay
--     timestamp, and a scheduled job to move the number forward, and it
--     disagrees with itself depending on when you look. A windowed pure
--     function over the log table is always exactly consistent with the logs
--     that produced it, needs no writes, and cannot be desynchronised.
--
--     Applied as: logs from the last 30 days only, and a score of 0 if the
--     most recent log is older than 30 days. A user who logs today scores as
--     if the 30 days before today were the only ones that ever counted. A
--     user who stopped in March has 0 in October, forever, until they log
--     again — the "no recovery" requirement.
--
-- (2) LIFETIME TOTAL. Decay must not silently erase history, so a separate
--     never-decaying figure is kept alongside it. `lifetime_total` counts
--     every log the user has ever made and is deliberately NOT part of the
--     decay rule. It is a record, not a rankable score: nothing sorts by it
--     and no multiplier applies to it.
--
--     The windowed boards (daily/weekly/monthly) already only ever counted
--     logs inside their window, so they needed no change; they now inherit the
--     same rule for free via devotion_index.
--
-- Nothing here changes the cadence, the exclusivity rule, or the multiplier
-- tiers. Only how long a log keeps counting.

-- ---------------------------------------------------------------------------
-- Drop the functions we are about to redefine.
--
-- Postgres refuses to change a function's OUT-parameter list with
-- CREATE OR REPLACE (42P13: "cannot change return type of existing function").
-- Each of these widens its return type, so every one needs an explicit DROP.
--
-- Order matters: the wrappers depend on devotion_leaderboard, which depends on
-- devotion_index. Dropping leaves-first keeps the dependency graph valid at every
-- step, so no statement here can fail on a missing dependency.
--
-- The existing profiles rows are untouched by all of this: these are read-only
-- scoring functions, and nothing reads them during the drop window.
-- ---------------------------------------------------------------------------
drop function if exists public.daily_leaderboard_weighted(int);
drop function if exists public.weekly_leaderboard_weighted(int);
drop function if exists public.monthly_leaderboard_weighted(int);
drop function if exists public.alltime_leaderboard_weighted(int);
drop function if exists public.devotion_leaderboard(interval, int);
drop function if exists public.devotion_index_alltime(uuid);
drop function if exists public.devotion_index(uuid, timestamptz);

-- ---------------------------------------------------------------------------
-- Lifetime totals. Never decays; the record that survives a reset.
-- ---------------------------------------------------------------------------
create or replace function public.devotion_lifetime(p_user uuid)
returns table (
  lifetime_wlw bigint, lifetime_wld bigint, lifetime_game_bonus bigint,
  lifetime_total bigint
) language sql stable as $$
  select
    count(*) filter (where l.kind = 'habit')::bigint,
    count(*) filter (where l.kind = 'denial')::bigint,
    floor(count(*) filter (where l.kind = 'affirmation') / 3)::bigint,
    count(*)::bigint
  from public.habit_logs l
  where l.user_id = p_user;
$$;

-- ---------------------------------------------------------------------------
-- Devotion Index, with the 30-day decay rule.
--
-- p_since = the start of a leaderboard window, or NULL for "the last 30 days"
-- under the decay rule. A windowed board passes now() - p_interval; the
-- decay floor is applied on top of whatever window is in force, so no board
-- can report a score earned outside its own window.
--
-- Signature is unchanged from 0008, so every existing call site keeps working.
-- The two added return columns are appended to the END of the list: Rust call
-- sites select named columns rather than `select *`, but a positional consumer
-- would still see the original six in their original order.
-- ---------------------------------------------------------------------------
create or replace function public.devotion_index(p_user uuid, p_since timestamptz)
returns table (
  wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, racism_penalty numeric, score numeric,
  lifetime_total bigint, decayed boolean
) language sql stable as $$
  -- The effective window. For a leaderboard period this is that period's start;
  -- for the all-time view (p_since NULL) it is the 30-day decay horizon, which
  -- IS the all-time rule rather than a leaderboard window.
  --
  -- This coalesce is load-bearing. Without it the all-time view counts EVERY
  -- log ever and then only asks whether the newest one is recent, so a single
  -- log today would resurrect a year of stale ones instead of wiping them.
  with bounds as (
    select coalesce(p_since, now() - interval '30 days') as since
  ),
  live as (
    select
      count(*) filter (where l.kind = 'habit')::bigint as wlw,
      count(*) filter (where l.kind = 'denial')::bigint as wld,
      floor(count(*) filter (where l.kind = 'affirmation') / 3)::bigint as game_bonus
    from public.habit_logs l, bounds b
    where l.user_id = p_user
      and l.logged_at >= b.since
  ),
  -- `decayed` is evaluated against the 30-day horizon, not against p_since, so
  -- a windowed board cannot report a user as "decayed" merely for being outside
  -- a 24-hour window. It means: has logs, but none inside 30 days.
  stale as (
    select
      exists (select 1 from public.habit_logs x where x.user_id = p_user)
      and not exists (
        select 1 from public.habit_logs x
        where x.user_id = p_user
          and x.logged_at >= now() - interval '30 days'
      ) as is_decayed
  ),
  lt as (select * from public.devotion_lifetime(p_user)),
  es as (select multiplier from public.exclusive_streak(p_user))
  select
    live.wlw,
    live.wld,
    live.game_bonus,
    es.multiplier,
    0::numeric as racism_penalty,
    round((live.wlw + live.wld + live.game_bonus)::numeric * es.multiplier, 2) as score,
    -- Lifetime is measured over ALL logs, independent of any window: a windowed
    -- board must not report a lifetime that shrinks when the window narrows.
    lt.lifetime_total,
    stale.is_decayed
  from live cross join lt cross join es cross join stale;
$$;

-- The all-time wrapper keeps its name but now MEANS "the current 30-day era",
-- which is the same figure the all-time board ranks by.
create or replace function public.devotion_index_alltime(p_user uuid)
returns table (
  wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, racism_penalty numeric, score numeric,
  lifetime_total bigint, decayed boolean
) language sql stable as $$
  select * from public.devotion_index(p_user, null);
$$;

-- ---------------------------------------------------------------------------
-- Leaderboard. Unchanged signature; the decay rule arrives via devotion_index.
--
-- `where di.score > 0` is unchanged and now also excludes everyone in the
-- wiped state, so a lapsed user falls off the board instead of sitting there
-- as a zero.
-- ---------------------------------------------------------------------------
create or replace function public.devotion_leaderboard(p_interval interval, p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  score numeric, wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, streak int, active_kind text, last_log_at timestamptz,
  lifetime_total bigint
) language sql stable as $$
  select
    p.id, p.username, p.display_name, p.avatar_url,
    di.score, di.wlw, di.wld, di.game_bonus, di.multiplier,
    es.streak::int, es.active_kind,
    (
      select max(l.logged_at) from public.habit_logs l
      where l.user_id = p.id
        and l.kind in ('habit', 'denial', 'affirmation')
        -- Same effective window the score uses: the period when given, else the
        -- 30-day decay horizon. Leaving this on the old NULL-means-everything
        -- rule would print a year-old "last seen" for a user whose score is 0.
        and l.logged_at >= coalesce(
          case when p_interval is null then null else now() - p_interval end,
          now() - interval '30 days'
        )
    ) as last_log_at,
    di.lifetime_total
  from public.profiles p
  cross join lateral public.devotion_index(
    p.id, case when p_interval is null then null else now() - p_interval end
  ) di
  cross join lateral public.exclusive_streak(p.id) es
  where di.score > 0
  order by di.score desc, last_log_at asc nulls last
  limit p_limit;
$$;

-- Named wrappers, so existing call sites keep working unchanged. Each must
-- redeclare the full return type, which is why `lifetime_total` is threaded
-- through all four.

create or replace function public.daily_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  score numeric, wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, streak int, active_kind text, last_log_at timestamptz,
  lifetime_total bigint
) language sql stable as $$
  select * from public.devotion_leaderboard(interval '24 hours', p_limit);
$$;

create or replace function public.weekly_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  score numeric, wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, streak int, active_kind text, last_log_at timestamptz,
  lifetime_total bigint
) language sql stable as $$
  select * from public.devotion_leaderboard(interval '7 days', p_limit);
$$;

create or replace function public.monthly_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  score numeric, wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, streak int, active_kind text, last_log_at timestamptz,
  lifetime_total bigint
) language sql stable as $$
  select * from public.devotion_leaderboard(interval '30 days', p_limit);
$$;

create or replace function public.alltime_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  score numeric, wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, streak int, active_kind text, last_log_at timestamptz,
  lifetime_total bigint
) language sql stable as $$
  select * from public.devotion_leaderboard(null, p_limit);
$$;
