# Plan — recover StreakForge features (2026-11-16)

Source: `docs/specs/2026-11-16-redeploy-lost-features.md`.

## 1. Reproduce

On thinkcentre: direct port 8001 answers 200 for `/admin`, `/metrics`, `/affirmations`; port 8035 (nginx) answers 404
for the same paths because `@streakforge_spa` allowlists only ten routes. `grep -rl 'HOW THESE ARE COUNTED'
/var/www/streakforge/_app` → no output. Deployed and freshly built backend SHA-256 are identical; migrations through
0022 are applied.

## 2. Build the frontend from HEAD

```
cd ~/code-local/rust/streakforge/web && node node_modules/vite/bin/vite.js build
```

Verify: local `web/build/_app/version.json` changes from `1793306982817`; `grep -rl 'HOW THESE ARE COUNTED'
web/build/_app` names a hashed node. Do not rebuild the backend: its deployed hash already matches HEAD.

## 3. Deploy without changing .env

Explicit rsync (NOT `scripts/deploy.sh`, which rewrites `SECURE_COOKIES=1` and would sign everyone out):

```
rsync -az --delete web/build/ thinkcentre:/var/www/streakforge/
# then patch the route allowlist in /etc/nginx/sites-available/streakforge and its mirror:
# (app|dashboard|denial|doctrine|drill|feed|leaderboard|login|settings)
# becomes the full set from web/src/routes, including admin/affirmations/cage/glossary/kpi/log/metrics/moderation/privacy/terms
ssh thinkcentre 'sudo nginx -t && sudo systemctl reload nginx'
```

Verify: `grep -aoc /api/admin/numbers /opt/streakforge/streakforge-api` non-zero; the three UI strings found in
`/var/www/streakforge/_app`; `ActiveEnterTimestamp` moves; `systemctl is-active streakforge` → active.

## 4. Live verification

Anonymous routes: `/api/health`, `/api/total`, `/api/doctrine`, `/api/affirmations`, `/`, `/admin`, `/metrics`,
`/affirmations` → 200 or the expected 401 for gated APIs.

Leader probe: grant `is_leader` to a throwaway profile over SQL without printing secrets, call
`/api/admin/numbers` and check real keys (`members`, `active_7d`, `conversion_rate`), revoke and delete, then read
counts back. Browser check: clear service worker/cache, load `/admin` and `/metrics`, confirm the new cards and copy.

## 5. Record

Prepend a dated entry to `STATUS.md` with before/after and the exact evidence. Commit the spec, plan and status
together; push `github main`. Report the hash and every command's real output.
