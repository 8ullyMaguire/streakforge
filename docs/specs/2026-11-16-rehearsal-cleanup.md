# Spec — clean the StreakForge restore rehearsal scratch database (2026-11-16)

## Problem

`scripts/restore-db.sh` deliberately leaves `streakforge_restore_check` behind "so a human can look at it". In practice
that is a wreckage policy: the database exists on production right now, and the estate already learned from suki's
drill that a rehearsal leaving its scratch behind becomes a disk-space incident.

## Goals

- Drop the scratch database on every exit path — success, restore failure, freshness failure, interrupt — after the
  counts have been printed, so the log still contains the evidence.
- Never touch the live database: the existing guard that refuses `TARGET = LIVE_DB` stays, and the trap only fires for
  the scratch name.
- Prove it by running a real rehearsal and reading `pg_database` back.

## Non-goals

- No change to the backup job, the restore arithmetic or the freshness rule.
- No touching the running service, the live DB or any member tree outside StreakForge.

## Success criteria

- After a rehearsal run, `select datname from pg_database where datname like '%_restore_check%'` returns no rows.
- The rehearsal's own `REHEARSAL OK` line and counts still appear in the log.
- A failure-path probe (missing dump) also leaves no scratch behind and exits non-zero.
- Repo commit and host copy both updated; STATUS records the evidence.

## Follow-up — the rehearsal exposed a real backup defect

The first rehearsal after adding the trap failed: `pg_restore: unrecognized configuration parameter "transaction_timeout"`.
The dump header says it was written by `pg_dump 18.7` while the server and the default `pg_restore` are 16.15. The nightly
backup script must use the pg_dump matching the server's major version; otherwise it produces dumps its own restore
cannot read. Both `backup-db.sh` and `restore-db.sh` now derive the server major from `show server_version` and select
`/usr/lib/postgresql/<major>/bin/pg_{dump,restore}` when present, with PATH fallback. The stale 2026-11-17 dump was
regenerated with pg_dump 16 before the rehearsal was accepted.
