# Plan — the consent the brainstorm asked for: time to cum is opt-in

Date: 2026-10-29 (late) · Repo: `~/code-local/rust/streakforge` · Follows the owner's standing
instruction: *"complete anything left you can do without my input, use your best judgement for any
decisions left; test, verify, etc as you go; update docs, commit, push, re-deploy."*

## What was left

Three items from my own handoff, taken in the order the evidence ranked them:

1. **`time_to_cum_board()` published a member's average with no opt-in** (idea #122, score 1.0, a live
   consent gap on a feature that shipped the same day).
2. **No moderation affordance where the deck content is met** (idea #4, "moderation is a destination,
   not a gesture"): hiding a line meant leaving the drill for `/affirmations`.
3. **A profile showed only counts** (idea #5): no trust level, no verified cage time, no average.

## 1. Time to cum is opt-in (0021)

| question | decision | why |
|---|---|---|
| Default? | `share_time_to_cum = false` — the only private-by-default switch on the site | a duration measured from the start of a session to its end is the most intimate number here; the owner asked for public *confessions*, not public durations |
| Does the aggregate count non-sharers anonymously (as idea #122 proposed)? | **No** — only opt-ins | with three members, an "anonymous" aggregate plus a per-person average plus a five-person board is not anonymous. The stricter rule is the only one that survives a small site |
| Own figures? | always shown, sharing or not | consent is about publication, not storage; hiding a number from the person it describes is a different bug |
| Revoking? | removes the rows from the pool, deletes nothing | turning it off must never be a disguised delete |
| Empty pool? | zero samples **and a sentence saying why** | `coalesce(avg(...), 0)` prints "0s" as if it were a measurement |

Verification: `time_to_cum_publishes_both_averages_but_only_for_members_who_share` (consent gates the
aggregate, the board needs consent *and* five reports, opting out leaves the rows) and the route test
`time_to_cum_is_opt_in_and_the_switch_is_a_setting` (walks the real PATCH, not the column).

## 2. Hide the line where it is read

The drill already fetched the deck; it now keeps each line's id and `can_review`, and prints
`withdraw mine` on your own line and `hide this line` on anyone else's when you are level 2+. Verified
in a browser with a temporary leader grant on a probe account: 19 lines → withdraw → 18, and the
control on someone else's line reads "hide this line". No curated line was touched; the probe's line
left with the probe's account.

## 3. The numbers block on a profile

`ProfileResponse` gained `trust_level`, `verified_cage_secs`, `time_to_cum_samples` and
`time_to_cum_avg_secs` (present only when shared), all from functions the rest of the site already
trusts (`trust_level()`, `cage_check_hours()`, `time_to_cum_stats()`). The page prints two more cards
and one sentence: `Time to cum: 12m 31s over 1 report — shared by choice`, or
`not shared (N reports kept private)`, or `No durations reported` — three different facts, three
different sentences.

## Files

- `backend/migrations/0021_time_to_cum_opt_in.sql`
- `backend/src/api.rs` — profile response/update, numbers block
- `backend/src/metrics.rs` — `sharing` on the response
- `backend/src/account.rs` — the export carries the switch
- `web/src/routes/settings/+page.svelte`, `web/src/routes/metrics/+page.svelte`,
  `web/src/routes/profile/[username]/+page.svelte`, `web/src/routes/drill/+page.svelte`
- `web/src/lib/types.ts`, `web/src/lib/api.ts`

## Verification

- 146 backend tests (22 unit · 11 consent · 14 guest · 66 integration · 26 routes · 7 edge), 145
  vitest, svelte-check 0/0, vite build clean.
- Live probe (`sf-ttc-probe.sh`): anonymous `samples 0 reporters 0`; a duration logged and not
  published (`mine samples 1 | aggregate samples 0`); the profile showing `avg None, samples 1`;
  PATCH on → `profile avg 751.0`, `public samples 1 reporters 1`; PATCH off → `public samples 0`
  with `mine samples still 1`.
- Browser: the settings switch present and unchecked by default, the metrics page saying "Nobody
  shares yet … not because the sessions are instant", then after the switch: `12m 31s · 1 report`,
  the board's "Nobody who shares has five reported sessions yet", and the profile reading
  `TRUST TL0 · VERIFIED CAGE 0s · Time to cum: 12m 31s over 1 report — shared by choice`.
- Probe account deleted; deck 18 curated, `is_leader` 0, public pool empty.

## What this does not do

- **No public "fastest" or "slowest" ranking of individuals.** The board stays the only ranking, and it
  is opt-in.
- **No per-log publication of durations.** A duration is an aggregate input, never a feed item.
- **No admin analytics** (idea #123) — that is its own pass, and it needs a TL4 page the site does not
  have yet.
