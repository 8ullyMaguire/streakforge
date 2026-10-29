# StreakForge — Session Context (Recovery Cheat Sheet)

> Read this first if you're a fresh agent (or a returning one after a cleared
> session). It summarizes everything, points to the deep docs, and lists the
> immediate next actions. Last updated: **2026-10-14**.

---

## TL;DR

- **What**: StreakForge — a whiteboi denial & accountability tracker with community
  leaderboards, wlw.grok.me-inspired dark theme (counter, marquee, red/gold, mono digits).
- **Stack**: Rust (axum) + SvelteKit (Svelte 5, adapter-static SPA) + PostgreSQL.
- **Where**: local repo `/home/alvaro/code-local/rust/streakforge`; production on
  thinkcentre (192.168.1.13) port 8001 (systemd `streakforge.service`, binary in
  `/opt/streakforge`); public at https://streakforge.polarisocial.xyz; GitHub remote
  `github` = github.com/8ullyMaguire/streakforge.
- **Status**: fully working + verified. **134 backend tests** (22 unit · 11 consent ·
  13 guest · 63 integration · 18 routes · 7 streak edge cases), **135 frontend tests**,
  migrations `0001`–`0018`, deployed and green. Since 2026-10-29 the site also has cage
  checks (peer-verified), trust levels with quorum moderation, time-to-cum metrics,
  public-by-default confessions with an opt-out, a one-click landing flow, a yesterday
  catch-up window, and account export/erasure (`/privacy`).

---

## Quick facts

| Fact | Value |
|------|-------|
| Repo | `/home/alvaro/code-local/rust/streakforge` |
| Backend dev port | 127.0.0.1:8787 (`scripts/dev.sh`) |
| Prod port | 127.0.0.1:8001 on thinkcentre — public https://streakforge.polarisocial.xyz |
| DB dev (this host) | a local cluster (e.g. `streakforge_dev` on 127.0.0.1:5499) |
| DB prod | thinkcentre `streakforge`, role `streakforge` (has CREATEDB). Password lives only in `/opt/streakforge/.env` |
| Migrations | `0001`–`0011`, applied automatically at startup |
| Frontend build | `web/build` (adapter-static), served by Rust — **rebuild after every frontend change** |
| Rate limits | wastes 5/UTC day (`WLW_DAILY_LIMIT`), denials 1/day, reps 5/day; a waste spends the day's denial, a denial never spends a waste (one-way, since 2026-10-29) |
| Login throttle | 5 failures/username or 30/address per 15 min → 429 + `Retry-After` |
| Score | `(days wasted + days denied + drill bonus) × multiplier`; counts DAYS; 24h account-age floor on boards |
| Session | 30 days (`SESSION_TTL_DAYS`); prod cookie is `__Host-streakforge_session` (Secure) |
| Backups | nightly 03:30 → `~/backups/streakforge` on thinkcentre; restore rehearsal: `/opt/streakforge/restore-db.sh` |
| CI | `.github/workflows/ci.yml` — postgres service + cargo test; npm ci + check + test + build |
| Docs | `docs/SPECIFICATION.md`, `docs/ARCHITECTURE.md`, `docs/OPERATIONS.md`, `docs/100_IDEAS.md`, `docs/plans/` |
| Status file | `STATUS.md` (keep updated) |
| Prod secrets | `~/.hermes/profiles/coding/.env` (`STREAKFORGE_SESSION_SECRET`, `STREAKFORGE_DB_PASSWORD`); the live `.env` on the host |

---

## What was built (chronology)

1. **Core tracker** (`7a938ce`) — profiles, habit_logs, streaks (gaps-and-islands),
   daily/weekly/alltime leaderboards, UOTD, feed, X OAuth + dev login, wlw theme,
   landing/dashboard/leaderboard/feed/profile/settings.
2. **Scripts** (`706dce1`) — `scripts/dev.sh`, `scripts/seed.sql`.
3. **Doctrine** (`71b52a2` + renames) — `/doctrine` page + `/api/doctrine*` serving
   markdown docs from `manifestos/` (11 docs); custom markdown renderer (XSS-safe).
4. **Affirmation Drill** (`8e01e64`) — `/drill` page with wlw counter + affirmation
   deck; `kind` column on habit_logs; per-kind rate limits; drill reps excluded from
   community surfaces.
5. **Deploy** (`1e99bce`, `c226861`) — `scripts/deploy.sh`; systemd service on
   thinkcentre port 8001 (mirrors fichub on 8000; nginx inactive there).
6. **Local auth** (`9aa2817`…`d667aea`) — X OAuth + dev-login replaced by local
   username/password (Argon2id) with bot-dissuasion; `social_url`; migration `0004`;
   secrets moved to the Hermes profile `.env`; unknown `/api/*` → 404.
7. **Devotion Index** (`0008`–`0010`) — the score replaced log counts:
   `(WLW + WLD + game bonus) × multiplier`, a 30-day decay rule, a lifetime
   record that does not decay, and the `/kpi` page. (The multiplier's basis is the
   daily-participation streak since `0012`, below.)
8. **Sessions, budget & the ranked 100** (2026-10-14) — the signing key derived from
   `SESSION_SECRET` (restart-logout fixed), `WLW_DAILY_LIMIT=5`, `docs/100_IDEAS.md`
   filed to the public Concord project, all 16 ideas scoring ≥ 2.5 shipped
   (`91fc795`): notes private, day-based scoring, board age floor, decay warning,
   `__Host-` cookie, nightly backups + rehearsal, CI, login throttle, og/twitter cards,
   18+ notice + `/terms`, `/glossary`, `/api/today`, `/log?kind=` deep links, budget +
   reset clock. Plan: `docs/plans/2026-10-14-tier-1-and-tier-2.md`.
