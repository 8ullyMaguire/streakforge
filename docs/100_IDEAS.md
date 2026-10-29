# StreakForge — 100 Ideas, ranked by impact ÷ effort

Written 2026-10-14, against the tree at `1fd713d` (migrations 0001–0010, 20 routes, 12 frontend routes). Companion file to `docs/FEATURE_IDEAS.md`, which is a shorter essay-shaped list; this one is 100 discrete, scored items.

**How to read the score.** Every idea carries an `impact` (1–10) and an `effort` (1–10), and the score is `impact ÷ effort` — value for the work, not importance. That is the same encoding Concord's `features` table uses (`impact`, `effort_score`, and a generated `impact_ratio`), so the same 100 items filed there sort identically; this is deliberate, not a coincidence. The top of the list is therefore dominated by small expensive-if-ignored things (a privacy leak, no backups) and the bottom by the large strategic bets — the keyholder, the most important product idea on this list, scores 1.25 because it is a week of work. Nothing reaches 10.0: that would need an impact-10 job costing one unit of effort, and nothing here is both. The privacy leak at the top is the closest thing, because the fix is one line.

**What is deliberately not here.** `Freeze days`, `longest streak as the headline` and `backdated entry` are already filed — in a second, local Concord instance (features 19-21 there, complaints 17/18), *not* in the public project this list was filed to, which held two features and two complaints when this was written. Re-proposing them would be noise either way. Nothing here proposes ads, a paywall on the counter or the board, email-required signup, or anything that rewards wasting over denying; see `FEATURE_IDEAS.md` for why.

**Status 2026-10-14: ranks 01–16 are SHIPPED** — every item scoring ≥ 2.5, in one
pass (`91fc795`, plan and per-item acceptance tests in
`docs/plans/2026-10-14-tier-1-and-tier-2.md`, evidence in SPEC §21). Ranked
highest-value first was the right order: the list's top item was a live privacy leak
that every one of the 512 logs in production was exposed to, and it was one line of
SQL. What the 16 items turned into is recorded in SPEC §3.6, §7.2, §7.4, §7.6, §14.1,
§15, §16 and §17; the honest summary is that four of them (01, 11, 13, 16) changed
what the site measures, and the other twelve changed what it tells you.

**A third pass on 2026-10-14 (late) read the ideas' own sentences for clauses never built at
all** and found four: 03's dump did not survive the disk dying (it now copies off-host), 10's
line was only on the landing page and not the dashboard the item names, 14's `/log/habit` path
form did not exist, and 15's reset clock never rendered (one field was answering two different
questions). All four are closed. Three of the four had shipped "one surface away" from the
item's own words.

**A second pass on 2026-10-14 re-read all 16 against what actually landed** and found
five places where the implementation was weaker than the idea: 03's rehearsal was not on
a timer, 06's backoff was flat and unlogged, 09's test compared signatures instead of
decoding, 12's glossary was not linked from the empty states, and `GET /health` was a 200
from the SPA fallback rather than a health check. All five are closed; the detail is in
SPEC §14.1, §16.4 and §21. The lesson is worth keeping: **an idea's text is the
requirement, and a plan that ships a reduced version of it has to say so out loud.**

Nothing below rank 16 was started. Ranks 17–20 are the next candidates by score, and
the note at the bottom of this file still stands: the strategic bets (the keyholder,
the lock seal) score low here **because effort is the denominator** — that is the
ranking working, not the ideas being weak.

