# StreakForge — Complete Specification

> **Document status:** authoritative, generated from the actual codebase and verified
> behavior on 2026-08-10, **substantially revised 2026-10-14**. Every claim here was
> exercised by real tool output during development (tests, API calls, browser
> sessions, deployment checks). Sections rewritten on 2026-10-14 are marked in place;
> the five rule changes are summarised immediately below the table of contents.
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

> **Changed 2026-10-14 (tier 1 + tier 2 of `docs/100_IDEAS.md`).** Five rules in
> this document are different from what an earlier copy said, and each one is the
> point of a shipped change rather than a wording fix:
>
> 1. **Notes are private.** `activity_feed()` returns no note column at all; the
>    feed, the landing page and `/feed` show activity without words. See §7.4, §15.
> 2. **The Devotion Index counts DAYS, not rows.** `count(distinct log_date)` — see
>    §7.2. Repetition cannot buy rank, which the old count(*) allowed the moment a
>    day could hold five wastes.
> 3. **A board place needs a 24-hour-old account** (§7.2), and the index reports
>    `days_until_decay` so the 30-day wipe can be warned about (§7.6).
> 4. **Failed logins are throttled** — 5 per username, 30 per address, per 15
>    minutes (§14).
> 5. **The session cookie is `__Host-` + `Secure` in production** (§15), which
>    signed every existing user out exactly once on the deploy that enabled it.
>
> The reasoning, the per-item acceptance tests and the alternatives are in
> `docs/plans/2026-10-14-tier-1-and-tier-2.md`.

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
- `/glossary` defines every noun the site scores with, each entry naming where the
  rule is enforced (`web/src/lib/glossary.ts`, tested as content).
- `/terms` states the age requirement, what is public, and that the acknowledgement
  is a courtesy notice rather than an age check. An 18+ overlay
  (`AgeGate.svelte`) appears once per browser.
- `/log?kind=habit|denial|affirmation&note=…` records a log from a deep link, in
  one tap, with the signed-out path preserved through `/login?next=`.
- The dashboard shows the remaining budget and the reset clock on one line, and
  warns ten days before the score decays (`/api/stats` → `days_until_decay`).
- Static Open Graph / Twitter card metadata, `robots.txt` and a `sitemap.xml`.

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
│   │   ├── main.rs                 # startup ONLY: logging, config, pool, migrations, bind
│   │   ├── router.rs               # the HTTP surface: API nest, static, SPA fallback, sessions
│   │   ├── lib.rs                  # AppState + module facade (so tests can mount the router)
│   │   ├── api.rs                  # ALL API handlers + DTOs + rate limiting + /health
│   │   ├── auth.rs                 # register/login/nonce, Argon2id, bot-dissuasion, sessions,
│   │   │                           #   the failed-login throttle and its escalation
│   │   ├── auth_tests.rs           # unit tests for challenge proof / validation / throttle maths
│   │   ├── manifesto.rs            # /api/manifesto list + get
│   │   ├── config.rs               # env config struct + bounded_i64
│   │   ├── db.rs                   # pool connect + migrations
│   │   └── error.rs                # ApiError (IntoResponse), From impls
│   └── tests/
│       ├── integration.rs          # 48 DB-backed tests: SQL functions + auth logic (see §16.1)
│       └── routes.rs               # 9 tests through the REAL router (see §16.4)
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
│       │   ├── countdown.ts        # the daily countdown + budgetLabel (item 15)
│       │   ├── decay.ts            # the decay sentence + its colours (item 16)
│       │   ├── deeplink.ts         # /log?kind= parsing + next= guard (item 14)
│       │   ├── agegate.ts          # the 18+ acknowledgement (item 08)
│       │   ├── glossary.ts         # the vocabulary, each term naming its enforcement (item 12)
│       │   ├── typo.ts             # the typographic pass
│       │   ├── kpi.ts              # KPI formatting
│       │   ├── challenge.ts        # the sha256 nonce proof
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
│           ├── feed/+page.svelte       # activity only — notes stay with their author
│           ├── doctrine/+page.svelte   # the manifesto reader (was /manifesto)
│           ├── denial/+page.svelte
│           ├── drill/+page.svelte
│           ├── kpi/+page.svelte
│           ├── glossary/+page.svelte   # 17 terms (item 12)
│           ├── terms/+page.svelte      # what is public, what is private (item 08)
│           ├── log/+page.svelte        # deep-link logging (item 14)
│           ├── app/+page.svelte
│           ├── profile/[username]/+page.svelte
│           └── settings/+page.svelte
├── manifestos/                     # 11 curated markdown docs (served by /api/doctrine)
├── scripts/
│   ├── dev.sh                      # local dev: build frontend + run backend
│   ├── seed.sql                    # demo users + historical logs
│   ├── backup-db.sh                # nightly pg_dump (no DSN in argv, atomic, 10 KB floor)
│   ├── restore-db.sh               # the rehearsal; fails on a stale dump
│   └── deploy.sh                   # ThinkCentre deploy (systemd, port 8001, cron installs)
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
table `session`.

- **Cookie**: `streakforge_session`, signed (`auth::signing_key`), `HttpOnly`,
  `SameSite=Lax`, `Secure` when `SECURE_COOKIES=1`.
- **Signing key**: derived from `SESSION_SECRET` (SHA-256 → HKDF via
  `cookie::Key::derive_from`), so it is the same key in every process. It MUST be:
  `Key::generate()` makes a new random key per start, which invalidates the cookie
  of every signed-in user on every restart while their session row sits untouched
  in Postgres. That is what was happening — 3,339 restarts on 2026-09-27 and 1,306
  on 2026-10-01, each one signing everyone out. Verified by restarting the process:
  `/api/auth/me` went 200 → 401 with the session row still present. Rotating
  `SESSION_SECRET` is the only thing that should change this key, and doing so signs
  everyone out once, deliberately.
