-- 0011_privacy_scoring_and_ops.sql — privacy, scoring, sybil floor, and the
-- pieces the ops items need.
--
-- Four independent-looking changes that shipped together because the site
-- should not sit in a half-state between them (the feed must stop leaking
-- BEFORE a scoring change makes the board worth gaming):
--
-- (1) THE FEED STOPS PUBLISHING NOTES. `activity_feed()` returned `note` to
--     unauthenticated callers, so the 140-character confession box was readable
--     by anyone with the URL the moment a log was written, and the live feed
--     was carrying real users' private text. The function no longer has the
--     column at all: a leak that the SQL cannot produce is a leak the HTTP layer
--     cannot reintroduce. A user's own notes still come back from
--     /api/stats (authenticated, own rows only).
--
-- (2) SCORING COUNTS DAYS, NOT ROWS. 0008 computed
--         WLW = count(*) filter (where kind = 'habit')
--     which was harmless while a day could hold one log. When the waste budget
--     became `WLW_DAILY_LIMIT` (default 5), the same behaviour started earning
--     5x: repetition bought rank. Both counts now use
--     `count(distinct log_date)`, so a day of wasting is worth one day however
--     many times it was logged. `devotion_lifetime` stays row-based on purpose —
--     it is a record of activity, not a score, and nothing sorts by it.
--
-- (3) A 24-HOUR FLOOR BEFORE A BOARD APPEARANCE. A brand-new account could top
--     a public board within a minute of registering and there was no cost to
--     making another one. Cheap farming stops being free; a determined farmer
--     can still wait a day, which is the honest limit of a rule that cannot
--     verify anything.
--
-- (4) `days_until_decay` ON THE INDEX. 0009's decay erases a score after 30 days
--     of silence and told nobody: the user found out by returning to a zero.
--     The index now reports the days left, so the dashboard can warn at 10.
--
-- Also here (used by the auth and public-counter work that shipped with it):
-- `today_counts()` and the `login_attempts` table.
--
-- Postgres note, same trap 0009 documents: CREATE OR REPLACE cannot change a
-- function's OUT list (42P13). Every function below whose return type changes is
-- DROPPED first, in dependency order, leaves before the functions they call.
-- The 30-day era, the multiplier tiers, the exclusivity rule, the UTC-day
-- cadence and the lock mechanic are all untouched.

-- ---------------------------------------------------------------------------
-- (1) activity_feed: no note column.
-- ---------------------------------------------------------------------------
drop function if exists public.activity_feed(bigint, int);

create or replace function public.activity_feed(p_cursor bigint, p_limit int default 50)
returns table (
  id bigint, user_id uuid, username text, display_name text, avatar_url text,
  logged_at timestamptz
) language sql stable as $$
  -- Public surface. If a note is ever wanted here it must arrive with an
  -- explicit per-user opt-in, not as a default the user never saw.
  --
  -- `kind = 'habit'` is carried over from 0003 and is load-bearing: the feed
  -- shows completions, never drill reps or denials. Leaving it out of this
  -- rewrite (as the first draft of this migration did) put every affirmation in
  -- the feed, and the suite's `affirmation_kind_separated_from_community` caught
  -- it — a good argument for recreating a function from its previous file rather
  -- than from memory.
  select l.id, l.user_id, p.username, p.display_name, p.avatar_url,
         l.logged_at
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where (p_cursor = 0 or l.id < p_cursor)
    and l.kind = 'habit'
  order by l.id desc
  limit p_limit;
$$;

-- ---------------------------------------------------------------------------
-- (2)+(4) Devotion Index: distinct days, plus the decay countdown.
-- ---------------------------------------------------------------------------
drop function if exists public.daily_leaderboard_weighted(int);
drop function if exists public.weekly_leaderboard_weighted(int);
drop function if exists public.monthly_leaderboard_weighted(int);
drop function if exists public.alltime_leaderboard_weighted(int);
drop function if exists public.devotion_leaderboard(interval, int);
drop function if exists public.devotion_index_alltime(uuid);
drop function if exists public.devotion_index(uuid, timestamptz);

