# StreakForge — Operations

How to run, deploy, monitor, and troubleshoot StreakForge. Covers local dev and the
ThinkCentre production instance.

---

## 1. Environments

| Environment | Host | Backend | DB | Frontend | Auth |
|-------------|------|---------|----|----------|------|
| local dev | this machine (gamingpc/dev box) | `127.0.0.1:8787` | local Postgres, `streakforge` / `streakforge_dev` | `web/build`, dev-login ON | dev-login (+ X if creds set) |
| production | thinkcentre (192.168.1.13) | `127.0.0.1:8001` (systemd) | ThinkCentre Postgres, `streakforge` / `streakforge_prod` | `/var/www/streakforge`, dev-login OFF | X only |

---

## 2. Local Development

### 2.1 Prereqs on the dev machine

- Rust stable (rustup), cargo.
- Node 20+ / npm.
- PostgreSQL running locally with role/db:
  ```bash
  sudo -u postgres psql -c "CREATE ROLE streakforge WITH LOGIN PASSWORD 'streakforge_dev' CREATEDB;"
  sudo -u postgres psql -c "CREATE DATABASE streakforge OWNER streakforge;"
  sudo -u postgres psql -c "CREATE DATABASE streakforge_test OWNER streakforge;"
  ```
- (Optional) X OAuth app credentials for real login.

### 2.2 One-shot dev

```bash
./scripts/dev.sh
# builds web (dev-login on), runs cargo run on :8787
```

### 2.3 Manual dev (two terminals)

```bash
# terminal A — backend
cd backend
ALLOW_DEV_LOGIN=1 \
  WEB_BUILD_DIR="$(pwd)/../web/build" \
  MANIFESTOS_DIR="$(pwd)/../manifestos" \
  RUST_LOG=info \
  cargo run

# terminal B — frontend (with HMR proxy to :8787)
cd web
npm install
npm run dev          # vite on :5173, proxies /api → 127.0.0.1:8787
```

> Use `npm run dev` (vite) during iteration; build once for production-style testing.

### 2.4 Seed demo data

```bash
psql -h 127.0.0.1 -U streakforge -d streakforge -f scripts/seed.sql
```

Creates iron_will (10-day streak), daily_dave (6-day), streak_queen (broken 12-day),
noob_forger, late_night (3 today).

### 2.5 Tests

```bash
cd backend
DATABASE_URL=postgres://streakforge:streakforge_dev@127.0.0.1:5432/streakforge_test \
  cargo test --test integration        # 6 tests, ~0.3s

cd ../web
npm run check                          # svelte-check
npx vitest run src/lib/api.test.ts     # targeted file(s) — 24 total
```

---

## 3. Production Deployment (ThinkCentre)

### 3.1 Deploy script

```bash
./scripts/deploy.sh thinkcentre
```

Steps (see also SPEC §17): build release, build web (no dev login), rsync binary +
migrations + manifestos → `/personal/documents/code/streakforge`, rsync web/build →
`/var/www/streakforge`, write `.env`, install/enable/restart systemd unit.

After first deploy (or any redeploy), set a real session secret:

```bash
SECRET=$(openssl rand -hex 32)
ssh thinkcentre "sudo sed -i 's|^SESSION_SECRET=.*|SESSION_SECRET=$SECRET|' /personal/documents/code/streakforge/.env && sudo systemctl restart streakforge"
```

### 3.2 What lives where on thinkcentre

```
/personal/documents/code/streakforge/
├── streakforge-api          # release binary
├── migrations/              # SQL migrations
├── manifestos/              # manifesto markdown (MANIFESTOS_DIR)
└── .env                     # prod env (owned by alvaro)
/var/www/streakforge/        # frontend build (WEB_BUILD_DIR, chown alvaro)
/etc/systemd/system/streakforge.service
```

### 3.3 systemd unit (as installed)

```ini
[Unit]
Description=StreakForge habit tracker API (Rust)
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=alvaro
WorkingDirectory=/personal/documents/code/streakforge
EnvironmentFile=/personal/documents/code/streakforge/.env
Environment=PATH=/home/alvaro/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
ExecStart=/personal/documents/code/streakforge/streakforge-api
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

### 3.4 Cloudflare tunnel

- Tunnel client: `cloudflared.service` runs `tunnel run --token-file /etc/cloudflared/token`.
- **Routes are managed in the Cloudflare dashboard** (no host config file).
- To expose StreakForge: add a route (hostname or path) → `http://192.168.1.13:8001`.
- FicHub already uses the tunnel for its domain on port 8000.

