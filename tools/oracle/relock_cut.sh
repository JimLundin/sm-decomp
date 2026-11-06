#!/usr/bin/env bash
# Run a re-lock seed-cut plan (tools/oracle/relock_plan.sh; docs/ORACLE.md, RELOCK
# step a2): one lsnes `movie` run a window, a few at a time, then the
# derivations, then the check.
#
# Usage: tools/oracle/relock_cut.sh PLAN OUT LOGS [PHASE] [WINDOW...]
#   PHASE   cut, realign, derive, check, or all of them in that order (default)
#   PLAN    the plan; OUT/<capture>/NNNN.lss (+ .record, or .words) are written
#   LOGS    one log a job (LOGS/<movie>-<rung>.log, LOGS/derive-<capture>.log),
#           the window CSVs (LOGS/csv/) and the check's report (LOGS/check.txt)
#   WINDOW  MOVIE-RUNG, e.g. 100-50000: only those windows (all by default);
#           derive and check then cover only those windows' seeds
# Environment:
#   ORACLE   the oracle command (`oracle` binary or a wrapper of it)
#   ROM      the ROM; INPUTS a directory with MOVIE.inputs for each movie
#   LADDERS  the ladders, LADDERS/<movie>/NNNNNN.lss
#   CAPTURES directories holding lsnes room captures as
#            <dir>/target/seeds/<capture>/NNNN-ROOM-wram.bin (the old bundles,
#            unpacked); space-separated
#   JOBS     concurrent jobs (8); STAGGER seconds between lsnes starts (4)
#
# realign: a capture usually follows step FRAME-1 of its movie, some follow
# step FRAME (the CSV can't tell, the game's frame counter can); those are
# re-cut one step later, from the seed itself.
#
# The check: every planned seed exists. A cut seed's record carries the
# memories (`movie` dumps them on save steps); its room word ($7E:079B) and
# frame counter ($7E:05B6) equal the CSV row's `room` and `nmi`, and its work RAM equals the lsnes room capture's
# byte for byte outside the stack page $1F00-$1FFF, as `movie --seeds` counts.
# A derived seed has its .words and derive-seed confirmed every word stuck at
# the load (it reads the written value, or other than the plain base, one step
# on: physics moves some words in that step). Exit 1
# when any seed fails.
set -euo pipefail

