# StreakForge — Development Status

Last updated: 2026-08-13

## What's built (all verified working)

### Backend (Rust / axum)
- [x] PostgreSQL schema: `profiles`, `habit_logs` (with generated `log_date`), indexes
- [x] `user_streak()` — current + longest streak (gaps-and-islands)
- [x] `user_streak_kind()` — kind-aware streaks (habit / affirmation / denial)
- [x] `daily_leaderboard()` / `weekly_leaderboard()` / `alltime_leaderboard()`
- [x] **Weighted leaderboards** (`*_leaderboard_weighted`): denial = 10 pts,
      waste = 1 pt, 3 affirmations = 1 pt; per-row waste/denial/affirmation counts
- [x] `user_of_the_day()` — count + earliest-log tiebreak (same weighting)
- [x] `activity_feed()` — cursor pagination
- [x] `profile_stats()` — today / week / alltime / streaks
- [x] Auth: local username/password (Argon2id) + session cookies; bot-dissuasion
      (honeypot, form timing, JS proof-of-work challenge); NO X OAuth, NO SMTP,
      NO dev-login in production
- [x] **Remember me by default**: sessions persist 30 days of inactivity
      (was: ended on browser close)
- [x] One social URL per profile (editable in settings, shown on public profile)
- [x] Rate limits: 1/hr + 5/day per kind (habit vs affirmation), server-enforced
- [x] **Denial kind + chastity lock**: `denial` log kind (1 per 24h, blocked if
      a waste was logged in the same 24h); `lock_sessions` table + `lock_info` /
      `lock_streak` / `total_denied` functions; `/api/denial`, `/api/lock`,
      `/api/unlock`; `/api/total` returns `{total, denied}`
- [x] Affirmation drill: `/api/drill` stats + `kind` on `/api/logs`
- [x] All endpoints: /auth/*, /logs, /stats, /drill, /denial, /lock, /unlock,
      /leaderboard/*, /user-of-the-day, /total, /feed, /profile/*, /profile (PATCH)
- [x] SPA static serving + client-route fallback (adapter-static build)
- [x] Doctrine: `/api/doctrine` + `/api/doctrine/{id}` — serves markdown docs
      from a configurable directory (path-traversal sanitized)
- [x] Integration tests (17 passing) for streak calc, leaderboards, UOTD, feed,
      affirmation-kind separation, denial/lock lifecycle, weighted leaderboards,
      auth (register/login/social_url)

### Frontend (SvelteKit / Svelte 5)
- [x] wlw-style dark theme (near-black, red/gold, Inter + Roboto Mono)
- [x] Landing: hero, big green/black tabular counters (**LOAD$ WASTED** +
      **LOAD$ DENIED**), marquee, UOTD card, top-25 daily board, recent activity
- [x] Login: username/password form (SIGN IN / REGISTER tabs), honeypot +
      timing + JS challenge bot-dissuasion, no external accounts
- [x] Dashboard ("KNOW YOUR PLACE"): stat cards, **WASTE A LOAD** button
      (rate-limit aware), 30-day heatmap, recent logs
- [x] **Denial page** (`/denial`): live lock timer, LOCK / UNLOCK (with reason),
      REPORT A DENIAL, lock-streak + denial stats
- [x] **Typed affirmation drill** (`/drill`): user must TYPE the affirmation
      (typo-tolerant Levenshtein ≤3 edits) before REPEAT enables; gate
      affirmations "I'm racist" / "I submit!"
- [x] Leaderboard ("WHITEBOIS WHO KNOW THEIR PLACE"): weighted points,
      waste/denial/affirmation breakdown per row
- [x] Feed: reverse-chronological, LOAD MORE pagination, avatars
- [x] Profile: public stats, DiceBear avatar fallback
- [x] **Mobile-progressive**: bottom nav (Place/Deny/Drill/Board/Text), responsive
      counters, card-list tables, 2-col stat grid
- [x] **Installable app**: PWA manifest + spade icon, `/app` install guide,
      auth-aware landing CTA (logged in → WASTE A LOAD), 9-digit 3-color counters
- [x] **Leaderboard windows**: DAILY = rolling 24h, WEEKLY = rolling 7 days
      (was UTC calendar day/week — the board showed empty at US-evening peak hours)
- [x] Settings: edit username / display name / avatar URL
- [x] Manifesto: sidebar doc list + rendered markdown viewer (dark theme)
- [x] Drill: wlw-style big counter, affirmation deck, REPEAT (rate-limit aware),
      streak cards + heatmap
- [x] Toasts, skeletons, empty states, 401 redirect to /login
- [x] Frontend tests (33 passing): api utils, validation, challenge PoW, markdown
      renderer, affirmations deck
- [x] `npm run check` clean (0 errors)

## Verification evidence

- All API endpoints exercised via curl (auth, logging, rate limits, boards, feed,
  profile CRUD, drill separate budgets, doctrine + traversal 400).
- Browser-tested end-to-end: login → dashboard → log → leaderboard tabs → feed →
  profile → settings update → drill (counter/card/rate-limit hint) → doctrine.
- Backend: `cargo test --test integration` — 17/17 pass.
- Frontend: `vitest run` — 47/47 pass; `svelte-check` — 0 errors.
- Deployed: thinkcentre port 8001, systemd active, all routes 200, auth enforced,
  dev-login removed (routes 404); public at https://streakforge.polarisocial.xyz;
  pushed private to opencommit.eu/MagicZhang/streakforge.

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
