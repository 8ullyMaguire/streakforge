# StreakForge

A whiteboi denial & accountability tracker with community leaderboards — dark,
counter-driven, and social. Log wastes when permitted, report denials, track
chastity locks, drill affirmations by typing them, and climb the board.

Built with **Rust (axum) + SvelteKit (Svelte 5)** and PostgreSQL. The UI theme
mirrors the aesthetic of wlw.grok.me: near-black background, red accent, gold
highlights, mono-spaced tabular counters, and a scrolling marquee.

> **Content note:** the core tracker is themed around BNWO denial kink per owner
> request — "Embrace Defeat. The Future Is Black." — and hosts the owner's curated
> BNWO doctrine texts (`/doctrine`) plus a typed affirmation drill (`/drill`).

---

## Documentation

Full context lives in `docs/` — read these before working on the code. **If a doc and
the code disagree, the code wins and the doc is a bug**: several sections here and in
the spec were describing pre-0008 behaviour until 2026-10-14, which is how a stale
sentence becomes a wrong decision six months later.

| File | Contents |
|------|----------|
| `docs/SPECIFICATION.md` | Complete spec (1058 lines): product, data model, API, business logic, security, testing, deployment, decision log, verification evidence |
| `docs/ARCHITECTURE.md` | Deep dive: module map, data flow, concurrency, migration strategy, extension points |
| `docs/OPERATIONS.md` | Run/deploy/monitor/troubleshoot (local + thinkcentre prod) |
| `docs/SESSION_CONTEXT.md` | Session-recovery cheat sheet |
| `docs/100_IDEAS.md` | 100 ideas ranked by impact ÷ effort; ranks 01–16 shipped 2026-10-14 |
| `docs/plans/2026-10-14-tier-1-and-tier-2.md` | The plan for those 16 items: a decision and an acceptance test each |
| `docs/OPERATIONS.md` | including backups (§6): the nightly dump and the restore rehearsal |

Plus `STATUS.md` for the living dev-status checklist.

---

## Features

- **One-button logging** — large "Log Completion" button on the dashboard.
- **Affirmation Drill** — a `/drill` page with a wlw-style big tabular counter,
  a cycling affirmation deck, and a "REPEAT" button. Affirmation reps log under
  a separate `kind` with their own rate-limit budget (1/hr, 5/day), so the drill
  doesn't consume (or pollute) your habit budget.
- **Rate limiting (server-enforced, per kind, per UTC calendar day)**:
  - Loads wasted: **5 per day**, configurable with `WLW_DAILY_LIMIT` (1–100).
  - Denials: 1 per day. A denial day is a stand.
  - Exclusivity is one-way: a waste spends the day's denial; a denial never
    spends a waste slot.
  - Drill reps: 5 per day, their own budget.
  - Clear 429 messages that name both the limit and the time it resets.
- **Personal dashboard** — today's count, current streak, longest streak, this
  week, all-time, recent logs, and a 30-day streak heatmap.
- **Community leaderboards** — Daily (rolling 24h), Weekly (7d), Monthly (30d) and
  All-time, scored by the **Whiteboi Devotion Index**
  (`(days wasted + days denied + drill bonus) × streak multiplier`), not by log count:
  five logs in one day count once, because consistency should beat volume. An account
  must be 24 hours old to hold a place. Top 50 each.
- **User of the Day** — highest today's count; ties broken by earliest log.
- **Community KPI dashboard** — a public `/kpi` page: loads wasted/denied
  totals, denial rate, affirmations, active (24h) and new (7d) users,
  currently-locked count, all-time lock hours, a 30-day stacked trend chart,
  and the top weekly board. No auth, like the landing counters.
- **Public activity feed** — reverse-chronological, cursor-paginated. **Activity
  only: notes are private** and only their author ever reads them back, on their own
  dashboard. (They used to be published here; see `docs/100_IDEAS.md` #01 and SPEC
  §7.4.)
- **A day's budget, a live clock, and a decay warning** — the dashboard shows how many
  logs are left today and when the cadence resets, warns ten days before 30 days of
  silence would wipe the score, and explains the score by its components.
- **Deep-link logging** — `/log?kind=habit|denial|affirmation&note=…` records in one
  tap from a phone shortcut, including the signed-out path through `/login?next=`.
- **A health check that can fail** — `/api/health` round-trips a query and answers 503
  when the database cannot, which `/health` never did (there was no such route: the SPA
  fallback answered 200 with `index.html`).
