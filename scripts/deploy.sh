#!/usr/bin/env bash
# StreakForge deploy to ThinkCentre (mirrors FicHub setup).
# - Syncs release binary + frontend build + migrations + manifestos to ThinkCentre
# - Sets up /var/www/streakforge (static) + /personal/documents/code/streakforge (bin)
# - Installs systemd service on port 8001 + nginx site (proxy /api -> 8001)
# Usage: scripts/deploy.sh [thinkcentre-host]
set -euo pipefail

HOST="${1:-thinkcentre}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEPLOY_DIR="/personal/documents/code/streakforge"
WWW_DIR="/var/www/streakforge"

echo "==> 1/6 Building release binary (local)"
(cd "$ROOT/backend" && cargo build --release)

echo "==> 2/6 Building frontend (no dev login)"
(cd "$ROOT/web" && VITE_ALLOW_DEV_LOGIN=0 npm run build >/dev/null)

echo "==> 3/6 Syncing to $HOST"
ssh "$HOST" "mkdir -p $DEPLOY_DIR && sudo mkdir -p $WWW_DIR && sudo chown -R alvaro:alvaro $WWW_DIR"
rsync -az --delete "$ROOT/backend/target/release/streakforge-api" "$HOST:$DEPLOY_DIR/"
rsync -az --delete "$ROOT/backend/migrations" "$HOST:$DEPLOY_DIR/"
rsync -az --delete "$ROOT/manifestos" "$HOST:$DEPLOY_DIR/"
rsync -az --delete "$ROOT/web/build/" "$HOST:$WWW_DIR/"

echo "==> 4/6 Writing .env"
ssh "$HOST" "cat > $DEPLOY_DIR/.env" <<'ENV'
DATABASE_URL=postgres://streakforge:streakforge_prod@127.0.0.1:5432/streakforge
SESSION_SECRET=CHANGE_ME_to_a_long_random_string_for_production_0123456789abcdef
PUBLIC_URL=http://127.0.0.1:8001
ALLOW_DEV_LOGIN=0
SECURE_COOKIES=0
WEB_BUILD_DIR=/var/www/streakforge
MANIFESTOS_DIR=/personal/documents/code/streakforge/manifestos
BIND_ADDR=127.0.0.1:8001
RUST_LOG=info
ENV

echo "==> 5/6 Installing systemd service"
ssh "$HOST" "sudo tee /etc/systemd/system/streakforge.service >/dev/null" <<'UNIT'
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
UNIT

echo "==> 6/6 Installing nginx site (port 8001) — SKIPPED: backend serves static + API directly on 8001 (mirrors fichub, which serves on 8000 itself)"
echo "==> Enabling + starting"
ssh "$HOST" "sudo systemctl daemon-reload && sudo systemctl enable streakforge && sudo systemctl restart streakforge"

echo "==> Done. Verify: curl http://127.0.0.1:8001/"
