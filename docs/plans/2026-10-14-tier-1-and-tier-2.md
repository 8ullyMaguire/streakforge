# Plan — tier 1 and tier 2 of the ranked 100

Written 2026-10-14, one day after `docs/100_IDEAS.md` was filed (tree at `1ff8634`).
Scope: **16 items** — every idea scoring ≥ 2.5 (impact ÷ effort). Tier 1 is ranks 01–04,
tier 2 is ranks 05–16. Everything below is decided: where the original idea left a choice
open, the decision is stated with the reason, so the plan can be implemented without going
back to the owner.

Owner's standing rule applies: a plan must be complete enough to implement from alone. The
code that lands is in `backend/` and `web/`; this file is the reasoning and the acceptance
test for each item.

## Where the items came from

`docs/100_IDEAS.md` ranks 100 ideas by value-for-work. Filed into the public Concord project
`concord.polarisocial.xyz/projects/streakforge` (project id 6 on thinkcentre) as features
1074–1173, each with a linked validated complaint (118–217).

Tier 1: 01 notes leak (9.0) · 02 secure cookie (6.0) · 03 backups (5.0) · 04 robots+sitemap (5.0)
Tier 2: 05 CI (3.5) · 06 login throttle (3.5) · 07 OG meta (3.0) · 08 18+ interstitial (3.0) ·
09 pin feed columns (3.0) · 10 today counter (3.0) · 11 score by day (2.67) · 12 glossary (2.5) ·
13 anti-sybil board age (2.5) · 14 deep-link logging (2.5) · 15 budget+clock (2.5) ·
16 decay warning (2.5)

## Global decisions

- **One migration, `0011_privacy_scoring_and_ops.sql`.** The SQL changes are interdependent
  (the feed no longer returns notes; the boards gain an age floor; scoring counts days) and
  shipping them separately would leave the site in states nobody wants. Migration 0009's own
  header records the Postgres trap: `CREATE OR REPLACE` cannot change a function's OUT
  parameters (42P13), so every function whose return type changes is dropped and recreated in
  dependency order, leaves first.
- **Notes become private, globally.** No per-user `public_feed` toggle. The confession box is
  the only place on the site that reads as private, nothing in the UI says it is public, and a
  toggle would ship the leak switched on for every existing user. A user's own notes stay in
  the authenticated `/api/stats` payload, which is what the dashboard renders.
- **Scoring counts DAYS, not rows** (item 11). This is the fix for a defect the daily budget
  change introduced: `devotion_index` counted `count(*)`, so going from 1 waste/day to 5 made
  a day of waste worth 5×. Repetition must not buy rank. Row counts stay visible as
  `lifetime_total`, which is a record rather than a score.
- **No new HTTP-surface for ops items** (03, 05). Backups and CI are scripts and workflows,
  not endpoints.
- **The 24-hour board age floor is 24 hours, not a verified day** (item 13). A determined
  farmer can wait a day; the point is that casual farming stops being free.

## Item-by-item

### 01 — notes stop being published to the anonymous feed  (9.0, S)
Files: `backend/migrations/0011_*.sql`, `backend/src/api.rs`, `web/src/lib/types.ts`,
`web/src/routes/feed/+page.svelte`.
Change: `activity_feed()` loses its `note` OUT column; `get_feed` selects the named columns and
maps to a new `PublicFeedItem` (no note field) instead of the shared `FeedItem`. `FeedItem`
keeps `note` for the authenticated recent-logs list in `compute_stats`.
Acceptance: `curl -s https://streakforge.polarisocial.xyz/api/feed | python3 -c "import
sys,json;rows=json.load(sys.stdin)['items'];print(len(rows), any('note' in r for r in rows))"`
→ prints `50 False`. And `/api/stats` (signed in) still returns notes for the caller's own logs.