| # | Score | Idea | Impact | Effort | Size | Area |
|---|-------|------|--------|--------|------|------|
| ~~01~~ | ~~9.0~~ | ~~Stop publishing log notes to the anonymous feed~~ **SHIPPED** | 9 | 1 | S | Privacy |
| 01 | **9.0** | Stop publishing log notes to the anonymous feed | 9 | 1 | S | Privacy |
| 02 | **6.0** | Set Secure and the __Host- prefix on the session cookie **— SHIPPED** | 6 | 1 | S | Privacy |
| 03 | **5.0** | Back up the production database **— SHIPPED** (nightly, rehearsed monthly on a timer, and the newest dump copied to a second machine nightly) | 10 | 2 | S | Ops |
| 04 | **5.0** | robots.txt and a sitemap.xml **— SHIPPED** | 5 | 1 | S | SEO |
| 05 | **3.5** | CI: run the gate on every push **— SHIPPED** (fmt advisory, clippy errors-only) | 7 | 2 | S | Ops |
| 06 | **3.5** | Throttle failed logins per account and per IP **— SHIPPED** (doubling past 8/day, 2h cap, every refusal logged) | 7 | 2 | S | Auth |
| 07 | **3.0** | Add Open Graph and Twitter meta tags **— SHIPPED** (static, honestly so) | 6 | 2 | S | Growth |
| 08 | **3.0** | An 18+ interstitial and a clear terms line **— SHIPPED** (an acknowledgement, and it says so) | 6 | 2 | S | Compliance |
| 09 | **3.0** | Pin activity_feed's column list with a test **— SHIPPED** (five signatures pinned, plus ten router-level tests that decode the JSON) | 6 | 2 | S | Ops |
| 10 | **3.0** | Who denied today: a live same-day counter **— SHIPPED** (`/api/today`, on the landing page AND on the dashboard, which is where the item put it) | 6 | 2 | S | Social |
| 11 | **2.67** | Score by day, not by row, now that a day can hold five logs **— SHIPPED** | 8 | 3 | M | Scoring |
| 12 | **2.5** | A glossary for the language **— SHIPPED** (17 terms, each naming its enforcement, linked from the empty states) | 5 | 2 | S | Content |
| 13 | **2.5** | Anti-sybil: account age before a board appearance **— SHIPPED** (24h) | 5 | 2 | S | Scoring |
| 14 | **2.5** | Log from a deep link or one-key URL **— SHIPPED** (`/log?kind=` and the path form `/log/habit`, `/log/denial`, `/log/affirmation` that the item names) | 5 | 2 | S | Streaks |
| 15 | **2.5** | Show the day's remaining budget and the clock together **— SHIPPED** (`4 of 5 left · resets in 5h 3m`, from a field that is the day's clock and not the log lock's) | 5 | 2 | S | Streaks |
| 16 | **2.5** | Warn before the 30-day decay erases a score **— SHIPPED** (`days_until_decay`) | 5 | 2 | S | Scoring |
| 17 | **2.33** | A consistency board: percentage of days logged | 7 | 3 | M | Scoring |
| 18 | **2.33** | A running lock-streak board | 7 | 3 | M | Lock |
| 19 | **2.0** | A public ledger: dates and kinds, nothing else | 6 | 3 | M | Trust |
| 20 | **2.0** | A verifiable lock seal | 6 | 3 | M | Trust |
| 21 | **2.0** | Notify at the halfway point and an hour before a lock ends | 6 | 3 | M | Lock |
| 22 | **2.0** | Streak-at-risk state before the day closes | 6 | 3 | M | Retention |
| 23 | **2.0** | Uptime monitoring that is not the deploy script | 6 | 3 | M | Ops |
| 24 | **2.0** | A home-screen install prompt at the right moment | 4 | 2 | S | Growth |
| 25 | **2.0** | A key rotation procedure, documented | 4 | 2 | S | Ops |
| 26 | **2.0** | A public changelog with RSS | 4 | 2 | S | Growth |
| 27 | **2.0** | An abuse policy and a takedown path | 4 | 2 | S | Compliance |
| 28 | **2.0** | Empty states that teach the rule | 4 | 2 | S | Craft |
| 29 | **2.0** | Notification quiet hours | 4 | 2 | S | Retention |
| 30 | **1.75** | Doctrine as real pages with real titles | 7 | 4 | M | Content |
| 31 | **1.75** | Let a user fix a log within 15 minutes | 7 | 4 | M | Streaks |
| 32 | **1.67** | A per-kind streak alongside the combined one | 5 | 3 | M | Streaks |
| 33 | **1.67** | Age-gate the deepest surfaces, not just the door | 5 | 3 | M | Compliance |
| 34 | **1.67** | An embeddable streak badge | 5 | 3 | M | Growth |
| 35 | **1.67** | Emergency unlock, recorded and rate-limited | 5 | 3 | M | Lock |
| 36 | **1.67** | Lock milestones with badges | 5 | 3 | M | Lock |
| 37 | **1.67** | Public "why is this ranked here" for a board row | 5 | 3 | M | Scoring |
| 38 | **1.67** | Separate boards per kind | 5 | 3 | M | Scoring |
| 39 | **1.67** | Signed proof of a streak or a lock | 5 | 3 | M | Trust |
| 40 | **1.6** | Web push: one notification a day | 8 | 5 | M | Retention |
| 41 | **1.5** | A weekly recap card | 6 | 4 | M | Retention |
| 42 | **1.5** | Extend the lock as a penalty | 6 | 4 | M | Lock |
| 43 | **1.5** | Reduce the first-session to one decision | 6 | 4 | M | Growth |
| 44 | **1.5** | Self-serve account deletion with a 7-day undo | 6 | 4 | M | Auth |
| 45 | **1.5** | Share a milestone to Bluesky | 6 | 4 | M | Growth |
| 46 | **1.5** | Structured reasons on a waste, not just free text | 6 | 4 | M | Streaks |
| 47 | **1.5** | Tests for the frontend rules, not just the helpers | 6 | 4 | M | Craft |
| 48 | **1.5** | A rate limit on complaint and feedback submissions | 3 | 2 | S | Ops |
| 49 | **1.4** | Recovery codes instead of an email reset | 7 | 5 | M | Auth |
| 50 | **1.33** | A CLI for logging | 4 | 3 | M | Platform |
| 51 | **1.33** | A comparison page for a user against the community | 4 | 3 | M | Scoring |
| 52 | **1.33** | A migration drift test | 4 | 3 | M | Ops |
| 53 | **1.33** | A podium with the reigning holder | 4 | 3 | M | Scoring |
| 54 | **1.33** | A status page for the community | 4 | 3 | M | Ops |
| 55 | **1.33** | An audit log of account-level changes | 4 | 3 | M | Ops |
| 56 | **1.33** | An import/export of a user's whole history as a file | 4 | 3 | M | Platform |
| 57 | **1.33** | Per-page titles and canonical URLs | 4 | 3 | M | SEO |
| 58 | **1.33** | Push notification preferences per kind | 4 | 3 | M | Retention |
| 59 | **1.33** | Search across users | 4 | 3 | M | Social |
| 60 | **1.33** | Show lock history on the calendar | 4 | 3 | M | Lock |
| 61 | **1.33** | Sitemap generation from the data | 4 | 3 | M | SEO |
| 62 | **1.33** | Track doctrine progress | 4 | 3 | M | Content |
| 63 | **1.25** | A keyholder: someone else holds the key | 10 | 8 | L | Lock |
| 64 | **1.25** | A public read-only API with tokens | 5 | 4 | M | Platform |
| 65 | **1.25** | A rank history sparkline on the profile | 5 | 4 | M | Scoring |
| 66 | **1.25** | Accessibility pass over the interactive surfaces | 5 | 4 | M | Craft |
| 67 | **1.25** | Error tracking for 500s | 5 | 4 | M | Ops |
| 68 | **1.25** | Feature flags so a change can be turned off without a deploy | 5 | 4 | M | Ops |
| 69 | **1.25** | Kudos on feed items | 5 | 4 | M | Social |
| 70 | **1.25** | Structured request logging with a request id | 5 | 4 | M | Ops |
| 71 | **1.25** | Zero-downtime restart | 5 | 4 | M | Ops |
| 72 | **1.2** | A session list with revoke-one and sign-out-everywhere | 6 | 5 | M | Auth |
| 73 | **1.2** | An offline-capable shell with a service worker | 6 | 5 | M | Retention |
| 74 | **1.2** | Badges and achievements | 6 | 5 | M | Scoring |
| 75 | **1.2** | Behaviour insight panel from the user's own logs | 6 | 5 | M | Streaks |
| 76 | **1.2** | Follow users and a following-only feed | 6 | 5 | M | Social |
| 77 | **1.2** | Lock schedules: recurring windows, not manual locks | 6 | 5 | M | Lock |
| 78 | **1.17** | Acceptance pairs: two users, one streak | 7 | 6 | L | Social |
| 79 | **1.17** | User-chosen day boundary instead of enforced UTC | 7 | 6 | L | Streaks |
| 80 | **1.14** | Keyholder approves or denies an unlock | 8 | 7 | L | Lock |
| 81 | **1.0** | Community challenges with a shared target | 6 | 6 | L | Social |
| 82 | **1.0** | A staging environment | 5 | 5 | M | Ops |
| 83 | **1.0** | Rivals: a head-to-head page for two users | 5 | 5 | M | Social |
| 84 | **1.0** | A one-click report on any public item | 4 | 4 | M | Safety |
| 85 | **1.0** | A profanity and abuse filter on the public feed's remaining free text | 4 | 4 | M | Safety |
| 86 | **1.0** | A proper long-term activity chart | 4 | 4 | M | Craft |
| 87 | **1.0** | Personal invite links with attribution | 4 | 4 | M | Growth |
| 88 | **1.0** | A lightweight on-site bug report | 3 | 3 | M | Ops |
| 89 | **1.0** | An RSS/Atom feed of the public activity | 3 | 3 | M | SEO |
| 90 | **1.0** | Keyboard shortcuts on the log surfaces | 3 | 3 | M | Craft |
| 91 | **0.86** | Teams or guilds with an aggregate board | 6 | 7 | L | Social |
| 92 | **0.83** | Mentions and a lightweight notification inbox | 5 | 6 | L | Social |
| 93 | **0.8** | Blocklists for the social surfaces | 4 | 5 | M | Safety |
| 94 | **0.8** | Email, optional, for exactly two events | 4 | 5 | M | Retention |
| 95 | **0.8** | Webhooks for milestones | 4 | 5 | M | Platform |
| 96 | **0.75** | A dark/light theme switch | 3 | 4 | M | Craft |
| 97 | **0.67** | TOTP two-factor, opt-in | 4 | 6 | L | Auth |
| 98 | **0.6** | A monthly community newsletter | 3 | 5 | M | Growth |
| 99 | **0.57** | A passwordless magic-link is a bad fit; a passkey is a good one | 4 | 7 | L | Auth |
| 100 | **0.5** | A second deploy target so this host is not a single point of failure | 4 | 8 | L | Ops |


## Tier 1 — do these first. Best value on the list; most are a day or less.

### 01. Stop publishing log notes to the anonymous feed
*Score 9.0 — impact 9 ÷ effort 1 (S) — Privacy*

activity_feed() returns `note` to unauthenticated callers, so the 140-char confession box is world-readable the moment it is logged. Drop `note` from the anonymous feed (keep it in the authenticated /api/stats response) or add a per-user public_feed toggle defaulting to off. One-line change with a real disclosure behind it.

