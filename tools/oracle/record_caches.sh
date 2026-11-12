#!/usr/bin/env bash
# Records the lockstep record cache (docs/ORACLE.md §6) of a list of rows, several at a time.
#
#   tools/oracle/record_caches.sh <rows> <bundle> <data> <rom> <cache> [jobs]
#
# <rows>    a file of row ids, one per line; `#` starts a comment. `-` reads stdin.
# <bundle>  the decrypted relock bundle's `relock-v2-lsnes/` directory (its `pads/` and `seeds/`).
# <data>    the decrypted bundles of scenarios/lock.toml, one directory per bundle stem: corpus
#           still reads each row's start and its movie from the old lock (ORACLE §9).
# <rom>     the unheadered ROM.
# <cache>   the cache root; each row is written to `<cache>/<row>/`.
# [jobs]    rows recorded at once, default 8. Each is one lsnes process.
#
# Each row is `oracle corpus ... --oracle lsnes --native-seeds <bundle>/seeds
# --from-frame-pads <dir> --cache <cache> --only <row>`, with the row's pad file copied to a
# directory of its own, since corpus writes `<row>.played` beside the pads. No pictures, so
# corpus writes the lean tier (`lean.bin`, `lean.txt`): the harnesses and lockstep don't read
# pictures, and the lean tier is about a hundredth of the full one's bytes.
#
# Resumable: a row whose `lean.txt` exists (the writer writes it last) with the bundle's pad
# hash is skipped. One line per row on stdout: `<row> cached|recorded|failed ...`; each row's
# corpus output is kept in `<cache>/.logs/<row>.log`.
#
# ORACLE is the command that runs the oracle's command line, default `target/verify/oracle`.
# Where Defender blocks fresh executables (docs/ORACLE.md §8), it is the Python DLL runner, for
# example ORACLE="python $TEMP/collab/run.py $TEMP/collab/oracle_cache.dll".
#
# WATCH_PCS, when set, is the comma-separated list of instruction addresses passed to corpus as
# `--watch-pcs`, so each row also gets its `hits.txt` (docs/ORACLE.md §6): the harnesses that hear
# requests from hits (blocks, progress, the HUD) need it. The records are the same either way.
set -euo pipefail

if [ $# -lt 5 ] || [ $# -gt 6 ]; then
  echo "usage: $0 <rows> <bundle> <data> <rom> <cache> [jobs]" >&2
  exit 2
fi
rows_file=$1
bundle=$2
data=$3
rom=$4
cache=$5
jobs=${6:-8}
here=$(cd "$(dirname "$0")/../.." && pwd)
lock=$here/scenarios/lock.toml
read -r -a oracle <<<"${ORACLE:-$here/target/verify/oracle}"
watch=()
if [ -n "${WATCH_PCS:-}" ]; then
  watch=(--watch-pcs "$WATCH_PCS")
fi

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
mkdir -p "$cache/.logs" "$cache/.pads"

mapfile -t rows < <(sed -e 's/#.*//' -e 's/[[:space:]]//g' -e '/^$/d' "$rows_file")

# One row: skip it when cached, else record it. Prints the row's one line.
record() {
  local row=$1 pads=$bundle/pads/$1.pads index=$cache/$1/lean.txt
  local log=$cache/.logs/$1.log work=$cache/.pads/$1
  if [ ! -f "$pads" ]; then
    echo "$row failed: no $row.pads in the bundle"
    return
  fi
  local sha
  sha=$(sha256sum <"$pads" | cut -d' ' -f1)
  if [ -f "$index" ] && grep -qx "pads $sha" "$index"; then
    echo "$row cached $(sed -n 's/^frames //p' "$index") frames"
    return
  fi
  mkdir -p "$work"
  cp "$pads" "$work/"
  local started=$SECONDS status=0
  "${oracle[@]}" corpus "$rom" "$lock" "$data" --oracle lsnes --native-seeds "$bundle/seeds" \
    --from-frame-pads "$work" --cache "$cache" --only "$row" "${watch[@]}" >"$log" 2>&1 || status=$?
  local seconds=$((SECONDS - started))
  local line
  line=$(grep -E "^  $row cache: " "$log" || true)
  if [ "$status" -eq 0 ] && [ -f "$index" ] && [ -n "$line" ]; then
    echo "$row recorded ${line#*cache: } in ${seconds} s"
    rm -rf "$work"
  else
    echo "$row failed: exit $status after ${seconds} s, see $log"
  fi
}

running=0
for row in "${rows[@]}"; do
  record "$row" &
  running=$((running + 1))
  if [ "$running" -ge "$jobs" ]; then
    wait -n
    running=$((running - 1))
  fi
done
wait