### 02 — secure cookie, and the `__Host-` prefix  (6.0, S)
Files: `backend/src/auth.rs` (`session_cookie_name`), `backend/src/main.rs`, `docs/OPERATIONS.md`.
Change: the cookie is named `__Host-streakforge_session` when `SECURE_COOKIES=1` and
`streakforge_session` otherwise, so local HTTP development still works while production gets
the prefix (which browsers only accept with `Secure`, `Path=/`, no `Domain`). `SECURE_COOKIES=1`
in `/opt/streakforge/.env`.
Decision: do it despite the one-time sign-out. A cookie that may travel in plaintext on an
HTTPS-only site is a defect, and the sign-out is a single event whose cost is one login.
Acceptance: `curl -si -X POST .../api/auth/register | grep -i set-cookie` → `__Host-streakforge_session=…; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000; Secure`. Then `/api/auth/me` with that cookie → 200.

### 03 — nightly database backup with a rehearsed restore  (5.0, S)
Files: `scripts/backup-db.sh`, `scripts/restore-db.sh`, `docs/OPERATIONS.md`.
Change: `pg_dump -Fc` from `DATABASE_URL` into `~/backups/streakforge/streakforge-YYYY-MM-DD.dump`,
keeping 14 dailies (and the 1st of each month for 6 months); installed as a cron entry at 03:30
on thinkcentre. `restore-db.sh` restores the newest dump into a scratch database
`streakforge_restore_check` and prints row counts for `profiles`, `habit_logs`, `lock_sessions`.
Decision: dumps land under the user's home rather than `/var/backups` (no sudo in the backup
path, and 69 GB free on `/`); the script never echoes `DATABASE_URL`.
Acceptance: run both scripts for real; the restore must print counts matching production
(`profiles 67`, `habit_logs 512` at the time of writing) — a backup nobody has restored is
not a backup.

### 04 — `robots.txt` and a `sitemap.xml`  (5.0, S)
Files: `web/static/robots.txt`, `web/static/sitemap.xml`.
Change: `robots.txt` allows everything and names the sitemap; the sitemap lists the public
routes plus `/doctrine/1..10`. Both are served as themselves by `ServeDir`, which runs before
the SPA fallback in `main.rs`.
Acceptance: `curl -s https://streakforge.polarisocial.xyz/robots.txt` returns the file (not the
SPA shell), and the sitemap URL count equals 8 routes + 10 doctrine pages.

### 05 — CI on every push  (3.5, S)
Files: `.github/workflows/ci.yml`.
Change: two jobs on push and pull_request. `backend`: postgres:16 service, `cargo fmt --check`,
`cargo clippy -- -D warnings` (warn-only if the tree is not clean at the time of writing, stated
in a comment), `cargo test` with `DATABASE_URL` pointing at the service. `frontend`: `npm ci`,
`svelte-kit sync`, `svelte-check`, `vitest run`, `vite build`. `#[sqlx::test]` creates its own
scratch database per test, so the service role needs CREATEDB — the CI user is
`postgres`, which has it.
Acceptance: push, then `gh run list --workflow=ci.yml --limit 1` shows a completed success.
A workflow file that has never run is a claim, not a gate.

### 06 — throttle failed logins  (3.5, S)
Files: `backend/migrations/0011_*.sql` (`login_attempts`), `backend/src/auth.rs`, `docs/SPECIFICATION.md` §14.
Change: `login_attempts(id, username, ip, attempted_at)` with an index on
`(username, attempted_at)`. On a failed login, insert and count the last 15 minutes: 5 for the
username or 30 for the IP → `429` with a `Retry-After` header and the seconds remaining. A
successful login deletes that username's rows. The per-IP ceiling is deliberately 6× the
per-username one so a shared NAT does not lock a household out while an attacker's guesses at
one account still stop.
Acceptance: six bad logins for one username → the sixth is 429 and the correct password for
that account is refused while the throttle stands; a different username from the same IP is
unaffected (that is the whole reason for two counters).

