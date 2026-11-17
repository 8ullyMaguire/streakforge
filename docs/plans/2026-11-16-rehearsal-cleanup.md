# Plan — rehearsal cleanup (2026-11-16)

1. Reproduce: on thinkcentre query `pg_database` for `*_restore_check`; expect `streakforge_restore_check`.
2. Patch `scripts/backup-db.sh` and `scripts/restore-db.sh` to select the pg_dump/pg_restore matching the server's
   major version (the rehearsal proved the PATH default was pg_dump 18.7 against a 16.15 server).
3. Patch `scripts/restore-db.sh`: after the `TARGET = LIVE_DB` guard, install `trap` that runs
   `psql -v ON_ERROR_STOP=1 -q -c "drop database if exists \"$TARGET\""` with `PGDATABASE=postgres` on EXIT. Print one
   log line (`scratch database cleaned: $TARGET`) only when a database was actually created. Keep the counts and
   `REHEARSAL OK` line before the trap runs.
4. Install both patched scripts to `/opt/streakforge/` with mode 755; regenerate today's dump with the pinned pg_dump.
5. Run the real rehearsal with the newest dump. Capture the log. Verify: `REHEARSAL OK` present, counts present,
   `pg_database` has no scratch row.
6. Failure-path probe: run with a nonexistent dump path; exit non-zero; verify no scratch row.
7. Prepend a STATUS entry, commit the script + spec + plan + STATUS, push `github main`, and report the hash.
