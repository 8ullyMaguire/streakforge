-- 0022 — the product numbers, for the one person allowed to read them.
--
-- Owner (2026-10-29): "#123 a TL4 /admin page with product numbers (weekly active humans,
-- guest→account conversion, retention)".
--
-- Why this is SQL and not a handler full of queries: the numbers are definitions, and a
-- definition written twice drifts. Every figure below is computed in one function so the page,
-- the tests and anyone with psql read the same arithmetic. The handler only gates access and
-- serialises.
--
-- Definitions, stated because a metric without one is a number with a story attached:
--
--   * **member** — a non-guest profile that has not been folded into another (`merged_into is
--     null`). Guests and tombstones are counted separately, never mixed in.
--   * **active** — a member with at least one log in the window. Logs of every kind count,
--     including affirmation reps: the question is "did this person open the site and use it".
--   * **week** — ISO week, Monday 00:00 UTC, from `date_trunc('week', <ts at utc>)`.
--   * **converted guest** — a guest row carrying `merged_into`, counted in the week the GUEST
--     appeared, not the week it registered: the funnel question is "of the people who tried
--     anonymously, how many came back with a name", and the denominator is the week they tried.
--   * **retention, W*n*** — a cohort member with a log in the n-th week after their signup week.
--     W*n* is NULL, not zero, while that week is still in the future; a zero there would read as
--     a total loss of everyone who joined on Monday.
--   * **median logs per active member** — `percentile_cont(0.5)` over per-member counts, not an
--     average of averages: one person logging forty times must not move the typical member.

-- ---------------------------------------------------------------- overview ----

create or replace function public.admin_overview()
returns table (
  members bigint,
  active_1d bigint,
  active_7d bigint,
  new_7d bigint,
  guests_created bigint,
  guests_converted bigint,
  conversion_rate float8,
  logs_1d bigint,
  logs_7d bigint,
  median_logs_per_active_7d float8,
  verified_cage_secs bigint,
  affirmations_live bigint
) language sql stable as $$
  with m as (
    select id, created_at from public.profiles
    where not is_guest and merged_into is null
  ),
  g as (
    select created_at, merged_into from public.profiles where is_guest
  ),
  per_member as (
    select l.user_id, count(*)::float8 as n
    from public.habit_logs l
    join m on m.id = l.user_id
    where l.logged_at > now() - interval '7 days'
    group by l.user_id
  )
  select
    (select count(*) from m)::bigint,
    (select count(distinct l.user_id) from public.habit_logs l join m on m.id = l.user_id
      where l.logged_at > now() - interval '1 day')::bigint,
    (select count(distinct l.user_id) from public.habit_logs l join m on m.id = l.user_id
      where l.logged_at > now() - interval '7 days')::bigint,
    (select count(*) from m where created_at > now() - interval '7 days')::bigint,
    (select count(*) from g)::bigint,
    (select count(*) from g where merged_into is not null)::bigint,
    case
      when (select count(*) from g) = 0 then null
      else ((select count(*) from g where merged_into is not null)::float8
            / (select count(*) from g)::float8)
    end,
    (select count(*) from public.habit_logs where logged_at > now() - interval '1 day')::bigint,
    (select count(*) from public.habit_logs where logged_at > now() - interval '7 days')::bigint,
    coalesce((select percentile_cont(0.5) within group (order by n) from per_member), 0)::float8,
    coalesce((select public.total_verified_cage_hours()), 0)::bigint,
    (select count(*) from public.affirmations where status = 'live')::bigint;
$$;

comment on function public.admin_overview() is
  'One row of product totals for the TL4 page (0022). Members never appears mixed with guests.';

-- ------------------------------------------------------- the weekly series ----

create or replace function public.admin_weekly(p_weeks int default 12)
returns table (
  week_start date,
  active_members bigint,
  new_members bigint,
  logs bigint,
  guests bigint,
  guests_converted bigint
) language sql stable as $$
  with weeks as (
    select generate_series(
      date_trunc('week', (now() at time zone 'utc'))::date
        - make_interval(weeks => greatest(p_weeks, 1) - 1),
      date_trunc('week', (now() at time zone 'utc'))::date,
      interval '1 week'
    )::date as wk
  )
  select
    w.wk,
    coalesce((select count(distinct l.user_id)
                from public.habit_logs l
                join public.profiles p on p.id = l.user_id
               where not p.is_guest and p.merged_into is null
                 and (l.logged_at at time zone 'utc') >= w.wk
                 and (l.logged_at at time zone 'utc') < w.wk + interval '1 week'), 0)::bigint,
    coalesce((select count(*) from public.profiles
               where not is_guest and merged_into is null
                 and (created_at at time zone 'utc') >= w.wk
                 and (created_at at time zone 'utc') < w.wk + interval '1 week'), 0)::bigint,
    coalesce((select count(*) from public.habit_logs
               where (logged_at at time zone 'utc') >= w.wk
                 and (logged_at at time zone 'utc') < w.wk + interval '1 week'), 0)::bigint,
    coalesce((select count(*) from public.profiles
               where is_guest
                 and (created_at at time zone 'utc') >= w.wk
                 and (created_at at time zone 'utc') < w.wk + interval '1 week'), 0)::bigint,
    coalesce((select count(*) from public.profiles
               where is_guest and merged_into is not null
                 and (created_at at time zone 'utc') >= w.wk
                 and (created_at at time zone 'utc') < w.wk + interval '1 week'), 0)::bigint
  from weeks w
  order by w.wk;