### 02. Set Secure and the __Host- prefix on the session cookie
*Score 6.0 — impact 6 ÷ effort 1 (S) — Privacy*

SECURE_COOKIES=0 in /opt/streakforge/.env while the public site is HTTPS-only, so the cookie may travel in plaintext. SECURE_COOKIES=1 plus a __Host-streakforge_session name is config plus one constant, no logic.

### 03. Back up the production database
*Score 5.0 — impact 10 ÷ effort 2 (S) — Ops*

No pg_dump exists in the thinkcentre crontab and no streakforge dump is anywhere under /var/backups. A nightly pg_dump plus a monthly restore rehearsal into a scratch database turns "the disk died" from the end of the site into an hour. Cheapest catastrophe insurance on the list; the restore half is what makes it real.

### 04. robots.txt and a sitemap.xml
*Score 5.0 — impact 5 ÷ effort 1 (S) — SEO*

web/static holds only icon.svg and manifest.webmanifest, so crawlers get no guidance and no URL list. Two static files: allow everything public, disallow nothing private because nothing private is fetchable for a crawler anyway, and list the static routes.


## Tier 2 — strong value, real work.

### 05. CI: run the gate on every push
*Score 3.5 — impact 7 ÷ effort 2 (S) — Ops*

Nothing runs cargo test, svelte-check, vitest or a build except a human remembering to. A GitHub Actions workflow on push to main makes "56 tests green" a claim with a URL. The suite already exists; only the wiring is missing.

### 06. Throttle failed logins per account and per IP
*Score 3.5 — impact 7 ÷ effort 2 (S) — Auth*

`login` verifies a password and nothing else - no failure counter, no backoff, no lockout. A password is only as strong as the guesses per second you allow. Count failures by username and by IP, back off exponentially after 5, and log the event.

### 07. Add Open Graph and Twitter meta tags
*Score 3.0 — impact 6 ÷ effort 2 (S) — Growth*

app.html has zero `og:` tags, so every share of streakforge.polarisocial.xyz is a bare link. Static tags plus a per-page title give every paste into a chat or a Bluesky post a card, which is free reach the site currently throws away.

### 08. An 18+ interstitial and a clear terms line
*Score 3.0 — impact 6 ÷ effort 2 (S) — Compliance*

The site's content is unmistakably adult and nothing states an age requirement. A first-visit confirmation (cookie, no account needed) plus a footer link to plain terms is a small build and the difference between a defensible site and an indefensible one.

### 09. Pin activity_feed's column list with a test
*Score 3.0 — impact 6 ÷ effort 2 (S) — Ops*

The feed calls a SQL function whose OUT parameters sqlx decodes positionally, exactly the shape that 500'd both leaderboards twice (a numeric into f64, an INT4 into i64). One test that calls activity_feed and decodes every column turns the next function-signature change from an outage into a red test.

### 10. Who denied today: a live same-day counter
*Score 3.0 — impact 6 ÷ effort 2 (S) — Social*

kpi_totals already counts today's denials. A line on the dashboard - "17 whitebois denied today, 4 wasted" - gives a solitary activity an audience and costs one public read of a number the KPI page already publishes.

### 11. Score by day, not by row, now that a day can hold five logs
*Score 2.67 — impact 8 ÷ effort 3 (M) — Scoring*

devotion_index counts count(*) filter (where kind='habit'), so the day the waste budget went 1 -> 5 the same behaviour started earning 5x. Score the DAY (count(distinct log_date)) and keep the row count as a separate "how much" figure. Fix it before anyone games it, not after.

### 12. A glossary for the language
*Score 2.5 — impact 5 ÷ effort 2 (S) — Content*

WLW, WLD, devotion, drill, doctrine and lock all appear in the UI with no definition anywhere a newcomer can read. A static page with the vocabulary, linked from the empty states and the doctrine index, is the single cheapest onboarding fix.

### 13. Anti-sybil: account age before a board appearance
*Score 2.5 — impact 5 ÷ effort 2 (S) — Scoring*

A new account can top a board within minutes, and there is no cost to making one. Requiring 24 hours of age (or one verified day) to appear on public boards does not stop a determined farmer and does make casual farming pointless.

### 14. Log from a deep link or one-key URL
*Score 2.5 — impact 5 ÷ effort 2 (S) — Streaks*

POST /api/logs needs JSON and a session, so a bookmark or a phone shortcut cannot log in one tap. A confirm page at /log/habit that shows what will be recorded and takes one button makes the habit reachable without opening the app.

### 15. Show the day's remaining budget and the clock together
*Score 2.5 — impact 5 ÷ effort 2 (S) — Streaks*

daily_limit and remaining_today now come from the server, so the dashboard can say "3 of 5 left, resets in 4h12m" instead of only counting down after the budget is gone. Same data, earlier warning: a rule rather than a trap.

### 16. Warn before the 30-day decay erases a score
*Score 2.5 — impact 5 ÷ effort 2 (S) — Scoring*

A user who stops for 31 days returns to a zero score and no explanation (the decay is documented in a migration and nowhere a user looks). A notice on the dashboard at day 20, day 28 and on return turns a silent punishment into a rule people know.


## Tier 3 — worth doing once the above are solid.

### 17. A consistency board: percentage of days logged
*Score 2.33 — impact 7 ÷ effort 3 (M) — Scoring*

Ranking by raw count rewards volume and quietly punishes everyone who logs once a day forever. "Days logged in the last 30" as a parallel board measures the thing the product is named after.

### 18. A running lock-streak board
*Score 2.33 — impact 7 ÷ effort 3 (M) — Lock*

The lock is the most distinctive mechanic on the site and has no leaderboard. lock_sessions already computes current and longest durations, so a public "locked longest right now" board is a query and a page, and it shows the thing this audience actually compares.

### 19. A public ledger: dates and kinds, nothing else
*Score 2.0 — impact 6 ÷ effort 3 (M) — Trust*

Streaks are unverifiable claims with a public board on top. A paginated per-user list of log dates and kinds (no notes, ever) makes a streak checkable by a stranger and costs one read-only route over data the site already holds.

### 20. A verifiable lock seal
*Score 2.0 — impact 6 ÷ effort 3 (M) — Trust*

A screenshot of "locked 14 days" is worth nothing. A short code rendered on the lock page and reproducible from lock_start + user_id + a server secret makes a claim checkable by anyone with the algorithm, and it is what makes a share card credible.

### 21. Notify at the halfway point and an hour before a lock ends
*Score 2.0 — impact 6 ÷ effort 3 (M) — Lock*

A lock ending is currently a non-event, which wastes the one moment the user is most engaged. Two notifications off the existing countdown, opt-in, using the same push channel as the daily reminder.

### 22. Streak-at-risk state before the day closes
*Score 2.0 — impact 6 ÷ effort 3 (M) — Retention*

Nothing warns a user their streak is about to break. Derive "at risk" from user_streak_kind plus the clock (>= 20h since the last log, or no log and under 6h left) and surface it on the dashboard and later as a push. Cheapest retention lever on the list.

### 23. Uptime monitoring that is not the deploy script
*Score 2.0 — impact 6 ÷ effort 3 (M) — Ops*

The only health check that exists runs during a deploy. An external probe (a cron on another machine, or any uptime service) hitting /api/total and /api/kpi every minute with an alert turns "the service died at some point today" into a page.

### 24. A home-screen install prompt at the right moment
*Score 2.0 — impact 4 ÷ effort 2 (S) — Growth*

