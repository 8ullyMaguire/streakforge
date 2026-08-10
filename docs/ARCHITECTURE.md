# StreakForge — Architecture

A deep dive into how the code is organized and how data flows through the system.
This complements SPECIFICATION.md (the "what") with the "where" and "how".

---

## 1. High-Level Diagram

```
┌──────────────────────────────┐         ┌─────────────────────────────────────┐
│  Browser (SPA)               │  HTTP   │  Rust backend (axum)                │
│                              │ ──────▶ │                                     │
│  SvelteKit 5 / adapter-static│  /api/* │  main.rs (router)                  │
│  routes/*.svelte             │         │   ├─ api.rs      (habit/drill/board)│
│  lib/api.ts (fetch client)   │ ◀────── │   ├─ auth.rs     (OAuth/session)    │
│  lib/markdown.ts             │  JSON   │   ├─ manifesto.rs (markdown docs)   │
│  lib/affirmations.ts         │         │   └─ spa_fallback (index.html)     │
│                              │         │          │                          │
│  static assets from /_app    │         │          ▼                          │
│                              │         │  sqlx (pool) ──▶ PostgreSQL         │
└──────────────────────────────┘         │  tower-sessions ─▶ session table    │
                                         │  fs (read) ──▶ manifestos/*.md      │
                                         └─────────────────────────────────────┘
```

Production topology (ThinkCentre M720q):

```
Cloudflare Tunnel (token-based, dashboard-managed route)
        │  hostname → http://192.168.1.13:8001
        ▼
streakforge-api (systemd, User=alvaro, binds 127.0.0.1:8001)
        │
        ├─ serves /var/www/streakforge (frontend build) via WEB_BUILD_DIR
        ├─ /api/* → Postgres (127.0.0.1:5432, role streakforge)
        ├─ /api/manifesto/* → /personal/documents/code/streakforge/manifestos
        └─ sessions → tower_sessions.session (Postgres)
```

---

## 2. Backend Module Map

### 2.1 `main.rs` — composition root

- Reads env → `Config::from_env()`.
- Connects pool (`db::connect`), runs embedded migrations (`db::run_migrations`).
- Migrates the session store (`PostgresStore::migrate()`).
- Builds the session layer (ephemeral `Key::generate()` — see SPEC §25).
- Builds `AppState { pool, cfg, x_oauth }`.
- CORS: allows `cfg.public_url` origin, credentials, GET/POST/PATCH/DELETE/OPTIONS.
- Router:
  - `/api/*` → handlers.
  - `/_app/*` → `ServeDir` for immutable build assets.
  - everything else → `spa_fallback` (reads index.html from WEB_BUILD_DIR).
- Layers: session_layer, cors, TraceLayer.

### 2.2 `lib.rs` — crate facade

`AppState` lives here so both the binary and integration tests can use the crate
without duplicating state. `pub mod api/auth/config/db/error/manifesto`.

### 2.3 `api.rs` — all API logic

Constants: `HOURLY_LIMIT=1`, `DAILY_LIMIT=5`.

Public handlers: `log_habit`, `get_stats`, `get_drill`, `get_leaderboard`,
`get_user_of_the_day`, `get_total`, `get_feed`, `get_profile`, `update_profile`.

Internal helpers: `require_user(session)`, `check_rate_limits(pool, uid, kind)`,
`compute_stats(pool, uid, kind)`, `week_start_utc(now)`, `parse_kind`,
`next_allowed_at(today, last_60m)`, `valid_username`.

DTOs: `LogRequest`, `LogResponse`, `DrillResponse`, `StatsResponse`, `FeedItem`,
`LeaderboardEntry`, `LeaderboardResponse`, `UserOfTheDay`, `FeedResponse`,
`ProfileResponse`, `UpdateProfileRequest`.

### 2.4 `auth.rs` — authentication

- Local username/password (Argon2id PHC) + tower-sessions cookies.
- `hash_password` / `verify_password` — Argon2id `m=19456, t=2, p=1`.
- Bot-dissuasion helpers: `validate_auth_form` (honeypot → form timing
  [3s,10min] → sha256 PoW proof), `compute_challenge_proof` /
  `challenge_proof_valid`, `fresh_nonce()` (16 random bytes hex).
- `register` / `login` handlers insert/verify `profiles` and start a session.
- Handlers: `me`, `logout`, `register`, `login`, `nonce`.
- `fetch_profile(pool, id)` refreshes the SessionUser from the DB.
- `sanitize_username(raw)` — ASCII alnum/underscore, 2..30 chars.

### 2.5 `manifesto.rs` — document server

- `list(state)` reads `MANIFESTOS_DIR`, collects `*.md`, sorts by id, returns titles
  derived from filenames.
- `get(state, id)` sanitizes id, reads `{id}.md` to string, returns
  `ManifestoContent {id, content}`.

### 2.6 `config.rs` / `db.rs` / `error.rs`

- config: env struct + defaults (see SPEC §18).
- db: `connect(dsn)` (10 max conns), `run_migrations(pool)`.
- error: `ApiError {status, message}` with `IntoResponse` (JSON `{error}`),
  `From<sqlx::Error>` (unique → 409, RowNotFound → 404), `From<redis::RedisError>`,
  `From<serde_json::Error>`, `From<session Error>`.

---

## 3. Frontend Module Map

### 3.1 `app.css` — the whole theme

