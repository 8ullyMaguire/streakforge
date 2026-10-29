# StreakForge — Development Status

## 2026-10-29 (fifth pass) — the consent the brainstorm asked for, and a profile that can be read

Standing instruction: finish what is left without asking. Plan:
`docs/plans/2026-10-29-time-to-cum-consent.md`.

- **Time to cum is opt-in (0021).** 0017 published every reporter's average; idea #122 called that a
  live consent gap, and it was right. `profiles.share_time_to_cum` defaults **false** — the only
  private-by-default switch on the site — and `time_to_cum_aggregate()` / `time_to_cum_board()` count
  only members who opted in. The board's five-sample floor sits behind the consent gate.
  `time_to_cum_stats()` is untouched: your own numbers are yours whether you share them or not, and
  turning the switch off removes them from the pool without deleting a row.
  **Stricter than the idea proposed:** #122 suggested the aggregate keep counting everyone
  anonymously. With three members, an anonymous aggregate plus a per-person average plus a board is
  not anonymous, so the aggregate is opt-in too. And with no one sharing, the page says "nobody
  shares yet — zero because there is nothing to average", instead of a `coalesce(..., 0)` printing
  "0s" as if it were a measurement.
- **The profile carries a numbers block.** `ProfileResponse` gained `trust_level`,
  `verified_cage_secs`, `time_to_cum_samples`, and `time_to_cum_avg_secs` (only when shared) — from
  `trust_level()`, `cage_check_hours()` and `time_to_cum_stats()`, no new arithmetic. The page prints
  three different sentences for three different facts: `12m 31s over 1 report — shared by choice`,
  `not shared (1 report kept private)`, `No durations reported`.
- **Hide a deck line where it is read.** The drill keeps each line's id and `can_review`, so a
  level-2 voice gets `hide this line` on the card itself and an author gets `withdraw mine` — idea #4
  of the first brainstorm list ("moderation is a destination, not a gesture"), applied where the
  content is actually met.
- The account export carries `share_time_to_cum` too, so a data download describes the setting as
  well as the rows.

Verified: 146 backend tests (22 unit · 11 consent · 14 guest · 66 integration · 26 routes · 7
edge-case), 145 vitest, svelte-check 0/0, vite build clean. Live (`sf-ttc-probe.sh`): anonymous
`samples 0 reporters 0`; a duration logged and **not** published (`mine samples 1 | aggregate 0`); the
profile reading `avg None, samples 1`; PATCH on → `profile avg 751.0`, `public 1 / 1`; PATCH off →
`public 0` with `mine samples still 1`. In a browser: the settings switch present and unchecked by
default; the metrics page saying "Nobody shares yet … not because the sessions are instant"; after the
switch, `12m 31s · 1 report` and the board's "Nobody who shares has five reported sessions yet"; the
profile reading `TRUST TL0 · VERIFIED CAGE 0s · Time to cum: 12m 31s over 1 report — shared by
choice`; and in the drill, `withdraw mine` → 19 lines became 18, with `hide this line` on someone
else's. Probe account deleted; deck 18 curated, `is_leader` 0, pool empty.

## 2026-10-29 (fourth pass) — a deck the members write, and a ladder that fits the site

The owner asked for two things: members should be able to add entries to the affirmation drill, and
the trust requirements should be loosened "while there are few users". Plan:
`docs/plans/2026-10-29-affirmation-deck-and-soft-ladder.md`.

- **The deck is a table (0019).** 18 curated lines seeded with `author_id` null, plus member
  submissions. A submission is `pending` until a level-2 voice admits it — or is `live` at write time
  if the author already holds level 2. 1 an hour, 3 a day. `affirmations_one_live_per_line`
  (partial unique on `lower(btrim(body))`) is what makes "the same line twice" a 409 instead of a
  race, and `affirmations_live_is_reviewed` refuses a member line that is live without a reviewer.
  The author can hide or delete their own; a curated line only a leader can touch; deletion cascades
  with the account, so erasure still means erasure. `/affirmations` is the page, the drill reads the
  deck from the API, and the old constant stays as the offline fallback.
- **The ladder has two bands (0019/0020).** Rungs come from `trust_thresholds()`, which picks a band
  by `active_accounts()` and switches at ten. Small band: L1 24h/1 log, L2 48h/3 active days/5
  contributions, L3 7 days/8 active days/15 contributions. Grown band: the 0016 rungs. It tightens
  back by itself when the site grows, because the band is computed at read time.
