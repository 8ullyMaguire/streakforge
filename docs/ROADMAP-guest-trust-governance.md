# Guest access, trust levels and self-governance — streakforge

Status: **GS-1 … GS-4 shipped (2026-10-27/28/29)**; GS-5 stays deferred (nothing to govern until the deck
and the feed need it — and the deck now has an admission rule of its own, below).
The estate-wide standard (items GS-1…GS-5, every setting configurable, defaults and firewalls) is:

    secondbrain/10-Projects/2026-10-16-guest-access-trust-and-self-governance.md

This file is the site-specific half: what exists here today, what this site adopts, and the
firewalls that differ from the standard because this site is not the same shape as the others.

Stack: Rust + SvelteKit (web/).

## Where streakforge stands today (measured 2026-10-16, updated 2026-10-27)

- accounts exist; no role or trust columns yet (no `trust_level`/`moderator`/`is_admin` hits)
- **GS-3 consent gate is live** (2026-10-27) and **GS-1 guest identity + GS-2 fold** (2026-10-28):
  see *GS-1 and GS-2, as built* and *GS-3, as built* below

## What this site adopts

- GS-1 … GS-3 now; GS-4 is the natural next thing here (streak data *is* a tenure signal), GS-5 only when user-generated content exists to moderate

## Config deltas (everything else is the standard's default)

- `trust.axes` should be built from the streak signal the site already computes — no second score
- `guest.mau_limit = 500`

## Firewalls specific to this site

- A streak is a personal record: folding a guest's streak into the new account is the feature's whole point, and `merge.grants_tenure = true` is what makes the first day not feel like day one.

## Order of work

GS-3 → GS-1 → GS-2 → GS-4 (GS-5 deferred until there is content to govern). The standard's §9
explains the ordering: consent first, because the guest identity *is* a cookie and the gate has to
exist before the thing it gates; the fold second, because offering guest access without it creates
work to clean up later.

## If this is implemented

Every step needs a test that fails without it, and the numbers belong in this file when they
are real — the estate's rule is that a claim of done carries the command that proves it. The
two items in the standard that nothing in this repo has yet are the elastic trust rule (§4)
and the jury drawn from the eligible pool (§5): both are cheap to fake and therefore need a
test that would pass for the wrong reason to be written, and killed, on purpose.

## GS-3, as built (2026-10-27)

Files: `backend/migrations/0013_consent_gate.sql`, `backend/src/consent.rs`,
`backend/src/audit.rs`, `web/src/lib/consent.ts`,
`web/src/lib/components/ConsentToast.svelte`, `web/src/routes/terms` (a section that says what is
stored), `backend/tests/consent.rs`.

- **The refusal is the API's.** `axum::middleware::from_fn_with_state` is applied to the whole `/api`
  router, so a route added later is gated by existing rather than by remembering to call a check.
  Mutating methods without consent answer `403 {"consent":"required","version":N}`; every read stays
  open.
- **Three places hold the answer**, because they answer three different questions: the tower-sessions
  session (does *this browser* have consent? — a signed cookie over a server-side row), 
  `profiles.consented_at` / `.consent_version` (does *this account*? — so a new device is not asked
  again), and `audit_log` (did they, and to which version? — answerable after the fact).
- **Exemptions, each with a reason in the code:** `POST /api/consent` (the key would be inside the
  locked door), `POST /api/auth/logout` (ending a session needs no permission), and
  `POST /api/auth/login`, which is exempt **from the middleware only** and enforces the identical
  rule in its handler, because the account's `consented_at` is the one legitimate second answer and
  the middleware cannot see an account.
- **Versioning:** `CONSENT_VERSION` (default 1). `>=` comparison while reading, so a visitor who
  accepted 2 is not re-asked by 1, and bumping it re-asks everyone below it.
- **Refusal is honoured:** no row, no persisted client flag, and the toast remembers only in
  `sessionStorage` for that tab — it asks again next visit, which is the honest behaviour for a site
  with an anonymous door.