The manifest and the icons ship and the install instructions are a page nobody opens. Intercept beforeinstallprompt and offer it after a second successful log, when the user has a reason to keep the thing.

### 25. A key rotation procedure, documented
*Score 2.0 — impact 4 ÷ effort 2 (S) — Ops*

SESSION_SECRET is now load-bearing (it derives the cookie key) and rotating it signs everyone out. Writing the procedure down - generate, update both hosts, deploy, expect one mass sign-out - is a paragraph that prevents a panicked guess during an incident.

### 26. A public changelog with RSS
*Score 2.0 — impact 4 ÷ effort 2 (S) — Growth*

The site changes (the waste budget changed tonight) and users find out by noticing. A CHANGELOG.md rendered as a page with a feed turns every change into a reason to return and costs almost nothing.

### 27. An abuse policy and a takedown path
*Score 2.0 — impact 4 ÷ effort 2 (S) — Compliance*

There is a rules page for members and no statement of what the operator will remove or why. Two paragraphs and an address is the difference between an enforceable policy and an argument in a comment thread.

### 28. Empty states that teach the rule
*Score 2.0 — impact 4 ÷ effort 2 (S) — Craft*

A new account sees zeros and a disabled button with no explanation of why a waste and a denial cannot share a day. One sentence per empty state, written from the rule rather than the schema, removes the first-session confusion that currently looks like a bug.

### 29. Notification quiet hours
*Score 2.0 — impact 4 ÷ effort 2 (S) — Retention*

A daily reminder that fires at 3am is worse than none. Derive quiet hours from the user's day offset (once it exists) and suppress anything outside waking hours.

### 30. Doctrine as real pages with real titles
*Score 1.75 — impact 7 ÷ effort 4 (M) — Content*

The ten manifestos are the best content the site has and they are a JSON blob behind /api/doctrine inside a client-rendered shell, so nothing is indexable. Server-rendered pages (or prerendered HTML with per-page meta) with a next-read chain is the one passive acquisition channel available.

### 31. Let a user fix a log within 15 minutes
*Score 1.75 — impact 7 ÷ effort 4 (M) — Streaks*

A mis-tapped kind or a typo is permanent: habit_logs has no update or delete endpoint at all. A 15-minute window to edit or delete your own row, with the change recorded in an audit column, removes a whole class of support requests without making history editable.

### 32. A per-kind streak alongside the combined one
*Score 1.67 — impact 5 ÷ effort 3 (M) — Streaks*

user_streak_kind already exists and each page uses it privately; the dashboard shows one number. Showing waste / denial / drill streaks side by side makes the exclusivity rule legible instead of surprising.

### 33. Age-gate the deepest surfaces, not just the door
*Score 1.67 — impact 5 ÷ effort 3 (M) — Compliance*

The lock mechanic is the part that reads as sexual and it is reachable in two clicks from a link. Keeping the interstitial idea but gating the lock and drill surfaces behind an explicit acknowledgement makes the site's posture legible, which matters the first time a payment processor or a host asks.

### 34. An embeddable streak badge
*Score 1.67 — impact 5 ÷ effort 3 (M) — Growth*

An iframe or SVG a user can put on a blog or a link-in-bio page makes every external profile a billboard. Only safe once the privacy default is settled, and the SVG half is a template over data the site already serves.

### 35. Emergency unlock, recorded and rate-limited
*Score 1.67 — impact 5 ÷ effort 3 (M) — Lock*

Unlock takes a free-text reason and has no limit, so a lock is only as binding as the user's mood at 2am. Name that honestly: one emergency unlock per 30 days, reason required, shown on the profile.

### 36. Lock milestones with badges
*Score 1.67 — impact 5 ÷ effort 3 (M) — Lock*

7 / 30 / 90 / 365 days are the natural thresholds and none are marked. Deriving them is a query, and a badge is a visual record that survives the score reset the decay rule performs.

### 37. Public "why is this ranked here" for a board row
*Score 1.67 — impact 5 ÷ effort 3 (M) — Scoring*

The devotion multiplier is documented in a migration and invisible in the UI. A small panel per row showing the components (WLW, WLD, bonus, multiplier, decay window) makes the ranking arguable instead of opaque - the same principle Concord applies to its own standings.

### 38. Separate boards per kind
*Score 1.67 — impact 5 ÷ effort 3 (M) — Scoring*

One blended board makes a waste look like a cheap denial and hides the two populations. Boards for denials, wastes and drill reps - each with its own weights and its own top three - let a user compete in the mode they actually practise.

### 39. Signed proof of a streak or a lock
*Score 1.67 — impact 5 ÷ effort 3 (M) — Trust*

The public ledger makes a claim checkable against the server, which does not help someone who wants to prove it elsewhere. An HMAC over (user_id, kind, date range) with a published key id makes an offline-verifiable badge, and it is what an embed would carry.

### 40. Web push: one notification a day
*Score 1.6 — impact 8 ÷ effort 5 (M) — Retention*

The manifest exists and no service worker does, so nothing can notify and nothing is installable. A service worker plus one opt-in push at the user's own cutoff time is the difference between a habit product and a diary, and the daily cadence is already shaped for it.

### 41. A weekly recap card
*Score 1.5 — impact 6 ÷ effort 4 (M) — Retention*

A Sunday summary (logs, streak held, days denied, a rank delta) is the one message that can be sent without nagging, and it doubles as a share card. Derivable from data already stored.

### 42. Extend the lock as a penalty
*Score 1.5 — impact 6 ÷ effort 4 (M) — Lock*

Today an early unlock is free: the streak resets and nothing else happens. "An early unlock adds 24h to the next lock" gives the mechanic teeth, which is what the audience is there for.

### 43. Reduce the first-session to one decision
*Score 1.5 — impact 6 ÷ effort 4 (M) — Growth*

Registration drops the user on a dashboard with seven numbers, a heatmap and a disabled button. An onboarding that asks one thing (what are you here for: deny, drill, or lock), sets the relevant page as home and logs the first entry inline converts curiosity into a record, which is the whole conversion.

### 44. Self-serve account deletion with a 7-day undo
*Score 1.5 — impact 6 ÷ effort 4 (M) — Auth*

profiles is only ever inserted and updated; leaving means asking someone. A deleted_at soft delete that hides the profile and feed entries immediately, with a hard purge after a week, is what a privacy-minded audience expects.

### 45. Share a milestone to Bluesky
*Score 1.5 — impact 6 ÷ effort 4 (M) — Growth*

The audience is already there and bnwo-feed.polarisocial.xyz already exists as a custom feed, so the overlap is reachable. A button on a milestone (30-day denial, new longest lock) that posts a card with the lock seal behind it turns a personal record into distribution.

### 46. Structured reasons on a waste, not just free text
*Score 1.5 — impact 6 ÷ effort 4 (M) — Streaks*

The confession box is free text so nothing can be counted. A small tag set (bored / alone / angry / tired / scrolled into it) gives the user a private insight panel - "3 of your 8 wastes this month were after midnight" - which is the only item here that might reduce the number.

### 47. Tests for the frontend rules, not just the helpers
*Score 1.5 — impact 6 ÷ effort 4 (M) — Craft*

vitest covers countdown, typo, markdown and the API client, and no component test covers the log button's disabled logic - the exact surface the waste budget just changed. The repo already has @testing-library/svelte installed and unused; a test that a full budget disables the button with the right message would have caught a regression here.