- **"Active" means this week.** 0019 counted 30 days; measured on production that was 18 accounts
  while three members had logged that day, so the band never bent and the softening would have
  shipped inert. 0020 counts 7 days, with a test that a member who logged 20 days ago is in the
  month and not in the week.
- **The moderation page stopped lying.** It hardcoded `needs: '7 active days, 10 contributions'` for
  level 2 while the database granted it at 3. `/api/trust` now ships `band`, `active_accounts` and
  `thresholds`, and the page prints those (`web/src/lib/trust.ts`, unit-tested) plus a
  "SMALL SITE — 3 active members" note explaining why the numbers are low.
- **The fold takes the deck with the account.** `affirmations.author_id` folds like a log;
  `reviewed_by` stays with the tombstone like every other record of a decision about someone else's
  content. A guest's line is always `pending`, so the fold cannot collide with the live index.

Verified: 22 unit · 11 consent · 14 guest · 66 integration · 25 routes · 7 edge-case = 145 backend
tests, 145 vitest, svelte-check 0/0, vite build clean. Live: anonymous deck 18 curated; a level-0
line `pending` with `remaining_today: 2`; a second submission in the hour 429 with
`retry-after: 3600`; self-approval at level 0 403 ("Level 2 admits lines to the deck. You are level
0."); `/api/trust` → `band small, active_accounts 3, rungs [(1,24,0,1),(2,48,3,5),(3,168,8,15)]`. In a
browser: `/drill` footer "19 lines in the deck · write one", a line typed into `/affirmations` landed
under "YOUR LINES · WAITING · just now", and `/moderation` printed "a day old, 1 log / 3 active days,
5 contributions / 8 active days, 15 contributions" with the SMALL SITE note. Both probe accounts were
deleted afterwards; the deck is back to 18 curated lines and the leader flag went with them.

## 2026-10-29 (third pass) — the yesterday window, and the data rights the terms promised

Two things: a log may now be filed for yesterday (idea #110), and the member can take their
data away or have it erased (idea #111 plus the erasure the terms already described). Plan:
`docs/plans/2026-10-29-catch-up-and-gdpr.md`.

- **The catch-up** (`POST /api/logs {"day":"yesterday"}`): one per member per FILING day, only
  into a day that holds nothing (otherwise 409 — "it fills a gap, it cannot add to a day you
  kept"), and judged by **that day's** budget and exclusivity rule via
  `check_rate_limits_on(…, target_date)`. The row lands at 12:00 UTC of the day it describes
  with `is_catch_up`, and the 15-minute undo now reads `created_at` — a catch-up describes
  yesterday but was written now, and the window is about the writing.
- **One per day is an index, not a check.** `habit_logs_one_catch_up_per_filing_day`
  (partial unique) is what the 409 maps from, so two concurrent requests cannot both win.
  A shape constraint refuses `is_catch_up` without `catch_up_on`.
- **Export** (`GET /api/account/export`): one `json_build_object` statement over the caller's
  own rows — account, logs, locks, cage checks, attestations given, flags filed, actions
  taken, audit events, session count — served as an attachment with `no-store`. No
  `password_hash`. The route test asserts the section list, so a new table that the export
  forgets is a failing test rather than a silent gap.
- **Erasure** (`POST /api/account/erase`): password + the word `DELETE`, then one
  transaction — session rows cleared, `audit_log.detail` emptied (the WHEN is kept; it is the
  record a deletion request is answered from), profile deleted (cascading to logs, locks,
  checks, attestations, flags), one anonymous `account.erased` row carrying counts only.
  `moderation_actions` keeps the action with a null actor: a decision about someone else's
  content outlives its author.
- **The password answers to the sign-in throttle**, not a laxer one written for this endpoint
  — a wrong password here must cost an attacker what a wrong password at `/login` costs, or
  erasure becomes the cheaper oracle. (The first version recorded failures with a NULL
  `login_attempts.ip`, which is NOT NULL, so the insert was swallowed and the throttle never
  counted. The purge test found it: `23502 null value in column "ip"`.)
- **The feed says when a row was filed late** (`activity_feed` grew `catch_up`, which required
  `drop function` — `create or replace` cannot widen an OUT list, 42P13).
- **Retention is enforced**: `streakforge-api purge` (a mode of the same binary, cron 04:15 via
  `deploy.sh`) removes guest tombstones older than 365 days, `login_attempts` past 30,
  and audit rows past a year except consent and erasure records. `/privacy` states every
  window it enforces, and says what the site cannot do — no email means no reset and no
  receipt, nothing is end-to-end encrypted, and deletion is irreversible.

Verified: 22 unit + 11 consent + 13 guest + 63 integration + 18 routes + 7 edge-case,
135 vitest, svelte-check 0/0, vite build clean. Live: the catch-up filed 201 with
`logged_at: 2026-10-28T12:00:00Z`, `today_count: 0`, `current_streak: 1`, a second attempt
409 and a named day 400; the export arrived as
`attachment; filename="streakforge-<user>-2026-10-29.json"` with all eight sections and no
hash; erase answered 400 / 403 / 204 and the cookie 401'd afterwards, after which the prod
row count and `/api/total` returned to their pre-probe values and the only trace was one
anonymous `account.erased` row; `./streakforge-api purge` on the host printed
`{"audit_rows":0,"guest_tombstones":0,"login_attempts":0}` (nothing is old enough yet). In a
browser: the "YESTERDAY IS EMPTY" card moved the streak 0 → 1 and replaced itself with
"Catch-up used today", the erase button stayed disabled until `DELETE` was typed, and
`/privacy` rendered its seven sections.

## 2026-10-29 (later) — cage checks, quorum moderation, time to cum, public confessions

Four asks in one pass, plus the one-click landing flow and the reversal that a confession is
public unless its author says otherwise. Spec: `docs/specs/2026-10-29-cage-checks-trust-time-to-cum.md`;
plan: `docs/plans/2026-10-29-cage-trust-time-to-cum.md`.

- **Cage checks** (`/cage`, `0015`): one check per UTC day while a lock is open, an 8-hex seal
  code issued at creation, claimed time capped at 72h, and **hours that only exist once peers
  attest** — 2 confirmations from distinct members, or 1 from level 3. `cage_check_hours()` publishes
  claimed / verified / pending / disputed, a verification rate, and verified-day streaks;
  `total_verified_cage_hours()` is the site-wide total. Nothing is uploaded: a check is a claim and a code.
- **Trust levels + quorum** (`/moderation`, `0016`): levels 0–4 are *computed* by `trust_level()`
  (age, active days, contributions = logs + attestations, upheld flags), never stored — the only
  stored bit is `profiles.is_leader`. Three flags with at least one from level ≥ 2 hide a target,
  two restore votes (or one leader action) bring it back, and every hide/restore is appended to
  `moderation_actions` with its actor.
- **Time to cum** (`/metrics`, `0017`): `habit_logs.time_to_cum_secs`, the owner's definition —
  start of the session to release, per log. The page publishes the pooled average, the median, and
  an average of per-person averages, because the two answer different questions and one heavy
  logger must not define the site.
- **Public confessions, private on request**: `profiles.logs_public` defaults to true and
  `activity_feed()` — not the handler — applies it, so a private author's rows leave the feed while
  their logs, streak and score are untouched. The setting is a checkbox on `/settings` with the
  consequence spelled out, and the feed page states the default rather than hoping nobody notices.
- **One-click logging**: the landing page's two buttons POST directly. The duration and a 15-minute
  undo are offered *after* the fact, inline — `PATCH /api/logs/{id}` and `DELETE /api/logs/{id}`
  both refuse past the window with 403.

**A green suite hid a 500 until the guest tests ran.** `POST /api/auth/register` failed for any
guest once the new tables existed: `src/fold.rs` walks `pg_constraint` for every foreign key into
`profiles(id)` and refuses to half-move a person's history, and four new keys were unclassified.
`cage_checks.user_id` and `cage_attestations.attester_id` now fold (own tenure — 0016 counts
attestations towards contributions, so dropping them would silently reset a converting member to
level 0); `flags.reporter_id` and `moderation_actions.actor_id` stay with the tombstone, because a
vote about someone else's content and an action log must not be re-cast under a new name. New test:
`tests/guest.rs::the_fold_moves_cage_tenure_and_leaves_votes_with_the_tombstone`.

Verified: 22 unit + 11 auth + 13 guest + 59 integration + 15 routes + 7 edge-case backend tests,
135 vitest, `svelte-check` 0 errors / 0 warnings, `vite build` clean — then live, twice: by curl (a
one-click POST wrote the row, `PATCH` set 900s and the aggregate moved, the confession appeared in
`/api/feed` and vanished after `logs_public:false` while `/api/stats` still counted it, a check
without a lock 409'd, a second check that day 429'd, a self-attestation 403'd, undo 204'd) and in a
real browser session signed in as a throwaway account: the WASTE A LOAD button recorded on click,
the counter moved 202 → 203, the duration chip saved, the refusal read *"WLWs and WLDs are one way:
a whiteboi who cums cannot claim a denial"*, and UNDO took the counter back.

**The browser found what the tests could not.** A refused second attempt used to clear `recorded`
before the request, so the denial that the server correctly rejected also wiped the undo card for
the log that *had* landed — the one moment the card matters. `logNow()` now replaces `recorded`
only on success and leaves it alone on failure; re-proved live afterwards (waste → card → denied →
refusal shown, card and undo both still there → undo → counter back to 203).

## 2026-10-29 — exclusivity had a direction all along, and the code had it the wrong one

The owner: *"I should be able to click waste load right after denial, but not the other way
around."* Nothing was mis-styled. `check_rate_limits()` refused a **waste** for the rest of a
UTC day that already held a denial, and `compute_stats()` mirrored that into
`GET /api/stats` as `can_log: false, blocked_by_exclusivity: true` — which is not a hint, it is
literally the WASTE A LOAD button's `disabled` attribute. The button was doing what the server
said.

The rule is now one-way, the way the scoring always behaved:

| sequence (same UTC day) | before | now |
|---|---|---|
| waste → denial | 429 | 429 (a whiteboi who cums cannot claim a denial) |
| denial → waste | 429 | **201** (the relapse is the event the site counts) |
| denial → denial | 429 | 429 |
| 6th waste | 429 | 429 |

`blocked_by_exclusivity` is therefore now only ever true on a denial payload, and the dead
waste-side copy is gone from `LogButton`, the dashboard and the denial page. The Devotion
Index is untouched: `exclusive_streak()` still follows the most recent kind, so switching still
costs the active streak — the block only ever cost the *record*.

Verified: RED first (the new route test failed on the old code with
`blocked_by_exclusivity: true`), then GREEN — 22 unit + 51 integration + 11 routes + 11 auth +
7 edge-case, 128 vitest, `svelte-check` 0/0 — and then live on the deployed binary with a probe
account that logged denial → waste (201) → denial (429), deleted afterwards. Docs updated with
it: SPEC §14, README, SESSION_CONTEXT, the glossary's `Exclusivity` and `Denial cadence`
entries. Plan: `docs/plans/2026-10-29-one-way-exclusivity.md`.

## 2026-10-15 — the streak edge-case table, and the `pub` day function it proved wrong

The 2026-10-04 ranking's verification ask for this project was "add an edge-case table for
streak calculation (timezones, DST, grace periods)". `backend/tests/streak_edge_cases.rs`
(commit `3b8bec5`) is that table: 7 tests, **83 → 90 green** (22 unit + 51 integration +
10 routes + 7 edge-case), mutation gate **8 killed / 0 survived / 0 not-tested**.

It found a real defect on the way: `next_utc_midnight(now)` documented "Next UTC midnight"
but took `now.date()` — the date on the **caller's** clock — so a non-UTC caller got a
midnight a day wrong (a +02 caller at 01:30 on the 15th, i.e. 23:30 UTC on the 14th, got the
16th's). Latent only because every call site passes `now_utc()`. It now converts to UTC first,
and the non-UTC case is pinned. The table's most valuable row is not a date but the fixture
that makes the timezone property testable: 00:30 and 23:30 UTC are ONE day for a +02 member
whose local clock reads them as two.

Also recorded rather than left to be rediscovered: the repo is **not** `cargo fmt`/clippy
clean at HEAD, and this pass did not change that. `cargo fmt --check` wants the same 7 files
formatted before and after (the new file is clean on its own) and `clippy -D warnings`
reports 5 lib errors — `api.rs:300` a derivable `impl Default`, three very-complex types,
`auth.rs:399` a manual `!RangeInclusive::contains`. Because `-D warnings` makes clippy abort
at the lib it never reached the new file, so that file was linted separately:
`cargo clippy --test streak_edge_cases --no-deps` → exit 0, zero lints. Clearing the debt is
a separate, mechanical pass.

## 2026-10-14 (late) — the third reading: four clauses that were never shipped

The plan's scope is sixteen items and all sixteen are marked shipped. This pass read the
ideas' own sentences again — not for weaker implementations this time, but for **clauses
that were never built at all** — and found four. All four are closed and verified live.

| # | the clause | was | now |
|---|---|---|---|
| 03 | the dump must survive what the item names ("turns *the disk died* from the end of the site into an hour") | dump and rehearsal both on the disk that dies | `scripts/offhost-backup.sh` copies the newest dump to the other host nightly; five checks, a pruning window, and a control that proves each one fires |
| 10 | "a line **on the dashboard**" | the endpoint existed and the landing page showed it; the dashboard, where a logging user looks, did not | the dashboard shows `2 whitebois denied today · 3 wasted`, linked to `/kpi` |
| 14 | "a confirm page at **/log/habit**" | only `/log?kind=habit`; the path form hit the app's not-found page | `/log/habit`, `/log/denial`, `/log/affirmation` are real routes, each rendering the right kind |
| 15 | "**resets in 4h12m**" | the label read `4 of 5 left today` for most of every day | `4 of 5 left · resets in 1h 54m` |

**15 was a semantic collision, not a missing feature.** `next_allowed_in_secs` was zeroed
whenever `can_log` was true — the normal state — so the day's clock was reported as 0 and the
label fell back to "left today". One field, two questions ("may I log now?", "when does the
allowance refill?"). There are two fields now, and the route test asserts the distinction:
`can_log == true` while `next_allowed_in_secs == 0` while `secs_until_reset` is 1..86400.

**Not a bug anyone would have reported.** The endpoint worked, the numbers were right, and
the sentence never appeared. Three of the four are the same shape as the five found in the
audit pass — the deliverable shipped one surface away from the one the item names — which is
now the single most productive thing to look for in this codebase.

**Also**: three mutations, each reddening exactly the tests that assert the behaviour (the
mismatch reporter, the empty state, the budget clock); one observation I could not explain
(an integration run reporting 47 failures in 0.01s, unreproduced on two later full runs, the
error text lost to my own grep); and the CI story is unchanged — push triggers still create no
runs, so this commit is dispatched by hand (SPEC §16.3).

## 2026-10-14 (evening) — the tier 1+2 audit: five gaps between the ideas and the code

The first tier 1+2 pass shipped 16 items. This pass re-read each of the 16 ideas against
what actually landed, and found five places where the plan had shipped a weaker version
than the idea asked for. All five are now closed. None of them were bugs a user would
have reported, which is exactly why they needed looking for.

- **06 asked for exponential backoff**; the plan shipped a flat 15-minute window. An
  attacker got 5 guesses per 15 minutes forever. Now the wait doubles past eight
  failures in a day (30m / 1h / 2h cap) for that username, and every refusal is logged
  with the username and address. The address deliberately does NOT escalate — a cafe
  shares it.
- **06 also asked to log the event.** It did not. It does now, and OPERATIONS §4.1 says
  where to read it, why it is the only personal data the app logs, and what a flood means.
- **03 asked for a *monthly* restore rehearsal.** Only the nightly dump was on a timer;
  the rehearsal was a script someone had to remember. It is now a cron entry on the 1st at
  04:30, and it fails when the newest dump is over 48h old — which makes it the only
  thing that notices the backup job has died.
- **09 asked for a test that *decodes* the feed's columns.** The pinning test compared
  SQL signatures, which catches schema drift but cannot see a handler/JSON mismatch —
  the exact shape of the two 500s this project has had. There are now nine tests that
  mount the real router and read the response body; to do that the HTTP surface moved
  from `main.rs` to `router.rs`.
- **12 asked for the glossary to be linked from the empty states and the doctrine
  index.** It was in the footer only. Now both, plus the doctrine page.
- **And one thing that was worse than missing**: `GET /health` returned 200 on a broken
  database, because no such route existed and the SPA fallback served `index.html`. A
  status code describing the app shell is worse than no health check. There is now a real
  `/api/health` that round-trips a query and answers 503.
- **16's plan line claimed the decay notice was "covered by a component test"** while the
  copy lived inline in the template where no test could reach it. The three sentences now
  live in `web/src/lib/decay.ts` with seven tests asserting the strings themselves, on
  both sides of the ten-day threshold.
- **A sixth gap, found only after deploying**: the fix for the health check was itself
  half a fix. `/api/health` became real, and `/health` — the path a monitor guesses first —
  kept answering 200 with the SPA shell, i.e. the exact defect one layer out. Both paths
  serve the same handler now. It was found by curl-ing the live site after the deploy, not
  by reading the code I had just written.

**Acceptance sweep**: all sixteen tier 1+2 items re-observed against production and in a
browser on 2026-10-14, from scratch on a probe account that was created and deleted by the
sweep — the table is SPEC §21. Fifteen observed clean; the one red was my sweep's own
`/terms` regex, and tracing it found the page saying what the item asked for.

**Gates**: `cargo test` 22 lib + 48 integration + 10 route (80); `vitest run` 100 across 12
files; `svelte-check` 0/0; `vite build` green. Three of the new tests were broken on
purpose before being trusted (the feed DTO, the escalation, the decay threshold), each
reddening exactly one test with the defect named, then restored and re-run green.

**CI: push-triggered runs stopped being created; the workflow itself is fine.** The only
`push` run is `3b8096c`; every push after 21:26 on 2026-10-14 created none, including an
empty commit pushed purely as a control. Ruled out by experiment: the token (has the
`workflow` scope and pushed those commits), the YAML (GitHub accepted an edit to it), and
Actions themselves (a manual `gh workflow run ci.yml --ref main` started run `38643450176`
on `32e0b90`, **both jobs success, 80 backend tests — the same numbers as the local gate,
verified on a machine that is not this one**). `workflow_dispatch:` is now a trigger, so the gate is one command instead of
automatic. The reason `push` no longer schedules runs is only visible in the repository's
Actions tab in a browser — that is the one thing left to read (SPEC §16.3).

## 2026-10-14 — tier 1 and tier 2 of the ranked 100 (`91fc795`)

Sixteen items from `docs/100_IDEAS.md` — everything scoring ≥ 2.5 — shipped in one
pass. Plan with a decision and an acceptance test per item:
`docs/plans/2026-10-14-tier-1-and-tier-2.md`. Evidence: SPEC §21.

**The four that changed what the site measures**

- **Notes are private (01).** `activity_feed()` returned `note` to unauthenticated
  callers, so the confession box was published to the anonymous feed and rendered on
  the landing page. Every one of the 512 logs in production had a note. The column is
  gone from the function's result type; the feed maps into a `PublicFeedItem` that has
  no note field; the dashboard still shows you your own.
- **The score counts days, not rows (11).** `count(distinct log_date)` for both WLW and
  WLD. Raising the waste budget to 5 a day had made a day of wasting worth 5×.
- **A board place needs a 24-hour-old account (13).**
- **The decay warns instead of surprising (16).** `days_until_decay` on the index, ⚠ at
  10 days, and a different message once the score is already gone.

**The twelve that changed what the site tells you**

02 `__Host-`+`Secure` cookie (signs everyone out once, done) · 03 nightly backups at
03:30 with a REHEARSED restore · 04 robots.txt + sitemap · 05 CI on every push ·
06 login throttle (5/username, 30/address, 15 min) · 07 og/twitter cards + a real
1200×630 PNG · 08 18+ acknowledgement + /terms that admits it is not enforcement ·
09 five public SQL function signatures pinned as literals · 10 /api/today ·
12 /glossary · 14 /log?kind= deep links · 15 budget + reset clock on one line.

**Verified on the live site, not only in tests**: the `__Host-` cookie and
`/api/auth/me`; 50 feed items with no `note` key while 512 notes remain in Postgres;
`/api/today` matching a direct SQL count; five bad logins → 429 with `Retry-After:
899`, the correct password refused while throttled, and a different username still
able to sign in; robots/sitemap/og.png served; and in a real browser the 18+ overlay,
the note-free landing page, all 17 glossary terms, /terms, and `/log?kind=denial`
plus the unknown-kind fallback.

**Gates**: backend `cargo test` 20 lib + 48 integration; frontend 93 vitest across 11
files, `svelte-check` 0/0, `vite build` green. Deploy: 15/15 health checks 200.

Last updated: 2026-10-14 (evening pass)

## 2026-09-28 — the `wip/recover-2026-08-work` review

`docs/specs/2026-09-28-branch-review-and-frontend-install.md` has the full
write-up. Summary:

**The work is wanted and the recovery was correct.** `98d61a8` is finished
authored work (denial kind, chastity lock, weighted leaderboards). `bc08d5b`
restored 77 files that a botched `git rm -r --cached` had staged as deleted
while they sat on disk — `git reset` brought back the index without touching the
working tree, so nothing was lost. The recovered content is the working state of
2026-08-14, not speculative scaffolding.

**The backend was already honest.** 18/18 integration tests against a
zero-table database on the first attempt, three consecutive runs. It documents
its own `streakforge_test` precondition in the file header.

**The frontend could not be installed at all.** `npm ci` died on
`sharp@0.34.5` (no prebuilt linux-x64 binary, source build fails), so
`node_modules` was never populated and `build` / `test` / `check` all reported
`command not found` — which reads exactly like "unverified frontend".

Cause: `@sveltejs/enhanced-img` was registered in `vite.config.ts` and is a
direct devDependency, but **no component uses `<enhanced:img>`** and the only
three `<img>` tags take runtime URLs, so it had nothing to optimize. It was a
native build dependency doing zero work. Removed from both `vite.config.ts` and
`package.json`, with the reasoning at the removal site.

Also cleared two Svelte 5 deprecations in `Navbar.svelte` (a `<slot />` no
caller ever filled, and a self-closing `<div />`).

**Now verified, not assumed:**

    backend  cargo test --lib                 8 passed
             cargo test --test integration    18 passed (x3)
    frontend svelte-check                     0 errors, 0 warnings
             vitest                           6 files, 47 tests passed
             vite build                       built, adapter-static wrote build/

**Still open:** `denial-retheme` and `wip/recover-2026-08-work` now hold
identical content and `main` has neither. Merging to `main` and collapsing the
duplicate branch name is left as a separate, explicit step.

## What's built (all verified working)

### Backend (Rust / axum)
- [x] PostgreSQL schema: `profiles`, `habit_logs` (with generated `log_date`), indexes
- [x] `user_streak()` — current + longest streak (gaps-and-islands)
- [x] `user_streak_kind()` — kind-aware streaks (habit / affirmation / denial)
- [x] `daily_leaderboard()` / `weekly_leaderboard()` / `alltime_leaderboard()`
- [x] **Weighted leaderboards** (`*_leaderboard_weighted`): denial = 10 pts,
      waste = 1 pt, 3 affirmations = 1 pt; per-row waste/denial/affirmation counts
- [x] `user_of_the_day()` — count + earliest-log tiebreak (same weighting)
- [x] `kpi_totals()` / `kpi_trend(n)` — community KPI dashboard functions
      (migration `0010_kpis.sql`), served by `GET /api/kpi`
- [x] `activity_feed()` — cursor pagination
- [x] `profile_stats()` — today / week / alltime / streaks
- [x] Auth: local username/password (Argon2id) + session cookies; bot-dissuasion
      (honeypot, form timing, JS proof-of-work challenge); NO X OAuth, NO SMTP,
      NO dev-login in production
- [x] **Remember me by default**: sessions persist 30 days of inactivity
      (was: ended on browser close)
- [x] One social URL per profile (editable in settings, shown on public profile)
- [x] Rate limits: 1/hr + 5/day per kind (habit vs affirmation), server-enforced
- [x] **Denial kind + chastity lock**: `denial` log kind (1 per 24h, blocked if
      a waste was logged in the same 24h); `lock_sessions` table + `lock_info` /
      `lock_streak` / `total_denied` functions; `/api/denial`, `/api/lock`,
      `/api/unlock`; `/api/total` returns `{total, denied}`
- [x] Affirmation drill: `/api/drill` stats + `kind` on `/api/logs`
- [x] All endpoints: /auth/*, /logs, /stats, /drill, /denial, /lock, /unlock,
      /leaderboard/*, /user-of-the-day, /total, /feed, /profile/*, /profile (PATCH)
- [x] SPA static serving + client-route fallback (adapter-static build)
- [x] Doctrine: `/api/doctrine` + `/api/doctrine/{id}` — serves markdown docs
      from a configurable directory (path-traversal sanitized)
- [x] Integration tests (17 passing) for streak calc, leaderboards, UOTD, feed,
      affirmation-kind separation, denial/lock lifecycle, weighted leaderboards,
      auth (register/login/social_url)

### Frontend (SvelteKit / Svelte 5)
- [x] wlw-style dark theme (near-black, red/gold, Inter + Roboto Mono)
- [x] Landing: hero, big green/black tabular counters (**loads wasted** +
      **loads denied**), marquee, UOTD card, top-25 daily board, recent activity
- [x] **Community KPI dashboard** (`/kpi`): nine stat cards, a 30-day stacked
      trend chart, top weekly board. Public, no auth. Footer-linked.
- [x] Login: username/password form (SIGN IN / REGISTER tabs), honeypot +
      timing + JS challenge bot-dissuasion, no external accounts
- [x] Dashboard ("KNOW YOUR PLACE"): stat cards, **WASTE A LOAD** button
      (rate-limit aware), 30-day heatmap, recent logs
- [x] **Denial page** (`/denial`): live lock timer, LOCK / UNLOCK (with reason),
      REPORT A DENIAL, lock-streak + denial stats
- [x] **Typed affirmation drill** (`/drill`): user must TYPE the affirmation
      (typo-tolerant Levenshtein ≤3 edits) before REPEAT enables; gate
      affirmations "I'm racist" / "I submit!"
- [x] Leaderboard ("WHITEBOIS WHO KNOW THEIR PLACE"): weighted points,
      waste/denial/affirmation breakdown per row
- [x] Feed: reverse-chronological, LOAD MORE pagination, avatars
- [x] Profile: public stats, DiceBear avatar fallback
- [x] **Mobile-progressive**: bottom nav (Place/Deny/Drill/Board/Text), responsive
      counters, card-list tables, 2-col stat grid
- [x] **Installable app**: PWA manifest + spade icon, `/app` install guide,
      auth-aware landing CTA (logged in → WASTE A LOAD), 9-digit 3-color counters
- [x] **Leaderboard windows**: DAILY = rolling 24h, WEEKLY = rolling 7 days
      (was UTC calendar day/week — the board showed empty at US-evening peak hours)
- [x] Settings: edit username / display name / avatar URL
- [x] Manifesto: sidebar doc list + rendered markdown viewer (dark theme)
- [x] Drill: wlw-style big counter, affirmation deck, REPEAT (rate-limit aware),
      streak cards + heatmap
- [x] Toasts, skeletons, empty states, 401 redirect to /login
- [x] Frontend tests (33 passing): api utils, validation, challenge PoW, markdown
      renderer, affirmations deck
- [x] `npm run check` clean (0 errors)

## Verification evidence

- All API endpoints exercised via curl (auth, logging, rate limits, boards, feed,
  profile CRUD, drill separate budgets, doctrine + traversal 400).
- Browser-tested end-to-end: login → dashboard → log → leaderboard tabs → feed →
  profile → settings update → drill (counter/card/rate-limit hint) → doctrine.
- Backend: `cargo test` — 20 unit + 48 integration pass (2026-10-14; an earlier count
  of 17 integration tests in this file predates both the auth and the 0011 work).
- Frontend: `vitest run` — 93/93 pass across 11 files; `svelte-check` — 0 errors,
  0 warnings (2026-10-14; the two long-standing warnings are gone).
- Deployed: thinkcentre port 8001, systemd active, all routes 200, auth enforced,
  dev-login removed (routes 404); public at https://streakforge.polarisocial.xyz;
  pushed private to opencommit.eu/MagicZhang/streakforge.

## Documentation

- `docs/SPECIFICATION.md` (1058 lines), `docs/ARCHITECTURE.md`, `docs/OPERATIONS.md`,
  `docs/SESSION_CONTEXT.md`, expanded `README.md`.
- Keep all of these current after feature work.

## Known issues / notes

- No email/SMTP by design — accounts are username/password only.
- Legacy 'x'/'dev' provider rows (pre-migration) have no password hash and
  cannot log in; they are harmless orphans.
- Session cookie secret must be rotated per environment (`SESSION_SECRET`).

## Not in scope (v1)

Multiple habit types, notifications, friends/following, complex analytics,
native apps, payments/premium.

## Guest access, trust levels and self-governance (adopted 2026-10-16)

The owner's 2026-10-16 request is specified per site in `docs/ROADMAP-guest-trust-governance.md`, against the estate standard
`secondbrain/10-Projects/2026-10-16-guest-access-trust-and-self-governance.md` (items GS-1…GS-5, each configurable). This repo's deltas, firewalls and order of work are
in that file; it is the place to start, not this pointer.