- **`CONSENT_REQUIRED=0`** turns the gate off without a rebuild, because that is how a deployment
  answers an incident.

Evidence:

    DATABASE_URL=postgres://sf:test@127.0.0.1:5499/streakforge_dev cargo test
    # 22 + 11 + 51 + 10 + 7 = 101 passed, 0 failed (tests/consent.rs is the 11)

Browser-walked on the dev instance (`:8787`, built with the gate on): the toast renders behind the
age gate (one question at a time), declining hides it and sets `sf_consent_refused` in
`sessionStorage`, `POST /api/auth/register` from the page answers `403 {"consent":"required"}`, a
reload in the same tab does not re-ask, a fresh tab does, and accepting with `analytics` ticked
returns `{"accepted":true,"categories":{"analytics":true,...}}` — after which `/kpi`, `/leaderboard`,
`/doctrine` and `/terms` all render with no repeat ask.

What GS-3 does **not** do: it gates no guest identity, because GS-1 does not exist yet. Until it
does, the gate's only effect is on anonymous visitors who try to register or log in — which is
exactly the ordering the standard asks for (consent first, then the cookie it makes safe).

## GS-1 and GS-2, as built (2026-10-28)

Files: `backend/migrations/0014_guest_identity.sql`, `backend/src/guest.rs`,
`backend/src/fold.rs`, `backend/src/audit.rs` (MAU), `backend/tests/guest.rs`,
`web/src/routes/login` (the door), `web/src/lib/components/Navbar.svelte` (the marker).

**A guest is a real `profiles` row** with `is_guest = true` and **no credential at all** —
`password_hash` was already nullable, and that is what makes the row unloggable without inventing a
placeholder hash some future login path might accept. The identity is carried by the session, which
is the cookie; `POST /api/guest` is a mutation, so it goes through GS-3's gate, which is the ordering
the standard asks for made literal: the gate really does gate the identity.

**The door closes on MAU**, and the count is `public.monthly_active_accounts(window_days)` — distinct
accounts with a habit log *or* an audit row in the window, one definition in SQL rather than two in
Rust. `GUEST_MAU_LIMIT` (default 500; `0` shuts the door, it does not mean unlimited),
`GUEST_MAU_WINDOW_DAYS` (30), `GUEST_ENABLED`. At the cap: no new identities, no guest writes, reads
untouched (§1's grace). A subtlety the tests pin: **a mint is itself an action**, so a guest with a
limit of 1 is at the cap before it does anything.

**The fold is schema-driven and transactional.** `fold_guest` asks `pg_constraint` for every column
that references `profiles(id)`, fails on any it does not handle (`FoldError::Unhandled`), re-parents
`habit_logs` and `lock_sessions`, tombstones the guest (`folded_<uuid12>`, `merged_into` set,
credentialless), and runs on the *same transaction as the account insert*, so a fold that cannot
finish takes the account with it. Idempotent: `merged_into` is the fact, so a second fold reports
`folded = false` and moves nothing.

**Tenure, not score.** The streak is computed from the moved logs, so the new account's first day is
not day zero — that is `merge.grants_tenure = true` on this site, and it is the whole feature. There
is no trust ladder yet, so `merge.grants_trust` has nothing to grant; when GS-4 lands, the fold is
the function that must not touch it.

Evidence:

    DATABASE_URL=postgres://sf:test@127.0.0.1:5499/streakforge_dev cargo test
    # 22 + 11 + 12 + 51 + 10 + 7 = 113 passed, 0 failed   (tests/guest.rs is the 12)

    cd web && npm run check   # 0 errors, 0 warnings
    npx vitest run            # 128 passed
    npm run build             # clean

Browser-walked end to end on the dev instance: the door renders on `/login` beside the form with the
cap named; clicking it **without** consent is refused, the form says "Please choose your cookie
settings first" and the toast reopens (the `403 {"consent":"required"}` shape reaching the UI, not
just the API); accepting then mints, redirects, and the navbar shows `GUEST`; a logged waste counts
(`today=1 streak=1`); registering over that session lands on the new account with **the same
`today=1 streak=1 alltime=1`**, the navbar marker gone; in the database the guest row reads
`folded_44e79f7cf16d · is_guest · merged_into set · password_hash null` and its log now belongs to the
new account. Audit rows: `consent.accepted ×2`, `consent.refused ×1`, `guest.minted ×1`,
`guest.folded ×1`.