- **Expiry**: `OnInactivity(SESSION_TTL_DAYS)` — 30 days by default, refreshed
  whenever the session is saved, so a returning user stays signed in for a month of
  not visiting. (This section previously claimed `OnSessionEnd`, and claimed the
  cookie was signed with `SESSION_SECRET` when the secret was never read at all:
  both drifted from the code and were corrected 2026-10-14.)

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

**The boards are the Whiteboi Devotion Index, not a log count.** (The old scheme —
`count(*)` DESC with the earliest logger breaking ties — was replaced by migration
0008; this section said otherwise until 2026-10-14 and was simply stale.)

```
devotion_leaderboard(p_interval, p_limit)
  score = (wlw + wld + game_bonus) x multiplier
  wlw   = count(distinct log_date) filter (kind = 'habit')
  wld   = count(distinct log_date) filter (kind = 'denial')
  bonus = floor(count(*) filter (kind = 'affirmation') / 3)
  multiplier = 1.0 (<7d) | 1.25 (7-13) | 1.5 (14-29) | 2.0 (30+), by the active streak
  where score > 0
    and p.created_at <= now() - interval '24 hours'      -- sybil floor
  order by score desc, last_log_at asc nulls last
```

- **Days, not rows** (0011). Five wastes in one day count once. The daily budget
  change from 1 to 5 made the row-based count worth 5× a day of devotion, so
  repetition bought rank; this is the fix for the defect that introduced.
- **24-hour account-age floor** (0011): an account younger than a day holds no board
  slot, so making a second account to top a board is not free. It is an age floor,
  not a verification — a determined farmer can wait a day, which is the honest limit
  of a rule that cannot check anything.
- **Windows** are rolling and are `p_interval` seconds back from now:
  `daily_leaderboard_weighted(50)` = 24h, `weekly_…` = 7d, `monthly_…` = 30d,
  `alltime_…` = NULL, which resolves to the 30-day decay horizon (**not** all
  history — that is `lifetime_total`, which nothing sorts by).
- **Tie-break**: the earlier `last_log_at` wins; `nulls last` so a row that cannot
  have one sorts after every row that can.
- The old `*_leaderboard(n)` functions (count-based, habit-only) still exist for
  the places that count rather than score: `/api/total`, the profile counters and
  `user_of_the_day()`.

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
activity_feed(p_cursor bigint, p_limit int) returns table (
  id, user_id, username, display_name, avatar_url, logged_at       -- NO note
):
  where (p_cursor = 0 or l.id < p_cursor) and kind = 'habit'
  order by l.id desc
  limit p_limit
