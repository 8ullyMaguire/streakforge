-- 0010_kpis.sql — community KPI dashboard functions
--
-- kpi_totals(): one row of headline numbers for the /kpi page.
-- kpi_trend(n): per-UTC-day counts of each kind for the last n days (n default 30).
--
-- Note: denial_rate = denied / (wasted + denied) * 100  ("of all loads committed,
-- what % were denied"). Zero-divided -> 0.
--
-- Named 0010, not 0008 as the plan says: 0008_devotion_index.sql and
-- 0009_score_decay.sql landed after the plan was written on 2026-08-30. A
-- migration number is a position in a sequence, and reusing one that is
-- already applied is how a chain forks.
--
-- numeric is cast to float8 in both functions rather than decoded as
-- BigDecimal in Rust. The plan flagged this as the one compile risk and
-- recommended the SQL cast precisely so no Rust change is needed.

create or replace function public.kpi_totals()
returns table (
  total_wasted bigint,
  total_denied bigint,
  total_affirmations bigint,
  total_users bigint,
  active_24h bigint,
  new_7d bigint,
  currently_locked bigint,
  total_lock_hours double precision,
  denial_rate double precision
) language sql stable as $$
  select
    (select count(*) from public.habit_logs where kind = 'habit'),
    (select count(*) from public.habit_logs where kind = 'denial'),
    (select count(*) from public.habit_logs where kind = 'affirmation'),
    (select count(*) from public.profiles),
    (select count(distinct user_id) from public.habit_logs where logged_at >= now() - interval '24 hours'),
    (select count(*) from public.profiles where created_at >= now() - interval '7 days'),
    (select count(*) from public.lock_sessions where unlocked_at is null),
    (select coalesce(sum(extract(epoch from (coalesce(unlocked_at, now()) - locked_at)) / 3600.0), 0)::double precision
     from public.lock_sessions),
    (select case
       when (select count(*) from public.habit_logs where kind in ('habit','denial')) = 0 then 0::double precision
       else round(
         (select count(*) from public.habit_logs where kind = 'denial')::numeric
         / (select count(*) from public.habit_logs where kind in ('habit','denial')) * 100, 1)::double precision
     end)
$$;

-- The trend deliberately left-joins the day series so every one of the n days
-- is present, including days with no activity. A chart that omits empty days
-- silently compresses the x-axis and makes a quiet week look busy.
create or replace function public.kpi_trend(p_days int default 30)
returns table (day text, wasted bigint, denied bigint, affirmations bigint)
language sql stable as $$
  with days as (
    select generate_series(
      (now() at time zone 'utc')::date - (p_days - 1),
      (now() at time zone 'utc')::date,
      interval '1 day'
    )::date as day
  )
  select to_char(d.day, 'YYYY-MM-DD') as day,
         count(l.id) filter (where l.kind = 'habit')::bigint as wasted,
         count(l.id) filter (where l.kind = 'denial')::bigint as denied,
         count(l.id) filter (where l.kind = 'affirmation')::bigint as affirmations
  from days d
  left join public.habit_logs l on l.log_date = d.day
  group by d.day
  order by d.day;
$$;
