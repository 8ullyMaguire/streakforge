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
  grep -q '^MIGRATIONS_DIR=' $BIN_DIR/.env || echo 'MIGRATIONS_DIR=$DEPLOY_DIR/migrations' | sudo tee -a $BIN_DIR/.env
  grep -q '^MANIFESTOS_DIR=' $BIN_DIR/.env || echo 'MANIFESTOS_DIR=$DEPLOY_DIR/manifestos' | sudo tee -a $BIN_DIR/.env
  grep -E '^(MIGRATIONS_DIR|MANIFESTOS_DIR)=' $BIN_DIR/.env"

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

echo "==> 6/6 Installing nginx site (port 8001) — SKIPPED: backend serves static + API directly on 8001 (mirrors fichub, which serves on 8000 itself)"
echo "==> Enabling + starting"
ssh "$HOST" "sudo systemctl daemon-reload && sudo systemctl enable streakforge && sudo systemctl restart streakforge"

# The deploy used to print "Done" after a successful restart, which reported
# success while every API route 500'd. Assert the endpoints the frontend
# actually calls, and fail loudly if any of them do.
echo "==> Health check"
HEALTH_FAIL=0
# NOTE: /leaderboard/{period} has no bare form, and /feed takes a ?cursor
# query — so "/api/leaderboard" and "/api/feed/0" are 404s by design, not
# faults. These paths mirror what web/src/lib/api.ts actually requests.
for path in "/" "/api/total" "/api/leaderboard/monthly" "/api/leaderboard/alltime" \
            "/api/user-of-the-day" "/api/feed" "/api/doctrine" "/api/kpi"; do
  code=$(curl -s -o /dev/null -w '%{http_code}' "https://streakforge.polarisocial.xyz$path")
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