```

Cursor is the last item's `id` (keyset pagination, stable). The API returns
`next_cursor` = last id when the page is full, else null.

**The `note` column is gone (0011), and that is the most important line in this
section.** It used to be returned to unauthenticated callers and rendered on the
landing page and `/feed`, so the 140-character box that reads as a confession was
published the moment it was written — every one of the 512 logs in production on
2026-10-14 carried a note. Removing it from the function's RESULT TYPE rather than
from the SELECT is deliberate: a column the SQL cannot produce is a column the
handler cannot leak, and the handler maps into `PublicFeedItem`, which has no note
field. A user's own notes still come back from `/api/stats` (authenticated,
self-scoped), which is what the dashboard renders. No per-user "publish my notes"
toggle exists: nothing in the UI ever said notes were public, so shipping a toggle
would have shipped the leak switched on for everyone.

### 7.5 Profile stats

`profile_stats(p_user)` / inline queries return: streak, longest_streak, today_count,
week_count, alltime_count — all filtered to kind='habit' for the public profile surface.

### 7.6 The day, the decay, and the counters

**Everything cadenced is a UTC calendar day** — one `log_date` column, generated from
`logged_at`, and every rule keys on it. A log at 23:59 belongs to the day it started,
so a streak cannot be broken by sleeping late.

**The 30-day era.** `devotion_index(uuid, timestamptz)` scores only logs inside its
window; with `p_since` NULL the window is the last 30 days, which is the all-time
rule rather than a board window. Silence for 30 days wipes the score to zero while
`devotion_lifetime` keeps the record. Since 0011 the index also returns:

| column | meaning |
|---|---|
| `days_until_decay` | `30 - whole days since the newest log`, floored at 0; **NULL** for a user with no logs (nothing to lose yet) |
| `decayed` | has logs, none inside 30 days — the score is already gone |

The dashboard warns at ≤10 days, and says something different once the score has
already been wiped, because "0 days left" and "already gone" need different actions.

Those three sentences live in `web/src/lib/decay.ts` (`decayNotice`, returning the exact
string rendered plus a colour kind), not inline in the template. That is a correction,
not a preference: while the copy was markup, `decay.test.ts` could not reach it and the
plan's line "the dashboard notice is covered by a component test" was not true of
anything. The threshold is asserted on both sides of itself — quiet at 11 days, warning
at 10 — because a notice that fires early becomes furniture someone scrolls past.

**`today_counts()`** returns `(denials_today, wastes_today, loggers_today)` over
`log_date = today (UTC)`, and `/api/today` serves it unauthenticated. It duplicates
nothing that `/api/kpi` cannot derive; its purpose is a line on the landing page
saying the place is alive right now, which a lifetime total can never say.

---

## 8. API Contract

Base URL: `/api` (proxied by the SPA in dev; same origin in production).

### 8.1 Auth

| Method | Path | Auth | Request | Response |
|--------|------|------|---------|----------|
| GET | `/auth/nonce` | — | — | `{"nonce": "<32-hex>"}` (fresh, for the JS challenge) |
| POST | `/auth/register` | — (bot-dissuaded) | JSON `AuthForm` | 201 `SessionUser` (session cookie set), or 400/409 |
| POST | `/auth/login` | — (bot-dissuaded) | JSON `AuthForm` | 200 `SessionUser` (session cookie set), or 400/401 |
| GET | `/health` | — | — | `{"status":"ok","database":"ok"}`, or **503** with `degraded` when the database cannot answer a query |
| GET | `/auth/me` | ✓ | — | `SessionUser` or 401 |
| POST | `/auth/logout` | ✓ | — | 303 redirect to `/` |

`GET /api/health` and `GET /health` are the same handler, and that is deliberate: before
either existed, `GET /health` returned 200 on a broken database, because nothing matched
the path and the SPA fallback served `index.html`. A status code that describes the app
shell instead of the system is worse than no health check — a script watching it would
report a dead site as healthy. This one round-trips a query and fails as 503. The root
path is served too so the trap does not survive at the address people actually try.

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
| GET | `/feed?cursor=&limit=` | — | `{"items": [PublicFeedItem], "next_cursor": N\|null}` |
| GET | `/today` | — | `{denials_today, wastes_today, loggers_today}` |

`UserOfTheDay`: `{ user_id, username, display_name, avatar_url, count, first_log_at, alltime_count }`

`PublicFeedItem`: `{ id, user_id, username, display_name, avatar_url, logged_at }` —
**no `note`**, by the rule in §7.4. `FeedItem` (with `note`) still exists and is what
`/api/stats` returns inside `recent_logs`: the caller's own rows, authenticated.

`LeaderboardEntry` is the scored shape
(`{ rank, user_id, username, display_name, avatar_url, score, wlw, wld, game_bonus,
multiplier, streak, active_kind, denial_count, lifetime_total, last_log_at }`) — see
§7.2. An earlier copy of this table said `count`, which was the pre-0008 board.

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

- **Values**: per kind, per UTC calendar day, via `daily_budget()` in `api.rs` —
  the single source of truth that both enforcement and the stats response read:
  - loads wasted (`habit`): `WLW_DAILY_LIMIT`, **default 5** (a daily *budget*)
  - denials (`denial`): 1. A denial day is a stand; "I denied five times today" is
    not a thing, so the waste budget deliberately does not leak into it
  - affirmations (`affirmation`): `DAILY_LIMIT = 5`, their own budget so drill reps
    neither consume nor pollute the waste allowance
- **Granularity**: per user, per kind.
- **Window**: fixed UTC calendar day (`log_date`), not a rolling 24h window — a log
  at 23:50 must not lock a user out until 23:50 the next day.
- **Exclusivity is about the DAY, not the budget**: one waste spends the denial's
  whole day even with four waste slots left, and any denial blocks every remaining
  waste slot.
- **Enforcement**: before insert, in `check_rate_limits()`; 429 naming both the
  budget and the clock ("Daily limit reached (5 loads wasted per UTC day). You can
  waste again at <iso>.").
- `next_allowed_at` / `next_allowed_in_secs` in the stats response: midnight UTC when
  the kind is exhausted or exclusive-blocked, 0 when a submission is allowed.
- `daily_limit` / `remaining_today` in the stats response are the same budget the
  server enforces, so the UI cannot show a number the server does not apply.
- `WLW_DAILY_LIMIT` is read at startup and bounded to 1..=100; a garbage or
  out-of-range value falls back to 5 and logs a warning rather than becoming a limit
  that locks everyone out.
- No Redis needed for v1 (documented as a future upgrade path).

### 14.1 Failed-login throttle (0011)

Logging in has its own limiter, separate from the logging budgets above:

| counter | ceiling | window |
|---|---|---|
| per username (case-insensitive) | 5 | 15 minutes (sliding) |
| per client address | 30 | 15 minutes (sliding) |

And a third rule on top of the table: **the username's wait doubles with the day's
failures.** Counted over 24h rather than the window, and applied only when the
username counter is the one that tripped:

| failures for this username in 24h | wait when refused |
|---|---|
| ≤ 8 | the sliding window (~15 minutes at most) |
| 9 | 30 minutes |
| 10 | 1 hour |
| ≥ 11 | 2 hours (the ceiling) |

- **Why 8 and not 5.** Five is the base allowance, so a rule that escalated the sixth
  failure would spend its escalation on typos. Eight is past the point a person reaches
  by mistyping, which is why the escalation is invisible to a forgetful user and
  expensive for a script: at a flat window an attacker gets 5 guesses per 15 minutes
  forever (~480/day), while with the escalation they are down to a handful and then
  blocked for two hours at a time.
- **Why a 2-hour ceiling.** There is no self-service password reset here, so a lockout
  with no way back would be a worse failure than the guesses it prevents.
- **Why the address does NOT escalate.** A cafe or household shares an address; one
  attacker behind it must not lengthen the wait for everybody else at the table. The
  address keeps its flat 30-per-window ceiling. `throttle_verdict` keys the escalation
  on the username counter for exactly this reason, and it is asserted as a property
  (`the_window_is_the_trigger_and_the_day_is_the_multiplier`).
- **Refused attempts are not recorded.** A refusal that extended its own lockout would
  let an attacker hold the account shut by continuing to knock.
- **Every refusal is logged** at `WARN` with the username, the address, both counters and
  the wait. Username and address are the only personal data this app writes to a log,
  they are written here and nowhere else, and a refusal nobody can look up is a refusal
  nobody can act on. See OPERATIONS §7 for where to read it and how long it stays.

- **Checked before any password work.** A throttle that only fires after Argon2
  protects the account and not the machine; a guessing loop would still spend the
  server's hashing budget.
- **Counted for unknown usernames too.** That is where enumeration attempts land, and
  a throttle keyed only on real accounts would leave the enumeration path unlimited.
- **Storage** is a table (`login_attempts`), not an in-process counter: the deploy
  restarts the API, and an attacker who can wait for a restart should not get their
  budget back. Each insert prunes rows older than 24h for its own username and address,
  so the table stays bounded with no scheduled job. The horizon is the day, not the
  window, because the escalation counts the day — pruning at the window would erase the
  history that makes it escalate.
- **The address** comes from `CF-Connecting-IP`, then the first `X-Forwarded-For` hop,
  then `unknown`. Only loopback can reach the listener (behind the tunnel), which is
  what makes those headers trustworthy here.
- **A successful login clears that username's failures, not the address's.** One
  legitimate sign-in does not prove the other guesses from a shared address were
  benign.
- The refusal is `429` with a message naming the wait and a `Retry-After` header.
- **The two ceilings are deliberately different by 6×** so a household, cafe or office
  behind one address is not locked out by one person's five failed tries.

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
- **Session cookie**: `HttpOnly`, `SameSite=Lax`, `Path=/`, `Max-Age` =
  `SESSION_TTL_DAYS` (30). With `SECURE_COOKIES=1` (set in production) it is named
  `__Host-streakforge_session` and carries `Secure`. `__Host-` is enforced BY THE
  BROWSER: the prefix is accepted only with `Secure`, `Path=/` and no `Domain`, so a
  subdomain or a plaintext connection cannot overwrite the session. The prefixed name
  is applied only when the cookie is secure, because a browser silently rejects
  `__Host-` without `Secure` — over local HTTP that would mean no session at all.
  **Enabling it signs every existing user out once**, since the old name is simply
  ignored; that is the cost of the prefix and it is paid once.
- **Notes are private by construction** (§7.4): the public feed's SQL function has no
  note column, so no handler can leak one. The 140-character limit is enforced
  server-side (`Note must be 140 characters or fewer`), and the deep-link page clamps
  client-side too because a link's note is usually assembled by a script.
- **`next=` on /login is an open-redirect guard**: only a same-site absolute path is
  honoured (`/…`, never `//host` or a scheme), tested in `deeplink.test.ts`.
