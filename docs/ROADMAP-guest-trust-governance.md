# Guest access, trust levels and self-governance — streakforge

Status: **partly implemented — GS-3 (consent) shipped 2026-10-27**, GS-1/GS-2 are still proposals.
The estate-wide standard (items GS-1…GS-5, every setting configurable, defaults and firewalls) is:

    secondbrain/10-Projects/2026-10-16-guest-access-trust-and-self-governance.md

This file is the site-specific half: what exists here today, what this site adopts, and the
firewalls that differ from the standard because this site is not the same shape as the others.

Stack: Rust + SvelteKit (web/).

## Where streakforge stands today (measured 2026-10-16, updated 2026-10-27)

- accounts exist; no role or trust columns yet (no `trust_level`/`moderator`/`is_admin` hits)
- no guest identity
- **GS-3 consent gate is live** (2026-10-27): see *GS-3, as built* below

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
