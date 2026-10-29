-- 0021 — time to cum is shared on purpose, or not at all.
--
-- 0017 published every reporter's average. The brainstorm that followed called it out as a
-- live consent gap (idea #122, score 1.0) and it was right: a duration measured from the
-- start of a session to its end is the most intimate number this site holds, and publishing
-- it by default is a policy nobody agreed to. The confessions on the feed are public by
-- default because the owner said so explicitly; this is the other case.
--
-- What changes, and what deliberately does not:
--
--   * `profiles.share_time_to_cum` defaults to FALSE. It is the only switch on the site
--     whose default is private, and that asymmetry is the point.
--   * `time_to_cum_aggregate()` and `time_to_cum_board()` count only members who share.
--     The board was already gated on 5 samples per person; consent is the gate in front of it.
--   * `time_to_cum_stats(p_user)` is untouched: your own numbers are yours, and the metrics
--     page shows them to you whether you share them or not. Nothing here hides a person's
--     data from that person.
--
-- Consequence, stated rather than discovered later: on the day this lands the public
-- averages are empty, because nobody has opted in yet. The page says that ("nobody shares
-- yet") instead of printing a zero, which is what a `coalesce(avg(...), 0)` would have done.

alter table public.profiles
  add column if not exists share_time_to_cum boolean not null default false;

comment on column public.profiles.share_time_to_cum is
  'Opt-in (0021). When false — the default — this member''s reported durations are excluded from every public average and board. Their own page still shows them.';

-- The OUT list is unchanged, so `create or replace` is legal (widening needs a drop; see
-- 0018 and 0020 for the two times that mattered).
create or replace function public.time_to_cum_aggregate()
returns table (
  samples bigint, avg_secs float8, median_secs float8,
  avg_of_user_avgs_secs float8, reporters bigint
) language sql stable as $$
  with s as (
    select l.user_id, l.time_to_cum_secs
    from public.habit_logs l
    join public.profiles p on p.id = l.user_id
    where l.kind = 'habit'
      and l.time_to_cum_secs is not null
      and l.hidden_at is null
      and p.share_time_to_cum
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

-- Same signature, so this one too can be replaced in place. Consent first, then the
-- five-sample floor: one four-hour outlier must not be a ranking strategy, and a person who
-- did not opt in must not be in the ranking at all.
create or replace function public.time_to_cum_board(p_limit int default 10)
returns table (username text, avatar_url text, samples bigint, avg_secs float8)
language sql stable as $$
  select p.username, p.avatar_url, count(*)::bigint, avg(l.time_to_cum_secs)::float8
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.kind = 'habit'
    and l.time_to_cum_secs is not null
    and l.hidden_at is null
    and p.share_time_to_cum
  group by p.id, p.username, p.avatar_url
  having count(*) >= 5
  order by avg(l.time_to_cum_secs) desc, count(*) desc
  limit p_limit;
$$;
