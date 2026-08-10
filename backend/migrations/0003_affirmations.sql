-- 0003_affirmations.sql — support affirmation drill as a second log kind.
-- 'habit'       = normal daily completion (1/hr, 5/day budget)
-- 'affirmation' = affirmation drill rep (1/hr, 5/day budget, separate)

alter table public.habit_logs
  add column if not exists kind text not null default 'habit'
  check (kind in ('habit', 'affirmation'));

create index if not exists habit_logs_kind_user_logged_at_idx
  on public.habit_logs(kind, user_id, logged_at desc);

create index if not exists habit_logs_kind_log_date_idx
  on public.habit_logs(kind, log_date);

-- Kind-aware streak calculation (mirror of user_streak but filtered by kind).
create or replace function public.user_streak_kind(p_user uuid, p_kind text)
returns table (current_streak int, longest_streak int) language sql stable as $$
  with days as (
    select distinct log_date
    from public.habit_logs
    where user_id = p_user and kind = p_kind
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

-- Redefine leaderboard / feed / uotd / total functions to only count 'habit'
-- kind (affirmation drill reps stay out of the main community boards).
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
    and l.kind = 'habit'
  group by p.id
  order by count desc, min(l.logged_at) asc
  limit p_limit;
$$;

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
    and l.kind = 'habit'
  group by p.id
  order by count desc, min(l.logged_at) asc
  limit p_limit;
$$;

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
  where l.kind = 'habit'
  group by p.id
  order by count desc, min(l.logged_at) asc
  limit p_limit;
$$;

create or replace function public.user_of_the_day()
returns table (
  user_id uuid, username text, display_name text, avatar_url text,
  count bigint, first_log_at timestamptz, alltime_count bigint
) language sql stable as $$
  select p.id, p.username, p.display_name, p.avatar_url,
         count(l.id) as count,
         min(l.logged_at) as first_log_at,
         (select count(*) from public.habit_logs a where a.user_id = p.id and a.kind = 'habit') as alltime_count
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.log_date = (now() at time zone 'utc')::date
    and l.kind = 'habit'
  group by p.id
  order by count desc, min(l.logged_at) asc
  limit 1;
$$;

create or replace function public.total_logs()
returns bigint language sql stable as $$
  select count(*) from public.habit_logs where kind = 'habit';
$$;

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
    and l.kind = 'habit'
  order by l.id desc
  limit p_limit;
$$;
