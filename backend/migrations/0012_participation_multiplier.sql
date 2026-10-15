-- 0012: the multiplier follows DAILY PARTICIPATION, not one kind's run.
--
-- THE DEFECT. A member who shows up every UTC day but alternates what she does
-- -- a waste Monday, a denial Tuesday, a drill Wednesday -- earned 1.0x, the
-- same as someone with no unbroken run at all. The multiplier was read off
-- user_streak_kind(<the kind of her most recent log>), so ANY change of kind
-- reset the run to 1 and the tier collapsed with it. Measured before this
-- migration, 8 days present in every case:
--
--   alternating waste/denial  -> streak 1, multiplier 1.0   (must be 1.25)
--   waste daily + one drill   -> streak 3, multiplier 1.0   (must be 1.25)
--   7 days present, 2-day gap -> streak 0, multiplier 1.0   (correct, stays)
--
-- THE RULE NOW. Showing up is what the multiplier rewards; what you chose to do
-- that day is what the exclusive streak reports. The exclusive streak keeps its
-- exact former meaning (consecutive days of the ONE kind you last chose, which
-- may reset when you change kind). The multiplier moves onto a new reading:
-- consecutive UTC days with at least one log of ANY kind, drills included.
--
-- MONOTONIC. The participation streak counts a superset of the days any
-- per-kind streak counts, and keeps the same "today may still be empty" rule,
-- so no multiplier can fall on deploy and nobody loses score for this change.
--
-- The function below is user_streak_kind (0005) with the kind filter removed:
-- same gaps-and-islands, same today-or-yesterday branch. Keep the two in step.

create or replace function public.user_participation_streak(p_user uuid)
returns table (current_streak int, longest_streak int) language sql stable as $$
  with days as (
    select distinct log_date
    from public.habit_logs
    where user_id = p_user
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

-- The same row as before, with the multiplier moved onto participation.
--
--   active_kind  the kind of the most recent habit/denial log -- what the
--                exclusive streak is a run of. Affirmations never hold it.
--   streak       consecutive UTC days of THAT kind: it may reset when the
--                member changes kind. Unchanged, by design.
--   multiplier   the platform's ONE multiplier, read off the participation
--                streak above. It is returned here because every call site that
--                wants one of these wants both, and because a second,
--                differently-sourced multiplier column is how a future caller
--                picks the wrong one.
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
      select ps.current_streak from public.user_participation_streak(p_user) ps
    ), 0)::int) as multiplier
  from (select 1) one
  left join lk on true;
$$;

-- A kind-agnostic day scan is what user_participation_streak does, and
-- habit_logs_user_id_logged_at_idx covers the user_id prefix it filters on, so
-- no new index is needed here. (0005's habit_logs_user_kind_log_date_idx still
-- serves the per-kind scan.)
