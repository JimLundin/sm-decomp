#!/usr/bin/env bash
# The seed-cut plan of the lsnes re-lock (docs/ORACLE.md, RELOCK step a1): every
# seed the lock's rows start from that NATIVE doesn't hold yet.
#
# Usage: tools/oracle/relock_plan.sh NATIVE LADDERS > PLAN
#   NATIVE   lsnes native seeds: NATIVE/<capture>/NNNN.lss
#   LADDERS  the movie ladders: LADDERS/<movie>/ladder.txt
# Environment: PYTHON (default python3), with tomllib.
#
# It reads each seeded start from scenarios/locks/lsnes.toml, uses the room-seed
# tables movies/room-seeds-<capture>.toml and the scenario edit recipes, and adds
# each derivation's base seed. One line a seed, sorted so that each window's
# captures are adjacent and in frame order:
#   cut MOVIE RUNG CAPTURE INDEX ROOM FRAME
#       a movie capture: its seed in the capture's table, cut from ladder rung
#       RUNG of MOVIE (the latest at or before step FRAME-1; 0 is power-on).
#       A window is one (MOVIE, RUNG): one `movie --from` run.
#   derive CAPTURE INDEX BASE BASE_INDEX --word OFFSET=VALUE...
#       a scenario seed, `oracle derive-seed` from the base capture's seed.
#   skip CAPTURE INDEX REASON
#       a seed no lsnes movie gives.
set -euo pipefail

if [ $# -ne 2 ]; then
    echo "usage: $0 NATIVE LADDERS" >&2
    exit 2
fi
native=$1
ladders=$2
root=$(cd "$(dirname "$0")/../.." && pwd)
python=${PYTHON:-python3}

# The movie a capture was cut from; empty when no lsnes movie gives it.
movie_of() {
    case $1 in
        100-*) echo 100 ;;
        any-*) echo any ;;
        low-*) echo low ;;
        spore-*) echo spore ;;
        *) echo "" ;;
    esac
}

# The latest rung of MOVIE at or before step STEP; 0 (power-on) without one.
rung_for() {
    local file=$ladders/$1/ladder.txt
    if [ ! -f "$file" ]; then
        echo 0
        return
    fi
    awk -v step="$2" '!/^#/ && $1 <= step && $1 > best { best = $1 } END { print best + 0 }' "$file"
}

# The current lock names each native start directly.
seeds=$("$python" - "$root/scenarios/locks/lsnes.toml" <<'PYTHON'
import sys
import tomllib

with open(sys.argv[1], "rb") as lock:
    document = tomllib.load(lock)
if document["format"] != 2:
    sys.exit("expected format-2 lock")
print("\n".join(sorted({row["start"] for row in document["row"]} - {"power-on"})))
PYTHON
)

# Each scenario seed's base and words, from the scenario edit recipes.
derivations=$("$python" - "$root/tools" <<'PYTHON'
import sys

sys.path.insert(0, sys.argv[1])
from scenario_seeds import SCENARIOS, native_args

for name in sorted(SCENARIOS):
    native_args(name, "@")
PYTHON
)

# Whether NATIVE holds seed CAPTURE INDEX.
held() { [ -f "$(printf '%s/%s/%04d.lss' "$native" "$1" "$2")" ]; }

# A scenario capture's derivation: "BASE_CAPTURE BASE_INDEX WORDS..."; empty
# without a scenario definition.
derivation_of() {
    # "@/BASE/IIII.lss @/scenario-NAME/IIII.lss --word ..."
    local line base base_index words
    line=$(printf '%s\n' "$derivations" | grep -F " @/$1/" || true)
    if [ -z "$line" ]; then return; fi
    read -r base _ words <<<"$line"
    base=${base#@/}
    base_index=${base##*/}
    echo "${base%/*} $((10#${base_index%.lss})) $words"
}

# A derive's base is cut too, also when no row starts from it directly.
bases=$(for seed in $seeds; do
    capture=${seed%:*}
    case $capture in scenario-*) ;; *) continue ;; esac
    if held "$capture" "${seed#*:}"; then continue; fi
    read -r base_capture base_index _ <<<"$(derivation_of "$capture")" || true
    if [ -n "${base_capture:-}" ]; then echo "$base_capture:$base_index"; fi
done)
seeds=$(printf '%s\n' $seeds $bases | sort -u)

for seed in $seeds; do
    capture=${seed%:*}
    index=${seed#*:}
    if held "$capture" "$index"; then
        continue
    fi
    case $capture in
        scenario-*)
            derivation=$(derivation_of "$capture")
            if [ -z "$derivation" ]; then
                echo "skip $capture $index no-scenario-definition"
            else
                echo "derive $capture $index $derivation"
            fi
            continue
            ;;
    esac
    movie=$(movie_of "$capture")
    table=$root/movies/room-seeds-$capture.toml
    if [ -z "$movie" ]; then
        echo "skip $capture $index no-lsnes-movie"
        continue
    fi
    row=$("$python" -c 'import sys, tomllib; print(*next(([s["room"], s["movie_frame"]] for s in tomllib.load(open(sys.argv[1], "rb"))["seed"] if s["room_index"] == int(sys.argv[2])), []))' "$table" "$index")
    if [ -z "$row" ]; then
        echo "skip $capture $index no-table-seed"
        continue
    fi
    read -r room frame <<<"$row"
    echo "cut $movie $(rung_for "$movie" $((frame - 1))) $capture $index $room $frame"
done | sort -k1,1 -k2,2 -k3,3n -k7,7n -k4,4 -k5,5n
