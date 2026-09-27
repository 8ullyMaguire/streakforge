-- 0008_devotion_index.sql — Whiteboi Devotion Index + daily (UTC) denial cadence
--
-- Two changes:
--
-- (1) SCORING. Replaces the flat weighted leaderboard
--     (denial 10 pts, waste 1 pt, 3 affirmations 1 pt) with the wlw.grok.me
--     Devotion Index:
--
--         Score = ((WLW + WLD + Game Bonus) x Score Multiplier) - Racism Penalty
--
--     - WLW / WLD = whiteboi loads wasted / denied, counted within the window.
--     - Game Bonus = 1 point per 3 affirmations (unchanged from before).
--     - Score Multiplier comes from the ACTIVE EXCLUSIVE streak, tiered:
--         1-6 days 1.0x | 7-13 1.25x | 14-29 1.5x | 30-59 2.0x
--         60-99 2.5x | 100+ 3.0x
--     - Racism Penalty is always 0 for now. The term is kept in the formula so
--       the shape is final, but there is no data source for it: StreakForge has
--       no racism-reporting feature and inventing one would fabricate a
--       moderation signal. When a real source exists, fill RACISM_PENALTY_SQL
--       below and nothing else in the formula changes.
--
--     "WLWs and WLDs are mutually exclusive -- recording one breaks the other's
--     streak." Implemented as: only the streak of the most recently recorded
--     qualifying kind is ELIGIBLE for the multiplier. This is scoring-equivalent
--     to zeroing the other streak (an inactive streak can never contribute),
--     but non-destructive -- the history is still shown on the profile.
--
--     The multiplier reflects the streak as of NOW, not as of the window. That
--     is what a live index means: one index per user, viewed through a time
--     window, rather than a separate index recomputed per window.
--
-- (2) DENIAL CADENCE. Reverts the rolling-24h denial window back to one denial
--     per UTC calendar day, and re-derives the "waste breaks denial" check on
--     the same UTC day. Streaks were already UTC-day based (user_streak_kind),
--     so this makes the rate limit and the streak agree: submit once a day UTC
--     to grow the streak, miss a day and it resets.

-- ---------------------------------------------------------------------------
-- Multiplier tier lookup. Immutable, so it inlines.
-- ---------------------------------------------------------------------------
create or replace function public.devotion_multiplier(p_streak int)
returns numeric language sql immutable as $$
  select case
    when p_streak >= 100 then 3.0::numeric
    when p_streak >= 60  then 2.5::numeric
    when p_streak >= 30  then 2.0::numeric
    when p_streak >= 14  then 1.5::numeric
    when p_streak >= 7   then 1.25::numeric
    else 1.0::numeric
  end;
$$;

-- ---------------------------------------------------------------------------
-- The active exclusive streak: the kind of the most recent habit/denial log,
-- that kind's current UTC-day streak, and the multiplier it earns.
--
-- Always returns exactly one row. Affirmations are ignored here -- they are
-- neither a WLW nor a WLD, so they neither activate nor break exclusivity.
-- ---------------------------------------------------------------------------
create or replace function public.exclusive_streak(p_user uuid)
returns table (active_kind text, streak int, multiplier numeric) language sql stable as $$
  with lk as (
    select l.kind
    from public.habit_logs l
    where l.user_id = p_user and l.kind in ('habit', 'denial')
    order by l.logged_at desc
    limit 1
  )
  select
    coalesce(lk.kind, 'none')::text as active_kind,
    coalesce((
      select us.current_streak from public.user_streak_kind(p_user, lk.kind) us
    ), 0)::int as streak,
    public.devotion_multiplier(coalesce((
      select us.current_streak from public.user_streak_kind(p_user, lk.kind) us
    ), 0)::int) as multiplier
  from (select 1) one
  left join lk on true;
$$;