### 07 — Open Graph and Twitter meta  (3.0, S)
Files: `web/src/app.html`, `web/static/og.png` (generated), `web/src/routes/+layout.svelte`.
Change: static `og:title`, `og:description`, `og:type`, `og:url`, `og:image` (1200×630 PNG
rasterised from `favicon.svg`/`icon.svg` with `rsvg-convert`), `twitter:card`, and
`theme-color`. The layout keeps `document.title` per route; og:title stays static because an
SPA cannot serve per-route meta to a crawler that does not execute JavaScript, and pretending
otherwise would be worse than one honest static card.
Acceptance: `curl -s https://streakforge.polarisocial.xyz/ | grep -c 'og:'` ≥ 4, and
`curl -sI .../og.png | head -1` → 200 with `content-type: image/png`.

### 08 — 18+ acknowledgement and a terms page  (3.0, S)
Files: `web/src/routes/+layout.svelte`, `web/src/lib/components/AgeGate.svelte`,
`web/src/routes/terms/+page.svelte`, `web/src/lib/components/AgeGate.test.ts`.
Change: a first-visit overlay stating the site is for adults, with "I am 18 or older" (stores
`sf_age_ok` in `localStorage` and a matching cookie so it is not asked per tab) and a link to
`/terms`. `/terms` states the age requirement, that the site is a personal tracking tool, that
notes are **not** public, and that data may be exported or deleted on request.
Decision: an acknowledgement, not a wall — nothing server-side depends on it, and saying so in
the terms is more honest than an interstitial that a URL bar defeats.
Acceptance: vitest — the overlay renders for a fresh visitor, disappears on accept, and does
not render on a second visit; `/terms` serves 200.

### 09 — pin the SQL functions' columns with a test  (3.0, S)
Files: `backend/tests/integration.rs`.
Change: a test that reads `pg_get_function_result()` for `activity_feed()`, `devotion_index(uuid,
timestamptz)`, `devotion_leaderboard(interval,int)` and `kpi_totals()` and asserts the row
signature (names and types) equals a literal list. This is the guard for the failure that has
bitten the boards twice: a function's OUT parameters change, SQLx decodes positionally, and a
`numeric` into `f64` or an `INT4` into `i64` 500s a public endpoint.
Acceptance: `DATABASE_URL=… cargo test --test integration -- columns_pinned` passes, and
adding a column to any of those functions in a scratch copy makes it fail (verified by hand —
a pinning test that cannot fail is decoration).

