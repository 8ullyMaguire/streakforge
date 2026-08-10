# StreakForge — Complete Specification

> **Document status:** authoritative, generated from the actual codebase and verified
> behavior on 2026-08-10. Every claim here was exercised by real tool output during
> development (tests, API calls, browser sessions, deployment checks).
>
> **Purpose:** this is the single source of truth for StreakForge. It exists so a
> fresh coding agent (or a human) can reconstruct full context from this document
> alone — no need to reverse-engineer the code.

---

## Table of Contents

1. [Product Overview](#1-product-overview)
2. [Vision & Origins](#2-vision--origins)
3. [Feature Set](#3-feature-set)
4. [Tech Stack](#4-tech-stack)
5. [Repository Layout](#5-repository-layout)
6. [Data Model](#6-data-model)
7. [Business Logic](#7-business-logic)
8. [API Contract](#8-api-contract)
9. [Authentication](#9-authentication)
10. [Frontend Architecture](#10-frontend-architecture)
11. [Theme & UI Language](#11-theme--ui-language)
12. [Subsystem: Affirmation Drill](#12-subsystem-affirmation-drill)
13. [Subsystem: Manifesto](#13-subsystem-manifesto)
14. [Rate Limiting](#14-rate-limiting)
15. [Security](#15-security)
16. [Testing](#16-testing)
17. [Deployment](#17-deployment)
18. [Environment Configuration](#18-environment-configuration)
19. [Development Workflow](#19-development-workflow)
20. [Git History & Decision Log](#20-git-history--decision-log)
21. [Verification Evidence](#21-verification-evidence)
22. [Known Issues & Gotchas](#22-known-issues--gotchas)
23. [Roadmap & Out of Scope](#23-roadmap--out-of-scope)

---

## 1. Product Overview

StreakForge is a **habit & streak tracker with community leaderboards**. Users log
daily "I did it" completions, the system tracks individual streaks (current +
longest), totals (today/week/all-time), and exposes a public leaderboard, a
"User of the Day", and a public activity feed.

The UI is a deliberate **mechanics clone of wlw.grok.me**: a near-black theme with
red/gold accents, a giant tabular mono-spaced counter, a scrolling marquee, and a
leaderboard of usernames. The original site is an NSFW counter/leaderboard site;
StreakForge adapts the *aesthetic and interaction pattern* to a habit tracker.

Core goals (from the original design doc the project was built from):

- Extremely low-friction logging (one primary button).
- Strong visual feedback (streaks, counters, rankings).
- Light social competition via leaderboards and public activity.
- Built-in rate limiting to prevent spam/abuse.
- Clean, modern, mobile-first UI.

### Content note

The repo also contains a `/manifesto` page (a curated set of BNWO doctrine/training
texts) and an `/drill` page (affirmation repetitions with a counter). These match the
non-SFW subject matter of the reference site wlw.grok.me and were explicitly requested
by the owner. The core habit tracker itself is content-neutral.

---

## 2. Vision & Origins

The project was created in a single session on 2026-08-10, driven by a design document
("Habit & Streak Tracker with Community Leaderboards") plus follow-up requests:

1. **Initial ask:** "build a website like wlw.grok.me with same theme and features but
   built with rust+svelte and with more features" (streaks, leaderboards, rate limits,
   user of the day, feed, profiles).
2. **Theme clarification:** "still nsfw not sfw, same theme" — the site should keep the
   wlw dark/counter aesthetic; content is owner-curated.
3. **Manifesto:** "add also a manifestho like /home/alvaro/documents/text/nsfw/bnwo to
   the site" — a page serving the owner's BNWO guide collection.
4. **Affirmation drill:** "add the affirmation drill also with a counter from
   wlw.grok.me" + "what's the rate limit for completions? it should be around one per
   hour and 5 per day or so" — added a `/drill` page with its own counter and a
   per-kind rate-limit budget.
5. **Deploy:** "deploy to thinkcentre, and similar how fichub is setup (recall nginx)
   serve on port 8001 so I can add a route to it on cloudflare tunnel" — deployed as a
   systemd service on the ThinkCentre, port 8001, behind the existing Cloudflare tunnel.
6. **Publish:** "push to opencommit.eu, private" — pushed to a private Forgejo repo.
7. **Document:** "save a markdown file to the repo with all relevant context, at least
   1k lines, call it specification or whatever" — this document set.

---

## 3. Feature Set

### 3.1 Core habit tracking

- **One-button logging** — large "Log Completion" button on the dashboard.
- Optional note per completion (max 140 chars).
- Personal stats: today's count, current streak, longest streak, this week, all-time.
- Recent personal logs (last 20).
- 30-day streak heatmap.

### 3.2 Community features

- **Leaderboards**: Daily (UTC calendar day), Weekly (Monday 00:00 UTC), All-time.
  Top 50 each, columns: rank, avatar, username, count, last-log relative time.
- **User of the Day (UOTD)**: highest count on the current UTC date; ties broken by
  earliest first log, then all-time total. Shown as a gold crown card.
- **Public activity feed**: reverse-chronological, cursor-paginated (50/page, max 100),
  with avatars + relative times + notes.

### 3.3 Affirmation Drill (`/drill`)

- wlw-style big tabular counter (personal affirmation total, 8 digits, green/black).
- Cycling affirmation deck (16 phrases), prev/next navigation.
- Big red "REPEAT" button; green flash feedback on each rep.
- Separate stats: today, drill streak, longest, this week + 30-day heatmap.
- Affirmation reps use `kind='affirmation'` and their own rate-limit budget.

### 3.4 Manifesto (`/manifesto`)

- Sidebar list of 9 curated markdown documents + rendered viewer.
- Served from a configurable directory (`MANIFESTOS_DIR`, default `./manifestos`).
- Lightweight markdown renderer (headings, bold/italic, links, lists, blockquotes,
  code, hr) with full HTML escaping (no raw HTML passthrough).

### 3.5 Auth & profiles

- Local username/password auth (Argon2id, PHC strings), session cookies
  (tower-sessions, Postgres-backed store).
- Public register/login forms are bot-dissuaded: hidden honeypot field, form
  timing (3s–10min open window), and a JS proof-of-work challenge
  (`sha256(nonce || username || password)` truncated to 16 hex chars, nonce from
  `GET /api/auth/nonce`).
- Profiles: public username (unique, 2–30 chars, `[a-zA-Z0-9_]`), display name (≤50),
  avatar URL (http(s) only; DiceBear identicon fallback client-side), and an
  optional `social_url` (one external link, http(s) only) shown on the public
  profile when someone clicks the user's name.
- Public profile page with stats + "forging since" date.
- Settings page to edit username/display/avatar/social_url.
- Passwords: min 8 chars, hashed with Argon2id (~19 MiB, t=2, p=1); legacy
  X/dev rows have no hash and can't log in this way.

### 3.6 UX polish

- Toast notifications (success/error), loading skeletons, empty states.
- 401 redirect to `/login` on protected pages.
- Mobile-first responsive design.
- Dark mode only (system + manual toggle is not implemented — the theme is dark-only).

---

## 4. Tech Stack

| Layer | Choice | Notes |
|-------|--------|-------|
| Backend | Rust 2021 edition, axum 0.8 | async, tokio, `{capture}` route syntax |
| Sessions | tower-sessions 0.14 + tower-sessions-sqlx-store 0.15 | **version pairing matters** (see Gotchas) |
| Database | PostgreSQL 16 (local dev + ThinkCentre prod) | sqlx 0.8, runtime queries + embedded migrations |
| Frontend | SvelteKit 2 (Svelte 5), adapter-static | SPA mode, `fallback: 'index.html'` |
| Styling | Plain CSS + CSS custom properties | no Tailwind/shadcn — deliberate |
| Auth | argon2 0.5 (Argon2id), local username/password | bot-dissuasion: honeypot + form timing + sha256 nonce challenge |
| HTTP client | reqwest 0.12 (rustls) | (kept; no longer used for OAuth) |
| Icons | lucide-svelte | |
| Tests | Rust `#[sqlx::test]` integration + Vitest | targeted suites only (see Testing) |
| Deploy | systemd + Cloudflare tunnel (token-based) | ThinkCentre M720q, port 8001 — public https://streakforge.polarisocial.xyz |

Deliberately **not used** despite the original design doc suggesting them: Next.js,
Supabase, Tailwind, shadcn/ui, TanStack Query, Vercel. The actual stack is the
owner-mandated Rust + Svelte.

---

## 5. Repository Layout

```
streakforge/                        # repo root (git, private on opencommit.eu)
├── README.md                       # user-facing overview (expanded)
├── STATUS.md                       # dev-status checklist (kept current)
├── docs/
│   ├── SPECIFICATION.md            # THIS FILE
│   ├── ARCHITECTURE.md             # deep dive on code, flows, subsystems
│   ├── OPERATIONS.md               # deploy/dev/troubleshooting
│   └── SESSION_CONTEXT.md          # session-recovery cheat sheet
├── backend/
│   ├── Cargo.toml                  # deps (see Gotchas for version pins)
│   ├── Cargo.lock
│   ├── migrations/
│   │   ├── 0001_init.sql           # profiles, habit_logs, user_streak()
│   │   ├── 0002_leaderboards.sql   # daily/weekly/alltime/UOTD/total/feed/profile_stats
│   │   ├── 0003_affirmations.sql   # kind column, user_streak_kind(), kind-filtered redefs
│   │   └── 0004_local_auth.sql     # password_hash + social_url, provider default 'local' |
│   ├── src/
│   │   ├── main.rs                 # router, session layer, SPA fallback, startup
│   │   ├── lib.rs                  # AppState + module facade (for tests)
│   │   ├── api.rs                  # ALL API handlers + DTOs + rate limiting
│   │   ├── auth.rs                 # register/login/nonce, Argon2id, bot-dissuasion, sessions |
│   │   ├── auth_tests.rs           # unit tests for challenge proof / validation
│   │   ├── manifesto.rs            # /api/manifesto list + get
│   │   ├── config.rs               # env config struct
│   │   ├── db.rs                   # pool connect + migrations
│   │   └── error.rs                # ApiError (IntoResponse), From impls
│   └── tests/
│       └── integration.rs          # 6 DB-backed tests (see Testing)
├── web/
│   ├── package.json                # scripts: dev/build/check/test
│   ├── svelte.config.js            # adapter-static, SPA fallback
│   ├── vite.config.ts              # /api proxy to 8787 in dev
│   ├── vitest.config.ts
│   ├── tsconfig.json
│   └── src/
│       ├── app.html                # shell + Google Fonts (Inter + Roboto Mono)
│       ├── app.css                 # full theme (CSS vars, wlw-style)
│       ├── hooks.server.ts         # trivial handle
│       ├── lib/
│       │   ├── api.ts              # fetch client + timeAgo + formatCount
│       │   ├── types.ts            # API types mirroring backend DTOs
│       │   ├── markdown.ts         # tiny markdown renderer
│       │   ├── affirmations.ts     # affirmation deck + daily pick
│       │   ├── toasts.svelte.ts    # toast store (Svelte 5 runes)
│       │   ├── validation.ts       # isValidUsername
│       │   ├── components/
│       │   │   ├── Navbar.svelte
│       │   │   ├── LogButton.svelte
│       │   │   ├── StreakCalendar.svelte
│       │   │   ├── ToastStack.svelte
│       │   │   └── Marquee.svelte
│       │   └── *.test.ts           # vitest tests alongside
│       └── routes/
│           ├── +layout.svelte      # navbar + footer + toasts
│           ├── +error.svelte
│           ├── +page.svelte        # landing (hero, counter, marquee, UOTD, boards, feed)
│           ├── login/+page.svelte
│           ├── dashboard/+page.svelte
│           ├── drill/+page.svelte
│           ├── leaderboard/+page.svelte
│           ├── feed/+page.svelte
│           ├── manifesto/+page.svelte
│           ├── profile/[username]/+page.svelte
│           └── settings/+page.svelte
├── manifestos/                     # 9 curated markdown docs (served by /api/manifesto)
├── scripts/
│   ├── dev.sh                      # local dev: build frontend + run backend
│   ├── seed.sql                    # demo users + historical logs
│   └── deploy.sh                   # ThinkCentre deploy (systemd, port 8001)
└── .gitignore                      # target/, node_modules/, build/, .svelte-kit/, .env
```

---

## 6. Data Model

All timestamps are UTC. Tables live in the `public` schema.

### 6.1 `profiles`

| Column | Type | Constraints |
|--------|------|-------------|
| id | uuid | PK, default gen_random_uuid() |
| username | text | UNIQUE NOT NULL, check `^[a-zA-Z0-9_]{2,30}$` |
| display_name | text | check length ≤ 50 |
| avatar_url | text | nullable |
| password_hash | text | nullable (Argon2id PHC string; added in 0004) |
| social_url | text | nullable, check `^https?://` or empty (added in 0004) |
| provider | text | NOT NULL default 'local', check in ('local','x','dev') (0004) |
| provider_id | text | nullable; UNIQUE(provider, provider_id) |
| created_at | timestamptz | NOT NULL default now() |
| updated_at | timestamptz | NOT NULL default now() |

Notes:
- Local accounts: `provider='local'`, `provider_id=NULL`, keyed off unique
  `username`; `password_hash` holds the Argon2id PHC string.
- Legacy X/dev rows (provider 'x'/'dev') have `password_hash=NULL` and can't log
  in via the local form. `UNIQUE(provider, provider_id)` stays valid for them
  (multiple NULLs are allowed in Postgres).
- There is **no auth.users table** — sessions reference profiles directly.

### 6.2 `habit_logs`

| Column | Type | Constraints |
|--------|------|-------------|
| id | bigint | PK, generated always as identity |
| user_id | uuid | NOT NULL, FK → profiles(id) ON DELETE CASCADE |
| logged_at | timestamptz | NOT NULL default now() |
| note | text | check length ≤ 140 |
| log_date | date | GENERATED ALWAYS AS ((logged_at at time zone 'utc')::date) STORED |
| kind | text | NOT NULL default 'habit', check in ('habit','affirmation') (added in 0003) |

Indexes:
- `habit_logs_user_id_logged_at_idx` (user_id, logged_at desc)
- `habit_logs_log_date_idx` (log_date)
- `habit_logs_logged_at_idx` (logged_at desc)
- `habit_logs_kind_user_logged_at_idx` (kind, user_id, logged_at desc)
- `habit_logs_kind_log_date_idx` (kind, log_date)

### 6.3 Sessions (tower-sessions)

Created by `tower-sessions-sqlx-store`'s `migrate()` at startup: schema `tower_sessions`,
table `session`. Cookie is signed with `SESSION_SECRET`. Expiry: `OnSessionEnd`.

---

## 7. Business Logic

### 7.1 Streak calculation

A **day counts** if the user has ≥1 log of that kind on that UTC date.

- **Current streak** = consecutive days ending today — or ending yesterday if today
  has zero logs (you haven't broken it yet; today still counts as "in progress").
- **Longest streak** = longest historical run of consecutive logged days.

Implementation: gaps-and-islands via `row_number() OVER (ORDER BY log_date)`:

```sql
-- user_streak(p_user uuid) in 0001_init.sql; user_streak_kind(p_user, p_kind) in 0003
with days as (
  select distinct log_date from habit_logs
  where user_id = p_user [and kind = p_kind]
),
ordered as (
  select log_date,
         log_date - (row_number() over (order by log_date))::int as grp
  from days
),
groups as (
  select grp, min(log_date) as start_d, max(log_date) as end_d, count(*) as len
  from ordered group by grp
)
select
  case
    when g.end_d = t.d or g.end_d = t.d - 1 then g.len   -- current run touches today/yesterday
    else 0
  end as current_streak,
  coalesce(g2.longest, 0) as longest
from (select (now() at time zone 'utc')::date as d) t
left join groups g on g.end_d = (select max(end_d) from groups)
cross join (select max(len) as longest from groups) g2;
```

Key subtlety: the `current_streak` branch allows the run to end **yesterday** (today
empty = streak alive but 0). The **longest** uses only actually-logged days.

Verified cases:
- no logs → (0, 0)
- one log today → (1, 1)
- 3 consecutive days → (3, 3); adding a gap day 5-ago → still (3, 3)
- past 7-day run (days 5..11 ago) + current 3-day run → current=3, longest=7

### 7.2 Leaderboard ranking

- **Primary**: count DESC.
- **Secondary**: earliest max log (`min(l.logged_at)` ASC) — "first to reach the count".
- `daily_leaderboard(50)`: logs where `log_date = today(UTC)`, kind='habit'.
- `weekly_leaderboard(50)`: logs where `logged_at >= date_trunc('week', now() at time zone 'utc')` (Monday 00:00 UTC), kind='habit'.
- `alltime_leaderboard(50)`: all logs, kind='habit'.

### 7.3 User of the Day

```
user_of_the_day():
  count = count(l.id) today, first_log_at = min(l.logged_at),
  alltime_count = (select count(*) ... kind='habit')
  where log_date = today and kind = 'habit'
  order by count desc, min(l.logged_at) asc
  limit 1
```

Tie-break: earliest first log wins; all-time total is informational (displayed on the
card). Verified: two users with 1 log each — the earlier logger is UOTD.

### 7.4 Feed pagination

```
activity_feed(p_cursor bigint, p_limit int):
  where (p_cursor = 0 or l.id < p_cursor) and kind = 'habit'
  order by l.id desc
  limit p_limit
```

Cursor is the last item's `id` (keyset pagination, stable). The API returns
`next_cursor` = last id when the page is full, else null.

### 7.5 Profile stats

`profile_stats(p_user)` / inline queries return: streak, longest_streak, today_count,
week_count, alltime_count — all filtered to kind='habit' for the public profile surface.

---

## 8. API Contract

Base URL: `/api` (proxied by the SPA in dev; same origin in production).

### 8.1 Auth

| Method | Path | Auth | Request | Response |
|--------|------|------|---------|----------|
| GET | `/auth/nonce` | — | — | `{"nonce": "<32-hex>"}` (fresh, for the JS challenge) |
| POST | `/auth/register` | — (bot-dissuaded) | JSON `AuthForm` | 201 `SessionUser` (session cookie set), or 400/409 |
| POST | `/auth/login` | — (bot-dissuaded) | JSON `AuthForm` | 200 `SessionUser` (session cookie set), or 400/401 |
| GET | `/auth/me` | ✓ | — | `SessionUser` or 401 |
| POST | `/auth/logout` | ✓ | — | 303 redirect to `/` |

`AuthForm` (JSON body):
```json
{
  "username": "string",
  "password": "string",
  "form_opened_at": 1234567890000,
  "website": "",
  "challenge_proof": "<16 hex chars>",
  "challenge_nonce": "<32 hex chars>"
}
```
- `website` = honeypot, must be empty.
- `form_opened_at` = client ms epoch when the form was opened; must be within
  3s–10min of server time at submit.
- `challenge_proof` = `sha256(nonce || username || password)` truncated to the
  first 16 hex chars; recomputed server-side.
- Register: username 2–30 chars `[a-zA-Z0-9_]`, password 8–1024 chars; duplicate
  username → 409 "That username is already taken".
- Login: any failure (unknown user, missing hash, bad password) → generic
  401 "Invalid username or password" (timing-equalized for unknown users).

`SessionUser`:
```json
{ "id": "uuid", "username": "string", "display_name": "string|null",
  "avatar_url": "string|null", "provider": "local",
  "social_url": "string|null" }
```

### 8.2 Logs & stats

| Method | Path | Auth | Request | Response |
|--------|------|------|---------|----------|
| POST | `/logs` | ✓ | `{"note": "...", "kind": "habit|affirmation"}` | 201 `{"stats": StatsResponse}` or 429 |
| GET | `/stats` | ✓ | — | `StatsResponse` (kind='habit') |
| GET | `/drill` | ✓ | — | `{"stats": StatsResponse}` (kind='affirmation') |

`StatsResponse`:
```json
{
  "today_count": 0, "current_streak": 0, "longest_streak": 0,
  "week_count": 0, "alltime_count": 0,
  "recent_logs": [ /* FeedItem[] max 20 */ ],
  "last_60m": 0, "can_log": true, "next_allowed_at": "iso|null"
}
```

### 8.3 Community

| Method | Path | Auth | Response |
|--------|------|------|----------|
| GET | `/leaderboard/{period}` | — | `{"period": "daily|weekly|alltime", "entries": [LeaderboardEntry×50]}` |
| GET | `/user-of-the-day` | — | `UserOfTheDay \| null` |
| GET | `/total` | — | `{"total": N}` (habit kind only) |
| GET | `/feed?cursor=&limit=` | — | `{"items": [FeedItem], "next_cursor": N\|null}` |

`LeaderboardEntry`: `{ rank, user_id, username, display_name, avatar_url, count, last_log_at }`
`UserOfTheDay`: `{ user_id, username, display_name, avatar_url, count, first_log_at, alltime_count }`
`FeedItem`: `{ id, user_id, username, display_name, avatar_url, logged_at, note }`

### 8.4 Profiles

| Method | Path | Auth | Request | Response |
|--------|------|------|---------|----------|
| GET | `/profile/{username}` | — | — | `ProfileResponse` or 404 |
| PATCH | `/profile` | ✓ | `{"username"?, "display_name"?, "avatar_url"?, "social_url"?}` | `ProfileResponse` or 400/409 |

`ProfileResponse`: `{ id, username, display_name, avatar_url, social_url, created_at,
streak, longest_streak, today_count, week_count, alltime_count }`

PATCH validation:
- username: 2–30 chars `[a-zA-Z0-9_]` (400 otherwise), unique (409 "That username is already taken").
- display_name: ≤50 chars; empty → None.
- avatar_url: must start with http:// or https://; empty → None.
- social_url: must start with http:// or https:// (DB check too); empty → None.

### 8.5 Manifesto

| Method | Path | Auth | Response |
|--------|------|------|----------|
| GET | `/manifesto` | — | `{"docs": [{id, title, filename}]}` |
| GET | `/manifesto/{id}` | — | `{"id": "...", "content": "markdown string"}` |

`id` sanitizer: only `[a-zA-Z0-9_-]`; anything else → 400. Nonexistent → 404.

---

## 9. Authentication

Local username/password auth. No X OAuth, no dev-login, no SMTP. The public
register/login endpoints are bot-dissuaded with three layered checks.

### 9.1 Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/api/auth/nonce` | Fresh random 32-hex nonce for the JS challenge |
| POST | `/api/auth/register` | Create a local account (Argon2id) and start a session |
| POST | `/api/auth/login` | Verify credentials and start a session |
| GET | `/api/auth/me` | Current session user (401 if none) |
| POST | `/api/auth/logout` | Flush session, redirect to `/` |

Register/login both take the same JSON `AuthForm` body (see §8.1).

### 9.2 Password storage

- Argon2id (`argon2` crate 0.5), parameters: m=19456 KiB (~19 MiB), t=2, p=1.
- Stored as a PHC string in `profiles.password_hash` (migration 0004).
- Password rules: 8–1024 chars (register only).
- Login is timing-equalized: unknown usernames burn a dummy Argon2 hash so the
  response time doesn't reveal whether the username exists.

### 9.3 Bot-dissuasion (register + login forms)

Three layered checks in `validate_auth_form()` — all failures return a generic
400 "Invalid form submission" so bots can't probe which check tripped:

1. **Honeypot** — a hidden `website` field that humans never see. Bots that
   autofill every input fill it; any non-empty value is rejected.
2. **Form timing** — the form records `form_opened_at` (client ms epoch) when it
   loads. On submit the server requires the form to have been open between
   `FORM_MIN_OPEN_MS` (3s) and `FORM_MAX_OPEN_MS` (10 min). Instant submissions
   (bots) and stale sessions (harvested forms) are rejected.
3. **JS proof-of-work challenge** — the client fetches `GET /api/auth/nonce` for a
   fresh random nonce, computes `proof = sha256(nonce || username || password)`,
   and sends the first 16 hex chars plus the nonce back. The server recomputes and
   requires an exact match (constant-time compare). A bot that skips the JS cannot
   produce a valid proof without running the challenge.

Because the proof covers the actual password, the challenge can't be replayed and
the nonce is single-use in effect (fresh per page load).

### 9.4 Sessions

- tower-sessions `Session` extractor; store = PostgresStore (schema `tower_sessions`).
- Cookie signed with `SESSION_SECRET`; `SECURE_COOKIES=1` behind HTTPS.
- Handlers read the user from `session.get(SESSION_USER_KEY)`; 401 if absent.

### 9.5 Legacy X/dev rows

Profiles created before migration 0004 (`provider='x'` or `'dev'`) have
`password_hash = NULL` and cannot log in through the local form. They keep their
public surface (leaderboards, feed, profile pages) but are effectively read-only.

---

## 10. Frontend Architecture

### 10.1 SPA mode

- `svelte.config.js`: adapter-static with `pages: 'build'`, `assets: 'build'`,
  `fallback: 'index.html'`, `strict: false`; `prerender.entries = []`.
- The Rust backend serves `build/` statically and falls back to `index.html` for any
  non-`/api`, non-`/_app` route (client-side routing works on refresh).

### 10.2 API client (`web/src/lib/api.ts`)

- `request<T>(path, init)` wraps fetch with `credentials: 'include'`, JSON headers,
  parses `{error}` bodies into `ApiRequestError(message, status)`.
- Exposed methods: `me, logout, logHabit(note?, kind?), stats, drill, leaderboard,
  userOfTheDay, feed(cursor?), profile(username), updateProfile, manifestoList,
  manifestoDoc(id)`.
- Helpers: `timeAgo(iso)` (just now → s/m/h/d → locale date), `formatCount(n)` (pads
  to 8 digits, matching the wlw counter width).

### 10.3 State

- Svelte 5 runes (`$state`, `$derived`, `$props`) everywhere; no stores except the
  toast store (`toasts.svelte.ts`) which uses module-level `$state`.

### 10.4 Route guards

- Protected pages (dashboard, drill, settings) call their API on mount; on
  `ApiRequestError` with status 401 they `window.location.href = '/login'`.
- There is no SSR guard (SPA), so this is the mechanism.

### 10.5 Markdown renderer (`markdown.ts`)

- Pure function `renderMarkdown(src)` → HTML string.
- Escapes HTML first, then applies: headings h1–h6, paragraphs, `---` hr, `>` blockquote,
  `-/*/+` and numbered lists (→ `<ul>`), fenced code blocks, inline `**bold**`, `*italic*`,
  `` `code` ``, `[text](url)` links (http/https/relative only; target=_blank).
- No raw HTML passthrough. Used by the manifesto page via `{@html rendered}`.

---

## 11. Theme & UI Language

Sourced from the actual wlw.grok.me CSS (fetched and analyzed during development).

### 11.1 Palette (CSS vars in `app.css`)

| Var | Value | Use |
|-----|-------|-----|
| `--bg` | `#0a0a0a` | page background |
| `--bg-elev` | `#121212` | nav/skeleton surfaces |
| `--bg-card` | `#161616` | cards |
| `--border` | `#262626` | borders |
| `--text` | `#f0f0f0` | primary text |
| `--text-dim` | `#9aa0a6` | secondary text |
| `--red` | `#e31c23` | primary accent (buttons, active tabs, links) |
| `--red-soft` | `#c41e1e` | secondary red |
| `--gold` | `#d4a017` | UOTD card, longest-streak value, h2 accents |
| `--green` / `--digit-green` | `#22c55e` | counter digits (last 3), board counts, success |
| `--digit-black` | `#6b7280` | counter digits (leading) |

Fonts: **Inter** (sans) + **Roboto Mono** (mono) from Google Fonts, loaded in `app.html`.

### 11.2 Counter digits (wlw signature)

```css
.counter-digits {
  font-family: var(--font-mono);
  font-variant-numeric: tabular-nums lining-nums;
  font-feature-settings: 'tnum' 1, 'lnum' 1, 'zero' 0;
  letter-spacing: 0.08em;
}
```

Digits are rendered one span each; the **last 3** get `digit-green`, the rest
`digit-black` — matching wlw's "00000062" where the 062 is green. The landing hero
counter shows the **global habit total**; the drill counter shows the **user's
affirmation total**.

### 11.3 Marquee

wlw has a scrolling marquee of usernames. `Marquee.svelte` renders the items twice
(`[items, items]`) inside a track animated `marquee-scroll 40s linear infinite`, with
a mask fading edges and pause-on-hover. The landing page feeds it recent feed
usernames + relative times.

### 11.4 Other wlw-style elements

- `EMBRACE THE STREAK. THE FUTURE IS CONSISTENT.` tagline (mono, letter-spaced, red).
- "Sign in to forge your streak. Own your submission." under the CTA.
- "USER OF THE DAY" gold crown card.
- "TOP FORGERS TODAY" board with green mono counts, rank column (gold/silver/bronze for
  top 3).
- Red gradient "Log Completion" / "REPEAT" buttons with glow on hover.

---

## 12. Subsystem: Affirmation Drill

### 12.1 Data

`habit_logs.kind` distinguishes `'habit'` (dashboard) from `'affirmation'` (drill).
Both share the same table and indexes; all community queries filter `kind='habit'`.

### 12.2 Rate limits (per kind)

`check_rate_limits(pool, user_id, kind)`:
- Hourly: `count(*) where user_id=$1 and kind=$kind and logged_at >= now()-60min` ≥ 1 → 429.
- Daily: `count(*) where user_id=$1 and kind=$kind and log_date = today(utc)` ≥ 5 → 429.
- Each kind has an **independent** budget: logging an affirmation does NOT consume a
  habit slot, and vice versa. Verified by integration test + API check
  (affirmation 201/429, then habit still 201).

### 12.3 Affirmation deck

`web/src/lib/affirmations.ts` exports:
- `AFFIRMATIONS: Affirmation[]` — 16 curated phrases with `{text, source}`.
- `affirmationOfTheDay(seed = Date.now())` — deterministic daily pick:
  `AFFIRMATIONS[floor(seed/86400000) % length]`.

Phrases are drawn from the BNWO affirmation/mantra threads (steeltitan Bluesky post,
Breeder592 X thread) and the starting guide / commandments / plapping guide.

### 12.4 Drill page

- Counter: personal affirmation all-time total, 8-digit padded, green/black digits.
- Card shows current affirmation (index state), source tag, prev/next arrows.
- REPEAT button: `api.logHabit(undefined, 'affirmation')`; on success flips the card
  green (`card.flipped` class + glow), updates stats, shows toast.
- When rate-limited: button disabled + "Next rep allowed in ~1h 0m" hint.
- Stat cards: Today, Drill streak, Longest, This week + 30-day heatmap (StreakCalendar).
- Footer line: "Today's affirmation: ..." from `affirmationOfTheDay()`.

### 12.5 SQL

`user_streak_kind(p_user uuid, p_kind text)` — mirror of `user_streak` filtered by kind.
Migration 0003 also **redefines** the community functions (daily/weekly/alltime boards,
UOTD, total_logs, activity_feed) to filter `kind='habit'` so drill reps never appear
in leaderboards/feed/UOTD/total.

---

## 13. Subsystem: Manifesto

### 13.1 Content

9 curated markdown documents copied from `/home/alvaro/documents/text/nsfw/bnwo`
into `manifestos/` (repo root):

```
01_starting_guide.md        02_commandments.md       03_good_whiteboy_guide.md
04_get_partner_blacked.md   05_get_girlfriend_blacked.md   06_plapping_guide.md
07_plapping_inspiration.md  08_time_trials.md        09_socials.md
```

Titles are derived from filenames (strip numeric prefix + `.md`, replace `_` with
space, capitalize first letter).

### 13.2 Backend

- `GET /api/manifesto` lists docs from `MANIFESTOS_DIR` (default `./manifestos`),
  sorted by id.
- `GET /api/manifesto/{id}` reads `{id}.md`; strict id sanitization (alphanumeric +
  `_` + `-`) prevents path traversal; 404 on missing.

### 13.3 Frontend

- `/manifesto` page: sticky sidebar (doc buttons, active state red-left-border),
  content article with `.md-body` styles (gold h2, red h3, blockquote accents).
- Rendered with the custom `markdown.ts` renderer (no external md library).

---

## 14. Rate Limiting

- **Values**: `HOURLY_LIMIT = 1`, `DAILY_LIMIT = 5` (in `api.rs`).
- **Granularity**: per user, per kind.
- **Window**: rolling 60-minute window (query-based) + fixed UTC calendar day.
- **Enforcement**: before insert, in `check_rate_limits()`; 429 with a clear message
  ("Hourly limit reached (1 per hour). You can log again at <iso>." or "Daily limit
  reached (5 per day). Come back tomorrow.").
- `next_allowed_at` computed in the stats response: +60min for hourly cap, midnight UTC
  for daily cap.
- No Redis needed for v1 (documented as a future upgrade path).

---

## 15. Security

- **Auth on writes**: all mutation endpoints (POST /logs, PATCH /profile, logout)
  require a session user → 401 without.
- **Path traversal**: manifesto id sanitized; `/api/manifesto/%2e%2e%2fetc%2fpasswd`
  → 400 "Invalid manifesto id"; slashed variants fall through to the SPA shell (not a
  file leak). Verified.
- **XSS**: markdown renderer escapes HTML before rendering; `{@html}` only gets the
  escaped output. Avatar URL validated http(s). Username restricted to
  `[a-zA-Z0-9_]`.
- **Secrets**: `SESSION_SECRET` required in prod (random 64-hex, generated on
  deploy and saved in the Hermes profile `.env` as `STREAKFORGE_SESSION_SECRET`; the
  Postgres DB password is likewise random, saved as `STREAKFORGE_DB_PASSWORD`);
  git credential store (`~/.config/git/credentials`) holds the Forgejo token —
  never commit tokens.
- **Bot-dissuasion**: public register/login forms require an empty honeypot,
  3s–10min form timing, and a valid sha256 nonce challenge proof (see §9.3).
- **CORS**: dev CORS allows the configured PUBLIC_URL origin with credentials.
- Unknown `/api/*` paths return 404 (the SPA fallback refuses `/api/` prefixes).

---

## 16. Testing

### 16.1 Backend (Rust)

`backend/tests/integration.rs`, 13 tests, all `#[sqlx::test(migrations = "./migrations")]`
(creates a throwaway DB per test from the migrations):

1. `streak_calculation_basic` — no logs (0,0); one today (1,1).
2. `streak_multiple_days` — 3-day run, gap, 7-day past run: current=3, longest=7.
3. `leaderboard_ranking` — two users 3 vs 2 logs; order + counts.
4. `user_of_the_day_tiebreak` — equal counts; earlier logger wins.
5. `feed_pagination` — 2-per-page keyset pagination, ids strictly decreasing.
6. `affirmation_kind_separated_from_community` — drill streak counts only
   affirmations; alltime board/feed/total count only habits.
7. Auth integration (7 tests) — register creates a local profile + session;
   duplicate username → 409; login with correct/incorrect password; generic 401
   (no user-enumeration); bot-dissuasion (honeypot, form timing, bad challenge
   proof) → 400; social_url round-trip through PATCH /profile.

Run (needs local Postgres + `streakforge_test` DB):
```bash
cd backend
DATABASE_URL=postgres://streakforge:streakforge_dev@127.0.0.1:5432/streakforge_test cargo test --test integration
```
**Never run the full workspace test suite** (cargo ~10min). Target the integration
suite.

### 16.2 Frontend (Vitest)

`web/src/lib/*.test.ts`, 33 tests across 5 files:

- `api.test.ts` (12) — nonce/register/login payloads, error surfacing, timeAgo
  boundaries, formatCount padding.
- `validation.test.ts` (4) — isValidUsername.
- `markdown.test.ts` (8) — headings, bold/italic, links, escaping, lists, blockquote,
  code blocks, hr.
- `affirmations.test.ts` (4) — deck non-empty, fields present, deterministic daily pick,
  cycle wrap.
- `challenge.test.ts` (5) — sha256 proof computation matches the expected 16-hex
  prefix, nonce handling, edge inputs.

Run (targeted):
```bash
cd web
npx vitest run src/lib/api.test.ts src/lib/validation.test.ts   # or any file
```
Plus `npm run check` (svelte-check, 0 errors; 2 pre-existing warnings: deprecated
`<slot />` and a self-closing div).

### 16.3 Ad-hoc verification

During development a temp script (`/tmp/hermes-verify-streakforge.sh`) was used for
end-to-end checks: routes 200, SPA shell, rate-limit 201→429, separate budgets,
manifesto count, traversal 400. It is removed after each run (not committed).

---

## 17. Deployment

### 17.1 ThinkCentre (production)

- **Host**: ThinkCentre M720q, Linux Mint, hostname `thinkcentre` (192.168.1.13),
  user `alvaro`.
- **Service**: `streakforge.service` (systemd), `User=alvaro`,
  `WorkingDirectory=/personal/documents/code/streakforge`,
  `EnvironmentFile=/personal/documents/code/streakforge/.env`,
  `ExecStart=/personal/documents/code/streakforge/streakforge-api`.
- **Ports**: backend binds `127.0.0.1:8001` (BIND_ADDR). FicHub occupies 8000
  (fichub.service, same pattern).
- **Static**: `/var/www/streakforge` (frontend build; backend serves it directly via
  WEB_BUILD_DIR).
- **DB**: PostgreSQL on ThinkCentre, role `streakforge` / db `streakforge`, password
  `streakforge_prod` (dev-local is `streakforge_dev`). Note: role was created via
  `sudo -u postgres psql` (alvaro needs a TCP password; peer auth via sudo works).
- **Cloudflare**: tunnel runs token-based (`cloudflared tunnel run --token-file
  /etc/cloudflared/token`); routes are configured in the Cloudflare dashboard — the
  user adds a route pointing to `http://192.168.1.13:8001`.
- **Nginx**: installed but **inactive** on the ThinkCentre. FicHub serves on 8000
  directly from its binary; StreakForge does the same on 8001. No nginx site needed.

### 17.2 Deploy script

`scripts/deploy.sh [host=thinkcentre]`:
1. `cargo build --release`
2. `npm run build` (web — plain; no dev-login variant exists)
3. rsync release binary + migrations + manifestos → `$DEPLOY_DIR`;
   rsync web/build → `/var/www/streakforge`
4. write `.env` (prod values, placeholder SESSION_SECRET + DB password)
5. install systemd unit (enable + restart)
6. print verification hint

**Production secrets live in the Hermes profile `.env`** on the dev machine
(`~/.hermes/profiles/coding/.env`): `STREAKFORGE_SESSION_SECRET` (random 64-hex)
and `STREAKFORGE_DB_PASSWORD` (random; rotated on ThinkCentre). `deploy.sh` no
longer writes real secrets to the remote `.env` — set/rotate them on thinkcentre
before starting the service:

```bash
SECRET=$(openssl rand -hex 32)
ssh thinkcentre "sudo sed -i 's|^SESSION_SECRET=.*|SESSION_SECRET=$SECRET|' /personal/documents/code/streakforge/.env && sudo systemctl restart streakforge"
```

### 17.3 Local dev

`scripts/dev.sh` builds the frontend (`npm run build`) and runs the backend with
`WEB_BUILD_DIR` + `MANIFESTOS_DIR` set to repo paths, `cargo run`. Register a
local account in the UI to log in.

`scripts/seed.sql` inserts demo users + historical logs (iron_will 10-day streak,
daily_dave 6-day, streak_queen broken 12-day, noob_forger, late_night) for a lively
leaderboard:
```bash
psql -h 127.0.0.1 -U streakforge -d streakforge -f scripts/seed.sql
```

---

## 18. Environment Configuration

| Var | Local default | Prod (ThinkCentre .env) | Purpose |
|-----|---------------|-------------------------|---------|
| `DATABASE_URL` | `postgres://streakforge:***@127.0.0.1:5432/streakforge` | `postgres://streakforge:***@127.0.0.1:5432/streakforge` | Postgres DSN (prod password random, saved as `STREAKFORGE_DB_PASSWORD` in the Hermes profile `.env`) |
| `SESSION_SECRET` | dev-only-insecure... | random 64-hex (saved as `STREAKFORGE_SESSION_SECRET` in the Hermes profile `.env`) | cookie signing key |
| `PUBLIC_URL` | `http://127.0.0.1:8787` | `http://127.0.0.1:8001` | CORS origin |
| `SECURE_COOKIES` | `0` | `0` (set 1 behind HTTPS) | cookie Secure flag |
| `WEB_BUILD_DIR` | `./web/build` | `/var/www/streakforge` | SPA static dir |
| `MANIFESTOS_DIR` | `./manifestos` | `/personal/documents/code/streakforge/manifestos` | manifesto docs dir |
| `BIND_ADDR` | `127.0.0.1:8787` | `127.0.0.1:8001` | listen address |
| `RUST_LOG` | (env) | `info` | tracing filter |

Removed with the X OAuth / dev-login rework: `X_CLIENT_ID`, `X_CLIENT_SECRET`,
`ALLOW_DEV_LOGIN`, and the frontend build-time `VITE_ALLOW_DEV_LOGIN` no longer
exist anywhere in the codebase or deployment.

Note: `REDIS_URL` is in config.rs but unused by any code path (rate limiting is
DB-based).

---

## 19. Development Workflow

Per the owner's conventions (also used for FicHub):

- Commit often, small units, Conventional Commits
  (`feat:`, `fix:`, `test:`, `chore:`, `docs:`).
- One git worktree per feature when delegating; main agent integrates, subagents
  never commit.
- **Run only new/changed tests**, never full suites.
- Keep STATUS.md current after feature work (mark done items, update test counts).
- Update README + docs after features.
- Push mirror to `opencommit.eu` after merges (remote `github`).

---

## 20. Git History & Decision Log

```
d667aea fix(api): 404 unknown /api/* paths instead of SPA shell fallback
7e60488 chore(deploy): use random SESSION_SECRET + DB password (rotated on ThinkCentre)
33c296f docs: update README/STATUS + deploy.sh for local auth (drop X OAuth/dev-login)
917c1cd merge: local auth UI (login/register form, social URL)
7765323 feat(web): username/password auth UI + social URL, drop X/dev-login
9aa2817 feat(auth): local username/password auth replaces X OAuth + dev-login
c226861 docs: clarify deploy.sh serves directly on 8001 (no nginx)
1e99bce chore: add ThinkCentre deploy script (mirrors fichub pattern)
8e01e64 feat: add affirmation drill with wlw-style counter
71b52a2 feat: add manifesto page serving doctrine/training texts
706dce1 chore: add seed script and dev startup script
7a938ce feat: StreakForge habit tracker with wlw-style leaderboards
```

Decision log (why things are the way they are):

1. **Rust + Svelte, not Next.js/Supabase** — owner requirement; the original design
   doc's stack was overridden.
2. **axum 0.8 `{capture}` routes** — `:period` syntax panics at startup in axum 0.8;
   use `{period}` / `{username}`.
3. **tower-sessions version pairing** — tower-sessions 0.14 + sqlx-store 0.15 both use
   tower-sessions-core 0.14. Other pairs (0.13/0.13, 0.15/0.15, 0.14/0.14) mismatch.
4. **Local auth replaces X OAuth + dev-login** (`9aa2817`) — no external
   dependencies, no dev/prod auth asymmetry, and bot-dissuasion (honeypot + form
   timing + sha256 nonce challenge) protects the public forms. The `argon2` crate
   replaced the `oauth2` crate; `reqwest` is kept but unused for auth.
5. **AppState lives in lib.rs** — so integration tests and the binary share it; main.rs
   uses `streakforge_api::*`.
6. **SPA fallback must be a handler, not ServeDir** — ServeDir alone 404s SPA routes;
   main.rs has a `spa_fallback` handler serving index.html for non-/api, non-/`_app`.
   Since `d667aea`, unknown `/api/*` paths return 404 (the fallback refuses `/api/`).
7. **CSS `//` comments break the bundle** — Vite kept `//` comments in the CSS; the
   browser dropped the `:root` rule → all vars empty. Use `/* */`.
8. **adapter-static, not adapter-node** — for a Rust-hosted SPA, adapter-static with
   `fallback: 'index.html'` produces a servable static build.
9. **INT4 vs i64** — `user_streak()` returns integer; Rust decodes i64 → cast
   `::bigint` in queries.
10. **activity_feed limit param** — function takes `int`; bind i64 → `$2::int`.
11. **Migration 0003 redefines 0002 functions** — 0002 was already applied (checksum
    locked); edits to it would fail VersionMismatch. New function definitions go in
    new migrations. (0004 follows the same rule: new columns, never edits.)
12. **Secrets out of the repo** (`7e60488`) — random `SESSION_SECRET` + DB password
    live in the Hermes profile `.env` on the dev machine, not in deploy.sh or the
    remote `.env` writes.
13. **nginx not used on ThinkCentre** — fichub serves directly on 8000; streakforge
    mirrors on 8001.
14. **Rate limits per kind** — drill reps have their own 1/hr + 5/day budget so the
    drill doesn't consume the habit allowance; community surfaces filter kind='habit'.

---

## 21. Verification Evidence

All of the following were actually exercised (not assumed):

- **Backend**: `cargo test --test integration` → 13 passed (each run);
  `cargo test --lib` (auth unit tests) → 8 passed.
- **Frontend**: `npx vitest run` → 33 passed; `npm run check` → 0 errors.
- **API smoke (curl)**: register 201 (session set); duplicate register 409; login
  200 with correct credentials / 401 generic with wrong ones; bot-dissuasion
  rejects missing honeypot, out-of-window form timing, and bad challenge proof
  (400); log 201; second log 429 (hourly); 5th log 201 / 6th 429 (daily);
  leaderboard daily/weekly/alltime correct ranks; UOTD = late_night (6 today);
  profiles/streaks correct (iron_will 10, streak_queen 0/12); feed pagination;
  profile PATCH (update + 400 invalid username + 409 dup + social_url set/clear);
  unauth 401 on /stats, /logs POST, /profile PATCH.
- **Separate budgets**: affirmation 201 → affirmation 429 → habit 201.
- **Manifesto**: list 9 docs; content JSON; traversal `%2e%2e%2fetc%2fpasswd` → 400;
  missing → 404.
- **Browser (headless)**: landing (counter 00000062, marquee animating, UOTD, board,
  feed), dashboard (stats, log button, rate-limit disable), leaderboard tabs,
  feed pagination, profile page, settings update (DB verified), drill
  (counter 00000001, card flip, "Next rep allowed" hint), manifesto sidebar + docs.
- **Theme computed styles**: bg rgb(10,10,10); body Inter; counter Roboto Mono 88px;
  digits black #6b7280 / green #22c55e; UOTD gold #d4a017.
- **Deploy (ThinkCentre)**: systemd active; root/drill/api 200; assets 200;
  manifesto 9; unauth 401; register/login work (bot-dissuaded); unknown /api/*
  → 404; port 8001 listening; fichub 8000 untouched; survives restart; random
  SESSION_SECRET + DB password (in Hermes profile `.env`).
- **Push**: private repo created (id 241) at opencommit.eu/MagicZhang/streakforge;
  main pushed at c226861; private:true confirmed via API.

---

## 22. Known Issues & Gotchas

1. **svelte-check warnings (2)**: deprecated `<slot />` in Navbar.svelte (+layout) and
   a self-closing non-void `<div style="flex:1;" />` in +page.svelte. Cosmetic; no
   errors. (Can be cleaned with `{@render}` + explicit close tag.)
2. **Public auth forms require JS** — the sha256 nonce challenge, form timing, and
   honeypot are enforced server-side on register/login; a client with JS disabled
   or a skewed clock gets 400 "Invalid form submission". By design (bot-dissuasion).
3. **PUBLIC_URL matters for CORS** — must match the externally-reachable URL when
   behind the tunnel, or credentialed requests from the browser are blocked.
4. **Migration checksum lock** — never edit an applied migration; append new ones.
5. **Session cookie secret** — rotating it logs everyone out (fine); the deployed
   secret was set post-deploy via sed + restart.
6. **Streak "current" grace** — current streak survives an empty today (ends
   yesterday). The UI labels it "current streak"; a user who missed today still sees
   the streak until tomorrow.
7. **Feed ordering uses id, not logged_at** — keyset pagination on id desc is stable;
   inserts with backdated timestamps won't reorder history (seed data uses recent
   now() offsets so it looks right).
8. **Dev server proxy** — `vite.config.ts` proxies `/api` → 127.0.0.1:8787; if you
   change BIND_ADDR, update the proxy.
9. **Cloudflared is token-based** — no config file on the host; routes are dashboard-
   managed. Adding a route: point a hostname/path at `http://192.168.1.13:8001`.
10. **The `github` remote is actually opencommit.eu** — named `github` for
    consistency with FicHub's convention; the real upstream is the owner's Forgejo.

---

## 23. Roadmap & Out of Scope

In scope for future iterations (owner-driven):
- Cloudflare route for the tunnel (dashboard, not code).
- HTTPS termination (SECURE_COOKIES=1) behind the tunnel.
- Possibly multiple habit types (schema is `kind`-extensible; add values to the check
  constraint).

Deliberately out of scope (v1):
- Multiple user-defined habits per user.
- Notifications / emails.
- Friends / following.
- Complex analytics charts.
- Native mobile apps.
- Payments / premium tiers.
- Light/dark toggle (dark-only theme).

---

---

## 24. Request Lifecycle Walkthrough

### 24.1 Register / login (bot-dissuaded local auth)

```
Browser ──GET /api/auth/nonce──────────────────────────▶ axum
   ◀──200── {"nonce": "<32-hex>"}
Browser (JS) computes proof = sha256(nonce || username || password)[0..16]
   ──POST /api/auth/register | /api/auth/login────────▶ axum
   │  validate_auth_form(body):                       (all failures → 400 generic)
   │    honeypot "website" empty?                     ("Invalid form submission")
   │    form open time within 3s..10min?              (server clock vs form_opened_at)
   │    challenge_proof == sha256(nonce||user||pass)[0..16]?
   │  register: username valid (2-30 [a-zA-Z0-9_]), password 8-1024
   │            Argon2id hash → INSERT profiles (provider='local')
   │  login:    SELECT profile by username; Argon2id verify (dummy hash for
   │            unknown users → generic 401 "Invalid username or password")
   │  session.insert("user", SessionUser)
   ◀──201 (register) | 200 (login)── SessionUser JSON (Set-Cookie: id=<signed>)
```

### 24.2 Log a completion (rate-limited)

```
Browser ──POST /api/logs {note, kind}─────────────────▶ axum
   │  require_user(session) → SessionUser | 401
   │  validate note ≤ 140 chars → 400
   │  parse_kind("habit"|"affirmation") → LogKind
   │  check_rate_limits(pool, user_id, kind) → 429 | ok
   │  INSERT habit_logs (user_id, note, kind)
   │  compute_stats(pool, user_id, kind) → StatsResponse
   ◀──201── {"stats": {...}}
```

### 24.3 SPA navigation (any route)

```
Browser ──GET /dashboard──────────────────────────────▶ axum
   │  route match? no /api, no /_app → spa_fallback
   │  read {WEB_BUILD_DIR}/index.html → 200 text/html
   ◀──200── SPA shell (JS boots, client-side routing takes over)
```

---

## 25. Session Signing Detail (IMPORTANT)

The session layer is configured with an **ephemeral key**:

```rust
// main.rs
let key = tower_sessions::cookie::Key::generate();   // random per process boot
let session_layer = SessionManagerLayer::new(session_store)
    .with_expiry(Expiry::OnSessionEnd)
    .with_signed(key)
```

Consequences:

- Cookie signatures are valid only for the current process lifetime. A `systemctl
  restart streakforge` invalidates all sessions (users must log in again).
- The `SESSION_SECRET` env var is read into `Config` but is **not** currently wired to
  the session key. It is documented as "cookie signing key" — that intent is not yet
  realized in code.

Desired fix (not yet implemented):

```rust
let key = tower_sessions::cookie::Key::from(cfg.session_secret.as_bytes());
```

This makes sessions survive restarts and gives SESSION_SECRET its documented role.
Deployed impact today: acceptable for a personal tool, but if sessions dropping on
restart becomes a problem, apply the one-line fix.

---

## 26. Dependency Versions (pinned in Cargo.toml)

| Crate | Version | Why pinned |
|-------|---------|------------|
| axum | 0.8 | `{capture}` route syntax; `Key`/`Method` types |
| tokio | 1 (full) | async runtime |
| tower-http | 0.6 | cors, trace, ServeDir |
| tower-sessions | 0.14 | **must pair** with sqlx-store 0.15 |
| tower-sessions-sqlx-store | 0.15 | uses core 0.14 (same as sessions 0.14) |
| sqlx | 0.8 (postgres, migrate, time, uuid, chrono) | runtime queries + embedded migrations |
| argon2 | 0.5 | Argon2id password hashing (PHC strings) |
| rand | 0.8 | nonce generation, salt RNG |
| sha2 | 0.10 | JS challenge proof (`sha256(nonce‖username‖password)[0..16]`) |
| reqwest | 0.12 (rustls, json) | kept from the X-OAuth era; unused for auth |
| serde / serde_json | 1 | DTOs |
| time | 0.3 (serde-well-known) | OffsetDateTime, Rfc3339 |
| uuid | 1 (v4, serde) | profile ids |

Frontend deps (package.json): `svelte ^5`, `@sveltejs/kit ^2`, `@sveltejs/adapter-static
^3`, `lucide-svelte`, dev: `vitest ^2`, `svelte-check`, `typescript ^5`, `jsdom`,
`@testing-library/svelte`.

---

## 27. Future Work & Owner Notes

- Add the Cloudflare tunnel route in the dashboard: hostname → `http://192.168.1.13:8001`.
- Apply the session-key fix (Section 25) if restart-logout becomes annoying.
- Consider `SECURE_COOKIES=1` once behind HTTPS.
- The `kind` check constraint is extensible: adding a new habit type = new migration
  with `ALTER TABLE ... DROP CONSTRAINT ... ADD CONSTRAINT ... check (kind in (...))`
  plus a kind-filtered community query if it should stay private.
- If bot registrations ever become a problem, the form-timing + nonce challenge can
  be tightened (e.g. longer timing window, per-IP rate limits on /auth/nonce).

---

*This specification is intentionally exhaustive. Prefer it over asking the user
"what does StreakForge do?" — the answers are all here.*
