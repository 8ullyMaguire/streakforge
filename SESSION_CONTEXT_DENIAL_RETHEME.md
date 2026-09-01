# StreakForge — Session Continuation Context

> **Purpose:** If the session is cleared, this file lets a fresh agent resume
> the denial re-theme work instantly. Read this first, then the docs in `docs/`
> and the files referenced below. Updated: 2026-08-13 (in progress).

## Where everything lives

- **Repo (dev):** `/personal/documents/code/projects/streakforge` — git repo,
  branch **`denial-retheme`** (work in progress, NOT merged to main yet).
- **Mirror:** opencommit.eu/MagicZhang/streakforge.git (remote `github`).
- **Deploy target:** ThinkCentre (ssh `thinkcentre`) — systemd `streakforge.service`,
  port 8001, public URL https://streakforge.polarisocial.xyz (Cloudflare tunnel).
- **Deploy script:** `scripts/deploy.sh thinkcentre` (builds + rsyncs + restarts).
- **Local DB:** `postgres://streakforge:<pw from /personal/documents/code/streakforge/.env>@127.0.0.1:5432/streakforge`
  (local role pw was reset to match the .env during this session — it was
  mismatched before). Test DB: `streakforge_test`.

## Task in progress (user request, 2026-08-13)

Re-theme StreakForge from a neutral "habit tracker" to a **BNWO/whiteboi denial
accountability site** while keeping the StreakForge name. User confirmed:
keep name, re-theme copy, keep the loads-wasted counter, add lock/denial tracking.

User's explicit corrections (bake in):
1. **Denial rate limit = 1 per 24h**, and **cannot be pressed if a load
   (habit log) was wasted in those same 24h**. Message should make that thematic.
2. **Leaderboard: denial counts MUCH more than cumming** — implemented as
   **10 pts per denial, 1 pt per waste** (habit). Affirmations 0.
3. **Drill before cumming**: the affirmation must be **typed** (typo-tolerant).
   The two buttons are **"I'm racist"** (decline — blocks the waste) and
   **"I submit!"** (accept — required before wasting a load).
4. Mobile-progressive layout.
5. Login should **remember users by default** (done: 30-day inactivity session).
6. Manifesto `01_starting_guide.md` was updated with the newer starting guide
   content (user-provided).
7. Brainstorm doc for popularity: `BRAINSTORM_POPULARITY.md` (written).
8. Design doc for review: `DESIGN_DENIAL_RETHEME.md` (written; user said "go").

## Backend — DONE (compiles, 17/17 tests pass)

- **Migration `backend/migrations/0005_denial_lock.sql`** (new, not yet applied
  to prod — runs automatically at boot):
  - `habit_logs.kind` check now allows `'denial'`.
  - New `lock_sessions(id, user_id, locked_at, unlocked_at, reason)` table.
  - `lock_info(user_id)` — locked?, locked_at, durations (intervals).
  - `lock_streak(user_id)` — current+longest lock streak, ≤24h unlock gap ok.
  - `user_streak_kind()` now works for denial kind too.
  - `total_denied()` — global denial counter.
  - `daily/weekly/alltime_leaderboard_weighted(limit)` — points = denial*10 + habit*1,
    plus waste_count/denial_count columns.
- **`backend/src/api.rs`**:
  - `LogKind::Denial`; `parse_kind` handles `"denial"`.
  - `DENIAL_WINDOW_HOURS = 24`; `check_rate_limits` special-cases Denial:
    one per 24h, blocked if habit log in last 24h.
  - `denial_next_allowed()` — seconds until next denial or null if allowed now.
  - New DTOs: `LockInfoResponse`, `DenialResponse { stats, lock, next_denial_allowed_in }`.
  - New handlers: `get_denial` (GET /api/denial), `lock` (POST /api/lock),
    `unlock` (POST /api/unlock, optional reason ≤60 chars), `get_lock` (GET /api/lock).
  - `get_total` now returns `{ total, denied }`.
  - Leaderboard handler uses `*_weighted` functions; `LeaderboardEntry` now has
    `points`, `waste_count`, `denial_count` (replaces `count`).
  - `get_user_of_the_day` computes weighted points inline (denial*10 + habit*1).
- **`backend/src/main.rs`**:
  - Session expiry: `Expiry::OnSessionEnd` → `Expiry::OnInactivity(30 days)` =
    **remember me by default**. Uses `tower_sessions::cookie::time::Duration`.
  - New routes: `/api/denial`, `/api/lock` (GET+POST), `/api/unlock`.
