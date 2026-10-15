# Plan: the multiplier follows daily participation, not one kind's run

**Reported defect.** A member who shows up every single UTC day but alternates
what she does — waste Monday, denial Tuesday, drill Wednesday — earns no
multiplier. It reads 1.0x, the same as someone with no unbroken run at all,
because the multiplier is read off the *exclusive* streak
(`user_streak_kind(<the kind of the most recent log>)`), so any change of kind
resets the run to 1 and the tier collapses with it.

**Required behaviour (the owner's words).** "Only the multiplier — the streak
itself should reset, but the multiplier should remain so long as the user
participates every day." So: the exclusive streak keeps its present meaning
(consecutive days of the one kind you last chose, and it may reset on a change
of kind); the **multiplier** moves onto consecutive days of *presence*, any
kind, drill included.

**Reproduced before the change** (rolled-back probe against the live DB, 8 days
present in every case):

```
scenario                                            days_present  active_kind  streak  multiplier
A: alternating waste/denial, 8 days                            8  habit              1         1.0   <- defect
B: waste daily, one drill day, 8 days                          8  habit              3         1.0   <- defect
C: 7 days present, 2-day gap (must NOT reach 1.25)              7  habit              0         1.0   <- correct
```

After the change A and B must read 1.25 and C must stay 1.0.

## Design

- New `public.user_participation_streak(p_user)` — the same gaps-and-islands
  shape and the same today-or-yesterday rule as `user_streak_kind`, with the
  kind filter removed, so a day counts if **any** log exists on that UTC date.
- `public.exclusive_streak(p_user)` keeps its signature and its
  `active_kind`/`streak` columns; only its `multiplier` column changes source.
  One multiplier stays in the platform — the score's — so no future caller can
  pick the wrong one.
- **Monotonic:** the participation streak counts a superset of the days any
  per-kind streak counts, and keeps the same "today may still be empty" rule,
  so no multiplier can fall on deploy and nobody loses score.
- No Rust change: `api.rs` already reads `devotion_index`'s multiplier and
  `exclusive_streak`'s `active_kind`/`streak`, and both keep their shapes.

## Files, in order

1. `backend/migrations/0012_participation_multiplier.sql` — the two functions.
2. `backend/tests/integration.rs` — three new tests, one comment corrected.
3. `backend/tests/verify_devotion.sql` — CASE 4/5/6 expectations, one new CASE.
4. `docs/SPECIFICATION.md` — §7.1b + the §7.2 multiplier line.
5. `web/src/lib/glossary.ts` — the multiplier, active-streak and drill entries.
6. `web/src/routes/leaderboard/+page.svelte`, `web/src/routes/+page.svelte` —
   the two copies that say the multiplier comes from the exclusive streak.
7. `docs/plans/2026-10-15-participation-multiplier.md` — this file.

## Verification, step by step

| # | Command | Expected |
|---|---|---|
| 1 | `psql "$DATABASE_URL" -f <probe>` (rolled back) | A 1.25, B 1.25, C 1.0, streaks unchanged at 1 / 3 / 0 |
| 2 | `cargo test --test integration` | 115 passed / 0 failed (112 + 3 new) |
| 3 | mutation: point the multiplier back at `user_streak_kind` | the 3 new tests red, every neighbour green |
| 4 | `psql -f backend/tests/verify_devotion.sql` | CASE 4 = 1.25, CASE 5 = 11.25, new CASE = 1.25 |
| 5 | `cd web && node node_modules/vitest/vitest.mjs run` | 118 passed (118 before) |
| 6 | `node node_modules/vite/bin/vite.js build` + `svelte-check` | build ok, 0 errors |
| 7 | `scripts/deploy.sh thinkcentre` | EXIT=0, migrations applied, service up |
| 8 | live: `curl /api/leaderboard` after seeding nothing | multiplier column present, shapes unchanged |

## Not doing (and why)

- No change to the exclusive streak's meaning, and no new UI panel: the ask is
  the multiplier's basis, and the board already prints `streak` + `×multiplier`.
- No per-kind multiplier column kept alongside: two multipliers on one row is
  how a future caller picks the wrong one.
- The daily budget (5 WLW/day), the sybil floor and `user_of_the_day` are
  untouched.