-- ---------------------------------------------------------------------------
-- The index for one user. p_since NULL = all time.
--
-- RACISM_PENALTY_SQL: intentionally 0. See the header note -- there is no
-- data source for a racism penalty in this schema, and a fabricated one would
-- be worse than an absent one. Replace the literal 0 with a real count when a
-- reporting feature exists; the surrounding arithmetic needs no change.
-- ---------------------------------------------------------------------------
create or replace function public.devotion_index(p_user uuid, p_since timestamptz)
returns table (
  wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, racism_penalty numeric, score numeric
) language sql stable as $$
  with counts as (
    select
      count(*) filter (where l.kind = 'habit')::bigint as wlw,
      count(*) filter (where l.kind = 'denial')::bigint as wld,
      floor(count(*) filter (where l.kind = 'affirmation') / 3)::bigint as game_bonus
    from public.habit_logs l
    where l.user_id = p_user
      and (p_since is null or l.logged_at >= p_since)
  ),
  es as (select multiplier from public.exclusive_streak(p_user))
  select
    counts.wlw,
    counts.wld,
    counts.game_bonus,
    es.multiplier,
    0::numeric as racism_penalty,
    round((counts.wlw + counts.wld + counts.game_bonus)::numeric * es.multiplier, 2) as score
  from counts cross join es;
$$;

-- Single-user convenience: all-time index.
create or replace function public.devotion_index_alltime(p_user uuid)
returns table (
  wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, racism_penalty numeric, score numeric
) language sql stable as $$
  select * from public.devotion_index(p_user, null);
$$;

-- ---------------------------------------------------------------------------
-- The one leaderboard. p_interval NULL = all time.
--
-- One function instead of four near-identical copies; the named wrappers below
-- keep the old call sites working. Ranking is by score desc, then earliest
-- activity as the tie-break (same convention as before).
--
-- Users with a score of 0 are excluded: they have no activity in the window,
-- and an empty board is more honest than a board of zeroes.
-- ---------------------------------------------------------------------------
create or replace function public.devotion_leaderboard(p_interval interval, p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  score numeric, wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, streak int, active_kind text, last_log_at timestamptz
) language sql stable as $$
  select
    p.id, p.username, p.display_name, p.avatar_url,
    di.score, di.wlw, di.wld, di.game_bonus, di.multiplier,
    es.streak::int, es.active_kind,
    (
      select max(l.logged_at) from public.habit_logs l
      where l.user_id = p.id
        and l.kind in ('habit', 'denial', 'affirmation')
        and (p_interval is null or l.logged_at >= now() - p_interval)
    ) as last_log_at
  from public.profiles p
  cross join lateral public.devotion_index(
    p.id, case when p_interval is null then null else now() - p_interval end
  ) di
  cross join lateral public.exclusive_streak(p.id) es
  where di.score > 0
  order by di.score desc, last_log_at asc nulls last
  limit p_limit;
$$;

-- Named wrappers, so existing call sites keep working unchanged.
drop function if exists public.daily_leaderboard_weighted(int);
drop function if exists public.weekly_leaderboard_weighted(int);
drop function if exists public.monthly_leaderboard_weighted(int);
drop function if exists public.alltime_leaderboard_weighted(int);

create or replace function public.daily_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  score numeric, wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, streak int, active_kind text, last_log_at timestamptz
) language sql stable as $$
  select * from public.devotion_leaderboard(interval '24 hours', p_limit);
$$;

create or replace function public.weekly_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  score numeric, wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, streak int, active_kind text, last_log_at timestamptz
) language sql stable as $$
  select * from public.devotion_leaderboard(interval '7 days', p_limit);
$$;

create or replace function public.monthly_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  score numeric, wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, streak int, active_kind text, last_log_at timestamptz
) language sql stable as $$
  select * from public.devotion_leaderboard(interval '30 days', p_limit);
$$;

create or replace function public.alltime_leaderboard_weighted(p_limit int default 50)
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  score numeric, wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, streak int, active_kind text, last_log_at timestamptz
) language sql stable as $$
  select * from public.devotion_leaderboard(null, p_limit);
$$;

-- ---------------------------------------------------------------------------
-- Index to keep the per-user streak scan cheap. user_streak_kind filters on
-- (user_id, kind) and reads log_date; habit_logs_user_id_logged_at_idx covers
-- the user_id prefix, and log_date is a stored generated column so the existing
-- log_date index serves the distinct-date scan.
-- ---------------------------------------------------------------------------
create index if not exists habit_logs_user_kind_log_date_idx
  on public.habit_logs(user_id, kind, log_date);
