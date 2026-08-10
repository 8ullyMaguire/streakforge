# StreakForge

A habit & streak tracker with community leaderboards — dark, counter-driven, and
social. Log daily completions, build unbreakable streaks, climb the leaderboard,
and own your consistency.

Built with **Rust (axum) + SvelteKit (Svelte 5)** and PostgreSQL. The UI theme
mirrors the aesthetic of wlw.grok.me: near-black background, red accent, gold
highlights, mono-spaced tabular counters, and a scrolling marquee.

> **Note on content:** This project is a *mechanics clone* of wlw.grok.me's
> counter/leaderboard/feed pattern. The subject matter here is completely SFW —
> a habit tracker — with no NSFW elements.

---

## Features

- **One-button logging** — large "Log Completion" button on the dashboard.
- **Rate limiting (server-enforced)**:
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
- **Manifesto** — a `/manifesto` page serving curated doctrine/training texts
  (commandments, guides, socials) from the `manifestos/` directory, rendered
  markdown in the wlw-style theme.
- **Auth** — X (Twitter) OAuth 2.0 with PKCE, plus a local dev-login for testing.
- **wlw-style theme** — near-black, red/gold accents, Inter + Roboto Mono,
  tabular green counter digits, scrolling marquee, mobile-first.

## Tech Stack

| Layer     | Choice                                            |
|-----------|---------------------------------------------------|
| Backend   | Rust, axum 0.8, tower-sessions (Postgres store)   |
| Frontend  | SvelteKit (Svelte 5, SPA adapter-static), TypeScript |
| Styling   | Plain CSS with CSS variables (no framework)       |
| Database  | PostgreSQL 16, sqlx (runtime queries + migrations)|
| Auth      | X OAuth 2.0 PKCE + dev login                      |
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
│   │   ├── auth.rs          # X OAuth + dev login + session helpers
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
# optional: set X_CLIENT_ID / X_CLIENT_SECRET for real X login
ALLOW_DEV_LOGIN=1 WEB_BUILD_DIR=/abs/path/to/streakforge/web/build cargo run
```

The server listens on `http://127.0.0.1:8787` by default.

### 3. Frontend

```bash
cd web
npm install
VITE_ALLOW_DEV_LOGIN=1 npm run build   # include the dev-login button
```

The SPA is served by the Rust backend at `/` (adapter-static build).

## Configuration (env vars)

| Var               | Default                          | Purpose                              |
|-------------------|----------------------------------|--------------------------------------|
| `DATABASE_URL`    | postgres://streakforge:...@127.0.0.1:5432/streakforge | Postgres DSN |
| `SESSION_SECRET`  | dev-only-insecure-...            | Cookie signing key (set in prod!)    |
| `X_CLIENT_ID`     | —                                | X OAuth client ID                    |
| `X_CLIENT_SECRET` | —                                | X OAuth client secret                |
| `PUBLIC_URL`      | http://127.0.0.1:8787            | Public base URL (OAuth redirect)     |
| `ALLOW_DEV_LOGIN` | `1`                              | Enable `/api/auth/dev-login`         |
| `SECURE_COOKIES`  | `0`                              | Set to `1` behind HTTPS              |
| `WEB_BUILD_DIR`   | `./web/build`                    | SPA static dir served by Rust        |
| `MANIFESTOS_DIR`  | `./manifestos`                   | Directory of manifesto markdown docs |
| `BIND_ADDR`       | `127.0.0.1:8787`                 | Listen address                       |

## API Endpoints

| Method | Path                      | Auth | Description                          |
|--------|---------------------------|------|--------------------------------------|
| GET    | `/api/auth/me`            | ✓    | Current session user                 |
| POST   | `/api/auth/logout`        | ✓    | End session                          |
| GET    | `/api/auth/x`             | —    | Start X OAuth (redirects to X)       |
| GET    | `/api/auth/x/callback`    | —    | OAuth callback                       |
| POST   | `/api/auth/dev-login`     | —    | Dev login (if enabled)               |
| POST   | `/api/logs`               | ✓    | Log a completion (rate-limited)      |
| GET    | `/api/stats`              | ✓    | Personal stats + recent logs         |
| GET    | `/api/leaderboard/{period}` | —  | daily / weekly / alltime             |
| GET    | `/api/user-of-the-day`    | —    | Today's top user                     |
| GET    | `/api/total`              | —    | Total completions (hero counter)     |
| GET    | `/api/feed`               | —    | Public feed (cursor + limit)         |
| GET    | `/api/profile/{username}` | —    | Public profile stats                 |
| PATCH  | `/api/profile`            | ✓    | Update username/display/avatar       |
| GET    | `/api/manifesto`          | —    | List manifesto docs                  |
| GET    | `/api/manifesto/{id}`     | —    | Fetch a manifesto document (markdown)|

## Business Logic

### Streaks

A day counts if the user has ≥1 log on that UTC date.

- **Current streak** = consecutive days ending today (or yesterday if today is
  empty — you haven't broken it yet).
- **Longest streak** = longest historical run of consecutive logged days.

Implemented in `user_streak()` in `backend/migrations/0001_init.sql` (gaps-and-islands
with `row_number()` grouping).

### Rate limits

- 1 log / 60 min / user — checked against `logged_at >= now() - 60 min`.
- 5 logs / UTC day / user — checked against `log_date = today (UTC)`.
- Enforced in `check_rate_limits()` in `api.rs`, before any insert.

### User of the Day

Highest count on the current UTC date; ties broken by earliest first log of the
day, then all-time total. See `user_of_the_day()` in `0002_leaderboards.sql`.

## Testing

```bash
# Backend (needs local Postgres, streakforge_test DB)
cd backend
DATABASE_URL=postgres://streakforge:streakforge_dev@127.0.0.1:5432/streakforge_test cargo test

# Frontend
cd web
npm run check
npm test   # vitest
```

## Deployment Notes

- Build frontend with `VITE_ALLOW_DEV_LOGIN=1 npm run build` (or omit the env
  var to hide the dev button in production).
- Run the Rust binary from the repo root (or set `WEB_BUILD_DIR` to an absolute
  path) so the SPA fallback can find `index.html`.
- Set `SESSION_SECRET` to a long random value in production.
- Set `SECURE_COOKIES=1` behind HTTPS.
- Rate limiting is DB-query based (simple, reliable for v1); a Redis sliding
  window can replace it later if the free tier becomes limiting.

## Roadmap / Out of Scope (v1)

Deliberately **not** in v1: multiple habit types per user, notifications/emails,
friends/following, complex analytics, native mobile apps, payments/premium.

## License

MIT
