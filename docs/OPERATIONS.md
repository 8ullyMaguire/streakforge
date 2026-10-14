# StreakForge — Operations

How to run, deploy, monitor, and troubleshoot StreakForge. Covers local dev and the
ThinkCentre production instance.

---

## 1. Environments

| Environment | Host | Backend | DB | Frontend | Auth |
|-------------|------|---------|----|----------|------|
| local dev | this machine (gamingpc/dev box) | `127.0.0.1:8787` | local Postgres, `streakforge` / `streakforge_dev` | `web/build` | local username/password (register/login) |
| production | thinkcentre (192.168.1.13) | `127.0.0.1:8001` (systemd) | ThinkCentre Postgres, `streakforge` / `streakforge_prod` | `/var/www/streakforge` | local username/password (register/login) |

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
- No API keys needed — auth is local username/password (Argon2id); see
  SPEC §9 for the register/login/nonce endpoints and bot-dissuasion.

### 2.2 One-shot dev

```bash
./scripts/dev.sh
# builds web, runs cargo run on :8787
```

### 2.3 Manual dev (two terminals)

```bash
# terminal A — backend
cd backend
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
DATABASE_URL=postgres://streakforge:***@127.0.0.1:5432/streakforge_dev \
  cargo test                           # 56 tests: 16 unit + 40 integration

cd ../web
npm run check                          # svelte-check
npx vitest run                         # 69 tests (8 files)
```

Counts re-measured 2026-10-14 (16/40); the 17/47 and 11/37 lines before that had
been stale in a third direction (the plan for the KPI dashboard quoted 19/48, also
wrong). Re-measure rather than trusting any of these numbers.

The integration tests need a Postgres the role can create databases in
(`#[sqlx::test]` makes a throwaway DB per test) — `DATABASE_URL` must point at the
cluster, and the database named in it may not exist.

**The role and database must exist before the suite will run.** `sqlx::test`
provisions a scratch database per test by cloning the one in `DATABASE_URL`, so
it needs a role with CREATEDB *and* an existing database to clone. Without them
every integration test fails with `role "streakforge" does not exist` or
`database "streakforge_dev" does not exist`, which reads as a broken repo and
is not one:

```bash
sudo -u postgres psql -c "CREATE ROLE streakforge WITH LOGIN PASSWORD 'streakforge_dev' CREATEDB;"
sudo -u postgres psql -c "CREATE DATABASE streakforge_dev OWNER streakforge;"
```

**After adding a migration, touch the test file before re-running.**
`#[sqlx::test(migrations = "./migrations")]` embeds its migration set when the
test target is compiled; a new `.sql` file does not invalidate that binary, so
the new migration is simply absent from every database the suite provisions and
the symptom is a missing database object rather than a stale build.

```bash
touch tests/integration.rs
```

---

## 3. Production Deployment (ThinkCentre)

### 3.1 Deploy script

```bash
./scripts/deploy.sh thinkcentre
```

Steps (see also SPEC §17): build release, `npm run build` (plain — no dev-login
variant exists anymore), rsync binary + migrations + manifestos →
`/personal/documents/code/streakforge`, rsync web/build → `/var/www/streakforge`,
write `.env`, install/enable/restart systemd unit.

`deploy.sh` no longer writes real secrets to the remote `.env`: the random
`SESSION_SECRET` and the Postgres DB password live in the **Hermes profile `.env`**
(`~/.hermes/profiles/coding/.env` on this machine) as `STREAKFORGE_SESSION_SECRET`
and `STREAKFORGE_DB_PASSWORD`. When deploying, set them on thinkcentre before
starting the service (or rotate them there):

```bash
SECRET=$(openssl rand -hex 32)
ssh thinkcentre "sudo sed -i 's|^SESSION_SECRET=.*|SESSION_SECRET=$SECRET|' /opt/streakforge/.env && sudo systemctl restart streakforge"
```

**Rotating `SESSION_SECRET` signs every user out once** — the session cookie is
signed with a key derived from it (see SPEC §6.3). That is intended for a rotation
and unacceptable by accident: do NOT let a deploy rewrite it. The deploy script
manages ONLY these in `/opt/streakforge/.env`, because they are configuration rather
than secrets:

| Managed key | Value |
|---|---|
| `MIGRATIONS_DIR` | `$DEPLOY_DIR/migrations` |
| `MANIFESTOS_DIR` | `$DEPLOY_DIR/manifestos` |
| `SECURE_COOKIES` | `1` |

`SECURE_COOKIES=1` is what makes the cookie `Secure` and renames it to
`__Host-streakforge_session` (SPEC §15). **Turning it on signs everyone out once**,
because the old cookie name is simply ignored; after that the prefix is enforced by
the browser. `DATABASE_URL` and `SESSION_SECRET` are never touched by a deploy.

The deploy also copies `scripts/backup-db.sh` and `scripts/restore-db.sh` into
`/opt/streakforge/` (next to the `.env` they read) and installs this cron entry,
idempotently — the old line is removed before the new one is added:

```
30 3 * * * /opt/streakforge/backup-db.sh >> /home/alvaro/backups/streakforge/cron.log 2>&1
```

Two further session/limit knobs live in the same `.env` (both optional, both
defaulted, both read at startup):

| Variable | Default | Effect |
|----------|---------|--------|
| `SESSION_TTL_DAYS` | `30` | days of inactivity before a session expires |
| `WLW_DAILY_LIMIT` | `5` | loads wasted allowed per UTC day per user |

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

Verified against `systemctl cat streakforge` on thinkcentre, 2026-10-14. The paths
are under `/opt` and `/var/www`, not `/personal`: the binary used to run from the
`/personal` NFS mount, and when that mount wedged the kernel could not page in the
process's code pages, so SIGBUS killed the process mid-request (three times, in
August). Local ext4 is immune. This block had drifted from the live unit.

```ini
[Unit]
Description=StreakForge habit tracker API (Rust)
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=alvaro
WorkingDirectory=/opt/streakforge
EnvironmentFile=/opt/streakforge/.env
Environment=PATH=/home/alvaro/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
ExecStart=/opt/streakforge/streakforge-api
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

`Restart=on-failure` matters beyond uptime now: a restart used to sign every user
out (the signing key was regenerated per process). That is fixed, so restarts are
survivable — but the restart *count* is still worth watching, because a crash loop
of 3,339 restarts in a day was the symptom that hid the logout bug.

### 3.4 Cloudflare tunnel

- Tunnel client: `cloudflared.service` runs `tunnel run --token-file /etc/cloudflared/token`.
- **Routes are managed in the Cloudflare dashboard** (no host config file).
- Public URL: **https://streakforge.polarisocial.xyz** → `http://192.168.1.13:8001`
  (route added 2026-08-10; verified 200).
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

# Public health (via Cloudflare tunnel)
curl -s -o /dev/null -w "%{http_code}\n" https://streakforge.polarisocial.xyz/
curl -s https://streakforge.polarisocial.xyz/api/auth/nonce   # expect {"nonce":"<32-hex>"}