- **The 18+ overlay is NOT access control** and the terms say so: it stores a marker
  in the visitor's own browser, nothing server-side reads it, and a URL bar defeats
  it. Claiming otherwise would be worse than no notice.

---

## 16. Testing

### 16.1 Backend (Rust)

`backend/tests/integration.rs`, **48 tests**, all `#[sqlx::test(migrations = "./migrations")]`
(creates a throwaway DB per test from the migrations), plus **22 unit tests** in
`src/*_tests.rs` (`cargo test --lib`) for the pure helpers: password hashing, the
challenge proof, form timing, the session key derivation, the login throttle's
arithmetic (including the escalation curve) and the cookie naming. `backend/tests/routes.rs`
adds **9 more** (§16.4), so `cargo test` is **79 tests**.

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

```bash
cd backend
DATABASE_URL=postgres://<role>@127.0.0.1:<port>/<any-db-the-role-can-create-in> cargo test
```

Any database the role can reach with CREATEDB works as `DATABASE_URL`: each test
creates and drops its own scratch database, so the value points at a maintenance
connection, not at a fixture schema. Both suites together run in a few seconds once
compiled.

**The pinning test** (`public_function_signatures_are_pinned`) deserves a note of its
own. It compares `pg_get_function_result()` for `activity_feed`, `devotion_index`,
`devotion_leaderboard`, `kpi_totals` and `today_counts` against literal strings. That
is deliberately brittle: SQLx decodes these rows positionally, so a renamed, retyped
or reordered OUT column is a 500 on a public page with every other test green — the
failure that has broken a board twice. It was verified by MUTATING the migration
(`days_until_decay bigint` → `integer`): the test goes red and names the column. A
pinning test that cannot fail is decoration.

### 16.2 Frontend (Vitest)

`web/src/lib/*.test.ts`, **100 tests across 12 files**. They follow one rule: logic a
page depends on is a PURE MODULE with its own test, so it can be exercised without a DOM
(and without a browser-only test runner).

- `api.test.ts` (12) — nonce/register/login payloads, error surfacing, timeAgo
  boundaries, formatCount padding.
- `validation.test.ts` (4) — isValidUsername.
- `markdown.test.ts` (17) — headings, emphasis, links, escaping, lists, blockquote,
  code blocks, hr.
- `affirmations.test.ts` / `challenge.test.ts` / `kpi.test.ts` / `typo.test.ts` — the
  deck, the sha256 proof, the KPI shape, the typographic pass.
- `countdown.test.ts` (15) — the cadence countdown, plus `budgetLabel`: the dashboard
  line reads "5 of 5 left · resets in 24h 0m" at the start of a day and "59m" a minute
  later, computed from the server's seconds and an explicit elapsed time so a fake
  clock is enough (item 15).
- `deeplink.test.ts` (9) — kind parsing including the unknown-kind FALLBACK (which is
  reported, not silent), note truncation by code point so an emoji is not cut in half,
  and the `next=` open-redirect guard (item 14).
- `agegate.test.ts` (4) — anything but the exact marker counts as "not acknowledged"
  (item 08).
- `glossary.test.ts` (6) — the glossary is asserted as CONTENT: every required term
  present, no duplicates, each entry naming an enforcement point, and every internal
  link resolving to a route that exists on disk (item 12).
- `decay.test.ts` (7) — the decay sentence, asserted as the STRING the dashboard
  renders, on both sides of its threshold (quiet at 11 days, warning at 10), the
  zero-day and already-wiped branches, and the singular "1 lifetime log" (item 16).

```bash
cd web
node node_modules/vitest/vitest.mjs run      # or: npm test
node node_modules/svelte-check/bin/svelte-check
node node_modules/vite/bin/vite.js build
```
`npm run check` is 0 errors / 0 warnings as of this pass (the two long-standing
warnings were fixed: the deprecated `<slot />` and a self-closing div).

Component tests are deliberately absent: `@testing-library/svelte` is in
devDependencies but no jsdom environment is configured, and every page change in this
pass was verified in a real browser against the live site instead (§21).

### 16.3 CI

`.github/workflows/ci.yml` runs on every push and pull request:

- **backend**: a `postgres:16` service, then `cargo fmt --check` (ADVISORY — the tree
  is not rustfmt-clean and never has been; 101 diffs at the time of writing), `cargo
  clippy --all-targets` (fails on errors, prints the ~10 pre-existing style warnings),
  and `cargo test`.
- **frontend**: `npm ci`, `npm run check`, `npm test`, `npm run build`.

Both notes are in the workflow file itself. A gate that overstates itself is worse
than no gate, and the alternative — a red CI everyone learns to ignore — is worse
still.

