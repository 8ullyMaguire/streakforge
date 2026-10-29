-- 0018_catch_up_and_gdpr.sql — the yesterday window, and the data rights that make
-- an export and an erasure possible without a hand-written list of tables.
--
-- Two things drove this migration:
--
-- 1. People log the day they remember, which is often after midnight. A log filed
--    for the previous UTC day is allowed exactly once per member per filing day,
--    and only when that day is empty — so the streak is repaired, not inflated.
-- 2. The terms promise the member can take their data away and have it erased.
--    Neither was possible: there was no export endpoint and no path to deletion
--    except a row in psql.
--
-- `created_at` is the honest clock for the undo window: a backdated log describes
-- yesterday but was written now, and the 15-minute window is about the writing.

alter table public.habit_logs
  -- `now()` as the default rewrites the table once; it is 200-odd rows and the
  -- alternative (nullable + backfill + set not null) buys nothing but ceremony.
  add column if not exists created_at timestamptz not null default now(),
  add column if not exists is_catch_up boolean not null default false,
  -- The UTC date the catch-up was FILED on, not the date it describes. Two
  -- consecutive catch-ups would otherwise be legal for two different yesterdays.
  add column if not exists catch_up_on date;

-- A log is either an ordinary log or a catch-up with a filing date; never a
-- flag without the date it is counted by.
alter table public.habit_logs
  drop constraint if exists habit_logs_catch_up_shape;
alter table public.habit_logs
  add constraint habit_logs_catch_up_shape check (
    (is_catch_up and catch_up_on is not null)
    or (not is_catch_up and catch_up_on is null)
  );

-- One filed-late log per member per filing day. This is the rule the endpoint
-- reports as 409, enforced by the database so a race cannot produce two.
create unique index if not exists habit_logs_one_catch_up_per_filing_day
  on public.habit_logs (user_id, catch_up_on)
  where is_catch_up;

-- The undo window is measured from when the row was written (`created_at`), and
-- the "your own recent logs" query reads newest-written first.
create index if not exists habit_logs_user_id_created_at_idx
  on public.habit_logs (user_id, created_at desc);

-- ---- The feed says when a log was filed late ------------------------------
-- A backdated log appears on the wall with yesterday's date (correct: that is
-- when it happened), but a reader may fairly ask why it arrived today. One
-- boolean, so the answer travels with the row instead of being guessed from
-- `logged_at < created_at`.

-- `create or replace` cannot widen an OUT-parameter list (42P13: "cannot change
-- return type of existing function"), so the function is dropped and rebuilt. It
-- is a leaf: nothing else calls it, and it carries no grants to preserve.
drop function if exists public.activity_feed(bigint, integer);

create or replace function public.activity_feed(p_cursor bigint, p_limit int default 50)
returns table (
  id bigint, user_id uuid, username text, display_name text, avatar_url text,
  kind text, note text, logged_at timestamptz, catch_up boolean
) language sql stable as $$
  select l.id, l.user_id, p.username, p.display_name, p.avatar_url,
         l.kind, l.note, l.logged_at, l.is_catch_up
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where (p_cursor = 0 or l.id < p_cursor)
    and l.kind in ('habit', 'denial')
    and p.logs_public
    and l.hidden_at is null
  order by l.id desc
  limit p_limit;
$$;

-- ---- What a member's own history shows ------------------------------------
-- The stats endpoint lists the caller's recent logs per kind, including the
-- catch-up label; keeping it a function rather than an inline query means the
-- export below and the dashboard agree on what a log row is.

create or replace function public.my_recent_logs(p_user uuid, p_kind text, p_limit int default 20)
returns table (
  id bigint, user_id uuid, username text, display_name text, avatar_url text,
  logged_at timestamptz, note text, catch_up boolean
) language sql stable as $$
  select l.id, l.user_id, p.username, p.display_name, p.avatar_url,
         l.logged_at, l.note, l.is_catch_up
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where l.user_id = p_user and l.kind = p_kind
  order by l.created_at desc
  limit p_limit;
$$;