### 48. A rate limit on complaint and feedback submissions
*Score 1.5 — impact 3 ÷ effort 2 (S) — Ops*

There is no user-facing feedback path at all, and the moment one exists it will be spammed. A per-account hourly cap on any write endpoint beyond logging is cheap insurance against the day the site is noticed by a bored stranger.

### 49. Recovery codes instead of an email reset
*Score 1.4 — impact 7 ÷ effort 5 (M) — Auth*

There is no password reset at all: a forgotten password is a lost account, and there is no email by design. Ten single-use codes shown once at registration, hashed at rest and invalidated on use, keep the no-email promise and make recovery possible.

### 50. A CLI for logging
*Score 1.33 — impact 4 ÷ effort 3 (M) — Platform*

curl with a cookie works; nothing makes it pleasant. A tiny streakforge-cli (log, stats, board) published as a binary gives the tool-happy slice of the audience a way to make the site part of a shell, and it exercises the API the way a real client would.

### 51. A comparison page for a user against the community
*Score 1.33 — impact 4 ÷ effort 3 (M) — Scoring*

Percentiles ("stronger than 62% of active whitebois") are more motivating than a rank number and more robust to the population growing. One aggregate query against the same windowed data the boards use.

### 52. A migration drift test
*Score 1.33 — impact 4 ÷ effort 3 (M) — Ops*

Migrations are applied on startup and nothing asserts the resulting schema matches what the code expects. A test that applies every migration to a scratch database and compares the columns the handlers select against information_schema catches the class of failure that 0009 and 0010 both risked.

### 53. A podium with the reigning holder
*Score 1.33 — impact 4 ÷ effort 3 (M) — Scoring*

Top 50 tables read as a database dump. A three-slot podium with the current holder highlighted, plus "held it for 3 days", gives the board a face, and user_of_the_day already computes the winner.

### 54. A status page for the community
*Score 1.33 — impact 4 ÷ effort 3 (M) — Ops*

When the site is down there is nowhere to look and nothing to link. A static page generated by the uptime probe ("up since, last incident") answers the question before it becomes a message in someone's DMs.

### 55. An audit log of account-level changes
*Score 1.33 — impact 4 ÷ effort 3 (M) — Ops*

Nothing records a username change, a profile edit or a session revocation. A small append-only table on the account surface makes "someone changed my handle" answerable, and it is the same shape as the log audit the streak-edit idea needs.

### 56. An import/export of a user's whole history as a file
*Score 1.33 — impact 4 ÷ effort 3 (M) — Platform*

Export alone is a compliance gesture; export plus import makes the site's data portable between instances, which is the promise federated-adjacent audiences care about. Same route, one more direction.

### 57. Per-page titles and canonical URLs
*Score 1.33 — impact 4 ÷ effort 3 (M) — SEO*

The SPA shell sets one title for every route, so a shared link to a profile is indistinguishable from the home page in a browser history or a search result. SvelteKit head tags per route plus a canonical link is a small, purely additive change.

### 58. Push notification preferences per kind
*Score 1.33 — impact 4 ÷ effort 3 (M) — Retention*

Once notifications exist, the difference between "reminds me" and "harasses me" is granularity. One settings row per notification kind, off by default except the daily reminder, is the whole feature.

### 59. Search across users
*Score 1.33 — impact 4 ÷ effort 3 (M) — Social*

The board lists 50 and the feed shows 20; finding a username means typing the URL. A `GET /api/search?q=` over username and display_name, with trigram indexes, makes profiles discoverable and is the substrate for mentions later.

### 60. Show lock history on the calendar
*Score 1.33 — impact 4 ÷ effort 3 (M) — Lock*

lock_sessions records every session and nothing renders them. A tinted band over the existing 30-day StreakCalendar shows the lock shape beside the logs, which is the artifact people would screenshot.

### 61. Sitemap generation from the data
*Score 1.33 — impact 4 ÷ effort 3 (M) — SEO*

Once doctrine pages are prerendered, a generated sitemap listing them plus public profiles gives the crawler the whole surface rather than the eight static routes.

### 62. Track doctrine progress
*Score 1.33 — impact 4 ÷ effort 3 (M) — Content*

The manifestos are numbered and there is no record of which ones a user has read. A tick plus a progress line ("4 of 10 read") gives a reason to finish and a reason to return to the index.

### 63. A keyholder: someone else holds the key
*Score 1.25 — impact 10 ÷ effort 8 (L) — Lock*

The biggest missing mechanic versus the real-life kink and the only idea here that creates a two-sided network: whitebois bring keyholders, keyholders bring whitebois. v1 is keyholder_id on the profile plus an unlock_requests table and a page the keyholder can act on, with a stated escape hatch of N days without an answer.

### 64. A public read-only API with tokens
*Score 1.25 — impact 5 ÷ effort 4 (M) — Platform*

The stats, the boards and the ledger are exactly the data a community dashboard or a bot would consume. A token table already exists in the schema (api_tokens, unused by any route), so the missing half is one endpoint and one middleware.

### 65. A rank history sparkline on the profile
*Score 1.25 — impact 5 ÷ effort 4 (M) — Scoring*

Every board is a snapshot, so nothing shows movement. Storing a daily rank per user (one small table, written by a nightly job or derived on read) makes "up 4 this week" possible, and movement is the whole reason a leaderboard is fun.

### 66. Accessibility pass over the interactive surfaces
*Score 1.25 — impact 5 ÷ effort 4 (M) — Craft*

The app has aria-labels in places, no focus management between route changes, and a red-on-black palette that fails contrast in several labels. A pass with a keyboard and a contrast checker over the eight real pages is unglamorous and it is the difference between a site some people can use and one they cannot.

### 67. Error tracking for 500s
*Score 1.25 — impact 5 ÷ effort 4 (M) — Ops*

A 500 currently lands in journald and nowhere else, and nothing tells anyone. Even a webhook to a chat on 5xx with the route, the request id and the error string is enough to stop discovering outages from users.

### 68. Feature flags so a change can be turned off without a deploy
*Score 1.25 — impact 5 ÷ effort 4 (M) — Ops*

The waste budget went from 1 to 5 tonight as an env var, which is the right shape; the same pattern for behaviour changes (new rules, new surfaces) means a bad decision is a restart rather than a rollback. An env-read flags struct in config next to the two limits that already exist.

### 69. Kudos on feed items
*Score 1.25 — impact 5 ÷ effort 4 (M) — Social*

The feed is read-only, so recognition has no outlet short of a public comment (which needs moderation). A single "respect" reaction per user per item, no thread, no text, gives the good half of a comment system with almost none of the moderation.

### 70. Structured request logging with a request id
*Score 1.25 — impact 5 ÷ effort 4 (M) — Ops*

TraceLayer logs the shape of every request and nothing carries an identifier a user could quote. A request id echoed in the response and in the error body turns "it broke yesterday" into one grep, and it costs a middleware and a header.

### 71. Zero-downtime restart
*Score 1.25 — impact 5 ÷ effort 4 (M) — Ops*

systemctl restart drops in-flight requests; with sessions now stable across restarts, a socket-activated or hand-over restart is the last piece of making a deploy invisible.

### 72. A session list with revoke-one and sign-out-everywhere
*Score 1.2 — impact 6 ÷ effort 5 (M) — Auth*

Sessions live in tower_sessions.session but the user can neither see nor end them except here. A settings panel (device, first seen, last seen, revoke) plus a sign-out-everywhere that bumps a per-user generation makes a stolen cookie recoverable.

