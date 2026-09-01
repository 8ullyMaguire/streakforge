-- Seed script — creates demo users + historical logs for a lively leaderboard.
-- Run against the dev DB: psql -h 127.0.0.1 -U streakforge -d streakforge -f scripts/seed.sql

INSERT INTO profiles (username, display_name, provider, provider_id) VALUES
  ('iron_will', 'Iron Will', 'dev', 'dev-iron_will'),
  ('daily_dave', 'Daily Dave', 'dev', 'dev-daily_dave'),
  ('streak_queen', 'Streak Queen', 'dev', 'dev-streak_queen'),
  ('noob_forger', 'Noob Forger', 'dev', 'dev-noob_forger'),
  ('late_night', 'Late Night', 'dev', 'dev-late_night')
ON CONFLICT (username) DO NOTHING;

-- iron_will: 10-day streak ending today + 2 extra today (UOTD candidate)
INSERT INTO habit_logs (user_id, logged_at)
SELECT p.id, now() - (d || ' days')::interval + (random()*6)::int * interval '1 hour'
FROM profiles p, generate_series(0, 9) d WHERE p.username = 'iron_will'
ON CONFLICT DO NOTHING;

-- daily_dave: 6-day streak ending today
INSERT INTO habit_logs (user_id, logged_at)
SELECT p.id, now() - (d || ' days')::interval FROM profiles p, generate_series(0, 5) d WHERE p.username = 'daily_dave'
ON CONFLICT DO NOTHING;

-- streak_queen: long past streak, broken 3 days ago
INSERT INTO habit_logs (user_id, logged_at)
SELECT p.id, now() - (d || ' days')::interval FROM profiles p, generate_series(3, 14) d WHERE p.username = 'streak_queen'
ON CONFLICT DO NOTHING;

-- noob_forger: one log 2 days ago
INSERT INTO habit_logs (user_id, logged_at) SELECT p.id, now() - interval '2 days' FROM profiles p WHERE p.username = 'noob_forger'
ON CONFLICT DO NOTHING;

-- late_night: 3 logs today
INSERT INTO habit_logs (user_id, logged_at) SELECT p.id, now() - (h || ' hours')::interval FROM profiles p, generate_series(1, 3) h WHERE p.username = 'late_night'
ON CONFLICT DO NOTHING;
