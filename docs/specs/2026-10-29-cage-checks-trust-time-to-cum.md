# StreakForge — cage checks, peer moderation, time-to-cum, public confessions (spec)

Date: 2026-10-29 · Status: approved by the owner in-session · Repo: `~/code-local/rust/streakforge`

Owner's asks, verbatim where they were explicit:

1. "add cage check verified by users themselves; total verified cage hours and similar metrics"
2. "all moderation through discourse-style trust levels or quorum"
3. "time to cum, average time to cum per person, aggregate average, etc" — clarified: **"time it takes
   from start jerking to release"**, i.e. a per-log session duration, not the gap between releases
4. "logging waste/denial from landing page should be a single click rather than multiple"
5. "confessions/logs are public by default, user can choose to make them private through a setting"
6. fold the rest of the 20-idea brainstorm into the roadmap, implement what is worth implementing now

## 1. Cage checks — self-reported, peer-verified

A **cage check** is a timestamped claim that the cage is still on, carrying a **seal code** the site
issues at the moment the check is created. The user is told to write that code somewhere only a
person actually wearing the cage could reach (a card next to the lock, a note on the photo they keep
privately). Peers attest; nothing is uploaded, nothing is scanned, and no photo ever touches this
server.

| concept | definition |
|---|---|
| check | one per user per UTC day, only while a lock session is open |
| claimed hours | `min(now − previous check (or lock start), 72h)` — a longer silence cannot be banked as "hours" |
| attestation | a `confirm` or `dispute` from another user, one verdict per attester per check |
| verified | 2 confirming attestations from distinct users at trust level ≥ 1, or 1 from trust level ≥ 3 |
| disputed | any `dispute` from trust level ≥ 2 → hours leave the verified total until restored to verified by the same quorum that verifies |
| verified hours | the sum of `claimed_secs` over `status = 'verified'` checks; this is the metric that means something |

Refusals: no open lock → 409 ("nothing to check"); a second check the same UTC day → 429; attesting
on your own check → 403; attesting twice → 409 (one verdict, changeable with PATCH semantics through
the same endpoint).

Metrics (`cage_check_hours(p_user)`): total claimed, total verified, total pending, total disputed,
verification rate, checks count, average hours per check, current and longest run of **consecutive
verified days**, last check time. Global: `total_verified_cage_hours()`.

## 2. Moderation — Discourse-style trust levels, quorum for everything

Trust is **computed**, never stored (except the leader flag), from what an account demonstrably did:

| level | requirement | what it unlocks |
|---|---|---|
| 0 | any account | reading, logging |
| 1 | ≥ 24h old, ≥ 3 logs (any kind), no upheld flag in 30d | flagging content, attesting cage checks |
| 2 | ≥ 7 distinct active days, ≥ 10 contributions (logs + attestations), ≤ 1 upheld flag in 30d | moderation queue, restore votes |
| 3 | ≥ 30 distinct active days, ≥ 40 contributions, 0 upheld flags in 90d | a single attestation verifies a cage check |
| 4 | `profiles.is_leader` (manual, DB-granted) | unilateral hide and restore, all audited |

> **Superseded on the numbers, not on the shape (0019/0020).** The rungs above are the *grown* band.
> While fewer than ten members are active in a given week the ladder is softened to
> L1 1 day/1 log, L2 2 days/3 active days/5 contributions, L3 7 days/8 active days/15 contributions,
> and it tightens back by itself. `/api/trust` publishes the rungs in force, and the moderation page
> prints those rather than any copy of this table.

Quorum rules:

- **Auto-hide**: 3 distinct flags on one target, at least one from TL ≥ 2 → target hidden, audited as
  `auto_hide` with a null actor.
- **Restore**: 2 distinct restore votes from TL ≥ 2, or one action from TL 4.
- **Manual hide**: TL 4 only (TL 2–3 express that through flags, which is the quorum path).
- Every hide and restore writes `moderation_actions`; nothing is deleted, ever.

Targets: `log`, `cage_check`, `profile`. A hidden log leaves the public feed and the metrics; its
owner still sees it, and it still counts toward that owner's own score (hiding is about **visible
content**, not about rewriting someone's history — documented so nobody "fixes" it later).

## 3. Time to cum

Per log, optional `time_to_cum_secs` (0 … 86400): how long the session ran, start of stimulation →
release. Set at log time (API field) or afterwards within **15 minutes** (PATCH), which is also the
undo window (DELETE). Metrics:

- per person: samples, average, median, fastest, slowest, last
- aggregate: pooled average and median across every reported log, plus the **average of per-person
  averages** (the number the owner asked for separately — they differ when one person logs a lot)
- board: `longest edges` — the highest per-person average, 5-sample minimum so one 4-hour outlier is
  not a ranking strategy.

## 4. Confessions are public by default; private is a setting

`profiles.logs_public boolean not null default true`. The public feed carries `kind` and `note` for
logs whose owner has not turned it off, and excludes hidden logs and drill reps. Turning it off is
one setting (`PATCH /api/profile {logs_public:false}`), the copy everywhere stops promising "private
notes", and the glossary/terms/feed/landing sentences say the new default out loud.