### 73. An offline-capable shell with a service worker
*Score 1.2 — impact 6 ÷ effort 5 (M) — Retention*

adapter-static means the whole app is already static files; a service worker makes it load instantly and survive a bad connection. It is also a hard prerequisite for push, so it is the first half of the notification work rather than a separate idea.

### 74. Badges and achievements
*Score 1.2 — impact 6 ÷ effort 5 (M) — Scoring*

The score is a single number with no texture, and the decay rule can take it to zero. Non-decaying badges (first denial, 7-day denial streak, 100 affirmations, a 30-day lock) are the record that survives, and they are also what a profile has to show.

### 75. Behaviour insight panel from the user's own logs
*Score 1.2 — impact 6 ÷ effort 5 (M) — Streaks*

The site collects timestamps, kinds and notes and shows none of the patterns. Day-of-week and hour histograms, streak length distribution, longest gap - a private page of facts about yourself is the feature a tracking product owes its users.

### 76. Follow users and a following-only feed
*Score 1.2 — impact 6 ÷ effort 5 (M) — Social*

The feed is a single firehose of every log on the site. Following turns it into a personal list, which is both a reason to come back and the substrate any future notification needs.

### 77. Lock schedules: recurring windows, not manual locks
*Score 1.2 — impact 6 ÷ effort 5 (M) — Lock*

"Locked every weekend until further notice" is how this is practised. A lock_schedules row (weekday set, start hour, duration) that materialises into lock_sessions keeps the sessions table the single record of what happened and makes the habit structural.


## Tier 4 — strategic bets and speculative items. High impact, high cost; the score is deliberately low.

### 78. Acceptance pairs: two users, one streak
*Score 1.17 — impact 7 ÷ effort 6 (L) — Social*

Both members must log within the window or the pair's streak breaks; friend pressure outperforms app pressure by a wide margin. Watch the asymmetric failure: decide whether the pair breaks, the survivor is exempted, or nothing happens, and make it a setting.

### 79. User-chosen day boundary instead of enforced UTC
*Score 1.17 — impact 7 ÷ effort 6 (L) — Streaks*

Every rule keys off log_date = (logged_at at time zone 'utc')::date, so a user in California loses today at 5pm. A per-profile day_offset_minutes that log_date derives from stops the site being wrong about the calendar for most of its audience.

### 80. Keyholder approves or denies an unlock
*Score 1.14 — impact 8 ÷ effort 7 (L) — Lock*

The half that makes the keyholder a relationship rather than a label: the locked user asks, the keyholder decides, the decision is recorded. Reuses lock_sessions and the unlock endpoint; the new part is the request state machine plus notification.

### 81. Community challenges with a shared target
*Score 1.0 — impact 6 ÷ effort 6 (L) — Social*

"10,000 denials as a community this month" with a live progress bar converts a solitary practice into a collective one, and it gives the KPI dashboard a reason to exist beyond curiosity.

### 82. A staging environment
*Score 1.0 — impact 5 ÷ effort 5 (M) — Ops*

Every deploy goes straight to the only production instance, and the deploy script's own health check is the only gate. A second systemd unit on the same host, a copy of the database and a smoke suite makes a risky change testable, and the deploy script already knows how to install a unit.

### 83. Rivals: a head-to-head page for two users
*Score 1.0 — impact 5 ÷ effort 5 (M) — Social*

The boards are anonymous walls; the site's most competitive behaviour happens between two people who know each other. A rival link that pins a comparison (streaks, last 30 days, mutual days) is small once the profile route exists.

### 84. A one-click report on any public item
*Score 1.0 — impact 4 ÷ effort 4 (M) — Safety*

There is no moderation surface: an abusive profile display name stays up until someone edits it by hand in SQL. A report flag with a small queue (and a maintainer page) is the prerequisite for every social idea in this list, since each one adds a surface that needs one.

### 85. A profanity and abuse filter on the public feed's remaining free text
*Score 1.0 — impact 4 ÷ effort 4 (M) — Safety*

If notes stay public anywhere, they need a filter or a report button. A small word list plus a report link that emails nothing and logs everything is the minimum honest version, and the real decision is whether the text should be public at all.

### 86. A proper long-term activity chart
*Score 1.0 — impact 4 ÷ effort 4 (M) — Craft*

The KPI trend is 30 stacked bars and the profile has a heatmap; neither answers "am I getting stronger or weaker over months". A 12-month line per kind on the profile is a query and a small chart, and it is what people screenshot.

### 87. Personal invite links with attribution
*Score 1.0 — impact 4 ÷ effort 4 (M) — Growth*

The only way in is finding the site. A per-user invite link that records who brought whom gives the growth loop a number and lets a member be recognised for it, and it is the prerequisite for any reward-bound referral scheme.

### 88. A lightweight on-site bug report
*Score 1.0 — impact 3 ÷ effort 3 (M) — Ops*

The operator currently learns about bugs from a message, if at all. A footer link that captures the route, the user agent and the request id (see request ids) into a table turns feedback into something triaged rather than remembered.

### 89. An RSS/Atom feed of the public activity
*Score 1.0 — impact 3 ÷ effort 3 (M) — SEO*

The doctrine has no feed and the feed page has no format a reader can subscribe to. The Concord instance in this stack already emits feed.xml for exactly this reason; the same two handlers here make the site reachable from a feed reader.

### 90. Keyboard shortcuts on the log surfaces
*Score 1.0 — impact 3 ÷ effort 3 (M) — Craft*

The whole product is one button pressed once a day; making it reachable with a key (space to log, 1/2/3 to pick a kind) is small and it is the kind of detail this audience notices.

### 91. Teams or guilds with an aggregate board
*Score 0.86 — impact 6 ÷ effort 7 (L) — Social*

Aggregation changes who recruits whom: a team needs its weakest member to log, which is a retention mechanic aimed at exactly the user who churns. Schema is small (a teams table and a members join) and the board is one more weighted aggregate.

### 92. Mentions and a lightweight notification inbox
*Score 0.83 — impact 5 ÷ effort 6 (L) — Social*

Public notes plus profiles make "@username nice streak" natural and currently impossible. Mentions need a parse, an inbox and a moderation story, which is why it sits behind the report button.

### 93. Blocklists for the social surfaces
*Score 0.8 — impact 4 ÷ effort 5 (M) — Safety*

Once there are followers, kudos and pairs, blocking is the user's only tool that does not require an admin. A blocks table plus a filter on every read path using it; cheap now, painful to retrofit after a harassment incident.

### 94. Email, optional, for exactly two events
*Score 0.8 — impact 4 ÷ effort 5 (M) — Retention*

The no-email decision is right for signup and wrong for "someone asked to be your keyholder". A single provider behind an opt-in, used only for keyholder requests and a recovery-code reminder, gets the benefit without turning the site into a newsletter.

### 95. Webhooks for milestones
*Score 0.8 — impact 4 ÷ effort 5 (M) — Platform*

A lock ending, a streak breaking, a personal best: the events other people would build on. A signed POST to a user-supplied URL makes the data portable and gives the community the integrations before the site builds them.

### 96. A dark/light theme switch
*Score 0.75 — impact 3 ÷ effort 4 (M) — Craft*

The dark red-and-black palette is the brand and should stay the default, but a light mode removes a real accessibility barrier for daylight use. One CSS custom-property scope, since the styling is already variable-driven.

### 97. TOTP two-factor, opt-in
*Score 0.67 — impact 4 ÷ effort 6 (L) — Auth*

