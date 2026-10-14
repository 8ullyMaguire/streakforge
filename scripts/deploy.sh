#!/usr/bin/env bash
# StreakForge deploy to ThinkCentre (mirrors FicHub setup).
# - Syncs release binary + frontend build + migrations + manifestos to ThinkCentre
# - Sets up /var/www/streakforge (static) + /opt/streakforge (bin + .env, LOCAL disk)
# - Installs systemd service on port 8001 (backend serves static + API directly,
#   mirroring fichub on 8000 — nginx is inactive on the ThinkCentre)
#
# WHY /opt for the binary: the old layout ran the executable from
# /personal (fuse.mergerfs over NFS). When the NFS wedges, the kernel cannot
# page in the process's code pages -> SIGBUS kills the process mid-request
# (happened 3x: Aug 10/11/14). Local ext4 /opt is immune. Migrations +
# manifestos stay on /personal (read once at startup; read errors are
# recoverable, unlike executable page-in).
# Usage: scripts/deploy.sh [thinkcentre-host]
set -euo pipefail

HOST="${1:-thinkcentre}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEPLOY_DIR="/personal/documents/code/streakforge"
BIN_DIR="/opt/streakforge"
WWW_DIR="/var/www/streakforge"

echo "==> 1/6 Building release binary (local)"
# Ask cargo where it builds instead of assuming. ~/.cargo/config.toml sets a
# global `target-dir = /home/alvaro/.cache/cargo-target`, so a hardcoded
# backend/target is simply the wrong path on this machine -- and the rsync in
# step 3 would then ship whatever stale binary happened to be sitting there.
#
# This is not hypothetical. A deploy on 2026-09-30 did exactly that: it shipped
# a 3-day-old binary with the new migrations and the new frontend, and every
# health check PASSED, because the old binary served the old routes perfectly
# well. /api/kpi 404'd on a deploy the script reported as successful.
#
# Scope, since it was asked: the window is bounded. ~/.cargo/config.toml has
# mtime 2026-09-30 16:24; the last deploy before that was Sep 27 15:59 and it
# shipped a binary built that same afternoon, so builds were landing in
# backend/target correctly until the config appeared. The only affected deploy
# was the 19:06 one, which was caught and re-run clean. Nothing stale ever
# reached production.
(cd "$ROOT/backend" && cargo build --release)
CARGO_TARGET_DIR_ACTUAL="$(cd "$ROOT/backend" && cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
BIN_SRC="$CARGO_TARGET_DIR_ACTUAL/release/streakforge-api"
if [ -z "$CARGO_TARGET_DIR_ACTUAL" ] || [ ! -x "$BIN_SRC" ]; then
  echo "FATAL: cargo built no release binary (looked for $BIN_SRC)" >&2
  exit 1
fi
# And assert it is not older than the newest source file, so a build that
# silently did nothing cannot ship.
if [ "$BIN_SRC" -ot "$ROOT/backend/src/api.rs" ]; then
  echo "FATAL: $BIN_SRC is older than src/api.rs — the build did not run." >&2
  exit 1
fi
echo "    binary: $BIN_SRC"

echo "==> 2/6 Building frontend (production)"
(cd "$ROOT/web" && npm run build >/dev/null)

echo "==> 3/6 Syncing to $HOST"
ssh "$HOST" "mkdir -p $DEPLOY_DIR && sudo mkdir -p $BIN_DIR $WWW_DIR && sudo chown -R alvaro:alvaro $BIN_DIR $WWW_DIR"
rsync -az --delete "$BIN_SRC" "$HOST:$BIN_DIR/"
# The backup/restore scripts live next to the .env they read, and are copied on
# every deploy so the deployed copy cannot drift from this one. No --delete: the
# destination also holds the binary and the .env, which this source does not
# carry.
rsync -az "$ROOT/scripts/backup-db.sh" "$ROOT/scripts/restore-db.sh" "$HOST:$BIN_DIR/"
rsync -az --delete "$ROOT/backend/migrations" "$HOST:$DEPLOY_DIR/"
rsync -az --delete "$ROOT/manifestos" "$HOST:$DEPLOY_DIR/"
rsync -az --delete "$ROOT/web/build/" "$HOST:$WWW_DIR/"

