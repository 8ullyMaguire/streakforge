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

Full context lives in `docs/` — read these before working on the code:

| File | Contents |
|------|----------|
| `docs/SPECIFICATION.md` | Complete spec (1058 lines): product, data model, API, business logic, security, testing, deployment, decision log, verification evidence |
| `docs/ARCHITECTURE.md` | Deep dive: module map, data flow, concurrency, migration strategy, extension points |
| `docs/OPERATIONS.md` | Run/deploy/monitor/troubleshoot (local + thinkcentre prod) |
| `docs/SESSION_CONTEXT.md` | Session-recovery cheat sheet |

Plus `STATUS.md` for the living dev-status checklist.

---

## Features

- **One-button logging** — large "Log Completion" button on the dashboard.
- **Affirmation Drill** — a `/drill` page with a wlw-style big tabular counter,
  a cycling affirmation deck, and a "REPEAT" button. Affirmation reps log under
  a separate `kind` with their own rate-limit budget (1/hr, 5/day), so the drill
  doesn't consume (or pollute) your habit budget.
- **Rate limiting (server-enforced, per kind)**:
  - Max 1 log per 60 minutes per user.
  - Max 5 logs per UTC calendar day per user.
  - Clear 429 error messages with "next allowed at" hints.
- **Personal dashboard** — today's count, current streak, longest streak, this
  week, all-time, recent logs, and a 30-day streak heatmap.
- **Community leaderboards** — Daily (UTC), Weekly (Monday UTC start), and
  All-time. Top 50 each, with rank, avatar, username, count, and last-log time.
- **User of the Day** — highest today's count; ties broken by earliest log.
- **Public activity feed** — reverse-chronological, cursor-paginated, with notes.
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

## Tech Stack

| Layer     | Choice                                            |
|-----------|---------------------------------------------------|
| Backend   | Rust, axum 0.8, tower-sessions (Postgres store)   |
| Frontend  | SvelteKit (Svelte 5, SPA adapter-static), TypeScript |
| Styling   | Plain CSS with CSS variables (no framework)       |
| Database  | PostgreSQL 16, sqlx (runtime queries + migrations)|
| Auth      | Local username/password (Argon2id) + sessions, bot-dissuasion |
| Tests     | Rust `#[sqlx::test]` integration + Vitest          |

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
| GET    | `/api/feed`               | —    | Public feed (cursor + limit)         |
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

- 1 log / 60 min / user / kind — checked against `logged_at >= now() - 60 min`.
- 5 logs / UTC day / user / kind — checked against `log_date = today (UTC)`.
- `habit` and `affirmation` have independent budgets (drilling doesn't consume
  your daily habit allowance).
- Enforced in `check_rate_limits()` in `api.rs`, before any insert.

### User of the Day

Highest count on the current UTC date; ties broken by earliest first log of the
day, then all-time total. See `user_of_the_day()` in `0002_leaderboards.sql`.

## Testing

```bash
# Backend (needs local Postgres, streakforge_test DB)
cd backend
DATABASE_URL=postgres://streakforge:streakforge_dev@127.0.0.1:5432/streakforge_test cargo test

# 26 tests: 8 unit + 18 integration. The integration tests need the test
# database to exist AND the role's password to actually be the one above --
# sqlx's test harness panics with "failed to connect to setup test database"
# and a bare 28P01 if the password drifted, which reads as a broken suite
# rather than a credential mismatch. Reset it with:
#   sudo -u postgres psql -c "ALTER ROLE streakforge WITH PASSWORD '...';"

# Frontend
cd web
npm run check
npm test   # vitest
```

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
- Set `SECURE_COOKIES=1` behind HTTPS.
- Rate limiting is DB-query based (simple, reliable for v1); a Redis sliding window
  can replace it later.

## Roadmap / Out of Scope (v1)

Deliberately **not** in v1: multiple habit types per user, notifications/emails,
friends/following, complex analytics, native mobile apps, payments/premium.

## License

MIT
