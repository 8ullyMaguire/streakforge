#!/usr/bin/env bash
# Nightly dumps of the live StreakForge database, with pruning.
#
# Runs ON the host that owns the database (ThinkCentre), from cron at 03:30.
# Installed by scripts/deploy.sh, which also copies this file next to the .env
# it reads, so the deployed script and this one cannot drift.
#
# WHY THIS EXISTS: the site holds the only copy of every user's history — the
# logs, the streaks, and the private notes — on one local Postgres with no
# replication and no snapshot. A disk failure was a total loss. That was the
# single largest piece of uninsured risk left in the project (docs/100_IDEAS.md
# #03).
#
# Credentials: DATABASE_URL is read from the environment file, and then passed to
# pg_dump as PG* environment variables rather than as an argument. argv is
# readable via /proc by any local user, and this DSN carries the role's password.
# Nothing in this script ever echoes the DSN itself.
#
# Usage: backup-db.sh [--env-file PATH] [--dir PATH]
# Env:   STREAKFORGE_ENV, BACKUP_DIR, KEEP_DAILY, KEEP_MONTHLY, MIN_BYTES
set -euo pipefail

ENV_FILE="${STREAKFORGE_ENV:-/opt/streakforge/.env}"
BACKUP_DIR="${BACKUP_DIR:-$HOME/backups/streakforge}"
# 14 dailies is two weeks of "I noticed something was wrong" time; the monthly
# copies cover the slower failure (a bad import nobody spotted for a month).
KEEP_DAILY="${KEEP_DAILY:-14}"
KEEP_MONTHLY="${KEEP_MONTHLY:-6}"
# A dump smaller than this is a truncated or empty file, not a backup.
MIN_BYTES="${MIN_BYTES:-10240}"

while [ $# -gt 0 ]; do
  case "$1" in
    --env-file) ENV_FILE="$2"; shift 2 ;;
    --dir) BACKUP_DIR="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

if [ ! -r "$ENV_FILE" ]; then
  echo "FATAL: cannot read $ENV_FILE (set STREAKFORGE_ENV)" >&2
  exit 1
fi

# shellcheck disable=SC1090
set -a
. "$ENV_FILE"
set +a
: "${DATABASE_URL:?DATABASE_URL is not set in $ENV_FILE}"

# Turn a postgres:// URI into PG* variables. Falls back to leaving PGHOST etc.
# unset, which makes libpq use its own defaults for anything the URI omits.
parse_dsn() {
  local url="$1" rest userinfo hostport db
  rest="${url#*://}"
  userinfo="${rest%%@*}"
  rest="${rest#*@}"
  hostport="${rest%%/*}"
  db="${rest#*/}"
  db="${db%%\?*}"

  if [ "$userinfo" = "$rest" ]; then
    echo "FATAL: DATABASE_URL has no user@host part" >&2
    exit 1
  fi

  PGUSER="${userinfo%%:*}"
  case "$userinfo" in
    *:*) PGPASSWORD="${userinfo#*:}" ;;
    *)   PGPASSWORD="" ;;
  esac
  PGHOST="${hostport%%:*}"
  local port="${hostport#*:}"
  if [ "$port" != "$hostport" ] && [[ "$port" =~ ^[0-9]+$ ]]; then
    PGPORT="$port"
  else
    unset PGPORT
  fi
  PGDATABASE="$db"
  export PGUSER PGPASSWORD PGHOST PGDATABASE
  [ -n "${PGPORT:-}" ] && export PGPORT
  return 0
}
parse_dsn "$DATABASE_URL"
unset DATABASE_URL

mkdir -p "$BACKUP_DIR"
# 0700/0600 throughout: a dump contains every private note in the site. It is
# the most sensitive file on the host after the .env itself.
chmod 700 "$BACKUP_DIR"

STAMP="$(date -u +%F)"
OUT="$BACKUP_DIR/streakforge-$STAMP.dump"
TMP="$OUT.part"

echo "==> dumping $PGDATABASE on $PGHOST to $OUT"
pg_dump --format=custom --no-owner --no-privileges --file="$TMP"
chmod 600 "$TMP"

SIZE="$(stat -c%s "$TMP")"
if [ "$SIZE" -lt "$MIN_BYTES" ]; then
  echo "FATAL: the dump is only ${SIZE}B (under the ${MIN_BYTES}B floor) — refusing to publish it as a backup." >&2
  rm -f "$TMP"
  exit 1
fi

# Publish atomically: a reader (or the pruner) must never see a half-written
# file, and a failed run must not destroy yesterday's good dump.
mv "$TMP" "$OUT"

# The first of the month keeps its daily copy as the monthly one, by simply not
# being eligible for pruning below.
if [ "$(date -u +%d)" = "01" ]; then
  echo "    (first of the month — this dump is also the monthly copy)"
fi

echo "==> pruning (keeping $KEEP_DAILY dailies, $KEEP_MONTHLY monthlies)"
mapfile -t DUMPS < <(ls -1t "$BACKUP_DIR"/streakforge-*.dump 2>/dev/null || true)
MONTHLIES=0
for f in "${DUMPS[@]:-}"; do
  [ -n "$f" ] || continue
  base="$(basename "$f")"
  # streakforge-YYYY-MM-01.dump is a monthly copy: keep the newest KEEP_MONTHLY
  # of them regardless of age.
  if [[ "$base" =~ ^streakforge-[0-9]{4}-[0-9]{2}-01\.dump$ ]]; then
    MONTHLIES=$((MONTHLIES + 1))
    if [ "$MONTHLIES" -le "$KEEP_MONTHLY" ]; then
      continue
    fi
    echo "    removing old monthly $base"
    rm -f "$f"
  fi
done

# Then prune by position among the non-monthly dailies, newest first. Counted
# AFTER the monthly pass so a month of monthlies cannot eat the daily budget.
DAILIES=0
for f in "${DUMPS[@]:-}"; do
  [ -n "$f" ] || continue
  [ -f "$f" ] || continue
  base="$(basename "$f")"
  [[ "$base" =~ ^streakforge-[0-9]{4}-[0-9]{2}-01\.dump$ ]] && continue
  DAILIES=$((DAILIES + 1))
  if [ "$DAILIES" -gt "$KEEP_DAILY" ]; then
    echo "    removing old daily $base"
    rm -f "$f"
  fi
done

KEPT="$(ls -1 "$BACKUP_DIR"/streakforge-*.dump 2>/dev/null | wc -l)"
TOTAL="$(du -sh "$BACKUP_DIR" | cut -f1)"
# One line per run, so `journalctl`/cron mail shows the history. No DSN, ever.
echo "$(date -u +%FT%TZ) ok $OUT ${SIZE}B kept=$KEPT total=$TOTAL"
