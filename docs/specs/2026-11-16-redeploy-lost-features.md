# Spec — recover the lost StreakForge features by redeploying HEAD (2026-11-16)

## Problem

Reproduction changed the diagnosis. The deployed binary's SHA-256 equals the freshly built HEAD binary, it contains
the admin handler and all migrations through 0022 are applied — the backend is current. Two real defects remain:
1. The live nginx vhost (changed 2026-11-14) serves the shell only for an allowlist of client routes. That list omits
   `/admin`, `/affirmations`, `/metrics`, `/moderation`, `/cage`, `/glossary`, `/privacy`, `/terms`, `/kpi`, `/log`,
   so those pages return a real 404 even though the app routes exist. Direct `127.0.0.1:8001` answers 200 for all of
   them, proving the proxy is the blocker.
2. The deployed static bundle is older than the source: it lacks `HOW THESE ARE COUNTED`, `Nobody shares yet` and
   `withdraw mine`. A fresh vite build from HEAD contains all three.

## Goals

- Deploy the current frontend build from `main` at `9b6a891`; the backend needs no rebuild (hash already matches).
- Preserve `/opt/streakforge/.env` exactly: in particular do not flip `SECURE_COOKIES` as a side effect of this recovery.
- Verify the deployed bytes and the live routes, not just a green restart.
- Record the before/after in `STATUS.md` and commit/push the repo.

## Non-goals

- No new feature code, no migration edits, no schema changes, no backend redeploy (its hash already matches HEAD).
- No credential changes and no session-cookie policy change.
- No touches to suki, concord, darkhold or the estate substrate.

## Constraints

- Build the Rust binary locally with the repo's configured cargo target dir; build the frontend with the direct vite
  binary, not a package-manager wrapper.
- Deploy by rsync into the existing `/opt/streakforge` and `/var/www/streakforge` layout; restart with the existing
  `streakforge.service`.
- Clean up any probe account and read counts back.

## Success criteria

- The route allowlist contains every client route in `web/src/routes`; each page answers 200 through the proxy.
- The deployed bundle contains all three UI strings above; `/api/health`, `/api/total`, `/api/affirmations`,
  `/api/doctrine` and the key pages answer as before.
- A temporary leader can read `/api/admin/numbers` with real numbers; the grant is revoked and the probe deleted.
- `git status` clean after the documentation commit; commit and push named.