### 3.5 nginx

nginx is installed but **inactive** on thinkcentre. StreakForge serves everything
itself on 8001 (like fichub on 8000). Do not add nginx configs unless that changes.

---

## 4. Monitoring & Health Checks

From any machine on the LAN:

```bash
# Service + port
ssh thinkcentre 'systemctl is-active streakforge; ss -tln | grep 8001'

# HTTP health (local)
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:8001/
curl -s http://127.0.0.1:8001/api/total

# Logs
ssh thinkcentre 'sudo journalctl -u streakforge -n 50 --no-pager'
```

Expected healthy: service `active`; root 200; `/api/total` returns `{"total":N}`;
port 8001 listening.

---

## 5. Troubleshooting

### 5.1 Backend won't start: VersionMismatch(N)

You edited an already-applied migration. Fix: `git checkout` the migration back to its
committed state, and put changes in a new numbered migration. (The DB's
`_sqlx_migrations` table records checksums.)

### 5.2 Route panic at startup: "Path segments must not start with `:`"

axum 0.8 requires `{capture}` syntax, not `:capture`. `/leaderboard/:period` → panic;
`/leaderboard/{period}` → fine.

### 5.3 All CSS vars empty / Times New Roman everywhere

The built CSS contains `//` comments (Vite doesn't strip them); browsers drop the
`:root` rule after an invalid `//` line. Fix: use `/* */` comments in `app.css`, rebuild.

### 5.4 Session cookie invalid after restart

Expected: the signing key is `Key::generate()` per boot (SPEC §25). Apply the
`Key::from(cfg.session_secret.as_bytes())` fix if you want persistent sessions.

### 5.5 `/api/auth/x` returns 400 "X login not configured"

`X_CLIENT_ID`/`X_CLIENT_SECRET` are unset. Either set them (and PUBLIC_URL to the
reachable URL) or use dev-login locally.

### 5.6 Manifesto 404 / missing docs

Check `MANIFESTOS_DIR` env points at the dir containing `NN_*.md`. The service on
thinkcentre uses `/personal/documents/code/streakforge/manifestos`.

### 5.7 Deploy: "Permission denied" on /var/www/streakforge

Run `sudo mkdir -p /var/www/streakforge && sudo chown -R alvaro:alvaro /var/www/streakforge`
once; the deploy script does this automatically now.

### 5.8 Port conflicts

- 8000 = fichub (do not touch).
- 8001 = streakforge (backend). If occupied, change `BIND_ADDR` in .env + restart.

### 5.9 SvelteKit build fails on Google Fonts / external links

Only relevant offline; fonts are cosmetic (fallback stacks in CSS).

---

## 6. Backup / Restore

- Postgres: `pg_dump` the `streakforge` DB.
  ```bash
  ssh thinkcentre 'PGPASSWORD=streakforge_prod pg_dump -h 127.0.0.1 -U streakforge streakforge' > streakforge.sql
  ```
- Static content (manifestos) is in the repo; rebuild from source.
- The repo is mirrored private on opencommit.eu (remote `github`).

---

## 7. Routine Tasks

| Task | Command |
|------|---------|
| Redeploy after feature | `./scripts/deploy.sh thinkcentre` + secret re-check |
| Check prod health | see §4 |
| Add manifesto doc | copy to `manifestos/`, redeploy |
| Update docs | edit `docs/*.md`, keep STATUS.md in sync |
| Push mirror | `git push github main` |

---

## 8. Incident Notes (log)

- 2026-08-10: deployed to thinkcentre; discovered nginx is inactive and fichub serves
  directly on 8000; streakforge mirrors on 8001 (no nginx config).
- 2026-08-10: migration checksum lock hit when 0002 was edited after apply — resolved
  by reverting 0002 and moving redefinitions to 0003.
- 2026-08-10: session secret placeholder replaced with random 64-hex on prod.