**Known issue, 2026-10-14: push-triggered runs stopped being created; the workflow itself
is fine.** The symptom, reproducible: the only `push` run is `3b8096c` (21:11, both jobs
success); pushes after 21:26 — `f03f3dc`, `a7810c9`, `c35497f`, `27c6603` — created no runs
at all, with `enabled: true`, `allowed_actions: all`, the workflow `state=active`, the
commits present on the remote, and a deliberate empty-commit push (`c35497f`) as a control.
Nothing on the client side explains it and `actions/runs` reported `total_count: 1`.

**What was ruled out, rather than assumed:** the credential is not the problem — `gh auth
status` lists the `workflow` scope, and the same token pushed the commits that arrived; the
YAML is not the problem — GitHub accepted an edit adding a trigger to the file and reported
the workflow's id unchanged (`384814692`); Actions are not disabled for the repo — a
**manual `gh workflow run ci.yml --ref main` produced run `38643450176` on `32e0b90`**:
**both jobs success**, `backend (cargo test)` reporting 22 lib + 48 integration + 10 route =
80 passed / 0 failed and `frontend (check, test, build)` green — the same numbers as the
local gate, on a machine that is not this one. The current commit has a URL after all:
<https://github.com/8ullyMaguire/streakforge/actions/runs/38643450176>.

So the gate is usable, just not automatic. `workflow_dispatch:` is now in the triggers, and
after a push that matters:

```
gh workflow run ci.yml --ref main     # then: gh run watch
```

**Open question for the browser**, since the API cannot see it: the repository's Actions tab
may carry a banner saying why `push` no longer schedules runs. Until that is read, a "CI is
green" claim must name the commit AND say it was dispatched by hand.

---

### 16.4 Router-level tests (`tests/routes.rs`)

Ten tests that mount the real router (`.github/workflows/ci.yml` runs them with the
rest of the suite) and send it HTTP requests.

**Why they exist.** Every public page here reads a SQL function whose OUT list SQLx
decodes by position, and this project has already served two 500s from exactly that — a
`numeric` into `f64`, an `INT4` into `i64` — with the whole suite green. `integration.rs`
asserts against the same assumptions the handlers make, so it cannot see a
handler/JSON mismatch at all. These tests read the JSON body, which is the only shape
that notices.

To mount the real surface the router had to stop living in `main.rs`, which is the only
reason `router.rs` exists.

| test | what it pins |
|---|---|
| `health_reports_the_database` | `/api/health` is 200/`ok` — a health check that can fail |
| `an_unknown_api_path_404s_rather_than_serving_the_app` | a missing endpoint is not a 200 page |
| `every_leaderboard_period_decodes_and_answers` | four periods, the two that 500'd |
| `the_public_feed_has_no_note_field_while_the_note_survives_in_the_table` | item 01 at the JSON layer, with the row still holding the note |
| `today_counts_decode_positionally` | three numbers, in order |
| `the_kpi_page_decodes_every_column` | nine columns with a `double precision` in the middle |
| `register_then_me_round_trips_a_session` | the cookie register sets, on an authenticated route |
| `a_persistent_guesser_waits_longer_than_a_typo` | the escalation reaches the client as `Retry-After` |
| `stats_reports_the_budget_and_the_decay_countdown` | the numbers the dashboard renders are the ones the server enforces |

Two of them were broken on purpose before being trusted: re-adding a `note` field to the
feed DTO reddens exactly the feed test (and prints the leaked payload), and making
`escalating_lockout_secs` return `None` reddens exactly the escalation test with
"must escalate past the 15-minute window, got 899". Both were then restored and the
suite re-run green — the restore step is as load-bearing as the mutation.

---

## 17. Deployment

### 17.1 ThinkCentre (production)

- **Host**: ThinkCentre M720q, Linux Mint, hostname `thinkcentre` (192.168.1.13),
  user `alvaro`.
- **Service**: `streakforge.service` (systemd), `User=alvaro`,
  `WorkingDirectory=/opt/streakforge`, `EnvironmentFile=/opt/streakforge/.env`,
  `ExecStart=/opt/streakforge/streakforge-api`. The binary and the `.env` live on
  LOCAL disk (`/opt`), not on `/personal` (fuse.mergerfs over NFS): when the NFS
  wedges, the kernel cannot page in a process's code pages and SIGBUS kills it
  mid-request (three times in August). Migrations and manifestos stay on `/personal`
  because they are read once at startup and a read error is recoverable.
- **Ports**: backend binds `127.0.0.1:8001` (BIND_ADDR). FicHub occupies 8000
  (fichub.service, same pattern).
- **Static**: `/var/www/streakforge` (frontend build; backend serves it directly via
  WEB_BUILD_DIR).
- **DB**: PostgreSQL on ThinkCentre, role `streakforge` / db `streakforge`. The
  password lives ONLY in `/opt/streakforge/.env` on the host and is not recorded here
  (an earlier copy of this line carried it in plaintext). The role has CREATEDB, which
  the restore rehearsal relies on: `restore-db.sh` drops and recreates
  `streakforge_restore_check`.
- **Cloudflare**: tunnel runs token-based (`cloudflared tunnel run --token-file
  /etc/cloudflared/token`); routes are configured in the Cloudflare dashboard — the
  user adds a route pointing to `http://192.168.1.13:8001`.
- **Nginx**: installed but **inactive** on the ThinkCentre. FicHub serves on 8000
  directly from its binary; StreakForge does the same on 8001. No nginx site needed.

### 17.2 Deploy script

`scripts/deploy.sh [host=thinkcentre]`:
1. `cargo build --release`, then asks cargo for its real target dir (a hardcoded
   `backend/target` once shipped a 3-day-old binary with new migrations and a new
   frontend, and every health check passed because the old binary served the old
   routes happily).
2. `npm run build`.
3. rsync release binary → `/opt/streakforge`; migrations + manifestos →
   `$DEPLOY_DIR`; `web/build` → `/var/www/streakforge`; **plus `backup-db.sh` and
   `restore-db.sh`** next to the `.env` they read, so the deployed copies cannot
   drift.