$$;

comment on function public.admin_weekly(int) is
  'One row per ISO week (Monday UTC) for the last N weeks, including the incomplete current one.';

-- ---------------------------------------------------------- the cohort grid ----

create or replace function public.admin_retention(p_weeks int default 8)
returns table (
  cohort_start date,
  cohort_size bigint,
  w1 bigint,
  w2 bigint,
  w3 bigint,
  w4 bigint
) language sql stable as $$
  with weeks as (
    select generate_series(
      date_trunc('week', (now() at time zone 'utc'))::date
        - make_interval(weeks => greatest(p_weeks, 1) - 1),
      date_trunc('week', (now() at time zone 'utc'))::date,
      interval '1 week'
    )::date as wk
  ),
  cohorts as (
    select w.wk,
           (select count(*) from public.profiles p
             where not p.is_guest and p.merged_into is null
               and (p.created_at at time zone 'utc') >= w.wk
               and (p.created_at at time zone 'utc') < w.wk + interval '1 week') as size
    from weeks w
  ),
  kept as (
    select c.wk,
           (select count(distinct l.user_id) from public.habit_logs l
              join public.profiles p on p.id = l.user_id
             where not p.is_guest and p.merged_into is null
               and (p.created_at at time zone 'utc') >= c.wk
               and (p.created_at at time zone 'utc') < c.wk + interval '1 week'
               and (l.logged_at at time zone 'utc') >= c.wk + interval '1 week'
               and (l.logged_at at time zone 'utc') < c.wk + interval '2 weeks') as w1,
           (select count(distinct l.user_id) from public.habit_logs l
              join public.profiles p on p.id = l.user_id
             where not p.is_guest and p.merged_into is null
               and (p.created_at at time zone 'utc') >= c.wk
               and (p.created_at at time zone 'utc') < c.wk + interval '1 week'
               and (l.logged_at at time zone 'utc') >= c.wk + interval '2 weeks'
               and (l.logged_at at time zone 'utc') < c.wk + interval '3 weeks') as w2,
           (select count(distinct l.user_id) from public.habit_logs l
              join public.profiles p on p.id = l.user_id
             where not p.is_guest and p.merged_into is null
               and (p.created_at at time zone 'utc') >= c.wk
               and (p.created_at at time zone 'utc') < c.wk + interval '1 week'
               and (l.logged_at at time zone 'utc') >= c.wk + interval '3 weeks'
               and (l.logged_at at time zone 'utc') < c.wk + interval '4 weeks') as w3,
           (select count(distinct l.user_id) from public.habit_logs l
              join public.profiles p on p.id = l.user_id
             where not p.is_guest and p.merged_into is null
               and (p.created_at at time zone 'utc') >= c.wk
               and (p.created_at at time zone 'utc') < c.wk + interval '1 week'
               and (l.logged_at at time zone 'utc') >= c.wk + interval '4 weeks'
               and (l.logged_at at time zone 'utc') < c.wk + interval '5 weeks') as w4
    from cohorts c
  )
  select
    c.wk,
    c.size::bigint,
    -- NULL, not 0, while the window is in the future: a zero would read as "everyone left".
    case when c.wk + interval '1 week' <= (now() at time zone 'utc')::date then k.w1 else null end,
    case when c.wk + interval '2 weeks' <= (now() at time zone 'utc')::date then k.w2 else null end,
    case when c.wk + interval '3 weeks' <= (now() at time zone 'utc')::date then k.w3 else null end,
    case when c.wk + interval '4 weeks' <= (now() at time zone 'utc')::date then k.w4 else null end
  from cohorts c
  join kept k on k.wk = c.wk
  order by c.wk desc;
$$;

comment on function public.admin_retention(int) is
  'Per-signup-week cohorts with the members still logging in weeks 1-4 after joining; future windows are NULL (0022).';
