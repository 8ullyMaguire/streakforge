#!/usr/bin/env bash
# A second copy of the production dump, on a machine that is not the one that
# would lose it.
#
# WHY (docs/100_IDEAS.md #03): the nightly pg_dump and the monthly restore
# rehearsal both live on thinkcentre. The idea names its own purpose -- turning
# "the disk died" from the end of the site into an hour -- and a dump on the disk
# that dies does not do that. This pulls the newest dump somewhere else.
#
# WHY A PULL AND NOT A PUSH: the host holding the database then needs no
# credential on the machine that stores the copy, and the machine that keeps the
# copy decides what it accepts. A push would mean keeping a write path into the
# backup store on the host most likely to be compromised.
#
# WHAT IT DOES *NOT* DO, so nobody over-reads the reassurance: a copy on the same
# LAN does not survive the house. This is off-HOST, not off-SITE. Adding a real
# off-site target is one line at the end of this script (see OPERATIONS).
#
# Usage:  ./scripts/offhost-backup.sh            # normal run
#         DRY_RUN=1 ./scripts/offhost-backup.sh  # say what it would do
set -uo pipefail

REMOTE="${REMOTE:-thinkcentre}"
REMOTE_DIR="${REMOTE_DIR:-/home/alvaro/backups/streakforge}"
DEST="${DEST:-$HOME/backups/streakforge-offhost}"
KEEP="${KEEP:-14}"
MIN_BYTES="${MIN_BYTES:-10000}"
MAX_AGE_HOURS="${MAX_AGE_HOURS:-48}"
DRY_RUN="${DRY_RUN:-0}"

ssh_() { ssh -o BatchMode=yes -o ConnectTimeout=15 "$REMOTE" "$@"; }
fatal() { echo "FATAL: $*" >&2; exit 1; }

mkdir -p "$DEST" || fatal "cannot create $DEST"

# 1. Find the newest dump ON THE OTHER HOST. If there is none, that is the
#    failure this script exists to notice, so it is fatal here rather than a
#    quiet success with nothing to copy.
newest=$(ssh_ "ls -1t $REMOTE_DIR/*.dump 2>/dev/null | head -1") ||
  fatal "cannot reach $REMOTE to list $REMOTE_DIR"
[ -n "${newest:-}" ] || fatal "$REMOTE has no $REMOTE_DIR/*.dump at all — the nightly job is not running"

read -r mtime size < <(ssh_ "stat -c '%Y %s' '$newest'") || fatal "cannot stat $newest on $REMOTE"
[ -n "${mtime:-}" ] && [ -n "${size:-}" ] || fatal "unreadable stat output for $newest"

now=$(date +%s)
age_h=$(( (now - mtime) / 3600 ))

# 2. A stale dump means the BACKUP is broken, not that this copy failed. Same
#    rule the restore rehearsal uses, checked here too because this script runs
#    on a different machine and will notice a host that is up but idle.
if [ "$age_h" -gt "$MAX_AGE_HOURS" ]; then
  fatal "$REMOTE's newest dump is ${age_h}h old (limit ${MAX_AGE_HOURS}h). That is a BACKUP failure, not a copy failure — check the nightly cron on $REMOTE"
fi
[ "$size" -ge "$MIN_BYTES" ] || fatal "$newest is only ${size}B (floor ${MIN_BYTES}B) — a truncated or empty dump"

base=$(basename "$newest")
here="$DEST/$base"

# 3. Copy only when it is not already here at the same length, so a re-run is
#    free and a partial file can never be mistaken for a complete one.
if [ -f "$here" ] && [ "$(stat -c %s "$here")" = "$size" ]; then
  echo "ok   $base already here ($size bytes, ${age_h}h old) — nothing to copy"
elif [ "$DRY_RUN" = "1" ]; then
  echo "would copy $base ($size bytes, ${age_h}h old) to $here"
  exit 0
else
  tmp="$DEST/.$base.part"
  rm -f "$tmp"
  ssh_ "cat '$newest'" > "$tmp" || { rm -f "$tmp"; fatal "the copy from $REMOTE failed"; }
  got=$(stat -c %s "$tmp")
  [ "$got" = "$size" ] || { rm -f "$tmp"; fatal "short copy: $got of $size bytes — refused to keep it"; }
  mv "$tmp" "$here" || fatal "cannot move the copy into place"
  echo "ok   copied $base ($size bytes, ${age_h}h old)"
fi

# 4. Length is not the same as readability: prove the archive can be opened.
if command -v pg_restore >/dev/null 2>&1; then
  lines=$(pg_restore --list "$here" 2>/dev/null | grep -c . || true)
  [ "${lines:-0}" -gt 0 ] || fatal "$base is not a readable pg_restore archive"
  echo "ok   pg_restore lists $lines entries in $base"
else
  # Length + a full listing is what the host itself checks; do the one check
  # available here and say which check was skipped rather than implying it ran.
  echo "note pg_restore is not installed here, so only the length was verified"
fi

# 5. Keep a rolling window. Old copies are the least useful ones and the whole
#    point is that a fresh copy always fits.
if [ "$KEEP" -gt 0 ]; then
  removed=$(ls -1t "$DEST"/*.dump 2>/dev/null | tail -n +$((KEEP + 1)) | tee /dev/stderr | wc -l)
  ls -1t "$DEST"/*.dump 2>/dev/null | tail -n +$((KEEP + 1)) | xargs -r rm -f
  [ "$removed" -gt 0 ] && echo "ok   pruned $removed beyond the last $KEEP"
fi

echo "ok   $DEST holds $(ls -1 "$DEST"/*.dump 2>/dev/null | wc -l) dumps, newest $base"
