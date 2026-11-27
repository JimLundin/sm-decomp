#!/usr/bin/env bash
# Records the lockstep record cache (docs/ORACLE.md §6) of a list of rows, several at a time.
#
#   tools/oracle/record_caches.sh <rows> <bundle> <rom> <cache> [jobs]
#
# <rows>    a file of row ids, one per line; `#` starts a comment. `-` reads stdin.
# <bundle>  the decrypted relock bundle's `relock-v2-lsnes/` directory (its `pads/` and `seeds/`).
# <rom>     the unheadered ROM.
# <cache>   the cache root; each row is written to `<cache>/<row>/`.
# [jobs]    rows recorded at once, default 8. Each is one lsnes process.
#
# Each row is `oracle corpus ROM LOCK BUNDLE --cache CACHE --only ROW`, which writes
# `lean.bin` and `lean.txt`.
#
# Resumable: reuse a readable lean row with its pinned inputs and all requested watches.
# Each row reports cached, recorded or failed, with corpus output in <cache>/.logs/<row>.log.
#
# ORACLE runs the oracle command line (default target/verify/oracle).
# SM_TOOLS runs the cache reader (default target/verify/tools, built before use).
# SM_LOCK names the format-2 lock (default scenarios/locks/lsnes.toml).
#
# WATCH_PCS, when set, is the comma-separated list of instruction addresses passed to corpus as
# `--watch-pcs`, so each row also gets its `hits.txt` (docs/ORACLE.md §6): the harnesses that hear
# requests from hits (blocks, progress, the HUD) need it. The records are the same either way.
set -euo pipefail

if ((BASH_VERSINFO[0] < 5 || (BASH_VERSINFO[0] == 5 && BASH_VERSINFO[1] < 1))); then
  echo "cache recording requires Bash 5.1 or later" >&2
  exit 2
fi

