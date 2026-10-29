-- 0019 — the deck belongs to the members, and the ladder bends while the site is small.
--
-- Two decisions, both from the owner's ask ("allow users to add entries to the
-- affirmation drill; soften trust level requirements while there are few users"):
--
-- 1. **The deck is a table, not a TypeScript constant.** 19 curated lines are seeded
--    here as `author_id = null` (nobody's, and nothing to cascade when a member
--    deletes their account); member submissions land beside them and are admitted
--    by trust rather than by an editor. The frontend constant survives only as the
--    offline fallback for a drill that cannot reach the API — it is no longer the
--    source of truth, and it says so.
--
-- 2. **The trust ladder has two bands, chosen by how many people actually show up.**
--    The 2026-10-29 thresholds (7 distinct days for level 2, 30 for level 3) were
--    written for a site with dozens of members; with three the moderation queue
--    starves and every cage check sits unverified. So the requirements scale with
--    `active_accounts()` — distinct non-guest members who logged in the last 30
--    days — and TIGHTEN BACK automatically at 10. The numbers a member sees come
--    from `trust_thresholds()`, which is what `/api/trust` now ships, so the page
--    cannot print a requirement the function does not enforce.

-- ---------------------------------------------------------------- the deck ----

create table if not exists public.affirmations (
  id bigserial primary key,
  -- null = curated seed. A member's row cascades with their account: an affirmation
  -- you wrote is yours, and the deck losing it is the honest consequence of erasure.
  author_id uuid references public.profiles(id) on delete cascade,
  body text not null check (char_length(btrim(body)) between 3 and 200),
  source text not null default 'member',
  status text not null default 'pending' check (status in ('pending', 'live', 'hidden')),
  created_at timestamptz not null default now(),
  reviewed_at timestamptz,
  reviewed_by uuid references public.profiles(id) on delete set null,
  hidden_at timestamptz,
  -- a submitted line cannot start live; admission is a decision someone made
  constraint affirmations_live_is_reviewed check (
    status <> 'live' or author_id is null or reviewed_at is not null
  )
);

comment on table public.affirmations is
  'The drill deck. Curated rows have author_id null; member rows are admitted by a trust-level-2 voice (or by their own author once they hold one).';

-- The same line twice is noise in a deck that is read out loud. Case- and
-- whitespace-insensitive, and only among the live ones: a hidden line may legitimately
-- be resubmitted, and two pending twins are one queue item, not a failure.
create unique index if not exists affirmations_one_live_per_line
  on public.affirmations (lower(btrim(body))) where status = 'live';

create index if not exists affirmations_status_idx on public.affirmations (status, created_at desc);
create index if not exists affirmations_author_idx on public.affirmations (author_id, created_at desc);

-- The curated deck, verbatim from what the frontend shipped before this migration —
-- 18 lines, not 19: "I'm racist and I submit!" shipped twice under two source tags
-- ("the drill gate", "required before wasting a load"), and a deck that reads the same
-- line out loud twice is a bug rather than a feature.
insert into public.affirmations (body, source, status, reviewed_at)
select v.body, v.source, 'live', now()
from (values
  ('Bnwo is not just a kink, it’s reality, present and future 🤍', 'repeat after me'),
  ('Only black men are real men.', 'today’s mantra'),
  ('Whitebois are pathetic.', 'today’s mantra'),
  ('I’m not man enough to fuck, I can’t get hard for pussy.', 'today’s mantra'),
  ('Pussy is meant for bbc, not whitebois.', 'today’s mantra'),
  ('I will be pussyfree all my life.', 'today’s mantra'),
  ('I will watch only interracial porn.', 'today’s mantra'),
  ('I belong in service to my superiors.', 'the commandments'),
  ('Every rep is a payment of my reparations.', 'the commandments'),
  ('I surrender my pleasure to the women I have offended.', 'plapping guide'),
  ('I am the whiteboi who knows his place.', 'the good whiteboy guide'),
  ('My purpose is found in devotion, not in dominance.', 'the good whiteboy guide'),
  ('I will build the BNWO through words, actions, and conviction.', 'commandment XI'),
  ('I will not take false idols or ideologies.', 'commandment III'),
  ('I pay my social, sexual, financial, and masochistic reparations.', 'commandment VII'),
  ('I accept my place and find joy in service.', 'affirmations'),
  ('I’m racist and I submit!', 'the drill gate'),
  ('I submit!', 'the drill gate')
) as v(body, source)
where not exists (
  select 1 from public.affirmations a where lower(btrim(a.body)) = lower(btrim(v.body))
);

-- ------------------------------------------------------- the trust ladder ----

-- Who counts as "the site" for the purpose of scaling requirements: members, not
-- guests (a guest cannot moderate, vote or attest), not tombstones, and only those
-- who logged something in the last 30 days — a dormant account is not a moderator.
create or replace function public.active_accounts()
returns int language sql stable as $$
  select greatest(1, count(distinct l.user_id)::int)
  from public.habit_logs l
  join public.profiles p on p.id = l.user_id
  where not p.is_guest
    and p.merged_into is null
    and l.logged_at > now() - interval '30 days'
$$;