if [ $# -lt 3 ]; then
    echo "usage: $0 PLAN OUT LOGS [cut|realign|derive|check|all] [WINDOW...]" >&2
    exit 2
fi
plan=$1
out=$2
logs=$3
phase=${4:-all}
shift $(($# < 4 ? $# : 4))
windows=" $* "
root=$(cd "$(dirname "$0")/../.." && pwd)
jobs=${JOBS:-8}
stagger=${STAGGER:-4}
mkdir -p "$logs/csv" "$out"

# Whether window MOVIE-RUNG is selected.
selected() { [ "$windows" = "  " ] || [[ $windows == *" $1 "* ]]; }

# The plan's cut lines of the selected windows.
cuts() {
    awk '$1 == "cut"' "$plan" | while read -r _ movie rung rest; do
        if selected "$movie-$rung"; then echo "$movie $rung $rest"; fi
    done
}

# Start a job once fewer than JOBS run, STAGGER seconds after the last start.
started=0
start_job() {
    while [ "$(jobs -rp | wc -l)" -ge "$jobs" ]; do wait -n || true; done
    if [ "$started" -gt 0 ]; then sleep "$stagger"; fi
    started=$((started + 1))
    "$@" < /dev/null &
}

# One window: MOVIE RUNG, then its captures as CAPTURE INDEX FRAME triples.
window() {
    local movie=$1 rung=$2 log=$logs/$1-$2.log arguments=() last=0 capture index frame csv
    local -A rows=()
    shift 2
    while [ $# -ge 3 ]; do
        capture=$1 index=$2 frame=$3
        shift 3
        rows[$capture]+=" $index"
        if [ "$frame" -gt "$last" ]; then last=$frame; fi
    done
    for capture in "${!rows[@]}"; do
        csv=$logs/csv/$movie-$rung-$capture.csv
        awk -F, -v keep="${rows[$capture]} " 'NR == 1 || index(keep, " " $1 " ")' \
            "$root/movies/room-seeds-$capture.csv" > "$csv"
        mkdir -p "$out/$capture"
        arguments+=(--native-seeds "$csv" "$out/$capture")
    done
    if [ "$rung" -gt 0 ]; then
        arguments+=(--from "$(printf '%s/%s/%06d.lss' "$LADDERS" "$movie" "$rung")" "$rung")
    fi
    # A seed after step F-1 is copied once step F has run.
    local status=0 begun=$SECONDS
    "$ORACLE" movie "$ROM" "$INPUTS/$movie.inputs" --oracle lsnes --no-memories --no-picture --ppu \
        --frames $((last + 1)) "${arguments[@]}" > "$log" 2>&1 || status=$?
    echo "relock_cut $movie-$rung exit $status in $((SECONDS - begun))s" >> "$log"
    echo "window $movie-$rung exit $status in $((SECONDS - begun))s"
}

cut_phase() {
    # Longest windows first: the frames from the rung to the last capture.
    cuts | awk '{ w = $1 " " $2; if ($6 - $2 > n[w]) n[w] = $6 - $2 } END { for (w in n) print n[w], w }' |
        sort -rn | while read -r _ movie rung; do echo "$movie $rung"; done > "$logs/windows.txt"
    while read -r movie rung; do
        # shellcheck disable=SC2046
        start_job window "$movie" "$rung" $(cuts | awk -v m="$movie" -v r="$rung" '$1 == m && $2 == r { print $3, $4, $6 }')
    done < "$logs/windows.txt"
    wait
}

# The derivations whose base a selected window cuts, or all.
derivations() {
    awk '$1 == "derive"' "$plan" | while read -r _ capture index base base_index words; do
        if [ "$windows" = "  " ] || cuts | awk -v c="$base" -v i="$base_index" '$3 == c && $4 == i { found = 1 } END { exit !found }' ||
            [ -f "$(printf '%s/%s/%04d.lss' "$out" "$base" "$base_index")" ]; then
            echo "$capture $index $base $base_index $words"
        fi
    done
}

derive() {
    local capture=$1 index=$2 base=$3 base_index=$4 log=$logs/derive-$1.log status=0
    shift 4
    "$ORACLE" derive-seed "$ROM" "$(printf '%s/%s/%04d.lss' "$out" "$base" "$base_index")" \
        "$(printf '%s/%s/%04d.lss' "$out" "$capture" "$index")" "$@" > "$log" 2>&1 || status=$?
    echo "relock_cut derive $capture exit $status" >> "$log"
    echo "derive $capture exit $status"
}

derive_phase() {
    started=0
    while read -r capture index base base_index words; do
        # shellcheck disable=SC2086
        start_job derive "$capture" "$index" "$base" "$base_index" $words
    done < <(derivations)
    wait
}

# Work RAM bytes of RECORD (after its 43-byte header) that differ from WRAM,
# outside the stack page.
differing() {
    cmp -l <(tail -c +44 "$1" | head -c 131072) "$2" 2>/dev/null |
        awk '{ offset = $1 - 1 } offset < 7936 || offset >= 8192 { n++ } END { print n + 0 }'
}

# Work RAM word OFFSET of RECORD, as an unsigned decimal.
record_word() { od -An -tu2 -j $((43 + $2)) -N 2 "$1" | tr -d ' '; }

# The game's frame counter ($7E:05B6) at a capture: the CSV's `nmi` column.
csv_counter() {
    awk -F, -v index_="$2" 'NR == 1 { for (i = 1; i <= NF; i++) if ($i == "nmi") column = i; next }
        $1 == index_ { print $column; exit }' "$root/movies/room-seeds-$1.csv" | tr -d '\r'
}

# A capture usually follows step FRAME-1, but some follow step FRAME: the
# record's frame counter is then one short of the capture's. Re-cut those one
# step later from the seed itself.
realign_phase() {
    local movie rung capture index room frame stem counter expected csv
    while read -r movie rung capture index room frame; do
        stem=$(printf '%s/%s/%04d' "$out" "$capture" "$index")
        if [ ! -f "$stem.record" ] || [ "$(wc -c < "$stem.record")" -lt $((43 + 131072)) ]; then continue; fi
        counter=$(record_word "$stem.record" $((0x05B6)))
        expected=$(csv_counter "$capture" "$index")
        if [ "$(((counter + 1) % 65536))" != "$expected" ]; then continue; fi
        csv=$logs/csv/realign-$capture-$index.csv
        printf 'room_index,room,movie_frame\n%s,%s,%s\n' "$index" "$room" $((frame + 1)) > "$csv"
        mkdir -p "$logs/realign"
        "$ORACLE" movie "$ROM" "$INPUTS/$movie.inputs" --oracle lsnes --no-memories --no-picture --ppu \
            --from "$stem.lss" "$frame" --frames $((frame + 2)) --native-seeds "$csv" "$logs/realign" \
            > "$logs/realign-$capture-$index.log" 2>&1
        mv -f "$(printf '%s/realign/%04d' "$logs" "$index").lss" "$stem.lss"
        mv -f "$(printf '%s/realign/%04d' "$logs" "$index").record" "$stem.record"
        echo "realign $capture:$index one step later (frame counter $counter, capture $expected)"
    done < <(cuts)
}

check_phase() {
    local report=$logs/check.txt failures=0 seeds=0 compared=0
    : > "$report"
    while read -r movie rung capture index room frame; do
        seeds=$((seeds + 1))
        local stem record capture_wram="" size dumped found counter expected
        stem=$(printf '%s/%s/%04d' "$out" "$capture" "$index")
        record=$stem.record
        if [ ! -f "$stem.lss" ] || [ ! -f "$record" ]; then
            echo "FAIL $capture:$index missing ($movie-$rung)" >> "$report"
            failures=$((failures + 1))
            continue
        fi
        size=$(wc -c < "$record")
        dumped=$(od -An -tu1 -j 42 -N 1 "$record" | tr -d ' ')
        if [ "$size" -lt $((43 + 131072)) ] || [ "$dumped" = 0 ]; then
            echo "FAIL $capture:$index record without memories ($size bytes)" >> "$report"
            failures=$((failures + 1))
            continue
        fi
        found=$(od -An -tx1 -j $((43 + 0x079B)) -N 2 "$record" | awk '{ print toupper($2 $1) }')
        for directory in ${CAPTURES:-}; do
            for file in "$directory"/target/seeds/"$capture"/"$(printf '%04d' "$index")"-*-wram.bin; do
                if [ -f "$file" ]; then capture_wram=$file; break 2; fi
            done
        done
        counter=$(record_word "$record" $((0x05B6)))
        expected=$(csv_counter "$capture" "$index")
        if [ "$found" != "$room" ]; then
            echo "FAIL $capture:$index room $found, CSV $room" >> "$report"
            failures=$((failures + 1))
        elif [ "$counter" != "$expected" ]; then
            echo "FAIL $capture:$index frame counter $counter, CSV $expected" >> "$report"
            failures=$((failures + 1))
        elif [ -z "$capture_wram" ]; then
            echo "ok   $capture:$index room $room frame $frame; no lsnes capture to compare" >> "$report"
        else
            local count
            count=$(differing "$record" "$capture_wram")
            compared=$((compared + 1))
            if [ "$count" -ne 0 ]; then
                echo "FAIL $capture:$index room $room frame $frame: $count WRAM bytes differ from $capture_wram" >> "$report"
                failures=$((failures + 1))
            else
                echo "ok   $capture:$index room $room frame $frame: 0 WRAM bytes differ" >> "$report"
            fi
        fi
    done < <(cuts)
    while read -r capture index _; do
        seeds=$((seeds + 1))
        local stem
        stem=$(printf '%s/%s/%04d' "$out" "$capture" "$index")
        if [ -f "$stem.lss" ] && [ -f "$stem.words" ] && grep -q "^relock_cut derive $capture exit 0" "$logs/derive-$capture.log" 2>/dev/null; then
            echo "ok   $capture:$index derived, every word kept at load" >> "$report"
        else
            echo "FAIL $capture:$index not derived" >> "$report"
            failures=$((failures + 1))
        fi
    done < <(derivations)
    echo "relock check: $seeds seeds, $compared compared with an lsnes capture, $failures failing" | tee -a "$report"
    [ "$failures" -eq 0 ]
}

case $phase in
    cut) cut_phase ;;
    derive) derive_phase ;;
    realign) realign_phase ;;
    check) check_phase ;;
    all) cut_phase; realign_phase; derive_phase; check_phase ;;
    *) echo "unknown phase $phase" >&2; exit 2 ;;
esac