Not done: `merge.notify` (there is no notification channel on this site — the toast on arrival is the
only place a message could go, and it is not worth inventing a table for), `guest.upload_quarantine`
(no uploads), `guest.directory_access` (there is no member directory to withhold), and per-fingerprint
rate limiting beyond recording `guest_fingerprint` — the cap is the mechanism §1 actually names.

## GS-4 — deferred, and here is why that is not laziness

`trust.axes` would come from the streak signal the site already computes, and the elastic rule is the
interesting half: when nobody clears the nominal bar, the requirement falls to the top of the actual
ladder with `trust.elastic_min_holders` holders, and returns to nominal as soon as enough people clear
the real bar, with no ratchet.

**Nothing on this site would read it.** GS-4 exists to feed the privileges in `trust.require.<priv>`
— `moderate`, `jury`, `invite`, `annotate` — and this site has none of them: there is no moderation
queue, no content to govern (GS-5 is deferred for the same reason), and no invite system. A
`trust_level` that no route consults is the "a field with no reader" failure this estate has already
paid for twice (a `default_hashtag` with no consumer; a `collapsible` column with no collapse route).

So GS-4 lands **with the first privilege it gates**, not before it — most likely the day GS-5 acquires
something to govern. The elastic rule is the part that must not be faked: it needs a test that fails
for the wrong reason (an empty pool, a pool of one, and a pool that later grows past the nominal bar)
before it can be called implemented.

## GS-4, as built (2026-10-29) — and the elastic rule, now that there are privileges to gate

GS-4 landed with the privileges it was waiting for: flagging, cage attestation, the moderation queue,
restore votes, and (0019) admission to the affirmation deck. What was built, and what the deferred
note above asked to be proved:

- **`trust_level()` is a function, not a column** (0016): level, active days, contributions, upheld
  flags and the distance to the next rung, computed from logs and attestations. The only stored
  privilege stays `profiles.is_leader` (level 4), and every use of it writes a `moderation_actions` row.
- **The elastic rule, as a band** (0019/0020). The standard's shape was "the requirement falls to the
  top of the actual ladder when nobody clears the nominal bar"; the shape here is a site-wide band
  chosen by `active_accounts()` — distinct non-guest members with a log in the last **7 days**, floored
  at 1 — switching at ten members between a soft ladder (L2 at 48h/3 active days/5 contributions) and
  the nominal one (L2 at 7 days/7/10). It returns to nominal on its own and has no ratchet, because
  the band is computed at read time from activity, not stored anywhere.
- **The three tests the note demanded**, as the pool-grows test and its neighbours:
  `the_ladder_softens_while_the_site_is_small` (one member → soft; ten more active → nominal; pool
  empties → soft again), `active_means_this_week_not_this_month` (a member who logged 20 days ago is
  in the month's count and not in the week's), and `trust_level_1_is_earned_by_a_day_and_a_log`
  (the floor still costs a day, so a burst of logs cannot buy attestation rights).
- **The fold does not grant trust** (still true, deliberately): a guest's tenure moves with their logs
  and attestations, so the level is the same on the other side of registration, but nothing is
  awarded for registering.
- **The UI reads the function's numbers** (`/api/trust` ships `band`, `active_accounts`,
  `thresholds`): the moderation page printed "7 active days" for a level the database granted at 3
  until this pass, which is the "a field with no reader" failure wearing a different hat — a number
  with no reader is one thing, a number whose reader has its own copy is worse.
