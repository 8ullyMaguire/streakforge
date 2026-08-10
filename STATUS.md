# StreakForge — Development Status

Last updated: 2026-08-10

## What's built (all verified working)

### Backend (Rust / axum)
- [x] PostgreSQL schema: `profiles`, `habit_logs` (with generated `log_date`), indexes
- [x] `user_streak()` — current + longest streak (gaps-and-islands)
- [x] `daily_leaderboard()` / `weekly_leaderboard()` / `alltime_leaderboard()`
- [x] `user_of_the_day()` — count + earliest-log tiebreak
- [x] `activity_feed()` — cursor pagination
- [x] `profile_stats()` — today / week / alltime / streaks
- [x] Auth: local username/password (Argon2id) + session cookies; bot-dissuasion
      (honeypot, form timing, JS proof-of-work challenge); NO X OAuth, NO SMTP,
      NO dev-login in production
- [x] One social URL per profile (editable in settings, shown on public profile)
- [x] Rate limits: 1/hr + 5/day per kind (habit vs affirmation), server-enforced
- [x] Affirmation drill: `/api/drill` stats + `kind` on `/api/logs`; drill reps
      kept out of leaderboards/feed/total (separate community surface)
- [x] All endpoints: /auth/*, /logs, /stats, /drill, /leaderboard/*, /user-of-the-day,
      /total, /feed, /profile/*, /profile (PATCH)
- [x] SPA static serving + client-route fallback (adapter-static build)
- [x] Manifesto: `/api/manifesto` + `/api/manifesto/{id}` — serves markdown docs
      from a configurable directory (path-traversal sanitized)
- [x] Integration tests (6 passing) for streak calc, leaderboards, UOTD, feed,
      affirmation-kind separation

### Frontend (SvelteKit / Svelte 5)
- [x] wlw-style dark theme (near-black, red/gold, Inter + Roboto Mono)
- [x] Landing: hero, big green/black tabular counter, marquee, UOTD card,
      top-25 daily board, recent activity
- [x] Login: username/password form (SIGN IN / REGISTER tabs), honeypot +
      timing + JS challenge bot-dissuasion, no external accounts
- [x] Dashboard: stat cards, Log button (rate-limit aware), 30-day heatmap,
      recent logs
- [x] Leaderboard: Daily / Weekly / All-time tabs + UOTD
- [x] Feed: reverse-chronological, LOAD MORE pagination, avatars
- [x] Profile: public stats, DiceBear avatar fallback
- [x] Settings: edit username / display name / avatar URL
- [x] Manifesto: sidebar doc list + rendered markdown viewer (dark theme)
- [x] Drill: wlw-style big counter, affirmation deck, REPEAT (rate-limit aware),
      streak cards + heatmap
- [x] Toasts, skeletons, empty states, 401 redirect to /login
- [x] Frontend tests (24 passing): api utils, validation, markdown renderer,
      affirmations deck
- [x] `npm run check` clean (0 errors)

## Verification evidence

- All API endpoints exercised via curl (auth, logging, rate limits, boards, feed,
  profile CRUD, drill separate budgets, manifesto + traversal 400).
- Browser-tested end-to-end: login → dashboard → log → leaderboard tabs → feed →
  profile → settings update → drill (counter/card/rate-limit hint) → manifesto.
- Backend: `cargo test --test integration` — 13/13 pass; `cargo test --lib` — 8/8 pass.
- Frontend: `vitest run` — 33/33 pass; `svelte-check` — 0 errors.
- Deployed: thinkcentre port 8001, systemd active, all routes 200, auth enforced,
  dev-login removed (routes 404); pushed private to opencommit.eu/MagicZhang/streakforge.

## Documentation

- `docs/SPECIFICATION.md` (1058 lines), `docs/ARCHITECTURE.md`, `docs/OPERATIONS.md`,
  `docs/SESSION_CONTEXT.md`, expanded `README.md`.
- Keep all of these current after feature work.

## Known issues / notes

- No email/SMTP by design — accounts are username/password only.
- Legacy 'x'/'dev' provider rows (pre-migration) have no password hash and
  cannot log in; they are harmless orphans.
- Session cookie secret must be rotated per environment (`SESSION_SECRET`).

## Not in scope (v1)

Multiple habit types, notifications, friends/following, complex analytics,
native apps, payments/premium.