comment on function public.active_accounts() is
  'Distinct non-guest members with a log in the last 30 days, floored at 1 so the ratio is never empty.';

-- One band at a time, so a member's level cannot be computed from two different
-- ladders inside a single statement.
--
--   small  (< 10 active members)  L1  1 day   1 log          L2  2 days  3 active days   5 contributions   L3  7 days   8 active days  15 contributions
--   grown  (>= 10 active members) L1  1 day   3 logs         L2  7 days  7 active days  10 contributions   L3 30 days  30 active days  40 contributions
--
-- `max_upheld_*` is a ceiling, not a target: null means "not checked at this level".
-- Level 3's 90-day ceiling exists because a person who had content taken down in the
-- last quarter is not the voice that verifies other people's cages.
create or replace function public.trust_thresholds()
returns table (
  band text,
  level int,
  min_age_hours int,
  min_active_days int,
  min_contributions int,
  max_upheld_30 int,
  max_upheld_90 int
) language sql stable as $$
  with n as (select public.active_accounts() as c)
  select * from (
    values
      ('small'::text, 1::int,  24::int, 0::int,  1::int, 0::int, null::int),
      ('small'::text, 2::int,  48::int, 3::int,  5::int, 1::int, null::int),
      ('small'::text, 3::int, 168::int, 8::int, 15::int, null::int, 0::int)
  ) s
  where (select c from n) < 10
  union all
  select * from (
    values
      ('grown'::text, 1::int,  24::int, 0::int,  3::int, 0::int, null::int),
      ('grown'::text, 2::int, 168::int, 7::int, 10::int, 1::int, null::int),
      ('grown'::text, 3::int, 720::int, 30::int, 40::int, null::int, 0::int)
  ) g
  where (select c from n) >= 10
$$;

-- Same OUT list as 0016, so `create or replace` is legal here (widening it is the
-- thing that needs a drop — see 0018). The thresholds come from the function above
-- instead of four literals repeated in the CASE.
create or replace function public.trust_level(p_user uuid)
returns table (
  level int,
  active_days int,
  contributions int,
  upheld_flags int,
  next_level_at int
) language sql stable as $$
  with me as (
    select id, is_leader, created_at from public.profiles where id = p_user
  ),
  activity as (
    select
      (select count(distinct log_date) from public.habit_logs
        where user_id = p_user and hidden_at is null)::int as active_days,
      (select count(*) from public.habit_logs where user_id = p_user)::int as logs,
      (select count(*) from public.cage_attestations where attester_id = p_user)::int as attestations
  ),
  upheld_30 as (
    select (
      (select count(*) from public.habit_logs
        where user_id = p_user and hidden_at > now() - interval '30 days')
      + (select count(*) from public.cage_checks
        where user_id = p_user and hidden_at > now() - interval '30 days')
    )::int as n
  ),
  upheld_90 as (
    select (
      (select count(*) from public.habit_logs
        where user_id = p_user and hidden_at > now() - interval '90 days')
      + (select count(*) from public.cage_checks
        where user_id = p_user and hidden_at > now() - interval '90 days')
    )::int as n
  ),
  tt as (
    select
      max(band) as band,
      max(min_age_hours)     filter (where level = 1) as l1_hours,
      max(min_contributions) filter (where level = 1) as l1_contrib,
      max(min_age_hours)     filter (where level = 2) as l2_hours,
      max(min_active_days)   filter (where level = 2) as l2_days,
      max(min_contributions) filter (where level = 2) as l2_contrib,
      max(max_upheld_30)     filter (where level = 2) as l2_flags,
      max(min_age_hours)     filter (where level = 3) as l3_hours,
      max(min_active_days)   filter (where level = 3) as l3_days,
      max(min_contributions) filter (where level = 3) as l3_contrib,
      max(max_upheld_90)     filter (where level = 3) as l3_flags
    from public.trust_thresholds()
  ),
  lv as (
    select
      case
        when coalesce(m.is_leader, false) then 4
        when a.active_days >= tt.l3_days
             and (a.logs + a.attestations) >= tt.l3_contrib
             and u90.n <= coalesce(tt.l3_flags, 2147483647) then 3
        when a.active_days >= tt.l2_days
             and (a.logs + a.attestations) >= tt.l2_contrib
             and u30.n <= coalesce(tt.l2_flags, 2147483647) then 2
        when m.created_at <= now() - make_interval(hours => tt.l1_hours)
             and a.logs >= tt.l1_contrib
             and u30.n = 0 then 1
        else 0
      end::int as level,
      a.active_days,
      (a.logs + a.attestations)::int as contributions,
      u30.n as upheld_30
    from activity a, upheld_30 u30, upheld_90 u90, tt
    left join me m on true
  )
  select
    lv.level,
    coalesce(lv.active_days, 0),
    coalesce(lv.contributions, 0),
    coalesce(lv.upheld_30, 0),
    case lv.level
      when 0 then greatest(0, tt.l1_contrib - coalesce(lv.contributions, 0))
      when 1 then greatest(0, tt.l2_contrib - coalesce(lv.contributions, 0))
      when 2 then greatest(0, tt.l3_contrib - coalesce(lv.contributions, 0))
      else null
    end::int as next_level_at
  from lv, tt;
$$;