For the subset who want it, a TOTP secret plus backup codes is a standard shape with no email dependency. Worth doing after sessions and rate limiting are settled, not before.

### 98. A monthly community newsletter
*Score 0.6 — impact 3 ÷ effort 5 (M) — Growth*

The audience is small and engaged, and there is no channel that reaches them off-site. Optional email (which this list argues for elsewhere) plus a monthly post (the biggest streaks, the week's lock record, the changelog) is a re-engagement surface the site currently lacks entirely.

### 99. A passwordless magic-link is a bad fit; a passkey is a good one
*Score 0.57 — impact 4 ÷ effort 7 (L) — Auth*

No email means no magic link, but WebAuthn needs no email at all: a passkey per device with the password as a fallback removes the password from the common path and the phishing story with it. High effort, and it is the honest answer to "how do I log in without email".

### 100. A second deploy target so this host is not a single point of failure
*Score 0.5 — impact 4 ÷ effort 8 (L) — Ops*

The dev box builds and the ThinkCentre serves; if either is unavailable nothing ships and nothing runs. Publishing the release binary as an artifact (a GitHub release, a container image) makes the deploy reproducible from anywhere, which is also what makes onboarding another operator possible.


---

## 2026-10-29 (second pass) — what the shipped features imply next

**Shipped since this section was written:** #110 (the yesterday catch-up window), #111
(export my history) and the erasure path beside it, plus #128's first half (the decode contract
of `activity_feed` is pinned) — see `STATUS.md` 2026-10-29 (third pass) and `/privacy`.

Scored the same way as everything else: **impact ÷ effort**, so 1.0 is the best possible ratio and
0.33 is a real idea that should wait. This batch is deliberately downstream of what landed today —
cage checks, trust levels, time to cum, public-by-default confessions — because the gaps that matter
now are the ones the new surfaces exposed (a metric with no ranking, a setting with no surface, a
mechanic with no clock).

### 101. Cage state on the dashboard
*Score 1.0 — impact 4 ÷ effort 4 (M) — Cage*

The check is the only input the cage metric needs, and it lives on a page nobody visits twice. A 'checked today / check due in Xh' strip on /dashboard is one query and removes the friction that makes verified hours starve.

### 102. Time to cum on every log surface, not just the landing page
*Score 1.0 — impact 4 ÷ effort 4 (M) — Metrics*

The field exists on the landing flow; the dashboard, /denial and /log still write rows with no duration, so the aggregate is fed by one path. Same input, same PATCH, four surfaces.

### 103. A verified-cage-hours board
*Score 1.0 — impact 3 ÷ effort 3 (S) — Cage*

cage_check_hours() and total_verified_cage_hours() are computed and published but ranked nowhere. One board makes the metric competitive, which is what makes members attest each other.

### 104. Flag and attest where the content is
*Score 0.75 — impact 3 ÷ effort 4 (M) — Moderation*

Moderation currently lives on /moderation, so nobody meets a bad log where they meet it. A flag control on feed items and confirm/dispute on the cage queue put the quorum at the point of contact.

### 105. The numbers block on every profile
*Score 0.75 — impact 3 ÷ effort 4 (M) — Identity*

A profile shows the score and the streak. Verified cage hours, trust level and a reported average (opt-in) make it the share surface — no OG image needed for the site to be worth screenshotting.

### 106. A WLD board of its own
*Score 0.75 — impact 3 ÷ effort 4 (M) — Boards*

kind_streaks already separates waste from denial, but the leaderboard collapses them. Denial is the 10× event and the site's actual differentiator, and it is currently invisible as a ranking.

### 107. Say what a flag needs to become a verdict
*Score 0.75 — impact 3 ÷ effort 4 (M) — Moderation*

'Three flags hide it' is in the doctrine; the queue item says only 'N flags'. Showing 'needs 1 more voice from a trusted member' turns a mysterious verdict into a task someone can finish.

### 108. Quorum abuse resistance
*Score 0.75 — impact 3 ÷ effort 4 (M) — Moderation*

A flag and an attestation are one INSERT each and the counting trusts the reporter's level. Weighting a voice, capping flags per member per day and never counting a guest-born flag towards a quorum is the difference between moderation and a griefing tool.

### 109. A pause that does not lie
*Score 0.67 — impact 4 ÷ effort 6 (L) — Retention*

A member who stops for a week loses the multiplier and, in 30 days, the score — so they leave rather than return. An explicit pause (visible on the board, decaying at a documented rate) keeps the history honest and the member reachable.

### 110. A catch-up window for yesterday only
*Score 0.6 — impact 3 ÷ effort 5 (M) — Retention*

People log the day they remember, which is often after midnight. Allowing exactly one backdated log for yesterday, once per day, recovers real activity without turning the streak into fiction.

### 111. Export my history
*Score 0.6 — impact 3 ÷ effort 5 (M) — Data*

No email, no data portability: the rows are the member's and there is no way to take them. A CSV/JSON download of one's own logs, locks and checks is small, and it is the honest answer to 'what happens to my data'.

### 112. Nudge a guest at the second log
*Score 0.6 — impact 3 ÷ effort 5 (M) — Growth*

Guest rows fold into the account already, so the fold is built and the conversion is not. A one-line prompt after log #2 ('keep this — create an account, your history moves with you') uses machinery that exists.

### 113. A referral link with attribution
*Score 0.6 — impact 3 ÷ effort 5 (M) — Growth*

The site has no way for a member to bring another, and the landing page is the only entry. A signed link plus a 'brought by' column on profiles is the smallest honest growth loop this site can have.

### 114. Web Push for the three clocks that matter
*Score 0.5 — impact 4 ÷ effort 8 (L) — Retention*

PWA is installed already, so a service worker plus a stored subscription reaches the member when the lock is stale, the check is due or the multiplier is at risk. Needs a push service, not email — the site's no-email constraint is not violated.

### 115. Cage hours as a second index on the board
*Score 0.5 — impact 3 ÷ effort 6 (L) — Boards*

Verified hours are the most expensive number on the site to fake and the least ranked. A second column on the leaderboard (score | verified hours) rewards the behaviour the cage exists for without rewriting the score.

### 116. A read-only keyholder link
*Score 0.43 — impact 3 ÷ effort 7 (L) — Identity*

One signed, revocable token that shows a member's lock, check history and streak to someone else without an account. It is the feature this genre's users ask for first, and it needs no new identity model.

### 117. A per-person time-to-cum trend
*Score 0.4 — impact 2 ÷ effort 5 (M) — Metrics*

The site publishes one average per member and nothing over time. A 30-day sparkline turns a curiosity into something a member can watch change, and the data is already there.

### 118. Doctrine as pages and a feed
*Score 0.4 — impact 2 ÷ effort 5 (M) — Growth*

The doctrine is served through an API and rendered client-side only. Static pages plus an RSS feed make it indexable and subscribable — the cheapest traffic surface the site has.

### 119. An OG share card per profile and per milestone
*Score 0.38 — impact 3 ÷ effort 8 (L) — Growth*

og.png is static. Rendering the score, streak and verified hours into a per-profile image makes every milestone shareable without a screenshot, which is how a small site gets a graph.

### 120. Sessions you can see and end
*Score 0.33 — impact 2 ÷ effort 6 (L) — Ops*

There is one session cookie and no list. 'Signed in on these devices, end this one' is standard hygiene for a site holding this kind of history, and it is a table plus two endpoints.


---

## 2026-10-29 (third pass) — what the shipped features still owe