if [ $# -lt 4 ] || [ $# -gt 5 ]; then
  echo "usage: $0 <rows> <bundle> <rom> <cache> [jobs]" >&2
  exit 2
fi
rows_file=$1
bundle=$2
rom=$3
cache=$4
jobs=${5:-8}
if [[ ! $jobs =~ ^[1-9][0-9]*$ ]]; then
  echo "jobs must be a positive integer" >&2
  exit 2
fi
here=$(cd "$(dirname "$0")/../.." && pwd)
lock=${SM_LOCK:-$here/scenarios/locks/lsnes.toml}
read -r -a oracle <<<"${ORACLE:-$here/target/verify/oracle}"
for directory in "$bundle/pads" "$bundle/seeds"; do
  if [ ! -d "$directory" ]; then
    echo "$directory: missing; <bundle> is the relock bundle's relock-v2-lsnes/" >&2
    exit 1
  fi
done
if [ ! -f "$rom" ]; then
  echo "$rom: missing" >&2
  exit 1
fi
mkdir -p "$cache/.logs" "$cache/.locks"

rows_text=$(awk '{ sub(/#.*/, ""); gsub(/[[:space:]]/, ""); if (length && !seen[$0]++) print }' "$rows_file")
if [ -z "$rows_text" ]; then
  exit 0
fi
mapfile -t rows <<<"$rows_text"

artifacts=${CARGO_TARGET_DIR:-$here/target}
checker=${SM_TOOLS:-$artifacts/verify/tools}
if [ -z "${SM_TOOLS:-}" ]; then
  cargo build --quiet --locked --profile verify --manifest-path "$here/Cargo.toml" \
    --target-dir "$artifacts" -p tools
fi
if ! command -v "$checker" >/dev/null; then
  echo "cache reader unavailable: $checker" >&2
  exit 2
fi

# Validate the command and lock before any worker can replace a row.
lock_rows=$("$checker" cache-check "$lock")
if [[ ! $lock_rows =~ ^[1-9][0-9]*$ ]]; then
  echo "cache reader returned no lock rows: $checker" >&2
  exit 2
fi

cache_check() {
  "$checker" cache-check "$lock" "$bundle" "$@"
}

publish_row() {
  if [ ! -e "$2" ]; then
    mv -T -- "$1" "$2"
    return
  fi
  # Exchange complete directories so concurrent readers never see a publish gap.
  # The old row moves into staging, where the worker's exit trap removes it.
  python3 - "$1" "$2" <<'PYTHON'
import ctypes
import os
import sys

AT_FDCWD = -100
RENAME_EXCHANGE = 2
rename = ctypes.CDLL("libc.so.6", use_errno=True).renameat2
rename.argtypes = (ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint)
source, target = map(os.fsencode, sys.argv[1:])
if rename(AT_FDCWD, source, AT_FDCWD, target, RENAME_EXCHANGE):
    error = ctypes.get_errno()
    sys.exit(f"{os.fsdecode(target)}: {os.strerror(error)}")
PYTHON
}

# A worker owns its staging directory. Only publish rows the reader accepts.
record() (
  row=$1
  if [[ ! $row =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]]; then
    echo "invalid cache row id: $row" >&2
    exit 1
  fi
  exec {row_lock}>"$cache/.locks/$row"
  flock "$row_lock"
  pads=$bundle/pads/$row.pads
  target=$cache/$row
  log=$cache/.logs/$row.log
  if [ ! -f "$pads" ]; then
    echo "$row failed: no $row.pads in the bundle"
    exit 1
  fi
  requested=${WATCH_PCS:-}
  if [ -f "$target/lean.txt" ]; then
    if frames=$(cache_check "$cache" "$row" "$requested" 2>/dev/null); then
      echo "$row cached $frames frames"
      exit 0
    fi
    if combined=$(cache_check "$cache" "$row" --carry-watches "$requested" 2>/dev/null); then
      requested=$combined
    fi
  fi
  watch=()
  if [ -n "$requested" ]; then watch=(--watch-pcs "$requested"); fi

  stage=$(mktemp -d "$cache/.record.XXXXXXXX")
  trap 'rm -rf -- "$stage"' EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
  started=$SECONDS
  status=0
  "${oracle[@]}" corpus "$rom" "$lock" "$bundle" --cache "$stage/cache" --only "$row" "${watch[@]}" \
    >"$log" 2>&1 || status=$?
  if [ "$status" -ne 0 ]; then echo "oracle exit $status" >>"$log"; fi
  if [ "$status" -eq 0 ] && frames=$(cache_check "$stage/cache" "$row" "$requested" 2>>"$log"); then
    publish_row "$stage/cache/$row" "$target"
    echo "$row recorded $frames frames in $((SECONDS - started)) s"
  else
    echo "$row failed after $((SECONDS - started)) s, see $log"
    exit 1
  fi
)

# Completed workers can disappear from wait -n before it sees them. Wait by
# PID for those jobs, and let wait -n pick the next active worker otherwise.
status=0
declare -A workers=()
reap_worker() {
  local running worker finished result
  while :; do
    running=$'\n'$(jobs -pr)$'\n'
    for worker in "${!workers[@]}"; do
      if [[ $running != *$'\n'"$worker"$'\n'* ]]; then
        wait "$worker" || status=1
        unset 'workers[$worker]'
        return
      fi
    done
    result=0
    wait -n -p finished "${!workers[@]}" 2>/dev/null || result=$?
    if [ -n "${finished:-}" ]; then
      if [ "$result" -ne 0 ]; then status=1; fi
      unset 'workers[$finished]'
      return
    fi
  done
}
for row in "${rows[@]}"; do
  record "$row" &
  workers[$!]=1
  if [ "${#workers[@]}" -ge "$jobs" ]; then
    reap_worker
  fi
done
while [ "${#workers[@]}" -gt 0 ]; do
  reap_worker
done
exit "$status"