9. **The multiplier follows participation, not one kind's run** (2026-10-15, `0012`) — the
   exclusive streak keeps its meaning (and may reset when the kind changes); the multiplier is
   read off `user_participation_streak`: consecutive UTC days with a log of ANY kind, drills
   included. Alternating waste/denial daily now keeps the tier instead of dropping to 1.0.

---

## Where things live (key files)

- Backend API: `backend/src/api.rs` (handlers, DTOs, budgets, the `/api/today` route).
- Auth: `backend/src/auth.rs` (register/login/nonce, Argon2id, bot-dissuasion, the
  login throttle, cookie naming) + `auth_tests.rs`.
- Router/startup: `backend/src/main.rs`. Errors: `error.rs` (status + optional
  `Retry-After`).
- Schema/functions: `backend/migrations/0001`…`0011` (0011 = privacy, day scoring,
  sybil floor, decay countdown, `today_counts()`, `login_attempts`).
- Frontend theme: `web/src/app.css`. API client: `web/src/lib/api.ts` (types in
  `types.ts`).
- Pure logic modules with their own tests: `countdown.ts`, `deeplink.ts`, `agegate.ts`,
  `glossary.ts`, `affirmations.ts`, `markdown.ts`, `validation.ts`, `challenge.ts`.
- Doctrine content: `manifestos/*.md` (11 docs).
- Ops: `scripts/deploy.sh`, `scripts/backup-db.sh`, `scripts/restore-db.sh`,
  `.github/workflows/ci.yml`, prod `.env` on thinkcentre.

---

## Verified state (do not re-verify everything)

- `cargo test` → **20 lib + 48 integration passed**; `vitest run` → **93 passed**;
  `svelte-check` → 0 errors / 0 warnings; `vite build` → green.
- Prod deploy → 15/15 health checks 200 (including `/api/today`, `/robots.txt`,
  `/sitemap.xml`, `/og.png`, `/glossary`, `/terms`, `/log`).
- Live-verified 2026-10-14: `__Host-` cookie + `/api/auth/me`; `/api/feed` returns no
  `note` key while 512 notes remain in Postgres; `/api/today` matched a direct SQL
  count; five bad logins → 429 (`Retry-After: 899`) with the correct password also
  refused and another username still able to sign in; the restore rehearsal matched
  production (`profiles 67`, `habit_logs 512`, `migrations 11`).
- Browser-verified: the 18+ overlay, the note-free landing page, all 17 glossary
  terms, `/terms`, `/log?kind=denial` and the unknown-kind fallback.

---

## Immediate next actions (if continuing)

1. **Tier 3+ of `docs/100_IDEAS.md`** — everything scoring under 2.5 was never
   started. Ranks 17–20 (consistency board, lock-streak board, public ledger,
   verifiable lock seal) are the next by score.
2. **Off-host backups.** The dumps are on the same machine as the database, so a lost
   host is still a total loss (SPEC §27).
3. **Schedule the restore rehearsal** so "we have backups" becomes "the last backup
   restored".
4. **Self-service export/delete** — `/terms` promises both on request, and today that
   is manual.
5. The `racism_penalty` score component has no data source and is always 0: either
   wire it or remove it from the formula.

---

## Conventions to respect

- Conventional Commits, small units.
- **Spec + plan before code** for anything non-trivial; keep `docs/` and `STATUS.md`
  in sync with the code (several doc sections were stale for months — if they
  disagree, the code wins and the doc is a bug).
- A gate has to be able to fail: mutation-check a new test before trusting it, and
  make sure a "green" claim comes from real output.
- Rebuild the frontend before believing a page works (`web/build` is what the Rust
  binary serves).
- Mirror push to `github` after merges.
- Don't touch fichub on port 8000.

---

## Gotchas worth remembering (full list in SPEC §22)

- axum 0.8: `{capture}` routes, not `:capture`.
- tower-sessions 0.14 pairs with sqlx-store 0.15.
- Never edit applied migrations (checksum lock). 0011 is the newest.
- `CREATE OR REPLACE FUNCTION` cannot change a function's OUT list (42P13) — drop
  leaves-first and recreate.
- SQLx decodes function results POSITIONALLY: a changed column list is a runtime 500
  with every test green. `public_function_signatures_are_pinned` guards the five public
  functions.
- usernames are `^[a-zA-Z0-9_]{2,30}$` — no hyphens (your own test fixtures included).
- CSS `//` comments break `:root` in the browser — use `/* */`.
- `user_streak()` returns INT4; cast `::bigint` in Rust queries.
- Auth forms need JS: register/login require the sha256 nonce challenge + 3s–10min
  form timing + an empty honeypot (SPEC §9). Any scripted probe must reproduce all
  three, including for FAILED login attempts.
- `i64::div_ceil` is not available on this toolchain — use `(a + b - 1) / b`.
- `habit_logs.log_date` is a GENERATED column (`generated always as ((logged_at at time zone
  'UTC')::date) stored`). Backdating a log means updating `logged_at`; setting `log_date`
  fails with `column "log_date" can only be updated to DEFAULT`. Anything that mocks a past
  day (a decay probe, a fixture) moves `logged_at` and lets the day follow.
- Behind the tunnel, the client IP comes from `CF-Connecting-IP` / `X-Forwarded-For`;
  the socket address is always loopback.

---

*Deep dives: SPECIFICATION.md (what/why), ARCHITECTURE.md (how), OPERATIONS.md (ops,
including backups), 100_IDEAS.md (what's next and why that order).*
