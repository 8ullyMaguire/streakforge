# Plan — cage checks, trust/quorum moderation, time-to-cum, public confessions, one-click logging

Spec: `docs/specs/2026-10-29-cage-checks-trust-time-to-cum.md` · Date: 2026-10-29

## Order of work

1. Migrations (three files, append-only; never edit an applied one).
2. Backend modules, then router wiring.
3. Tests: SQL-function integration + router tests. RED first where the behaviour is user-visible.
4. Frontend: landing one-click, `/cage`, `/metrics`, `/moderation`, dashboard extras, leaderboard
   boards, settings toggle, feed confessions, nav.
5. Build, deploy, live verify, docs and roadmap, commit, push.

## 1. Migrations

### `backend/migrations/0015_cage_checks.sql`
- `cage_checks` and `cage_attestations` per spec §6. `check_date` is a generated column
  (`(checked_at at time zone 'utc')::date`) with `unique (user_id, check_date)`; `claimed_secs`
  `check (between 0 and 259200)`; `status` in `pending|verified|disputed`.
- Indexes: `(status, checked_at desc)`, `(user_id, checked_at desc)`.
- `cage_check_hours(p_user uuid)` → `checks, claimed_secs, verified_secs, pending_secs,
  disputed_secs, verification_rate float8, avg_secs float8, current_verified_day_streak int,
  longest_verified_day_streak int, last_check_at timestamptz` — verified/disputed exclude
  `hidden_at is not null`; day streaks use the row_number date-gap trick already in 0005.
- `total_verified_cage_hours()` → bigint (excludes hidden).

### `backend/migrations/0016_trust_and_moderation.sql`
- `alter table profiles add is_leader bool not null default false`.
- `alter table habit_logs add hidden_at timestamptz`.
- `flags`, `moderation_actions` per spec §6.
- `trust_level(p_user uuid)` → `level int, active_days int, contributions int, upheld_flags int,
  next_level_at int`, implementing spec §2 exactly; contributions = logs + attestations;
  upheld = hidden logs + hidden checks owned, windowed 30/90 days by level.
- `flag_quorum(p_target_kind, p_target_id)` → `flags int, from_tl2 int` (used by the handler and by
  tests).
- `hide_target(kind, id)` / `restore_target(kind, id)` small SQL helpers so the route test and the
  handler run the same statement.

### `backend/migrations/0017_time_to_cum_and_public_logs.sql`
- `alter table habit_logs add time_to_cum_secs int` with a 0..86400 check.
- `alter table profiles add logs_public bool not null default true`.
- `activity_feed(p_cursor, p_limit)` recreated: `habit` **and** `denial` kinds, `note` and `kind`
  returned, `where p.logs_public and l.hidden_at is null`, same cursor semantics.
- `time_to_cum_stats(p_user)` → `samples, avg_secs, median_secs, min_secs, max_secs, last_secs`.
- `time_to_cum_aggregate()` → `samples, avg_secs, median_secs, avg_of_user_avgs_secs, reporters`.
- `time_to_cum_board(p_limit)` → `username, avatar_url, samples, avg_secs` (min 5 samples).
- `most_denied_board(p_limit)`, `consistency_board(p_limit)` (30d % of days), `lock_streak_board(p_limit)`.
  All exclude `hidden_at`.

## 2. Backend

| file | contents |
|---|---|
| `backend/src/cage.rs` | `CageResponse`, `CageCheckRequest`, `CageCheckItem`, `AttestRequest`; handlers `get_cage`, `create_check`, `queue`, `attest`; seal code from `rand` (8 hex); claimed-secs computation; quorum application |
| `backend/src/moderation.rs` | `TrustResponse`, `FlagRequest`, `ActionRequest`, queue DTOs; handlers `get_trust`, `flag`, `queue`, `action`; quorum + leader rules; audit rows |
| `backend/src/metrics.rs` | `TimeToCumResponse` (aggregate, mine, board), `BoardsResponse` (most denied, consistency, locks); handlers `time_to_cum`, `boards` |
| `backend/src/api.rs` | `LogRequest.time_to_cum_secs` (validated); `PATCH /api/logs/{id}` (`update_log`) and `DELETE /api/logs/{id}` (`delete_log`) with the 15-minute window; feed gains `kind` + `note`; `UpdateProfileRequest.logs_public` + `ProfileResponse.logs_public`; `StatsResponse` gains `time_to_cum` (own), `kind_streaks` (habit/denial/affirmation), `at_risk` (multiplier tier + secs left), `next_milestone` |
| `backend/src/router.rs` | wire the eight new routes + PATCH/DELETE on `/logs/{id}` |
| `backend/src/lib.rs` | `pub mod cage; pub mod moderation; pub mod metrics;` |