echo "==> 4/6 Writing .env (PRESERVE existing remote secrets)"
# Secrets live in the remote .env and are never written here. Only the two
# non-secret PATH settings below are managed, and they are REWRITTEN (not just
# appended when absent): the legacy values pointed at an NFS path
# (/personal/...) that no longer resolves, so the binary silently started with
# no migrations to apply and the Devotion Index never deployed. Appending-if-
# missing could not catch that, so the deploy now asserts the paths exist.
ssh "$HOST" "set -e
  if [ ! -f $BIN_DIR/.env ]; then
    echo 'FATAL: no .env at $BIN_DIR/.env' >&2; exit 1
  fi
  sudo sed -i 's|^MIGRATIONS_DIR=.*|MIGRATIONS_DIR=$DEPLOY_DIR/migrations|' $BIN_DIR/.env
  sudo sed -i 's|^MANIFESTOS_DIR=.*|MANIFESTOS_DIR=$DEPLOY_DIR/manifestos|' $BIN_DIR/.env
  # SECURE_COOKIES is deployment POLICY, not a secret, so it is managed here
  # rather than left to whoever set the .env up. Without it the session cookie
  # travels without Secure and without the __Host- prefix, which on a site that
  # only exists behind HTTPS is a defect (docs/100_IDEAS.md #02). Setting it
  # renames the cookie, so every signed-in user is signed out exactly once.
  sudo sed -i 's|^SECURE_COOKIES=.*|SECURE_COOKIES=1|' $BIN_DIR/.env
  grep -q '^MIGRATIONS_DIR=' $BIN_DIR/.env || echo 'MIGRATIONS_DIR=$DEPLOY_DIR/migrations' | sudo tee -a $BIN_DIR/.env
  grep -q '^MANIFESTOS_DIR=' $BIN_DIR/.env || echo 'MANIFESTOS_DIR=$DEPLOY_DIR/manifestos' | sudo tee -a $BIN_DIR/.env
  grep -q '^SECURE_COOKIES=' $BIN_DIR/.env || echo 'SECURE_COOKIES=1' | sudo tee -a $BIN_DIR/.env
  grep -E '^(MIGRATIONS_DIR|MANIFESTOS_DIR|SECURE_COOKIES)=' $BIN_DIR/.env"