# Logs
ssh thinkcentre 'sudo journalctl -u streakforge -n 50 --no-pager'
```

Manual QA account (live 2026-08-10): `sample_user` / `SamplePass123!` —
register/login/me/logout all verified against the public URL. Delete the row
from `profiles` when it's no longer needed.

Expected healthy: service `active`; root 200; `/api/total` returns `{"total":N}`;
port 8001 listening. Unknown `/api/*` paths return 404 (they do **not** fall
through to the SPA shell).

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

### 5.5 Register/login rejected with "Invalid form submission"

The public auth forms are bot-dissuaded: the honeypot field must be empty, the
form must have been open 3s–10min, and the JS challenge proof must match
`sha256(nonce || username || password)` truncated to 16 hex chars. If a legit
submission gets 400, the client clock is skewed (form timing) or the JS challenge
didn't run (hardened browser, script blocked). Retry after re-fetching the nonce.

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

Everything the site knows — every log, every streak and every private note — lives in
ONE Postgres database on ONE machine. Until 2026-10-14 there was no dump at all, so a
bad migration, a mistaken `DELETE` or a failing disk was a total loss. There are now
nightly dumps, and a rehearsal that proves they restore.

### 6.1 Nightly dumps

| | |
|---|---|
| Script | `scripts/backup-db.sh` (deployed to `/opt/streakforge/backup-db.sh`) |
| When | 03:30 daily, from the cron entry the deploy installs |
| Where | `~/backups/streakforge/streakforge-YYYY-MM-DD.dump` (`pg_dump -Fc`) |
| Kept | 14 dailies; the 1st of each month is kept as a monthly copy, 6 of those |
| Log | `~/backups/streakforge/cron.log` — one line per run, with the size and the kept count |

Details that matter if it ever has to be changed:

- The dump format is `-Fc` (custom), so `pg_restore` can list, filter and restore
  selectively, and it is compressed.
- The directory is `0700` and each dump is `0600`: a dump contains every private note
  on the site, which makes it the most sensitive file on the host after the `.env`.
- **The DSN never appears in `argv`.** The script reads `DATABASE_URL` from the
  environment file and passes it as `PG*` variables, because `/proc` exposes arguments
  to any local user and the DSN carries the role's password. It also never echoes the
  DSN in its own output.
- A dump under 10 KB is treated as a FAILURE and not published (the `.part` file is
  removed and yesterday's dump is left untouched): the classic silent backup failure
  is a file that exists and is empty.
- The write is atomic (`--file=….part`, then `mv`), so a reader or the pruner can never
  see a half-written file.

Run it by hand any time — it is idempotent for a given day:

```bash
ssh thinkcentre '/opt/streakforge/backup-db.sh'
```

### 6.2 Restore, and the rehearsal

```bash
ssh thinkcentre '/opt/streakforge/restore-db.sh'                 # newest dump
ssh thinkcentre '/opt/streakforge/restore-db.sh /path/to/one.dump my_check_db'
```

It drops and recreates the target (`streakforge_restore_check` by default), restores
into it, prints row counts, and compares `profiles` against the live database. It
refuses to run if the target name equals the live database's name.

**A backup nobody has restored is a hypothesis.** Last run, 2026-10-14 (after
migration 0011):

```
profiles 67   habit_logs 512   lock_sessions 43   notes 512   migrations 11
profiles: live=67 restored=67
```

Run it after any migration, after any change to either script, and whenever a dump
size looks wrong.

### 6.3 What is NOT covered

- **The host itself.** These dumps are on the same machine as the database. A lost
  ThinkCentre still loses everything; off-host copies are the next step (SPEC §27).
- **The rehearsal is manual.** Nothing runs `restore-db.sh` on a schedule yet.
- Static content (manifestos, frontend build) is in the repo and rebuilt from source.

### 6.4 Other copies

- The repo is mirrored private on opencommit.eu (remote `github`), plus a local
  `upstream` remote on this machine.

---

## 7. Routine Tasks

| Task | Command |
|------|---------|
| Redeploy after feature | `./scripts/deploy.sh thinkcentre` (+ secret re-check) |
| Check prod health | see §4 |
| Take a backup now | `ssh thinkcentre /opt/streakforge/backup-db.sh` |
| Prove a backup restores | `ssh thinkcentre /opt/streakforge/restore-db.sh` |
| Check the nightly log | `ssh thinkcentre 'tail -5 ~/backups/streakforge/cron.log'` |
| Check CI | `gh run list --workflow=ci.yml --limit 3` |
| Add doctrine doc | copy to `manifestos/`, redeploy |
| Update docs | edit `docs/*.md`, keep STATUS.md in sync |
| Push mirror | `git push github main` |

---

## 8. Incident Notes (log)

- 2026-08-10: deployed to thinkcentre; discovered nginx is inactive and fichub serves
  directly on 8000; streakforge mirrors on 8001 (no nginx config).
- 2026-08-10: migration checksum lock hit when 0002 was edited after apply — resolved
  by reverting 0002 and moving redefinitions to 0003.
- 2026-08-10: session secret placeholder replaced with random 64-hex on prod.
- 2026-10-14: `SECURE_COOKIES=1` enabled by the deploy script. The session cookie
  became `__Host-streakforge_session`, so every signed-in user was signed out exactly
  once (the old name is ignored). Verified with a live registration: `HttpOnly;
  SameSite=Lax; Secure; Path=/; Max-Age=2592000`.
- 2026-10-14: nightly backups installed and REHEARSED for the first time; the restore
  matched production (`profiles 67`, `habit_logs 512`). Before this, the project had no
  dump of the database at all.
- 2026-10-14: migration 0011 changed the Devotion Index to count distinct DAYS instead
  of rows. Scores drop for anyone who logged more than once in a day — that is the
  intent (repetition must not buy rank), and lifetime totals are unchanged.
