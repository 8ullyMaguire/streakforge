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
(cd "$ROOT/backend" && cargo build --release)

echo "==> 2/6 Building frontend (production)"
(cd "$ROOT/web" && npm run build >/dev/null)

echo "==> 3/6 Syncing to $HOST"
ssh "$HOST" "mkdir -p $DEPLOY_DIR && sudo mkdir -p $BIN_DIR $WWW_DIR && sudo chown -R alvaro:alvaro $BIN_DIR $WWW_DIR"
rsync -az --delete "$ROOT/backend/target/release/streakforge-api" "$HOST:$BIN_DIR/"
rsync -az --delete "$ROOT/backend/migrations" "$HOST:$DEPLOY_DIR/"
rsync -az --delete "$ROOT/manifestos" "$HOST:$DEPLOY_DIR/"
rsync -az --delete "$ROOT/web/build/" "$HOST:$WWW_DIR/"

echo "==> 4/6 Writing .env (PRESERVE existing remote secrets)"
# The remote .env (SESSION_SECRET + DB password) lives on /personal. Copy it to
# /opt so systemd's EnvironmentFile is on local disk too (a wedged NFS at boot
# would otherwise fail the whole unit). Never overwrite with hardcoded values.
# Also ensure MIGRATIONS_DIR is set (relative ./migrations would break under
# the new /opt WorkingDirectory).
ssh "$HOST" "if [ -f $DEPLOY_DIR/.env ]; then
  cp -n $DEPLOY_DIR/.env $BIN_DIR/.env && echo 'preserved .env -> /opt';
  grep -q '^MIGRATIONS_DIR=' $BIN_DIR/.env || echo 'MIGRATIONS_DIR=$DEPLOY_DIR/migrations' >> $BIN_DIR/.env;
  echo 'MIGRATIONS_DIR set:'; grep '^MIGRATIONS_DIR=' $BIN_DIR/.env;
else echo 'MISSING .env on remote!'; fi"

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

echo "==> Done. Verify: curl http://127.0.0.1:8001/"