This reverses item 01 of `docs/100_IDEAS.md` (which removed notes from the feed because they had never
been opt-in). The reasoning that made it right then — *no default the user never saw* — is honoured
by a visible setting that is stated on the log surfaces, not by silence.

## 5. One click from the landing page

`WASTE A LOAD` and `I DENIED` on `/` log immediately: one POST, no confirm page, no drill gate, no
navigate. The response offers two optional follow-ups in place — `time to cum?` (a duration picker
that PATCHes the just-written log) and `undo` (DELETE within 15 min). The drill stays as its own
practice on `/drill`; it is no longer a gate in front of logging.

Signed-out visitors get the same two buttons and a sign-in/guest prompt; the click still costs one
action after they are identified, because the server refuses an anonymous write by design.

## 6. Data model (deltas)

```
profiles      + logs_public bool not null default true
              + is_leader   bool not null default false
habit_logs    + time_to_cum_secs int null check 0..86400
              + hidden_at timestamptz null
cage_checks   id, user_id, lock_id, checked_at, check_date (generated, UTC),
              seal_code, note (140), claimed_secs (0..259200), status, verified_at, hidden_at
              unique (user_id, check_date)
cage_attestations  id, check_id, attester_id, verdict ('confirm'|'dispute'), created_at
                   unique (check_id, attester_id)
flags         id, target_kind ('log'|'cage_check'|'profile'), target_id, reporter_id,
              reason (140), created_at, unique (reporter_id, target_kind, target_id)
moderation_actions  id, actor_id null, action ('hide'|'restore'|'auto_hide'), target_kind,
              target_id, note, created_at
```

## 7. Endpoints

| method | path | auth | notes |
|---|---|---|---|
| POST | `/api/logs` | user | `time_to_cum_secs` optional |
| PATCH | `/api/logs/{id}` | owner, ≤15 min | set/clear `time_to_cum_secs`, note |
| DELETE | `/api/logs/{id}` | owner, ≤15 min | undo a mis-click |
| GET | `/api/cage` | user | lock, my check metrics, today's check, queue count |
| POST | `/api/cage-check` | user, locked | creates today's check, returns the seal code |
| GET | `/api/cage-check/queue` | user | other people's pending checks + my verdict |
| POST | `/api/cage-check/{id}/attest` | TL ≥ 1, not owner | `confirm` or `dispute`, applies quorum |
| GET | `/api/metrics/time-to-cum` | public | aggregate + board (+ mine when signed in) |
| GET | `/api/metrics/boards` | public | most-denied, consistency, lock streak |
| GET | `/api/trust` | user | level, progress, contributions |
| POST | `/api/flags` | TL ≥ 1 | idempotent per reporter/target |
| GET | `/api/moderation/queue` | TL ≥ 2 | flagged targets with counts |
| POST | `/api/moderation/action` | TL ≥ 2 (restore), TL 4 (hide) | quorum or leader |
| PATCH | `/api/profile` | user | `logs_public` |

## 8. Decisions taken (no further approval requested)

- **Seal codes are 8 hex chars, server-issued, shown once per check** — non-replayable without being
  a stored secret, and verifiers compare them, so a copied check is visible as a duplicate code.
- **72h cap** on claimed hours: a week of silence must not become a week of "verified" time.
- **TL3 verifying alone** is deliberate: it keeps a dead 2-person quorum from stalling once the base
  is small (it is: 3 active accounts).
- **Hidden content keeps scoring** (see §2) — the alternative silently edits people's history.
- **Feed now carries denials as well as wastes** (with `kind`), because denials are the 10× event and
  the feed ignoring them was a hole, not a policy.
- **Drill is no longer a gate**: the typed affirmation stays as practice and as the game bonus, but
  the one-click ask wins by owner instruction.

- **The guest fold had to learn the new tables.** `src/fold.rs` walks `pg_constraint` and refuses to
  register a guest whose rows it cannot re-parent, so four new foreign keys into `profiles(id)`
  surfaced as a 500 on `POST /api/auth/register` until each was classified:
  `cage_checks.user_id` and `cage_attestations.attester_id` **fold** (own tenure — and 0016 counts
  attestations towards `contributions`, so dropping them would reset a converting member's trust to
  level 0), while `flags.reporter_id` and `moderation_actions.actor_id` **stay with the tombstone** — a
  vote about someone else's content, or an action log, must not be re-cast under a new name (the flag
  index is per reporter; the audit precedent is `audit_log.profile_id`). Covered by
  `tests/guest.rs::the_fold_moves_cage_tenure_and_leaves_votes_with_the_tombstone`.

## 9. Out of scope this pass (in the roadmap, with reasons)

Outbound notifications (no transport), share cards (no image rendering), public API tokens + embed
badge, referral loop, doctrine RSS/pages, keyholder link, time trials. See `docs/100_IDEAS.md` (scored 0–1, impact ÷ effort).
