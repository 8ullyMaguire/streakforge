# Plan — a deck the members write, and a ladder that fits the site

Date: 2026-10-29 (evening) · Repo: `~/code-local/rust/streakforge` · Owner's ask, verbatim:

> "allow users to add entries to the affirmation drill; soften trust level requirements while there
> are few users. use your best judgement for any decisions left; test, verify, etc as you go. when
> done, update docs (readme, roadmap, etc), commit, push, re-deploy, etc"

## 1. Decisions taken

**The deck**

| question | decision | why |
|---|---|---|
| Where do lines live? | `affirmations` table (0019), 18 curated seeds with `author_id` null | a TypeScript constant makes "add a line" mean "deploy the site" |
| Who may submit? | anyone signed in, guests included | a guest session can already log; writing a line is the same act |
| What admits a line? | level 2 (`REVIEW_LEVEL`), or the author's own level at write time | the same trust currency as every other power on the site |
| Rate limits? | 1/hour, 3/day | the deck is read, not shouted at; enough for a friend group |
| Duplicates? | partial unique index on `lower(btrim(body))` where `live` | a check-then-insert loses the race; the index cannot |
| May an author delete? | yes, and it cascades with the account | a line is yours; erasure has to mean erasure |
| Curated lines? | author null; only a leader may hide or prune | the seeds are the site's copy, not a member's |
| Does the drill depend on the API? | no — the old constant stays as the offline fallback | a drill that refuses to drill because one GET failed is worse than a stale deck |

**The ladder**

| question | decision | why |
|---|---|---|
| What counts as "few users"? | distinct non-guest members with a log in the last **7 days** | measured on prod: 74 accounts, 18 in 30 days, **3 that week** — a month counts one-time visitors and never falls below the threshold |
| Where is the switch? | 10 active members | below it the soft rungs apply; at or above it the 0016 rungs |
| Which numbers? | soft L1 24h/1 log · L2 48h/3 active days/5 contributions · L3 7d/8 days/15 | reachable in a long weekend, and still gated by real activity |
| Does it tighten back? | yes, automatically at 10 active | a band, not a permanent loosening nobody remembers to undo |
| Where does the UI get the numbers? | `/api/trust` ships `band`, `active_accounts`, `thresholds` | the page hardcoded "7 active days" for a level granted at 3 — a UI that lies about its own rule |

## 2. Files

- `backend/migrations/0019_affirmation_deck_and_soft_trust.sql` — the table, the 18 seeds, the
  partial unique index, `active_accounts()`, `trust_thresholds()`, `trust_level()` rewritten to read it.
- `backend/migrations/0020_active_means_this_week.sql` — `active_accounts()` over 7 days.
- `backend/src/affirmations.rs` — list / submit / approve / hide / delete.
- `backend/src/moderation.rs` — `TrustResponse` gains `band`, `active_accounts`, `thresholds`.
- `backend/src/fold.rs` — `affirmations.author_id` folds with the account; `reviewed_by` stays.
- `web/src/routes/affirmations/+page.svelte` — the deck page.
- `web/src/routes/drill/+page.svelte` — the deck comes from the API, the constant is the fallback.
- `web/src/routes/moderation/+page.svelte` — the ladder printed from the server's rungs.
- `web/src/lib/trust.ts` (+ `.test.ts`) — `ageLabel`, `describeRung`, `bandNote`.
- `web/src/lib/api.ts`, `web/src/lib/types.ts`, `web/src/lib/components/Navbar.svelte`.
- `scripts/deploy.sh` — health checks for `/api/affirmations` and `/affirmations`.

## 3. Steps with their verification

1. **The migration applies and the band is right.**
   `cargo test --test integration the_ladder_softens` → `1 passed`: the same member is L2 on the
   small band and L1 on the grown one, and back to L2 when the site empties.
2. **The denominator is a week.** `cargo test --test integration active_means` → `1 passed`:
   a member who logged 20 days ago is in the month's count and not in the week's.
3. **The deck's lifecycle.** `cargo test --test routes` → `25 passed`: public read, level-0 line
   pending, level-0 approval 403, level-2 approval 204, second approval 404, duplicate 409,
   1/hour 429 with `Retry-After`, 3/day 429, author hide/delete 204, stranger 403, curated 403
   for a member and 204 for a leader.
4. **The fold does not lose a line.** `cargo test --test guest` → `14 passed`, including
   `a_guests_affirmation_folds_and_keeps_its_place_in_the_queue`.
5. **Frontend.** `node node_modules/vitest/vitest.mjs run` → `145 passed`; `svelte-check` →
   `0 errors and 0 warnings`; `node node_modules/vite/bin/vite.js build`.
6. **Live.** `./scripts/deploy.sh thinkcentre` → all health checks ok (including the two new paths).
   Probe: anonymous deck = 18 curated; a level-0 account's line is `pending` with
   `remaining_today: 2`; a second submission in the hour is 429 with `retry-after: 3600`; approving
   your own line at level 0 is 403 with the level named; `/api/trust` reports
   `band: small, active_accounts: 3` and the soft rungs.
7. **In a browser.** `/drill` says "19 lines in the deck · write one"; `/affirmations` accepts a
   typed line and shows it under "YOUR LINES · WAITING"; `/moderation` prints
   "a day old, 1 log · 3 active days, 5 contributions · 8 active days, 15 contributions" and the
   "SMALL SITE — 3 active members" note.

## 4. What this plan deliberately does not do

- **No editor role.** Admission is trust, like everything else; a leader can still prune a curated
  line, and that is the only human privilege in the deck.
- **No editing after submission.** Hide or delete and write it again; an edit history for 200
  characters is more machinery than the feature deserves.
- **No thumbnails, votes or "best of" ranking on deck lines.** The drill is not a popularity contest,
  and a ranked deck would let the most-typed line become the only line.
- **No per-member band.** One band per site, so a level can never be computed from two ladders.
