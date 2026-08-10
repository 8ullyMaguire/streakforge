# StreakForge — Session Context (Recovery Cheat Sheet)

> Read this first if you're a fresh agent (or a returning one after a cleared
> session). It summarizes everything, points to the deep docs, and lists the
> immediate next actions. Last updated: 2026-08-10.

---

## TL;DR

- **What**: StreakForge — habit & streak tracker with community leaderboards,
  wlw.grok.me-inspired dark theme (counter, marquee, red/gold, mono digits).
- **Stack**: Rust (axum) + SvelteKit (Svelte 5, adapter-static SPA) + PostgreSQL.
- **Where**: local repo `~/code/streakforge`; production on thinkcentre
  (192.168.1.13) port 8001 (systemd `streakforge.service`); private mirror on
  opencommit.eu/MagicZhang/streakforge.
- **Status**: fully working + verified. 6 backend tests, 24 frontend tests,
  deployed, pushed. Everything committed (`main` at `c226861`).

---

## Quick facts

| Fact | Value |
|------|-------|
| Repo | `/home/alvaro/code/streakforge` |
| Backend dev port | 127.0.0.1:8787 |
| Prod port | 127.0.0.1:8001 (thinkcentre) |
| DB dev | local `streakforge` / `streakforge_dev` |
| DB prod | thinkcentre `streakforge` / `streakforge_prod` |
| Frontend build | `web/build` (adapter-static), served by Rust |
| Rate limits | 1/hr + 5/day per kind (habit | affirmation) |
| Git remote `github` | https://opencommit.eu/MagicZhang/streakforge.git (private) |
| Docs | `docs/SPECIFICATION.md` (1058 lines), `docs/ARCHITECTURE.md`, `docs/OPERATIONS.md` |
| Status file | `STATUS.md` (keep updated) |

---

## What was built (chronology)

1. **Core tracker** (`7a938ce`) — profiles, habit_logs, streaks (gaps-and-islands),
   daily/weekly/alltime leaderboards, UOTD, feed, X OAuth + dev login, wlw theme,
   landing/dashboard/leaderboard/feed/profile/settings.
2. **Scripts** (`706dce1`) — `scripts/dev.sh`, `scripts/seed.sql`.
3. **Manifesto** (`71b52a2`) — `/manifesto` page + `/api/manifesto*` serving 9
   markdown docs from `manifestos/`; custom markdown renderer (XSS-safe).
4. **Affirmation Drill** (`8e01e64`) — `/drill` page with wlw counter + affirmation
   deck; `kind` column on habit_logs; per-kind rate limits; drill reps excluded from
   community surfaces.
5. **Deploy** (`1e99bce`, `c226861`) — `scripts/deploy.sh`; systemd service on
   thinkcentre port 8001 (mirrors fichub on 8000; nginx inactive there).
6. **Docs** — this set + expanded README.

---

## Where things live (key files)

- Backend API: `backend/src/api.rs` (all handlers, DTOs, rate limits).
- Auth: `backend/src/auth.rs`.
- Router/startup: `backend/src/main.rs`.
- Schema/functions: `backend/migrations/0001_init.sql`, `0002_leaderboards.sql`,
  `0003_affirmations.sql`.
- Frontend theme: `web/src/app.css`.
- API client: `web/src/lib/api.ts` (types in `types.ts`).
- Affirmation deck: `web/src/lib/affirmations.ts`.
- Markdown renderer: `web/src/lib/markdown.ts`.
- Manifesto content: `manifestos/*.md` (9 docs).
- Deploy: `scripts/deploy.sh`, prod .env on thinkcentre.

---

## Verified state (do not re-verify everything)

- `cargo test --test integration` → 6 passed.
- `npx vitest run` → 24 passed. `npm run check` → 0 errors.
- Prod: root/drill/api 200, unauth 401, dev-login disabled, port 8001, session
  secret set.
- Repo pushed private: opencommit.eu/MagicZhang/streakforge (id 241, main @ c226861).

---

## Immediate next actions (if continuing)

1. **Cloudflare route** (user-side, dashboard): add route → `http://192.168.1.13:8001`.
2. **(Optional) session-key fix** — SPEC §25: `Key::from(cfg.session_secret.as_bytes())`
   so sessions survive restarts.
3. **(Optional) real X OAuth** — set X_CLIENT_ID/SECRET + PUBLIC_URL to tunnel URL.
4. **(Optional) HTTPS** — `SECURE_COOKIES=1` once behind the tunnel.

---

## Conventions to respect

- Conventional Commits, small units.
- **Run only targeted tests** (`cargo test --test integration`, `npx vitest run <file>`)
  — never full suites (cargo ~10min).
- Keep STATUS.md + docs updated after features.
- Mirror push to `github` remote after merges.
- Don't touch fichub on port 8000.

---

## Gotchas worth remembering (full list in SPEC §22)

- axum 0.8: `{capture}` routes, not `:capture`.
- tower-sessions 0.14 pairs with sqlx-store 0.15.
- Never edit applied migrations (checksum lock).
- CSS `//` comments break `:root` in the browser — use `/* */`.
- `user_streak()` returns INT4; cast `::bigint` in Rust queries.
- Dev-login button requires `VITE_ALLOW_DEV_LOGIN=1` at build time.
- Sessions invalidate on restart (ephemeral key).

---

*Deep dives: SPECIFICATION.md (what/why), ARCHITECTURE.md (how), OPERATIONS.md (ops).*