echo "==> 4b/6 Verifying every configured path exists on the remote"
# A .env value pointing at a directory that is not there is an outage that only
# shows up as a 500 much later, so fail the deploy here instead.
ssh "$HOST" "set -e
  for d in $DEPLOY_DIR/migrations $DEPLOY_DIR/manifestos $WWW_DIR; do
    if [ ! -d \"\$d\" ]; then echo \"FATAL: missing dir \$d\" >&2; exit 1; fi
    echo \"ok \$d\"
  done"

echo "==> 5/6 Installing systemd service"
ssh "$HOST" "sudo tee /etc/systemd/system/streakforge.service >/dev/null" <<'UNIT'
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
UNIT

echo "==> 5b/6 Installing the backup and rehearsal cron entries"
# Two jobs: the nightly dump at 03:30, and the restore rehearsal on the 1st at
# 04:30 — an hour after the dump, so a rehearsal can never race the backup it is
# checking. The rehearsal is what turns "we have backups" into "the last backup
# restored", and it is also the only job that notices the dump job has died
# (restore-db.sh fails when the newest dump is older than 48 hours).
#
# Idempotent: both old lines are removed before the new ones are added, so
# re-running a deploy does not stack duplicate jobs (the classic way a host ends
# up running the same backup four times a night).
ssh "$HOST" 'bash -s' <<REMOTE
set -e
mkdir -p "\$HOME/backups/streakforge"
chmod 700 "\$HOME/backups/streakforge"
{
  crontab -l 2>/dev/null | grep -v 'backup-db.sh\|restore-db.sh' || true
  echo "30 3 * * * $BIN_DIR/backup-db.sh >> \$HOME/backups/streakforge/cron.log 2>&1"
  echo "30 4 1 * * $BIN_DIR/restore-db.sh >> \$HOME/backups/streakforge/rehearsal.log 2>&1"
} > /tmp/cron.streakforge
crontab /tmp/cron.streakforge
rm -f /tmp/cron.streakforge
echo "    installed:"
crontab -l | grep -E 'backup-db.sh|restore-db.sh' | sed 's/^/      /'
REMOTE

echo "==> 6/6 Installing nginx site (port 8001) — SKIPPED: backend serves static + API directly on 8001 (mirrors fichub, which serves on 8000 itself)"
echo "==> Enabling + starting"
ssh "$HOST" "sudo systemctl daemon-reload && sudo systemctl enable streakforge && sudo systemctl restart streakforge"

# The deploy used to print "Done" after a successful restart, which reported
# success while every API route 500'd. Assert the endpoints the frontend
# actually calls, and fail loudly if any of them do.
#
# And wait for the service to come UP first. `systemctl restart` returns as soon
# as the process is started (Type=simple), which is well before the listener
# exists, so the loop below could run against a socket nobody was bound to and
# report FAIL on a deploy that was perfectly fine. That is what happened on
# 2026-10-14: the script exited 1 with eight endpoints answering 200 a few
# seconds later, and the only reason it was caught is that the operator did not
# take the gate's word for it. A gate that cries wolf is worse than no gate.
echo "==> 6b/6 Waiting for $HOST to accept connections"
READY=0
for i in $(seq 1 30); do
  code=$(curl -s -o /dev/null -w '%{http_code}' "https://streakforge.polarisocial.xyz/api/total" || true)
  if [ "$code" = "200" ]; then
    echo "    serving (after ${i}s)"
    READY=1
    break
  fi
  sleep 1
done
if [ "$READY" -ne 1 ]; then
  echo "FATAL: the service did not answer within 30s of the restart." >&2
  ssh "$HOST" "sudo journalctl -u streakforge -n 30 --no-pager" >&2
  exit 1
fi

echo "==> Health check"
HEALTH_FAIL=0
# NOTE: /leaderboard/{period} has no bare form, and /feed takes a ?cursor
# query — so "/api/leaderboard" and "/api/feed/0" are 404s by design, not
# faults. These paths mirror what web/src/lib/api.ts actually requests.
#
# Each path gets three tries. A single retryable blip (a Cloudflare hiccup, one
# cold connection) must not read as "the deploy did NOT land cleanly"; only a
# path that fails three times does.
# The static files and the pages this pass added are checked here too: a
# robots.txt that has silently started returning the SPA shell, or a /glossary
# that 404s, is exactly the kind of breakage a "binary is running" check misses.
for path in "/" "/api/total" "/api/today" "/api/leaderboard/monthly" "/api/leaderboard/alltime" \
            "/api/user-of-the-day" "/api/feed" "/api/doctrine" "/api/kpi" \
            "/robots.txt" "/sitemap.xml" "/og.png" "/glossary" "/terms" "/log"; do
  code=000
  for _ in 1 2 3; do
    code=$(curl -s -o /dev/null -w '%{http_code}' "https://streakforge.polarisocial.xyz$path" || true)
    [ "$code" = "200" ] && break
    sleep 1
  done
  if [ "$code" = "200" ]; then
    echo "  ok   $path ($code)"
  else
    echo "  FAIL $path ($code)"
    HEALTH_FAIL=1
  fi
done
if [ "$HEALTH_FAIL" -ne 0 ]; then
  echo "FATAL: health check failed — the deploy did NOT land cleanly." >&2
  echo "Recent service log:" >&2
  ssh "$HOST" "sudo journalctl -u streakforge -n 30 --no-pager" >&2
  exit 1
fi

echo "==> Done. All endpoints healthy."