create or replace function public.devotion_index(p_user uuid, p_since timestamptz)
returns table (
  wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, racism_penalty numeric, score numeric,
  lifetime_total bigint, decayed boolean, days_until_decay bigint
) language sql stable as $$
  -- The effective window. For a leaderboard period this is that period's start;
  -- for the all-time view (p_since NULL) it is the 30-day decay horizon, which
  -- IS the all-time rule rather than a leaderboard window. The coalesce is
  -- load-bearing for the same reason 0009 says: without it a single log today
  -- would resurrect a year of stale ones.
  with bounds as (
    select coalesce(p_since, now() - interval '30 days') as since
  ),
  live as (
    select
      -- DISTINCT on the UTC day: five logs on one day are one day of activity.
      -- This is the anti-Goodhart rule -- the multiplier rewards consistency,
      -- so volume must not be able to buy rank on top of it.
      count(distinct l.log_date) filter (where l.kind = 'habit')::bigint as wlw,
      count(distinct l.log_date) filter (where l.kind = 'denial')::bigint as wld,
      -- Affirmations stay row-based: the drill is a repetition exercise, and its
      -- bonus is already 1 point per 3 reps rather than 1 per rep.
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
  newest as (
    select max(x.logged_at) as at from public.habit_logs x where x.user_id = p_user
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
    stale.is_decayed,
    -- Whole days of silence left before the next log is the difference between
    -- a score and a wiped score. Null for a user who has never logged (there is
    -- nothing to lose); 0 once the era has already ended, so the UI can say
    -- "logged out again" rather than "0 days left" for both states.
    case
      when newest.at is null then null
      else greatest(
        0,
        30 - floor(extract(epoch from (now() - newest.at)) / 86400)::bigint
      )::bigint
    end as days_until_decay
  from live cross join lt cross join es cross join stale cross join newest;
$$;

-- The all-time wrapper keeps its name but MEANS "the current 30-day era".
create or replace function public.devotion_index_alltime(p_user uuid)
returns table (
  wlw bigint, wld bigint, game_bonus bigint,
  multiplier numeric, racism_penalty numeric, score numeric,
  lifetime_total bigint, decayed boolean, days_until_decay bigint
) language sql stable as $$
  select * from public.devotion_index(p_user, null);
$$;

-- ---------------------------------------------------------------------------
-- (3) Leaderboard: same shape, plus the 24-hour account-age floor.
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
    -- Sybil floor: an account has to be a day old to be ranked. Without it a
    -- fresh account can hold a top slot within a minute of registering.
    and p.created_at <= now() - interval '24 hours'
  order by di.score desc, last_log_at asc nulls last
  limit p_limit;
$$;

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

-- ---------------------------------------------------------------------------
-- Today's community counters, for the landing page line.
--
-- Public and unauthenticated on purpose: every number in it is already on the
-- KPI page, and "the community is alive today" is the message. Counted on
-- log_date, which is the generated UTC-day column the rest of the site keys on,
-- so this cannot disagree with what a user sees as "today".
-- ---------------------------------------------------------------------------
create or replace function public.today_counts()
returns table (denials_today bigint, wastes_today bigint, loggers_today bigint)
language sql stable as $$
  select
    (select count(*) from public.habit_logs
      where kind = 'denial' and log_date = (now() at time zone 'utc')::date),
    (select count(*) from public.habit_logs
      where kind = 'habit' and log_date = (now() at time zone 'utc')::date),
    (select count(distinct user_id) from public.habit_logs
      where log_date = (now() at time zone 'utc')::date);
$$;

-- ---------------------------------------------------------------------------
-- Failed-login bookkeeping, for the throttle in auth::login.
--
-- A table rather than an in-process counter: the API can be restarted (the
-- deploy script does exactly that), and an attacker who can wait for a restart
-- should not get their budget back. Rows are pruned by the same statement that
-- inserts them, so the table stays bounded without a scheduled job.
--
-- No foreign key to profiles: the throttle must also count attempts against
-- usernames that do not exist, which is where enumeration attempts land.
-- ---------------------------------------------------------------------------
create table if not exists public.login_attempts (
  id bigserial primary key,
  username text not null,
  ip text not null,
  attempted_at timestamptz not null default now()
);

-- Case-insensitive on purpose: usernames are matched case-insensitively at
-- login, so a throttle keyed on the raw string would be bypassable by
-- retyping the same name with different capitalisation.
create index if not exists login_attempts_username_at_idx
  on public.login_attempts (lower(username), attempted_at desc);
create index if not exists login_attempts_ip_at_idx
  on public.login_attempts (ip, attempted_at desc);