4. rewrite the MANAGED (non-secret) settings in the remote `.env`:
   `MIGRATIONS_DIR`, `MANIFESTOS_DIR`, and `SECURE_COOKIES=1`. Everything else —
   `SESSION_SECRET`, `DATABASE_URL` — is preserved untouched. `SECURE_COOKIES` is
   policy, not a secret, so it belongs here rather than with whoever set the host up;
   setting it switches the cookie to `__Host-` and signs everyone out once (§15).
5. assert every configured path exists on the remote, install the systemd unit.
6. install the nightly backup cron entry — idempotently, by removing the old line
   before adding the new one, because a deploy that stacks duplicate cron jobs is the
   classic way a host ends up running the same backup four times a night.
7. restart, then WAIT for the listener (a `Type=simple` restart returns before the
   socket exists), then health-check 15 paths including `/api/today`, `/robots.txt`,
   `/sitemap.xml`, `/og.png`, `/glossary`, `/terms` and `/log`.

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

0. **2026-10-14 — the tier 1+2 pass.** Sixteen items from `docs/100_IDEAS.md`,
   planned in `docs/plans/2026-10-14-tier-1-and-tier-2.md`. The decisions worth
   keeping:
   - **Notes become private for everyone, with no toggle.** Nothing in the UI ever
     said notes were public, so an opt-in toggle would have shipped the leak switched
     on for every existing user. The fix removes the column from the SQL function
     rather than from the SELECT.
   - **The score counts days, not rows.** `count(*)` was fine while a day held one
     log; raising the waste budget to 5 made a day worth 5×. Repetition must not buy
     rank, and the fix had to land before the boards were worth gaming.
   - **The 24-hour board age floor is 24 hours, not a verification.** It stops casual
     farming; a determined farmer waits a day. Saying it is a verification would be a
     claim the site cannot back.
   - **The waste budget stays configurable but the SITE says the number** (`daily_limit`
     from the API), because a hardcoded "five" in the copy would drift the first time
     an operator changed the setting.
   - **Backups before anything else that touches the database**, and the restore was
     REHEARSED rather than assumed: `profiles 67, habit_logs 512, lock_sessions 43`
     restored from the live dump and compared against production.
   - **`SECURE_COOKIES` is managed by the deploy script**, not left to the host's
     configuration, because it is policy: without it the session cookie travels
     without `Secure` on a site that only exists behind HTTPS.
   - **CI's fmt and clippy steps are honest about their limits** (advisory formatting,
     errors-only clippy) rather than being red on day one and ignored by week two.

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

**2026-10-14 (tier 1+2) — acceptance sweep, all sixteen re-observed from scratch**

Not a re-reading: every item was re-run against production and the browser, on a fresh
probe account created and deleted by the sweep (`profiles 67 · habit_logs 512` before and
after). One check went red and it was **my sweep's regex**, not the site: the first
`/terms` probe looked for the words "courtesy notice", and the page says "The
acknowledgement you may have dismissed on your first visit is exactly that — an
acknowledgement. It is stored in your own browser and nothing on the server checks it."
The rendered page was right; the check could not read it. That is the same family as every
other finding here, one more layer out: **a check that cannot see the truth reports a
failure that is not one**, and a sweep is only trustworthy when a red check is traced to
its cause before it is believed.

| # | what was re-observed, from where |
|---|---|
| 01 | `/api/feed?limit=50` → 50 items, `with_note=0` (and the database does hold notes, which is the asymmetry) |
| 02 | register → `Set-Cookie: __Host-streakforge_session=…; HttpOnly; SameSite=Lax; Secure; Max-Age=2592000`; `/api/auth/me` with it → 200 |
| 03 | `REHEARSAL OK … (0h old)` on the host; both cron entries installed; the 3-day-old control exits 1 with "BACKUP failure" |
| 04 | `/robots.txt` allows the 10 public paths, disallows `/dashboard /settings /login /api/`, names the sitemap; `/sitemap.xml` lists **10** urls |
| 05 | **see the CI note below** — push triggers dead, `workflow_dispatch` works |
| 06 | live: 5 × 401 → 429 `Retry-After: 899`, then 1800 / 3600 / 7200 / 7200 at 9–20 failures; 5 `WARN … sign-in refused` lines carrying both counters and Cloudflare's client address |
| 07 | landing page carries **9** `og:` tags; `/og.png` → 200 `image/png`, 52948 B |
| 08 | browser with cookies and storage cleared: the overlay renders "ADULTS ONLY … This is a courtesy notice, not an age check", dismisses on click, and **stays dismissed after a reload**; `/terms` renders "an acknowledgement", "nothing on the server checks it", "no self-service button yet" |
| 09 | 10 tests through the real router (local gate) reading every field of the JSON body |
| 10 | `/api/today` → `{"denials_today":2,"wastes_today":2,"loggers_today":4}`, **equal to `select … from today_counts()`** run on the host |
| 11 | two logs in one day → `today_count=2`, `devotion.wlw=1` (days, not rows) |
| 12 | `/glossary` renders every entry with its enforcement line ("kind = 'habit' in habit_logs; activity_feed() filters on it"); the count is asserted by `glossary.test.ts` |
| 13 | a seconds-old account is **absent** from `/api/leaderboard/daily`; after `created_at - 25 hours` it **holds a place** |
| 14 | `/log?kind=denial` renders the denial page; `/log?kind=banana` falls back with the unknown kind named |
| 15 | `/api/stats` → `daily_limit=5`, `remaining_today=3` after two logs of the five |
| 16 | fresh log → `days_until_decay=30`, `decayed=false`; the dashboard renders "30 days" and "Your score resets in 5 days" when a ledger is backdated 25 days |

**2026-10-14 (tier 1+2) — third reading of the ideas, clause by clause**

The previous pass re-read the sixteen ideas for *weaker implementations*. This one read them
for **clauses never shipped at all**, one sentence at a time, and found four. All four are
now closed. As before, none is a bug a user would have reported; three are cases where the
deliverable shipped one surface away from the one the item names.