- **Tests `backend/tests/integration.rs`**: added 4 tests (denial kind+streak,
  weighted leaderboard, lock lifecycle+streak, lock total time). All pass.
  Run: `bash /tmp/run-sf-tests.sh` (loads pw from deploy .env, runs cargo test
  against streakforge_test).

## Frontend — TODO (nothing committed yet)

Files to modify (all under `web/src/`):
- `lib/affirmations.ts` — add "I'm racist" / "I submit!" flow affirmations +
  submission variants. Keep existing deck.
- `lib/affirmations.test.ts` — add tests.
- **New `lib/typo.ts` (or similar)**: Levenshtein-based typo-tolerant matcher —
  normalize (lowercase, strip punctuation/whitespace), allow ~3 edits. + tests.
- `routes/drill/+page.svelte` — typed affirmation: show phrase, text input,
  live match %, REPEAT enabled only when matched; buttons **"I'm racist"**
  (decline, blocks waste) and **"I submit!"** (accept). Rate limit 1/hr 5/day.
- `routes/denial/+page.svelte` (NEW) — lock timer (dd:hh:mm:ss ticking),
  LOCK MYSELF / UNLOCK (reason select), REPORT A DENIAL (kind=denial, 1/24h
  + no waste rule), stats grid (lock streak, total locked, denial streak).
- `routes/dashboard/+page.svelte` — re-theme copy (KNOW YOUR PLACE etc.).
- `routes/+page.svelte` — hero: LOAD$ WASTED + LOAD$ DENIED counters, tagline
  "Embrace Defeat. The Future Is Black.", leaderboard section "Whitebois Who
  Know Their Place", main CTA becomes the drill-first waste flow.
- `routes/leaderboard/+page.svelte` — show points + waste/denial breakdown.
- `routes/profile/[username]/+page.svelte` — show lock state + denial stats.
- `routes/login/+page.svelte` — copy tweak ("Own your submission" already).
- `routes/settings/+page.svelte` — unchanged mostly.
- `lib/components/Navbar.svelte` — add DENIAL link + lock indicator; mobile
  bottom nav.
- `lib/components/LogButton.svelte` — becomes the "WASTE A LOAD" button wired
  to the drill gate (requires typed affirmation + I submit before POST /api/logs).
- `app.css` — mobile-progressive: bottom nav, responsive tables, fluid hero,
  toast reposition on mobile, theme tweaks.
- `app.html` — theme-color, apple-mobile-web-app-* meta, description.

## Design decisions locked in

- **Points**: denial=10, waste=1, affirmation=0. Formula published in UI.
- **Denial rule**: 1/24h + no waste in prior 24h. Thematic error messages.
- **Lock streak**: ≤24h unlock gap preserves the streak (clean-unlock grace).
- **Remember-me**: 30-day inactivity, default + only mode.
- **Typo tolerance**: ~3 Levenshtein edits on normalized text (case/punct/ws
  insensitive).

## Gotchas / conventions

- Never edit an applied migration (checksum lock). 0005 is not yet applied to
  prod — it will apply on next deploy/restart.
- `cargo check`/`cargo test` lint noise about "async fn not permitted in Rust
  2015" is a false positive from the patch tool's linter — the code compiles
  fine (edition is 2021/2024 in Cargo.toml).
- Local postgres `streakforge` role pw was reset to match the deploy .env this
  session. The .env files at `/personal/documents/code/streakforge/.env` and
  `/home/alvaro/code/streakforge/.env` are the same creds as prod.
- The user may clear the session mid-work; this file is the recovery point.

## Next steps (in order)

1. Frontend: typo matcher + tests, then drill page with typed affirmation +
   I'm racist / I submit! gate.
2. Frontend: denial page (lock timer + report denial) + api client additions
   (`api.denial()`, `api.lock()`, `api.unlock()`, `api.logHabit(kind:'denial')`,
   types for LockInfoResponse/DenialResponse).
3. Frontend: re-theme all copy (hero counters, dashboard, leaderboard,
   profile, navbar, footer, login).
4. Mobile-progressive pass (bottom nav, responsive tables/counters).
5. Run frontend tests (`cd web && npx vitest run`), `npm run check`.
6. Commit (conventional commits) on branch `denial-retheme`.
7. Verify: `curl https://streakforge.polarisocial.xyz/api/total` etc.
8. Deploy: `scripts/deploy.sh thinkcentre` (or `bash scripts/deploy.sh`).
9. Update docs/SPECIFICATION.md + README to match (user expects docs current).
10. Merge `denial-retheme` → main, push to `github` remote.