Same scoring as everything else: **impact ÷ effort**. This batch is the debt the first two passes
created and the things a stranger would notice before the owner does: a metric published without
consent, a safety gap where the content is edge practice, and four numbers about the product that
nobody can currently report.

### 121. A safety and aftercare page, before any more features
*Score 1.0 — impact 5 ÷ effort 3 (S) — Safety*

The site tracks locking, denial and edges with no page about what to do when a lock hurts, when to stop, or how to come off a long denial. It is content, not code — and it is the one omission that turns a kink tracker into a liability. Doctrine is the right home: a safety section, an emergency-release protocol, and links that are honest about being external.

### 122. Time to cum is opt-in on the board, not published by default
*Score 1.0 — impact 4 ÷ effort 3 (S) — Privacy*

Right now one PATCH publishes a member's average to the world after five samples; nothing asked. The duration is the most personal number on the site. `profiles.ttc_public` defaulting to false, with the board built only from opted-in members and the aggregate still counting everyone anonymously, keeps the metric and returns the consent.

### 123. Numbers about the product, from the database
*Score 1.0 — impact 4 ÷ effort 3 (S) — Product*

Nobody can answer 'is this growing' without counting rows by hand. A `/admin` page (TL4) with weekly active humans, new accounts, guest→account conversion, median logs per active member per week, and a 12-week retention curve — all computed from tables that already exist — is what makes the next prioritisation decision evidence instead of taste.

### 124. Keep the retention promise: purge tombstones, audit rows and dead sessions
*Score 1.0 — impact 3 ÷ effort 3 (S) — Privacy*

The terms describe a bounded history and nothing implements the bound. Folded guest tombstones, audit rows past their window and sessions older than their TTL accumulate forever. One nightly job, one documented window per table, and a count published on the status page.

### 125. Publish the restore rehearsal, not just the backup
*Score 1.0 — impact 3 ÷ effort 3 (S) — Ops*

`restore-db.sh` already runs monthly on thinkcentre; its output goes to a log nobody reads. Surfacing last run's timestamp, restored row counts and the delta against production on a public status page converts an unverified backup into a tested one.

### 126. A pageview number that exists before anyone talks about RPM
*Score 1.0 — impact 3 ÷ effort 3 (S) — Growth*

nginx's stock format has no `$host`, so this estate cannot say how many pages any site serves. A second JSON `log_format` with `$host`, a written pageview definition (GET, 200, not `/api`, not an asset, not a bot) and a daily snapshot is the denominator every later monetisation or growth claim needs.

### 127. Measure which routes are actually used, then delete the rest
*Score 1.0 — impact 3 ÷ effort 3 (S) — Craft*

The site has ~20 routes and three active members; several are almost certainly dead weight kept alive by nothing but the cost of writing them. A pageview-per-route table for a month, then the honest deletion of what nobody opens — with the doctrine, glossary and terms kept regardless of traffic.

### 128. Pin the decode contract of every SQL function, types included
*Score 1.0 — impact 3 ÷ effort 2 (S) — Testing*

A `bigint` in Rust against an `int4` column is a runtime 500 that no compiler and no unit test sees — it happened today, in `cage.rs`, and only the route test caught it. Every function the API decodes positionally (`activity_feed`, `cage_check_hours`, `time_to_cum_stats`, `trust_level`, `flag_quorum`) needs its result types asserted, not just its existence.

### 129. Cage hours can never exceed the lock they came from
*Score 0.75 — impact 3 ÷ effort 4 (M) — Data*

A verified check claims elapsed time, and nothing compares the sum against the lock that started it. A nightly contradiction report (claimed > lock duration, two checks in one day, a verified check with no lock, a seal repeated) catches both bugs and liars, and turns the verified total into a number worth trusting.

### 130. A 'since your last visit' strip on the dashboard
*Score 0.75 — impact 3 ÷ effort 4 (M) — Retention*

A member returning after four days sees the same page as one who logged ten minutes ago. Three computed diffs — your rank moved, a record you held was beaten, what the feed did in your absence — cost one query and give the visit a reason to continue.

### 131. Optimistic logging: the row appears instantly, and stays only if the server agrees
*Score 0.75 — impact 3 ÷ effort 4 (M) — Craft*

The counter only moves after the round trip, so the click feels slower than the page looks. Render the row and the counter immediately, reconcile on the response, roll back with an explanation on failure — the same rule the undo card already follows, applied to the click itself.

### 132. A records page: the extremes nobody can beat by trying
*Score 0.75 — impact 3 ÷ effort 4 (M) — Boards*

Boards rank the last 30 days. A records page holds the all-time extremes — longest lock, longest denial run, longest verified cage streak, fastest and slowest reported edge, best week — which are cheap to compute, stable enough to be worth citing, and the only part of the site with a real chance of being screenshotted.

### 133. Quiet mode: track without being ranked
*Score 0.75 — impact 3 ÷ effort 4 (M) — Identity*

Every member is on the board whether they want to be or not, and some of them are here for the diary rather than the contest. One flag that removes a member from every ranking while leaving their own numbers intact is smaller than the consent argument it settles.

### 134. Decide the payment rail before designing anything paid
*Score 0.75 — impact 3 ÷ effort 4 (M) — Money*

This content is prohibited by Stripe, PayPal, Ko-fi and every mainstream ad network, so a supporters tier designed before the rail exists is a design that cannot ship. Write the decision: what is permitted (an adult-friendly processor, crypto, or nothing), what it constrains, and only then what the tier is.

### 135. Retract a log with a reason, beyond the 15-minute undo
*Score 0.6 — impact 3 ÷ effort 5 (M) — Data*

The undo window is fifteen minutes and the confessions are meant to be honest; a week-old row that was wrong has no path except deletion. A retraction that flags the row (excluded from boards and the feed, visible in history as retracted with a reason) keeps the record honest without pretending it never happened.

### 136. A supporters tier that buys cosmetics and nothing else
*Score 0.6 — impact 3 ÷ effort 5 (M) — Money*

The credible version of monetisation here never gates a feature that already exists: a badge, a colour on the board, early access to a doctrine section. It is also the version a community of three will tolerate — and it prices the thing actually on offer, which is upkeep, not access.

### 137. An ops page: database size, disk, backup freshness, log volume
*Score 0.6 — impact 3 ÷ effort 5 (M) — Ops*

The site runs on one host with no monitoring beyond a health check and a self-heal timer. Four numbers rendered once an hour — DB size and growth rate, disk free, hours since the last dump, log lines per day — catch the failures that a 200 does not: a table growing without bound, a backup that stopped.

### 138. Chained seals: tomorrow's code depends on today's
*Score 0.6 — impact 3 ÷ effort 5 (M) — Cage*

A seal is random per check, so a check can be fabricated by anyone who can read what the site issues. Deriving each day's code from the previous code plus a server secret makes a chain: a member who skipped a day cannot produce a consistent sequence, and a verifier comparing two consecutive codes can tell.

### 139. Streak repair, paid for in verified cage hours
*Score 0.5 — impact 3 ÷ effort 6 (L) — Game*

A broken streak is where members leave. Letting one be repaired by spending earned verified hours makes the newest metric the currency of the oldest one, and prices the repair in the only thing the site cannot fake: time actually attested by peers.

### 140. A per-member feed for the people who watch one person
*Score 0.5 — impact 2 ÷ effort 4 (M) — Growth*

The site feed is global, and members follow individuals rather than the site. `/u/<name>.json` plus an RSS view of one member's public events is the smallest social surface there is, needs no following table, and gives the keyholder shape a machine-readable half.