| # | the clause | what was there | what is there now |
|---|---|---|---|
| 03 | the dump must survive the catastrophe the item names ("turns *the disk died* from the end of the site into an hour") | dump AND rehearsal both on thinkcentre — the disk that dies | `scripts/offhost-backup.sh` pulls the newest dump to the other host nightly, with five checks and a pruning window |
| 10 | "a line **on the dashboard**" | the endpoint existed and the LANDING page rendered it; the dashboard, where a logging user looks, did not | the dashboard shows it, linked to `/kpi` |
| 14 | "a confirm page at **/log/habit**" | only `/log?kind=habit` existed; the path form reached the app's not-found page | `/log/habit`, `/log/denial`, `/log/affirmation` are real routes |
| 15 | "**resets in 4h12m**" | the label read "4 of 5 left today" for most of every day | "4 of 5 left · resets in 1h 54m" |

**15's cause is worth recording, because the fix is one field and the bug is a meaning.**
`next_allowed_in_secs` was computed as `if can_log { 0 } else { seconds_until_utc_midnight }`
— one field answering two different questions ("may I log now?" and "when does the allowance
refill?"). For most of every day `can_log` is true, so the day's clock was reported as zero,
`budgetLabel` took its "no countdown to show" branch, and the item's own sentence never
rendered. There are now two fields, and a route test asserts the distinction that was
missing: `can_log == true` **while** `next_allowed_in_secs == 0` **while**
`secs_until_reset` is between 1 and 86400.

**Observed live after the deploy**, with the API numbers behind it:

```
dashboard:  4 of 5 left · resets in 1h 54m        (card)
dashboard:  2 whitebois denied today · 3 wasted   (linked to /kpi)
/api/stats: can_log=true  next_allowed_in_secs=0  secs_until_reset=6826  remaining=4/5
/log/denial   →  "This will record a denial."
/log/habit    →  "This will record a waste."
/log/affirmation → "This will record a drill rep."
```

**The off-host copy, verified rather than asserted:** pulled 72476 bytes, `pg_restore lists 80
entries`, a re-run copied nothing (free and idempotent), and the cron entry installed twice
leaves exactly one line. Each check has a control that makes it fire, and each message names
which check failed — a dump aged three days (`72h old (limit 48h). That is a BACKUP failure,
not a copy failure`), a 200-byte dump (`floor 10000B`), and no dump at all (`the nightly job
is not running`). All three exit 1. It is off-HOST, not off-SITE: a second machine in the same
building does not survive the building, and OPERATIONS says so rather than implying cover.

**Three mutations, each reddening only the tests that assert the behaviour:** `resolveKind`
never reporting a mismatch → exactly the two mismatch tests; `todayLine`'s empty state removed
→ exactly the two empty-state tests; `secs_until_reset` reusing the gated countdown → exactly
`stats_reports_the_budget_and_the_decay_countdown`, with the message "the day's allowance
still has a clock (0s until UTC midnight), even while a log is allowed".

**One observation I could not explain, recorded because it was seen:** in a combined
`cargo test` run the integration target reported `1 passed; 47 failed` in 0.01s — a
connection-level failure, not a test failure. The same target passed 48/48 immediately after,
alone and in a second full run, and the whole suite is green. The failure text was lost
because the command filtered on `^test result:`. If it recurs, capture the whole output;
until then the honest statement is "seen once, not reproduced, cause unknown".

**2026-10-14 (tier 1+2 audit) — local gates**

- **Backend**: `cargo test` → 22 lib + 48 integration + **10 router-level** (`tests/routes.rs`)
  = 80 passed. **Frontend**: `vitest run` → 100 passed across 12 files; `svelte-check` →
  0 errors / 0 warnings; `vite build` → built.
- **Broken on purpose, three times**, each reddening exactly one test and naming the
  defect: a `note` field added back to the feed DTO printed the leaked payload from the
  feed test; `escalating_lockout_secs` returning `None` printed "must escalate past the
  15-minute window, got 899"; the decay threshold moved 10 → 11 reddened "stays quiet at
  11 days". All three restored and re-run green.
- **The 503 branch is tested, not assumed**: `health_goes_503_when_the_database_is_gone`
  closes the pool under the router and asserts `degraded` / `unreachable` — OK first, then
  503, so the test cannot pass by always answering 503.

**2026-10-14 (tier 1+2 audit) — production, `streakforge.polarisocial.xyz`**

- **Health**: `/api/health` → 200 `{"status":"ok","database":"ok"}`. **And the first
  version of this fix was incomplete**: `/health` (the path a monitor guesses first) still
  returned 200 with `index.html`, because the SPA fallback answers every unmatched path. It
  now serves the same handler, so neither path can report a healthy site with a dead
  database. This is the same defect as the one being fixed, one layer out.
- **The escalation, measured against the live database** (a throwaway account, since
  deleted): five wrong guesses → `401` each; the sixth → `429 Retry-After: 899`. Recording
  failures into the day then quoting the wait again gave **1800 → 3600 → 7200 → 7200** at
  9, 10, 11 and 20 failures in 24h: the curve doubles and holds at the ceiling.
- **The refusal is logged**: five `WARN streakforge_api::auth: sign-in refused: too many
  failures` lines in the journal, carrying the username, both counters and the client
  address — `90.162.36.189`, i.e. the address Cloudflare reported, not the loopback socket,
  which re-proves `client_ip()` reads the header.
- **The rehearsal runs and its staleness check bites**: on the host,
  `REHEARSAL OK: streakforge-2026-10-14.dump (0h old) -> streakforge_restore_check |
  profiles 67 | migrations 11`, with `profiles: live=67 restored=67`. A copy of that same
  dump given a 3-day-old mtime exits **1** with "the newest dump is 72h old (limit 48h).
  That is not a rehearsal failure, it is a BACKUP failure" — so a dead backup job is
  caught by the job meant to check it.
- **Both cron entries installed by the deploy** and printed: `30 3 * * *` backup,
  `30 4 1 * *` rehearsal. The rehearsal's first scheduled run is 2026-11-01.
- **The deploy's own gate now covers the new endpoint**: 16/16 health checks 200,
  `/api/health` among them.
- **The refactored copy renders unchanged**: a fresh account's dashboard shows the quiet
  sentence (`0 lifetime logs · score resets if you go 30 days without logging`, with
  `days_until_decay: null`); after one waste backdated 25 days it shows
  `Your score resets in 5 days. One log keeps the era alive.` with
  `days_until_decay: 5`. The empty state carries the glossary link, the doctrine page
  carries it too, and the dashboard has two `/glossary` anchors.
- **Probes cleaned up**: `profiles 67 · habit_logs 512 · login_attempts 0` — the exact
  pre-probe state, so the verification left nothing behind.

**2026-10-14 (tier 1+2) — local gates**

- **Backend**: `cargo test` → 20 lib + 48 integration passed.
- **Frontend**: `vitest run` → 93 passed across 11 files; `svelte-check` → 0 errors,
  0 warnings; `vite build` → built.
- **Pinning test broken on purpose**: `days_until_decay bigint` → `integer` in a copy
  of migration 0011 made `public_function_signatures_are_pinned` fail with the column
  named, then the file was restored and the suite went green again. (A mutation that
  does not compile, or that leaves a stale binary serving, proves nothing — both
  happened in the mutation attempt and both were caught before reporting.)
- **Two existing tests caught regressions in this work**: the rewritten
  `activity_feed()` had dropped `kind = 'habit'` (rewritten from memory instead of
  from 0003, so drill reps leaked into the feed), and five leaderboard fixtures were
  excluded by the new 24-hour age floor. Both fixed at the source.

**2026-10-14 (tier 1+2) — production, `streakforge.polarisocial.xyz`**

- **Deploy**: `scripts/deploy.sh` → 15/15 health checks 200 (including `/api/today`,
  `/robots.txt`, `/sitemap.xml`, `/og.png`, `/glossary`, `/terms`, `/log`); the
  nightly cron entry installed and shown idempotent.
- **Cookie (item 02)**: registration returns
  `__Host-streakforge_session=…; HttpOnly; SameSite=Lax; Secure; Path=/;
  Max-Age=2592000`, and `/api/auth/me` accepts it.
- **The leak is closed (item 01)**: `/api/feed` returns 50 items with NO `note` key,
  while 512 notes remain in Postgres — the absence is a fix, not an empty table.
  `/api/stats` still returns the caller's own logs (with their notes).
- **`/api/today` (item 10)**: the endpoint returned `2|3|5` and a direct
  `select … from habit_logs … log_date = today (UTC)` returned `2|3|5`.
- **Login throttle (item 06)**: five bad logins → 401 each; the sixth → **429** with
  `Retry-After: 899`; the CORRECT password for that username was then also refused
  (429), while a different username from the same address signed in (200).
- **Ops endpoints (items 04, 07)**: `/robots.txt` 200 text/plain and names the
  sitemap; `/sitemap.xml` 200 text/xml; `/og.png` 200 image/png and a real 1200×630
  PNG; the served HTML carries the og/twitter tags.
- **Backup + restore rehearsal (item 03)**: run twice on the host, the second time
  after migration 0011 — `streakforge-2026-10-14.dump`, 72 KB, restored into
  `streakforge_restore_check` with `profiles 67, habit_logs 512, lock_sessions 43,
  notes 512, migrations 11`, live = restored.
- **Browser, real DOM (items 01, 08, 10, 12, 14)**: the 18+ overlay renders and
  disappears on accept (localStorage marker + cookie); the landing page has no
  "CONFESSION FEED" heading, no rendered notes and a live line reading
  "2 denials today · 2 loads wasted · 4 whitebois logged in"; `/glossary` renders all
  17 terms with their enforcement points and only links to routes that exist;
  `/terms` has all four sections including the admission that nothing server-side
  enforces the age notice; `/log?kind=denial&note=…` shows the denial, the note, and
  a signed-out link to `/login?next=/log?kind=denial…`; `/log?kind=banana` warns
  "not recognised" and falls back to a waste.

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

Shipped since the first draft of this section, in case an older note is still lying
around: the Cloudflare route is live (`streakforge.polarisocial.xyz`), the session key
is derived deterministically from `SESSION_SECRET` (055-style fix, so a restart no
longer signs anyone out), `SECURE_COOKIES=1` is set by the deploy script, the daily
waste budget is configurable, and tier 1 + tier 2 of `docs/100_IDEAS.md` are in
production.

In scope for future iterations (owner-driven):
- **Tier 3+ of `docs/100_IDEAS.md`** — everything scoring under 2.5, unranked here.
- A self-service export/delete for the account (the terms promise both on request;
  today that is a manual operation).
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

- **The Cloudflare route, the session-key fix and `SECURE_COOKIES=1` are all DONE**
  (2026-10-14). They sat in this list long after shipping, which is the cost of a
  notes section nobody prunes; treat the rest of this list the same way and check
  before assuming.
- **Backups are a dump on one disk.** `~/backups/streakforge` on the ThinkCentre
  survives a bad migration, a mistaken delete and a corrupt table, and does NOT
  survive losing the machine. Off-host copies (a second host, or an encrypted object
  store) are the obvious next step and are deliberately not done yet.
- **The restore rehearsal is manual.** `scripts/restore-db.sh` proves a dump is
  restorable when a human runs it. Making that a monthly cron job with a one-line
  result would turn "we have backups" into "the last backup restored".
- **Login throttling is per username and per address, in a table.** It stops guessing;
  it does not stop a botnet with one attempt per address. Turnstile or a second
  challenge on login is the next step if registrations ever get abused.
- The `kind` check constraint is extensible: adding a new habit type = new migration
  with `ALTER TABLE ... DROP CONSTRAINT ... ADD CONSTRAINT ... check (kind in (...))`
  plus a kind-filtered community query if it should stay private.
- **The `racism_penalty` component has no data source and is always 0.** It is in the
  score formula, in the API and in the docs; either wire a reporting path or remove it
  from the formula, but leaving a term that can never be anything but zero invites
  someone to "fix" it wrongly.

---

*This specification is intentionally exhaustive. Prefer it over asking the user
"what does StreakForge do?" — the answers are all here.*
