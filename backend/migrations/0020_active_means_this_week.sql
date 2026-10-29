-- 0020 — "active" means this week, not this month.
--
-- 0019 chose the band by counting distinct non-guest members who logged in the last 30
-- days. Measured on production the same afternoon, that number was 18 — and three of
-- them had logged that day. Eighteen is not a community of eighteen; it is three people
-- and fifteen accounts that tried the site once. A band chosen by that number never bends,
-- so the softening the owner asked for would never have taken effect on the site that
-- needed it.
--
-- A week is the honest unit here for three reasons: the streak is measured in days and
-- every surface on the site talks in weeks, a week is long enough that a person on holiday
-- still counts, and it is short enough that a closed tab drops out. Ten is still the
-- threshold; the number of people it is counting is now the number of people who are here.

create or replace function public.active_accounts()
returns int language sql stable as $$
  select greatest(1, count(distinct l.user_id)::int)
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where not p.is_guest
    and p.merged_into is null
    and l.logged_at > now() - interval '7 days'
$$;

comment on function public.active_accounts() is
  'Distinct non-guest members who logged in the last 7 days, floored at 1. Measured over 30 days it counted one-time visitors and never fell below the small-site threshold (18 accounts, 3 of them present).';
