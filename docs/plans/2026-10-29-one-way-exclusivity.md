# Plan — one-way exclusivity: a waste after a denial is allowed, a denial after a waste is not

Date: 2026-10-29
Request: *"I should be able to click waste load right after denial, but not the other way around."*

## Current behaviour (measured, not assumed)

`check_rate_limits()` in `backend/src/api.rs` enforces exclusivity **in both directions**:

```rust
if kind == LogKind::Denial && habits_today >= 1 { 429 }   // keep this
if kind == LogKind::Habit  && denials_today >= 1 { 429 }  // remove this
```

and `compute_stats()` mirrors it:

```rust
let blocked_by_exclusivity = match kind {
    LogKind::Habit => denials_today >= 1,   // remove: makes can_log false
    LogKind::Denial => habits_today >= 1,
    LogKind::Affirmation => false,
};
```

`GET /api/stats` is the habit-kind payload the dashboard's WASTE A LOAD button reads, so
`can_log = false` + `blocked_by_exclusivity = true` after a denial is exactly the disabled
button the owner reported. The 429 path is the same rule seen through `POST /api/logs`.

## Target rule

| sequence | today | after |
|---|---|---|
| waste, then denial (same UTC day) | 429 | **429** (unchanged: a whiteboi who cums cannot claim a denial) |
| denial, then waste (same UTC day) | 429 | **201** (a relapse after a reported denial is the outcome the site counts) |
| denial, then denial | 429 | 429 (1/UTC day, unchanged) |
| waste ×6 | 429 on the 6th | unchanged (budget of 5) |

The expense of a waste day on the Devotion Index is unchanged: `exclusive_streak()` still
follows the most recent kind, so switching costs the active streak either way.

## Steps

1. `backend/src/api.rs` — `check_rate_limits()`: delete the `Habit && denials_today >= 1`
   block; drop `denials_today` from the count query (it only fed that branch). Comment states
   the direction and why.
2. `backend/src/api.rs` — `compute_stats()`: `blocked_by_exclusivity = kind == LogKind::Denial
   && habits_today >= 1`; drop the now-unused `denials_today` column from its query.
3. `web/src/lib/components/LogButton.svelte` — the waste-side "denial blocks the waste" copy
   becomes unreachable; keep only the denial-side message, reworded to the one-way rule.
4. `web/src/routes/dashboard/+page.svelte` — remove the `blocked_by_exclusivity` span (a
   habit-kind payload can no longer set it; leaving it is dead copy).
5. `web/src/routes/denial/+page.svelte` — reword the block reason: "a waste was recorded
   today, and that spends the denial's day".
6. `web/src/lib/glossary.ts` — `Exclusivity` and `Denial cadence` entries state the direction.
7. `docs/SPECIFICATION.md` §14 — replace the "any denial blocks every remaining waste slot"
   sentence with the one-way rule.
8. `docs/SESSION_CONTEXT.md` — the rate-limits table row.
9. Tests:
   - `backend/tests/integration.rs`: retarget `wlw_and_wld_are_mutually_exclusive` → the
     waste-then-denial 429 stays; the denial-then-waste case flips to `is_ok()`.
   - `backend/tests/routes.rs`: new test — log `denial`, then `GET /api/stats` must report
     `can_log: true`, `blocked_by_exclusivity: false`, and `POST /api/logs {"kind":"habit"}`
     must be 201; then a second denial must be 429.

## Verification (exact commands and expected output)

Test Postgres (this workstation has none):

```
ssh -f -N -L 5499:127.0.0.1:5432 thinkcentre
export DATABASE_URL=postgres://streakforge:<pw>@127.0.0.1:5499/streakforge
cd backend && cargo test --test integration --test routes --test streak_edge_cases
```
expected: `test result: ok` for all three binaries, 0 failed. Run the new routes test alone
before the fix and it must FAIL on the waste-after-denial assertion (RED), after the fix PASS
(GREEN).

Frontend:
```
cd web && node node_modules/vitest/vitest.mjs run     # expected: all green
./node_modules/.bin/svelte-kit sync && ./node_modules/.bin/svelte-check --tsconfig ./tsconfig.json
```
expected: `0 errors, 0 warnings` (or only the pre-existing count).

Live (the only evidence that counts for a behavioural ask):
```
./scripts/deploy.sh thinkcentre     # 16/16 endpoint health checks
```
then a probe account on `https://streakforge.polarisocial.xyz`:
`POST /api/logs kind=denial` → 201, then `kind=habit` → 201, then `kind=denial` → 429
with the "already recorded a load wasted today" message; delete the profile over ssh and
read the row counts back.
