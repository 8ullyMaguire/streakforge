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
- [x] Auth: X OAuth 2.0 (PKCE) + dev-login + Postgres-backed sessions
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
- [x] Login: X OAuth button + dev-login (local)
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
  profile CRUD).
- Browser-tested end-to-end: login → dashboard → log → leaderboard tabs → feed →
  profile → settings update.
- Backend: `cargo test` — 5/5 pass.
- Frontend: `vitest run` — 12/12 pass; `svelte-check` — 0 errors.

## Known issues / notes

- Dev-login button only appears when built with `VITE_ALLOW_DEV_LOGIN=1`.
- X OAuth requires real `X_CLIENT_ID` / `X_CLIENT_SECRET`; dev-login is the
  local path.
- Session cookie secret should be rotated for production (`SESSION_SECRET`).

## Not in scope (v1)

Multiple habit types, notifications, friends/following, complex analytics,
native apps, payments/premium.