- **Glossary and terms** — `/glossary` defines every noun the score is made of, each
  entry naming where the rule is enforced, and is linked from the empty states and the
  doctrine index; `/terms` states what is public, what is
  private, and that the 18+ notice is an acknowledgement rather than an age check.
- **Profiles** — public username, display name, avatar (DiceBear fallback),
  stats. Editable in Settings.
- **Doctrine** — a `/doctrine` page serving curated doctrine/training texts
  (commandments, guides, socials) from the `manifestos/` directory, rendered
  markdown in the wlw-style theme.
- **Installable app** — PWA shell (`/manifest.webmanifest` + spade icon) with an
  `/app` install guide (Firefox/Chrome/Safari/desktop "add to home screen").
- **Auth** — local username/password (Argon2id) with session cookies.
  Bot-dissuasion on the public forms: hidden honeypot field, form timing
  (3s–10min window), and a JS proof-of-work challenge (sha256 over a
  server nonce). No X OAuth, no SMTP, no dev-login in production.
- **Social URL** — each profile can store one external link (editable in
  Settings, shown on the public profile when you click a user's name).
- **wlw-style theme** — near-black, red/gold accents, Inter + Roboto Mono,
  tabular green counter digits, scrolling marquee, mobile-first.

## Tech stack and language policy

| Layer     | Choice                                            |
|-----------|---------------------------------------------------|
| Backend   | Rust, axum 0.8, tower-sessions (Postgres store)   |
| Frontend  | SvelteKit (Svelte 5, SPA adapter-static), TypeScript |
| Styling   | Plain CSS with CSS variables (no framework)       |
| Database  | PostgreSQL 16, sqlx (runtime queries + migrations)|
| Auth      | Local username/password (Argon2id) + sessions, bot-dissuasion |
| Tests     | Rust `#[sqlx::test]` integration + Vitest          |

What this repo is, in one line: a Rust axum 0.8 backend on PostgreSQL 16 with
sqlx, and a SvelteKit (Svelte 5) front end in `web/` — the table above is the
detail, and the two manifests are the source of truth.

The estate default for new work is an axum + Postgres core, a SvelteKit front
end, the shared design tokens for anything that draws a UI, and contracts
between services — work-manifest v1, OpenAPI, RSS/ActivityPub — instead of a
shared database. Services read each other's published contracts; they do not
read each other's tables.

This repo is the default: the stack above is the one to copy for a new
service, and styling stays plain CSS over the shared CSS variables rather
than a component framework.

The rules that apply to every repo in the estate are in
`10-Projects/estate-master-prompt.md` in the secondbrain vault.

## Repository Layout

```
streakforge/
├── backend/                 # Rust axum API
│   ├── migrations/          # SQL migrations (schema + functions)
│   ├── src/
│   │   ├── main.rs          # router, session layer, SPA fallback
│   │   ├── lib.rs           # AppState + module facade
│   │   ├── api.rs           # logging, stats, leaderboards, feed, profiles
│   │   ├── auth.rs          # register/login/nonce, Argon2id, bot-dissuasion
│   │   ├── config.rs        # env config
│   │   ├── db.rs            # pool + migrations
│   │   └── error.rs         # ApiError
│   └── tests/integration.rs # DB-backed integration tests
└── web/                     # SvelteKit SPA
    ├── src/
    │   ├── lib/
    │   │   ├── api.ts       # fetch client + time formatting
    │   │   ├── types.ts     # API types
    │   │   ├── components/  # Navbar, LogButton, StreakCalendar, etc.
    │   └── routes/          # landing, login, dashboard, leaderboard, feed, profile, settings
    └── build/               # adapter-static output (served by Rust)
```

## Setup

### Prerequisites

- Rust 1.80+ (edition 2021)
- Node 20+ / npm
- PostgreSQL 14+ (running locally)
- Redis (optional; rate limiting is DB-based for v1)

### 1. Database

```bash
sudo -u postgres psql -c "CREATE ROLE streakforge WITH LOGIN PASSWORD 'streakforge_dev' CREATEDB;"
sudo -u postgres psql -c "CREATE DATABASE streakforge OWNER streakforge;"
sudo -u postgres psql -c "CREATE DATABASE streakforge_test OWNER streakforge;"  # for tests
```

Migrations run automatically on backend startup.

### 2. Backend

```bash
cd backend
cargo build
DATABASE_URL=postgres://streakforge:***@127.0.0.1:5432/streakforge WEB_BUILD_DIR=/abs/path/to/streakforge/web/build cargo run
```

The server listens on `http://127.0.0.1:8787` by default.

### 3. Frontend

```bash
cd web
npm install
npm run build   # production build
```

The SPA is served by the Rust backend at `/` (adapter-static build).

## Configuration (env vars)

| Var               | Default                          | Purpose                              |
|-------------------|----------------------------------|--------------------------------------|
| `DATABASE_URL`    | postgres://streakforge:***@127.0.0.1:5432/streakforge | Postgres DSN |
| `SESSION_SECRET`  | dev-only-insecure-...            | Cookie signing key (set in prod!)    |
| `PUBLIC_URL`      | http://127.0.0.1:8787            | Public base URL                      |
| `SECURE_COOKIES`  | `0`                              | Set to `1` behind HTTPS              |
| `WEB_BUILD_DIR`   | `./web/build`                    | SPA static dir served by Rust        |
| `MANIFESTOS_DIR`  | `./manifestos`                   | Directory of manifesto markdown docs |
| `BIND_ADDR`       | `127.0.0.1:8787`                 | Listen address                       |

## API Endpoints

| Method | Path                      | Auth | Description                          |
|--------|---------------------------|------|--------------------------------------|
| GET    | `/api/auth/nonce`         | —    | Fresh nonce for the JS challenge     |
| POST   | `/api/auth/register`      | —    | Create local account (bot-dissuaded) |
| POST   | `/api/auth/login`         | —    | Sign in (bot-dissuaded)              |
| GET    | `/api/auth/me`            | ✓    | Current session user                 |
| POST   | `/api/auth/logout`        | ✓    | End session                          |
| POST   | `/api/logs`               | ✓    | Log a completion (rate-limited, `kind` = habit|affirmation) |
| GET    | `/api/stats`              | ✓    | Personal habit stats + recent logs         |
| GET    | `/api/drill`              | ✓    | Affirmation drill stats (separate budget)  |
| GET    | `/api/leaderboard/{period}` | —  | daily / weekly / alltime             |
| GET    | `/api/user-of-the-day`    | —    | Today's top user                     |
| GET    | `/api/total`              | —    | Total completions (hero counter)     |
| GET    | `/api/today`              | —    | Today's denials / wastes / distinct loggers (UTC) |
| GET    | `/api/health`             | —    | `{status, database}`; **503** when the DB cannot answer |
| GET    | `/api/feed`               | —    | Public feed (cursor + limit) — activity only, no notes |
| GET    | `/api/profile/{username}` | —    | Public profile stats (+ social_url)  |
| PATCH  | `/api/profile`            | ✓    | Update username/display/avatar/social_url |
| GET    | `/api/doctrine`          | —    | List doctrine docs                   |
| GET    | `/api/doctrine/{id}`      | —    | Fetch a doctrine document (markdown) |

## Business Logic

### Streaks

A day counts if the user has ≥1 log on that UTC date.

- **Current streak** = consecutive days ending today (or yesterday if today is
  empty — you haven't broken it yet).
- **Longest streak** = longest historical run of consecutive logged days.

Implemented in `user_streak()` in `backend/migrations/0001_init.sql` (gaps-and-islands
with `row_number()` grouping).

### Rate limits

- 5 logs / UTC day / user / kind (`WLW_DAILY_LIMIT`, default 5) — checked against
  `log_date = today (UTC)`. A denial is 1/day; drill reps have their own budget.
- `habit`, `denial` and `affirmation` have independent budgets (drilling doesn't
  consume your daily waste allowance). Exclusivity is one-way: a waste recorded today
  spends the day's denial, but a denial never spends a waste slot — the load wasted
  after a stand is the event the site counts.
- Enforced in `check_rate_limits()` in `api.rs`, **before any insert**.
- **Failed logins have their own throttle** (5 per username, 30 per address, per 15
  minutes), checked before any password hashing, in a table so an API restart does not
  reset it. A successful sign-in clears that username's failures, not the address's.
  Past eight failures in a day for one username the wait doubles (30m, 1h, then a 2h
  ceiling), and every refusal is logged with the username and address — see SPEC §14.1
  and OPERATIONS §4.1. The address never escalates: a cafe shares it.

### The score (Whiteboi Devotion Index)

```
score = (days wasted + days denied + drill bonus) x multiplier
```

- **Days, not logs.** Five wastes in one day are one day of devotion. Repetition does
  not buy rank — which is exactly what the old row-based count allowed once the daily
  budget became five.
- **Multiplier** comes from the active streak: 1.0 (<7 days), 1.25 (7–13), 1.5 (14–29),
  2.0 (30+).
- **Drill bonus** is one point per three reps. Drill reps never hold a streak.
- **Decay**: 30 days without any log wipes the score; `/api/stats` reports
  `days_until_decay` so the dashboard warns at ten days. Lifetime totals never decay.
- **A board place needs a 24-hour-old account**, so making a second account to top a
  board is not free.

### User of the Day

Highest count on the current UTC date; ties broken by earliest first log of the
day, then all-time total. See `user_of_the_day()` in `0002_leaderboards.sql`.

## Testing

```bash
# Backend (needs a local Postgres the role can create databases in)
cd backend
DATABASE_URL=postgres://<role>:<pw>@127.0.0.1:5432/<any-db> cargo test

# 79 tests: 22 unit + 48 integration + 9 that drive the real router (tests/routes.rs).
# Each DB-backed test creates and drops its
# own scratch database (sqlx::test), so DATABASE_URL is a maintenance connection
# rather than a fixture schema -- point it at any database the role can reach.
#
# Two failure modes read as a broken suite rather than as what they are:
#   * a bare 28P01 means the credential drifted, not that the tests are wrong;
#   * "failed to connect to setup test database" usually means the role lacks
#     CREATEDB, which sqlx::test needs for its per-test scratch databases.
#
# CI (.github/workflows/ci.yml) does exactly this against a postgres:16 service.

# Frontend
cd web
npm run check
npm test   # vitest
```

## Configuration

Read from the environment at startup (`backend/src/config.rs`):

| Variable | Default | Meaning |
|----------|---------|---------|
| `DATABASE_URL` | local streakforge DB | Postgres |
| `SESSION_SECRET` | dev placeholder | **sessions are signed with this.** The signing key is derived from it, so it must be set and must not change between deploys — changing it signs every user out once. Set in prod (64 hex chars). |
| `SESSION_TTL_DAYS` | `30` | days of inactivity before a session expires (1–365) |
| `WLW_DAILY_LIMIT` | `5` | loads wasted allowed per UTC day per user (1–100) |
| `SECURE_COOKIES` | `0` | `1` behind HTTPS. Sets `Secure` AND renames the cookie to `__Host-streakforge_session` (browser-enforced prefix: Secure, `Path=/`, no `Domain`). **Turning it on signs every existing user out once** — the old cookie name is ignored. Set to `1` by `deploy.sh` in production. |
| `PUBLIC_URL` | `http://127.0.0.1:8787` | CORS origin |
| `WEB_BUILD_DIR` | `./web/build` | static frontend root |
| `MIGRATIONS_DIR` | `./migrations` | sqlx migrations |
| `MANIFESTOS_DIR` | `./manifestos` | doctrine pages |
| `BIND_ADDR` | `127.0.0.1:8787` | listen address |

An out-of-range or unparsable limit falls back to its default and logs a warning; it
is never silently clamped to 0, which would lock every user out of logging.

---

## Deployment Notes

- Production runs on **thinkcentre** (192.168.1.13) port **8001** as
  `streakforge.service` (systemd), serving static + API directly (mirrors fichub on
  8000; nginx is inactive there). Public URL: **https://streakforge.polarisocial.xyz**
  (Cloudflare tunnel, dashboard-managed route → `http://192.168.1.13:8001`).
- Deploy with `./scripts/deploy.sh thinkcentre` (builds release, syncs, installs unit).
- `SESSION_SECRET` + the Postgres DB password are random and kept in the Hermes
  profile `.env` on the dev machine (`~/.hermes/profiles/coding/.env` →
  `STREAKFORGE_SESSION_SECRET` / `STREAKFORGE_DB_PASSWORD`); rotate them on
  thinkcentre after deploy (see OPERATIONS.md §3.1).
- Build the frontend with a plain `npm run build` (no dev-login variant exists).
- Set `SECURE_COOKIES=1` behind HTTPS (the deploy does it, and it renames the cookie to
  `__Host-streakforge_session`, which signs everyone out exactly once).
- **Backups**: the deploy installs two cron entries — a nightly `pg_dump` at 03:30 and a
  restore rehearsal on the 1st at 04:30. The rehearsal is the half that makes the backup
  real, and it is also what notices if the nightly job stops running (it fails when the
  newest dump is over 48h old). Details in OPERATIONS.md §6.
- Health: check `/api/health` (200 `ok`, 503 `degraded`), not `/health` and not `/`.
- Rate limiting is DB-query based (simple, reliable for v1); a Redis sliding window
  can replace it later.

## Roadmap / Out of Scope (v1)

Deliberately **not** in v1: multiple habit types per user, notifications/emails,
friends/following, complex analytics, native mobile apps, payments/premium.

## License

MIT
