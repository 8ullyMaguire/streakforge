-- Verify the Whiteboi Devotion Index against hand-computed cases.
-- Each case states what the wlw tier table requires, so a mismatch is a real bug.
--
-- Run against a scratch DB:  psql -f tests/verify_devotion.sql
--
-- GOTCHA worth knowing: habit_logs.log_date is
--   ((logged_at at time zone 'utc')::date)
-- so a backdated timestamp MUST be built as `... at time zone 'utc'`. Built as
-- a bare timestampt literal it pins to the session timezone, and midnight in
-- +02 is 22:00 the PREVIOUS day in UTC -- which silently shifts every backfilled
-- log_date back one day and breaks the streak chain. That cost a debugging round
-- here; keep the `at time zone 'utc'` in any backfill.
\set QUIET on
\pset pager off

delete from public.habit_logs;
delete from public.profiles;

insert into public.profiles (username) values ('devtest');
insert into public.profiles (username) values ('devtest2');
insert into public.profiles (username) values ('devtest3');
create temporary table t  as select id from public.profiles where username = 'devtest';
create temporary table t2 as select id from public.profiles where username = 'devtest2';
create temporary table t3 as select id from public.profiles where username = 'devtest3';

-- CASE 1: no logs -> multiplier 1.0, score 0, active_kind 'none'
select 'CASE 1 (empty user): expect none / 0 / 1.0 / 0.00' as case;
select es.active_kind, es.streak, es.multiplier, di.wlw, di.wld, di.game_bonus, di.score
from public.exclusive_streak((select id from t)) es,
     public.devotion_index_alltime((select id from t)) di;

-- CASE 2: one waste -> streak 1 -> 1.0x -> score = 1
insert into public.habit_logs (user_id, kind)
select (select id from t), 'habit';
select 'CASE 2 (1 WLW, streak 1): expect habit / 1 / 1.0 / wlw=1 / 1.00' as case;
select es.active_kind, es.streak, es.multiplier, di.wlw, di.score
from public.exclusive_streak((select id from t)) es,
     public.devotion_index_alltime((select id from t)) di;

-- CASE 3: 7 consecutive UTC days of waste -> streak 7 -> 1.25x -> 7*1.25 = 8.75
insert into public.habit_logs (user_id, kind, logged_at)
select (select id from t), 'habit',
       (((now() at time zone 'utc')::date - (g || ' days')::interval) at time zone 'utc')
from generate_series(1, 6) g;
select 'CASE 3 (7-day WLW streak): expect 1.25 / wlw=7 / 8.75' as case;
select es.active_kind, es.streak, es.multiplier, di.wlw, di.score
from public.exclusive_streak((select id from t)) es,
     public.devotion_index_alltime((select id from t)) di;

-- CASE 4: a denial today flips the ACTIVE exclusive streak to the denial streak,
-- so the streak reads 1 -- but the MULTIPLIER holds at 1.25, because she has shown
-- up 7 days running and the multiplier follows participation, not one kind's run
-- (0012). Before 0012 this case read 1.0x and called that correct; it was the
-- reported defect, so this case now pins the fix.
insert into public.habit_logs (user_id, kind)
select (select id from t), 'denial';
select 'CASE 4 (denial today): expect denial / 1 / 1.25 / wlw=7 wld=1 / 10.00' as case;
select es.active_kind, es.streak, es.multiplier, di.wlw, di.wld, di.score
from public.exclusive_streak((select id from t)) es,
     public.devotion_index_alltime((select id from t)) di;

-- CASE 5: game bonus = 1 per 3 affirmations, multiplied like everything else
insert into public.habit_logs (user_id, kind)
select (select id from t), 'affirmation' from generate_series(1, 3);
select 'CASE 5 (+3 affirmations): expect game_bonus=1, score 11.25 (9 x 1.25)' as case;
select di.wlw, di.wld, di.game_bonus, di.multiplier, di.score
from public.devotion_index_alltime((select id from t)) di;

-- CASE 5b: drills count as showing up, but hold no exclusive streak. devtest3 has
-- seven days of NOTHING BUT affirmations: active_kind stays 'none', the exclusive
-- streak stays 0, and the multiplier still reaches 1.25. Before 0012 this was 1.0x.
insert into public.habit_logs (user_id, kind, logged_at)
select (select id from t3), 'affirmation',
       (((now() at time zone 'utc')::date - (g || ' days')::interval) at time zone 'utc')
from generate_series(0, 6) g;
select 'CASE 5b (drill-only 7 days): expect none / 0 / 1.25' as case;
select es.active_kind, es.streak, es.multiplier
from public.exclusive_streak((select id from t3)) es;

-- CASE 6: the board ranks by Devotion score, not raw log count. devtest2 gets 20
-- logs -- 5 on each of 4 days, the daily ceiling, so volume with no unbroken run --
-- and scores 4 x 1.0 = 4. devtest has 9 scored days in one unbroken run for 11.25.
-- Fewer than half the logs must outrank it.
-- (Before 0012 this case said "40 logs at 1.0x"; a 40-day run actually earns 2.0x,
-- so the fixture never demonstrated the claim its comment made. 5/day over 4 days does.)
-- The 24-hour sybil floor (0011) hides an account younger than a day, so these
-- fixtures must be backdated or the case prints 0 rows and checks nothing -- which
-- is what it did before, on top of the fixture being wrong.
update public.profiles set created_at = now() - interval '2 days'
where username like 'devtest%';
insert into public.habit_logs (user_id, kind, logged_at)
select (select id from t2), 'habit',
       ((((now() at time zone 'utc')::date - (g || ' days')::interval) at time zone 'utc')
         + (r || ' hours')::interval)
from generate_series(0, 3) g, generate_series(0, 4) r;
select 'CASE 6 (board): devtest (9 days, one run, 11.25) must outrank devtest2 (20 logs, 4 days, 4.00)' as case;
select username, score, wlw, multiplier, streak, active_kind
from public.monthly_leaderboard_weighted(10)
where username like 'devtest%'
order by score desc;

-- CASE 7: users with no activity in the window are absent, not zero rows
select 'CASE 7: zero-score users excluded' as case;
select count(*) as zero_score_rows
from public.devotion_leaderboard(interval '1 day', 50)
where score = 0;

-- CASE 8: the multiplier tier table, exhaustively at the boundaries
select 'CASE 8 (tiers): 1-6=1.0 7-13=1.25 14-29=1.5 30-59=2.0 60-99=2.5 100+=3.0' as case;
select s, public.devotion_multiplier(s) as mult
from (values (0),(1),(6),(7),(13),(14),(29),(30),(59),(60),(99),(100),(1000)) v(s);

-- cleanup
delete from public.habit_logs;
delete from public.profiles;
