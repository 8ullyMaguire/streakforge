#!/usr/bin/env bash
# StreakForge dev startup script.
# Builds frontend (with dev-login), starts the backend, serves on :8787.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Building frontend (dev-login enabled)"
(cd web && VITE_ALLOW_DEV_LOGIN=1 npm run build >/dev/null)

echo "==> Starting backend on http://127.0.0.1:8787"
cd backend
ALLOW_DEV_LOGIN=1 \
  WEB_BUILD_DIR="$(pwd)/../web/build" \
  RUST_LOG=info \
  cargo run