CSS custom properties on `:root` (palette §11 of SPEC), plus component classes:
`.nav`, `.btn`, `.log-btn`, `.card`, `.counter-digits`, `.marquee`, `.board`, `.tabs`,
`.toast`, `.skeleton`, `.stat-grid`, `.avatar`, `.feed-item`, `.empty`, `.uotd`,
`.hero`, `.form-*`, `.footer`.

### 3.2 `lib/api.ts` — the single fetch layer

Every page talks to the backend through `api.*`. Error handling: any non-2xx JSON
`{error}` becomes `ApiRequestError`; pages catch it and either toast (429) or
redirect (401).

### 3.3 Components

- `Navbar.svelte` — brand, 5 links (DASHBOARD/DRILL/LEADERBOARD/FEED/MANIFESTO),
  user menu (username → /settings, Log out). Uses `$app/state` `page` for active state.
- `LogButton.svelte` — textarea (140) + big red button; props `stats()` getter and
  `onLogged(stats)` callback; disables while rate-limited; toasts 429 messages.
- `StreakCalendar.svelte` — 30-cell heatmap grid; intensity by count
  (rgba red scale); title tooltips.
- `ToastStack.svelte` — renders toasts from the runes store.
- `Marquee.svelte` — wlw scrolling ticker (duplicated groups, CSS animation).

### 3.4 Pages

| Route | Data source | Key behaviors |
|-------|-------------|---------------|
| `/` | userOfTheDay, leaderboard(daily), feed, /api/total | hero counter, marquee, UOTD card, top-25 board, recent activity |
| `/login` | — | SIGN IN / REGISTER tabs (honeypot + timing + JS challenge) |
| `/dashboard` | stats() | stat cards, LogButton, 30-day heatmap, recent logs; 401→/login |
| `/drill` | drill() | counter, affirmation card + prev/next, REPEAT, stats, heatmap |
| `/leaderboard` | leaderboard(period), userOfTheDay | tabs DAILY/WEEKLY/ALL-TIME, UOTD card, board |
| `/feed` | feed(cursor) | paginated list, LOAD MORE, avatars |
| `/manifesto` | manifestoList, manifestoDoc(id) | sidebar + rendered article |
| `/profile/[username]` | profile(username) | avatar, stats, joining date |
| `/settings` | me(), updateProfile | edit username/display/avatar; save → toast |

### 3.5 Utils

- `types.ts` — mirrors backend DTOs (keep in sync).
- `markdown.ts` — escape-then-render mini renderer (SPEC §10.5).
- `affirmations.ts` — deck + deterministic daily pick.
- `toasts.svelte.ts` — runes store: `pushToast`, `getToasts`, `dismissToast`.
- `validation.ts` — `isValidUsername`.

---

## 4. Data Flow Patterns

### 4.1 Stats round-trip (habit)

`GET /api/stats` → `compute_stats(pool, uid, LogKind::Habit)`:

1. today/week/alltime counts (single query, 3 correlated subselects, kind filter).
2. `last_60m` count (hourly window).
3. streak via `user_streak()` → cast `::bigint` for i64 decode.
4. recent 20 logs joined with profiles.
5. `can_log = last_60m < 1 && today < 5`; `next_allowed_at` if not.

Same shape for drill with `LogKind::Affirmation` + `user_streak_kind()`.

### 4.2 Logging round-trip

POST → rate check → insert → recompute stats → 201 with fresh stats. The SPA uses the
returned stats to update counters immediately (no second fetch).

### 4.3 Community reads

Leaderboards/UOTD/total/feed all call SQL functions defined in migrations (stable,
indexed, kind-filtered). No caching (fine at this scale).

---

## 5. Concurrency & Consistency Notes

- Single Postgres pool (10 conns); rate checks + insert are not in an explicit
  transaction, but the insert is the only write and the check is advisory — worst
  case a user squeaks past the limit under race (acceptable for v1; noted).
- `log_date` is a **generated column** — always consistent with `logged_at` in UTC.
- Session store uses Postgres (not memory) so sessions survive individual requests;
  the signing key is per-boot (SPEC §25).
- No background jobs/cron in the app.

---

## 6. Migration Strategy

```
0001_init.sql            profiles + habit_logs + user_streak()
0002_leaderboards.sql    daily/weekly/alltime/UOTD/total/feed/profile_stats
0003_affirmations.sql    ALTER habit_logs ADD kind; user_streak_kind();
                         redefines 0002 functions with kind='habit' filter
```

Rule: **never edit an applied migration** (sqlx checksum lock → VersionMismatch at
boot). Add new numbered files. 0003 exists precisely because 0002 was already applied
when the `kind` column was introduced.

---

## 7. Test Architecture

- Backend tests live in `backend/tests/integration.rs`; each `#[sqlx::test]` gets a
  fresh throwaway DB, migrations applied, and runs SQL-level assertions (no HTTP
  layer). This keeps them fast (~0.3s) and deterministic.
- Frontend tests are colocated `*.test.ts` under `web/src/lib/`; vitest with jsdom.
- Ad-hoc end-to-end verification used curl + a headless browser (see SPEC §21).

---

## 8. Extension Points

- **New log kind**: migration adds to the `kind` check constraint; add a variant to
  `LogKind` in api.rs; optionally a new community surface.
- **New manifesto doc**: drop a `NN_name.md` into `manifestos/` (title derived).
- **New page**: SvelteKit route under `web/src/routes/`; uses `api.*`; SPA fallback
  serves it automatically.
- **Redis rate limiting**: replace `check_rate_limits` DB queries with a Redis
  sliding window; `REDIS_URL` is already in config.