Validation: `time_to_cum_secs` 0..86400 → 400 otherwise; PATCH/DELETE only own log and only inside
900 s (`429`-adjacent 403 with the window named).

## 3. Tests (RED before the fix, then green)

- `backend/tests/integration.rs`
  - cage: claim cap (72h), one per UTC day (unique), verified hours only after quorum, dispute removes
    hours, verified-day streak math.
  - trust: level 0 → 1 → 2 progression from rows; `is_leader` = 4; upheld flag demotion window.
  - quorum: 3 flags (≥1 TL2) hides; 2 TL2 restores; 1 TL4 hides alone; audit rows written.
  - time to cum: per-user avg/median, aggregate pooled vs per-user average, min-5 board cutoff.
  - feed: public by default, `logs_public = false` hides the row, hidden log absent, drill absent,
    denial present with its kind.
- `backend/tests/routes.rs` (through the real router)
  - `POST /api/logs` with `time_to_cum_secs`, then `PATCH` inside the window sets/clears it;
    `DELETE` removes the row and `/api/stats` count drops; after 15 min (row backdated) both 403.
  - `POST /api/cage-check` with no lock → 409; with a lock → 201 with an 8-hex seal; second same
    day → 429; attest from the owner → 403; two TL1 peers → check `verified` and
    `/api/cage` verified_hours > 0.
  - `GET /api/feed` carries `note` for a default profile, and not for `logs_public=false`.

## 4. Frontend

| file | change |
|---|---|
| `web/src/routes/+page.svelte` | one-click `WASTE A LOAD` / `I DENIED`; inline "time to cum?" chips (5m/15m/30m/1h/custom) PATCHing the new log; undo button; signed-out state keeps one CTA |
| `web/src/routes/cage/+page.svelte` | new: lock state, cage check button with the seal code, verified-hours grid, verify queue (confirm/dispute), stale-lock check-in |
| `web/src/routes/metrics/+page.svelte` | new: time-to-cum aggregate + mine + longest-edge board; verified cage hours total |
| `web/src/routes/moderation/+page.svelte` | new: my trust level + progress, flag queue, hide/restore actions |
| `web/src/routes/dashboard/+page.svelte` | at-risk banner, per-kind streaks, milestone progress, return-after-gap banner |
| `web/src/routes/leaderboard/+page.svelte` | second section: most denied, consistency, lock streak, longest edge |
| `web/src/routes/settings/+page.svelte` | `logs_public` toggle with the honest explanation |
| `web/src/routes/feed/+page.svelte` | render confession text + kind; copy states public-by-default |
| `web/src/lib/components/Navbar.svelte` | `CAGE`, `METRICS` links; `ORDER` link for TL ≥ 2 |
| `web/src/lib/{types,api}.ts` | new DTOs + client calls |
| copy: `glossary.ts`, `terms`, `app.html` meta, landing sub | "private notes" → public-by-default with an opt-out |

## 5. Verification

Backend (this workstation has no Postgres):
```
ssh -f -N -L 5499:127.0.0.1:5432 thinkcentre
export DATABASE_URL=postgres://streakforge:<pw>@127.0.0.1:5499/streakforge   # suffix-swap the prod role/db
cd backend && cargo test
```
expected: every binary `test result: ok`, 0 failed; the new tests named above appear by name.

Frontend:
```
cd web && node node_modules/vitest/vitest.mjs run
# svelte-check via a scratch .sh (the lifecycle guard refuses the 1MB+ bin directly)
```
expected: vitest all green, `svelte-check found 0 errors and 0 warnings`.

Live:
```
./scripts/deploy.sh thinkcentre      # 16→ health checks, add the new endpoints to the list
```
then a probe account: one-click waste 201 → PATCH time_to_cum 204 → feed shows the note → set
`logs_public=false` → feed no longer shows it → lock → cage check 201 with seal → second account
attests twice (needs two accounts at TL1: backdate `created_at` for the probe only, then delete).

Cleanup: delete probe profiles/logs/checks, read the counts back to their pre-probe values.
