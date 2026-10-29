# Plan — the yesterday window, and the data rights the terms already promised

Date: 2026-10-29 · Repo: `~/code-local/rust/streakforge` · Owner's ask, verbatim:

> "build 10 first, and gdp compliance (export, remove data, etc), use your best judgement for
> any decisions left; test, verify, etc as you go. when you are done update docs (readme,
> roadmap, spec, etc) if needed, commit, push, re-deploy."

Item 10 of the brainstorm is `docs/100_IDEAS.md` #110 (*a catch-up window for yesterday only*);
"GDP" here is the GDPR set the terms already claim: export, erasure, rectification, retention.

## 1. Decisions taken (no further approval requested)

| question | decision | why |
|---|---|---|
| How is "yesterday" expressed? | `POST /api/logs` gains `"day": "today"|"yesterday"` | one write path, one set of rules; a second endpoint would duplicate the budget logic |
| What if yesterday already has a log? | 409, and the message says the day was kept | a catch-up fills a gap; otherwise it is a way to file a sixth log for a day |
| Which budget judges a catch-up? | yesterday's (`check_rate_limits_on(…, target_date)`) | otherwise the catch-up is a budget-laundering door |
| How is "one per day" enforced? | partial unique index on `(user_id, catch_up_on) WHERE is_catch_up` | a check-then-insert loses the race; the database cannot |
| Where does the row land in time? | 12:00 UTC of the day described | the real time is unknown; midday is unambiguously inside that UTC date |
| What clock does the 15-minute undo use now? | `created_at`, not `logged_at` | a catch-up describes yesterday but was written now |
| Is the backdating visible? | yes — `FILED LATE` on the feed, in the caller's own history, and `catch_up` on the DTO | a backdated row that looks ordinary is a small lie |
| Export format | one JSON document, as an attachment, `no-store` | one statement, one test, no pagination to get wrong |
| Erasure proof | password + the word `DELETE` typed | the only irreversible action on the site |
| What survives an erasure? | an anonymous `account.erased` row with counts; `moderation_actions` with a null actor; consent rows with an emptied `detail` | answers "when was it deleted" without keeping the person |
| Retention | `streakforge-api purge` daily: tombstones >365d, `login_attempts` >30d, audit rows >365d except consent/erasure | the promise on `/privacy` needs an enforcer |

## 2. Files

- `backend/migrations/0018_catch_up_and_gdpr.sql` — columns (`created_at`, `is_catch_up`,
  `catch_up_on`), shape constraint, partial unique index, `activity_feed` rebuilt with
  `catch_up` (requires `drop function` — `create or replace` cannot widen an OUT list),
  `my_recent_logs()`.
- `backend/src/api.rs` — `LogRequest.day`, `check_rate_limits_on`, the catch-up branch,
  `created_at` in the undo window, `FeedItem.catch_up`, `PublicFeedItem.catch_up`,
  `StatsResponse.catch_up`.
- `backend/src/account.rs` — `export`, `erase`, `purge_expired`.
- `backend/src/main.rs` — `streakforge-api purge` mode.
- `backend/src/router.rs` — `/api/account/export`, `/api/account/erase`.
- `web/src/routes/dashboard/+page.svelte` — the "YESTERDAY IS EMPTY" card.
- `web/src/routes/settings/+page.svelte` — the "YOUR DATA" section.
- `web/src/routes/privacy/+page.svelte` — the policy page, linked from the footer.
- `web/src/routes/feed/+page.svelte`, `+layout.svelte`, `lib/api.ts`, `lib/types.ts`.
- `scripts/deploy.sh` — the daily purge cron entry.

## 3. Steps with their verification

Each step is done only when the command below it prints what it says.

1. **Migration applies from scratch and on top of prod.**
   `cargo test --test integration` → `test result: ok. 63 passed` (sqlx replays every
   migration into a fresh database per test, so a broken 0018 fails all 63).
2. **The window's rules.** `cargo test --test routes a_catch_up` →
   `test result: ok. 18 passed`; the test asserts 201, `catch_up: true`, `today_count: 0`,
   `current_streak: 1`, the second attempt 409, a kept day 409, a named day 400, and the
   PATCH inside the window 200.
3. **The index is the rule.** `cargo test --test integration one_catch_up` → passes; two
   inserts with the same `catch_up_on` collide.
4. **The export is complete and scoped.** `cargo test --test routes the_export` → 401
   anonymous, 200 with `Content-Disposition: attachment`, all eight sections, no
   `password_hash`, and the other account's name absent from the document.
5. **The erasure is precise.** `cargo test --test routes erasing` → 400 (bad word), 403
   (bad password), 204, rows gone in three tables, one anonymous audit row, the moderation
   action kept with a null actor, the cookie 401 afterwards, the bystander untouched.
6. **Retention runs and takes only the expired.** `cargo test --test integration
   the_purge` → 1/1/1 removed, consent + erasure rows kept.
7. **Frontend.** `node node_modules/vitest/vitest.mjs run` → `135 passed`;
   `svelte-check` → `0 errors and 0 warnings`; `node node_modules/vite/bin/vite.js build`.
8. **Live.** `./scripts/deploy.sh thinkcentre` → all health checks ok, including the purge
   cron line; then `scripts/sf-gdpr-probe.sh`-style probe: 201 catch-up with
   `logged_at: …T12:00:00Z`, 409 second, 400 named day, export as a download with eight
   sections, erase 400/403/204, session 401 after, `/api/total` back to its pre-probe pair.
9. **In a browser.** Signed in: the card renders, `WASTE FOR YESTERDAY` moves the streak
   0 → 1 and replaces itself with "Catch-up used today"; `/settings` shows the data
   section with the erase button disabled until `DELETE` is typed; `/privacy` renders its
   seven sections.

## 4. What this plan deliberately does not do

- **No 7-day undo on erasure.** Item #44 in the list wants one; the owner asked for removal,
  and a soft-delete would make "erased" untrue. The export is the safety net, and the UI
  says so before the button.
- **No account-deletion email.** There is no address on file anywhere in this system.
- **No bulk erasure of other people's rows.** A member's flags and attestations go with
  them (cascade); the content they flagged does not.
- **No `catch_up` backfill.** Existing rows are ordinary logs; the column defaults false,
  which is the truth.
