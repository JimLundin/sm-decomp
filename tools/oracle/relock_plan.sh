#!/usr/bin/env bash
# The seed-cut plan of the lsnes re-lock (docs/oracle/RELOCK.md, step a1): every
# seed the lock's rows start from that NATIVE doesn't hold yet.
#
# Usage: tools/oracle/relock_plan.sh NATIVE LADDERS > PLAN
#   NATIVE   lsnes native seeds: NATIVE/<capture>/NNNN.lss
#   LADDERS  the movie ladders: LADDERS/<movie>/ladder.txt
# Environment: PYTHON (default python), to run tools/scenario_seeds.py.
#
# It reads scenarios/lock.toml (each row's seed, by its pin), the room-seed
# CSVs movies/room-seeds-<capture>.csv and the scenario definitions, and adds
# each derivation's base seed. One line a seed, sorted so that each window's
# captures are adjacent and in frame order:
#   cut MOVIE RUNG CAPTURE INDEX ROOM FRAME
#       a movie capture: its row of the capture's CSV, cut from ladder rung
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
python=${PYTHON:-python}

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

# The distinct seeds of the lock's rows: a [[scenario]]'s id, through its pin.
seeds=$(awk '
    /^\[\[scenario\]\]/ { row = 1; next }
    row && /^id = "/ { split($0, part, "\""); rows[part[2]] = 1; row = 0; next }
    /\{ id = "/ && /seed = "/ {
        match($0, /id = "[^"]*"/); id = substr($0, RSTART + 6, RLENGTH - 7)
        match($0, /seed = "[^"]*"/); pins[id] = substr($0, RSTART + 8, RLENGTH - 9)
    }
    END { for (id in rows) if (id in pins) print pins[id] }
' "$root/scenarios/lock.toml" | sort -u)

# Each scenario seed's base and words, from the scenario definitions.
derivations=$(
    "$python" -c 'import sys; sys.path.insert(0, sys.argv[1]); import scenario_seeds; print("\n".join(sorted(scenario_seeds.SCENARIOS)))' \
        "$root/tools" | tr -d '\r' | while read -r name; do
        "$python" "$root/tools/scenario_seeds.py" native-args "$name" @ | tr -d '\r'
    done
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
    csv=$root/movies/room-seeds-$capture.csv
    if [ -z "$movie" ]; then
        echo "skip $capture $index no-lsnes-movie"
        continue
    fi
    row=$(awk -F, -v index_="$index" 'NR > 1 && $1 == index_ { print $2, $3; exit }' "$csv" | tr -d '\r')
    if [ -z "$row" ]; then
        echo "skip $capture $index no-csv-row"
        continue
    fi
    read -r room frame <<<"$row"
    echo "cut $movie $(rung_for "$movie" $((frame - 1))) $capture $index $room $frame"
done | sort -k1,1 -k2,2 -k3,3n -k7,7n -k4,4 -k5,5n
