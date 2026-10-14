#!/usr/bin/env bash
# Rehearse a restore from a dump into a scratch database.
#
# A backup nobody has restored is a hypothesis. This script restores the newest
# dump (or one you name) into `streakforge_restore_check`, prints the row counts
# of the three tables that matter, and leaves the database in place so a human
# can look at it. Run it after any migration and after any change to
# scripts/backup-db.sh.
#
# It never touches the live database: the target is dropped and recreated by
# name, and the script refuses to run if that name is the live one.
#
# Usage: restore-db.sh [dump-file] [target-db]
set -euo pipefail

ENV_FILE="${STREAKFORGE_ENV:-/opt/streakforge/.env}"
BACKUP_DIR="${BACKUP_DIR:-$HOME/backups/streakforge}"
TARGET="${2:-${RESTORE_DB:-streakforge_restore_check}}"

if [ ! -r "$ENV_FILE" ]; then
  echo "FATAL: cannot read $ENV_FILE (set STREAKFORGE_ENV)" >&2
  exit 1
fi

# shellcheck disable=SC1090
set -a
. "$ENV_FILE"
set +a
: "${DATABASE_URL:?DATABASE_URL is not set in $ENV_FILE}"

# Same reasoning as backup-db.sh: PG* variables, never the DSN in argv.
rest="${DATABASE_URL#*://}"
userinfo="${rest%%@*}"
rest="${rest#*@}"
hostport="${rest%%/*}"
db="${rest#*/}"; db="${db%%\?*}"
LIVE_DB="$db"
PGUSER="${userinfo%%:*}"
case "$userinfo" in *:*) PGPASSWORD="${userinfo#*:}" ;; *) PGPASSWORD="" ;; esac
PGHOST="${hostport%%:*}"
port="${hostport#*:}"
unset PGPORT
if [ "$port" != "$hostport" ] && [[ "$port" =~ ^[0-9]+$ ]]; then export PGPORT="$port"; fi
export PGUSER PGPASSWORD PGHOST
unset DATABASE_URL

if [ "$TARGET" = "$LIVE_DB" ]; then
  echo "FATAL: refusing to restore over the live database ($LIVE_DB)." >&2
  exit 1
fi

DUMP="${1:-}"
if [ -z "$DUMP" ]; then
  DUMP="$(ls -1t "$BACKUP_DIR"/streakforge-*.dump 2>/dev/null | head -1 || true)"
fi
if [ -z "$DUMP" ] || [ ! -f "$DUMP" ]; then
  echo "FATAL: no dump found (looked in $BACKUP_DIR; pass one as the first argument)." >&2
  exit 1
fi
DUMP="$(readlink -f "$DUMP")"

echo "==> rehearsing: $DUMP"
echo "    live database:  $LIVE_DB (untouched)"
echo "    scratch target: $TARGET"

# The maintenance connection must go to a database that always exists.
PGDATABASE=postgres
psql -v ON_ERROR_STOP=1 -q -c "drop database if exists \"$TARGET\""
psql -v ON_ERROR_STOP=1 -q -c "create database \"$TARGET\""

# --no-owner/--no-privileges for the same reason the dump used them: the
# rehearsal must not need the original role to exist, and must not change who
# owns anything.
echo "==> restoring"
pg_restore --no-owner --no-privileges --exit-on-error -d "$TARGET" "$DUMP" 2>&1 | sed -e 's/^/    /' || {
  echo "FATAL: pg_restore failed — this dump is NOT restorable." >&2
  exit 1
}

PGDATABASE="$TARGET"
echo "==> counts in the restored copy"
psql -At -F' ' -c "
  select 'profiles',     count(*) from profiles
  union all select 'habit_logs',   count(*) from habit_logs
  union all select 'lock_sessions', count(*) from lock_sessions
  union all select 'notes',        count(*) from habit_logs where note is not null
  union all select 'migrations',   count(*) from _sqlx_migrations" | sed -e 's/^/    /'

# Compare against the live database in the same run: a restore that works but
# silently loses a table is exactly the failure this script exists to catch.
LIVE_COUNTS="$(PGDATABASE="$LIVE_DB" psql -At -F' ' -c "select count(*) from profiles" )"
RESTORED_COUNTS="$(PGDATABASE="$TARGET" psql -At -F' ' -c "select count(*) from profiles")"
echo "==> profiles: live=$LIVE_COUNTS restored=$RESTORED_COUNTS"
if [ "$LIVE_COUNTS" != "$RESTORED_COUNTS" ]; then
  echo "WARN: profile counts differ. Expected if the dump is from an earlier day;" >&2
  echo "      compare the dumped day against the backup's timestamp before worrying." >&2
fi

echo "==> restore rehearsal complete. Scratch database '$TARGET' left in place."