### 10 — "who denied today" public counter  (3.0, S)
Files: `backend/migrations/0011_*.sql` (`today_counts()`), `backend/src/api.rs`
(`get_today`), `backend/src/main.rs` (route), `web/src/routes/+page.svelte`,
`web/src/lib/types.ts`.
Change: `GET /api/today` → `{denials_today, wastes_today, loggers_today}` from a `stable`
function over UTC today, no auth (it is already derivable from `/api/kpi`). The home page shows
one line: "17 denied today · 4 wasted · 21 whitebois logged in".
Acceptance: the endpoint's numbers equal `select count(*) from habit_logs where kind='denial'
and log_date = (now() at time zone 'utc')::date` run by hand.

### 11 — score days, not rows  (2.67, M)
Files: `backend/migrations/0011_*.sql`, `docs/SPECIFICATION.md` §7.2,
`backend/tests/integration.rs`.
Change: `devotion_index` computes `wlw`/`wld` as `count(distinct log_date) filter (where kind
= …)` — a day of wasting counts once however many times it was logged. `game_bonus` keeps its
row-based `floor(count/3)` (affirmations are a drill, not a claim about a day).
`devotion_lifetime` stays row-based and is documented as a record.
Acceptance: five logs for one user on one UTC day → `wlw = 1`, `score = 1.0 × multiplier`;
five logs on five days → `wlw = 5`. Both asserted in the integration suite.

### 12 — a glossary  (2.5, S)
Files: `web/src/routes/glossary/+page.svelte`, links from `doctrine/+page.svelte` and the
dashboard's empty state.
Change: one page defining WLW, WLD, devotion index, multiplier, decay, exclusivity, denial,
waste, drill, affirmation, lock, doctrine, streak — written from the product's own rules, each
entry naming where it is enforced.
Acceptance: the route builds and serves 200; the vitest test asserts the page contains every
defined term and that every internal link in it resolves to a route that exists.

### 13 — 24-hour account age before a board appearance  (2.5, S)
Files: `backend/migrations/0011_*.sql`, `docs/SPECIFICATION.md` §7.2.
Change: `devotion_leaderboard` adds `and p.created_at <= now() - interval '24 hours'`.
Acceptance: a profile created now with a log today does not appear in
`daily_leaderboard_weighted(50)`; the same profile with `created_at` moved back 25 hours does.

### 14 — logging from a deep link  (2.5, S)
Files: `web/src/routes/log/+page.svelte`, `web/src/lib/validation.test.ts` (kind parsing).
Change: `/log?kind=habit|denial|affirmation` renders what will be recorded and one button that
POSTs. Signed-out visitors get a link to `/login?next=/log?kind=…`. `?note=` is accepted but
truncated to the 140-character limit client-side; unknown kinds fall back to `habit` with a
visible warning rather than silently logging the wrong thing.
Decision: a query parameter rather than `/log/[kind]` — a static adapter has to enumerate
dynamic routes at build time, and a param page would either need prerender entries for three
values or a 404 for the fourth kind the schema allows later.
Acceptance: vitest covers kind parsing and the truncation; a live check loads
`/log?kind=habit` and confirms the button POSTs and the dashboard's count rises by one.

### 15 — remaining budget and the reset clock together  (2.5, S)
Files: `web/src/routes/dashboard/+page.svelte`, `web/src/lib/components/LogButton.svelte`,
`web/src/lib/countdown.ts` (reuse).
Change: the dashboard shows "3 of 5 left · resets in 4h 12m" whenever a log is still allowed,
and the existing countdown when it is not. Sourced from `daily_limit`,
`remaining_today`, `next_allowed_in_secs` — never recomputed in the UI, since the server's
budget is the enforced one.
Acceptance: vitest with fake timers — the label reads "5 of 5 left" at the start of a UTC day
and the countdown decreases by a minute after 60 s.

### 16 — warn before the 30-day decay erases a score  (2.5, S)
Files: `backend/migrations/0011_*.sql` (`devotion_index` returns `days_until_decay`),
`backend/src/api.rs` (`DevotionIndexResponse`), `web/src/routes/dashboard/+page.svelte`,
`web/src/lib/types.ts`.
Change: `days_until_decay` = 30 − days since the newest log (null with no logs, 0 when already
decayed, floored at 0). The dashboard shows a notice when it is ≤ 10, and a different one when
the score has already been wiped, explaining that logging once restores it.
Acceptance: an integration test asserts 0 for a 40-day-old log, 5 for a 25-day-old one, and
null for a user with no logs; the dashboard notice is covered by a component test.

## Order of work, and the gate that ends it

1. Migration 0011 + the backend items it feeds (01, 06, 09, 10, 11, 13, 16) → `cargo test`.
2. Frontend items (01, 07, 08, 10, 12, 14, 15, 16) → `svelte-check`, `vitest run`, `vite build`.
3. Ops and static (02, 03, 04, 05) → the restore rehearsal, the cookie probe, the CI run.
4. Docs (this plan's siblings: `SPECIFICATION.md`, `OPERATIONS.md`, `README.md`, `STATUS.md`,
   `SESSION_CONTEXT.md`), then deploy through `scripts/deploy.sh` and verify each item against
   the live site.

Done means: the suite is green, every acceptance line above has been run and its output
recorded, the deploy's own health checks pass, and the four items that need a real live
behaviour (01, 02, 10, 14) have been exercised on `streakforge.polarisocial.xyz`.
