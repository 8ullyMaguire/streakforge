# Guest access, trust levels and self-governance — streakforge

Status: **proposal**, from the owner's request of 2026-10-16. It records intent in this repo's own
terms; nothing here is implemented. The estate-wide standard (items GS-1…GS-5, every setting
configurable, defaults and firewalls) is:

    secondbrain/10-Projects/2026-10-16-guest-access-trust-and-self-governance.md

This file is the site-specific half: what exists here today, what this site adopts, and the
firewalls that differ from the standard because this site is not the same shape as the others.

Stack: Rust + SvelteKit (web/).

## Where streakforge stands today (measured 2026-10-16)

- accounts exist; no role or trust columns yet (no `trust_level`/`moderator`/`is_admin` hits)
- no guest identity, no consent gate

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
