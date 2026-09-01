-- 0001_init.sql — StreakForge core schema
-- Auth is handled by tower-sessions (server-side session store in the same DB).
-- No auth.users table — sessions reference profiles directly.

create table if not exists public.profiles (
  id uuid primary key default gen_random_uuid(),
  username text unique not null check (username ~ '^[a-zA-Z0-9_]{2,30}$'),
  display_name text check (char_length(display_name) <= 50),
  avatar_url text,
  provider text not null default 'x' check (provider in ('x', 'dev')),
  provider_id text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  unique (provider, provider_id)
);

create table if not exists public.habit_logs (
  id bigint generated always as identity primary key,
  user_id uuid not null references public.profiles(id) on delete cascade,
  logged_at timestamptz not null default now(),
  note text check (char_length(note) <= 140),
  log_date date generated always as ((logged_at at time zone 'utc')::date) stored
);

create index if not exists habit_logs_user_id_logged_at_idx on public.habit_logs(user_id, logged_at desc);
create index if not exists habit_logs_log_date_idx on public.habit_logs(log_date);
create index if not exists habit_logs_logged_at_idx on public.habit_logs(logged_at desc);

-- Streak calculation
-- A day counts if the user has >= 1 log on that UTC date.
-- Current streak = consecutive days ending today (or yesterday if today is empty).
create or replace function public.user_streak(p_user uuid)
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
