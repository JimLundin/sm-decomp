#!/usr/bin/env bash
# Re-cut the relock bundle's seed records with the PPU tier (`movie --ppu`) and
# repack the bundle (docs/oracle/RELOCK.md §9). A lean record decodes its PPU
# registers as the defaults, so the free-run gate's `ppu` stream differed from
# frame 0.
#
# Usage: tools/oracle/relock_ppu_recut.sh OLD PLAN WORK [ROW...]
#   OLD   the current bundle, decrypted: the relock-v2-lsnes/ directory
#   PLAN  the cut plan(s) the seeds were cut by (relock_plan.sh's plan and any
#         extra cut lines, concatenated); one cut line per seed record in OLD
#   WORK  written: WORK/seeds (the re-cut seeds), WORK/logs (relock_cut.sh's
#         logs), WORK/relock-v2-lsnes (the new tree) and the bundle
#   ROW   re-cut only the seeds these lock rows start from (default: every
#         record in OLD); the other records stay as they are
# Environment: ORACLE, ROM, INPUTS, LADDERS, JOBS (8), STAGGER (4), as
# relock_cut.sh reads them.
#
# Each re-cut record must be the old record byte for byte, then the PPU part
# alone ("OHS1", flag 4, 146 bytes); its state must equal the old state in every
# member but `rrdata`, lsnes's per-run rerecord data. The new tree keeps the old
# .lss files, so the lock's `seed_record` pins (the .lss SHA-256) stay valid.
# Prints the bundle's name, its ciphertext SHA-256 and the lock's new
# `bundles` line; exits 1 when a record or state fails the check.
set -euo pipefail

if [ $# -lt 3 ]; then
    echo "usage: $0 OLD PLAN WORK [ROW...]" >&2
    exit 2
fi
old=$1
plan=$2
work=$3
shift 3
root=$(cd "$(dirname "$0")/../.." && pwd)
lock=$root/scenarios/locks/lsnes.toml
# "OHS1", the flag byte, PpuRegisters to `setini` (58) and $43x0-$43xA (88).
ppu_part=$((4 + 1 + 58 + 88))
mkdir -p "$work/seeds" "$work/logs"

# The seeds to re-cut, as CAPTURE INDEX: those with a record in OLD, of ROWs' starts.
(cd "$old/seeds" && find . -name '*.record' | sed -E 's|^\./(.*)/([0-9]+)\.record$|\1 \2|') |
    sort > "$work/logs/records.txt"
if [ $# -gt 0 ]; then
    for row in "$@"; do
        # A row's `start` line follows its `id` line.
        start=$(awk -v id="id = \"$row\"" '$0 == id { getline; print; exit }' "$lock" |
            sed -nE 's/^start = "([^"]+):([0-9]+)"$/\1 \2/p')
        if [ -z "$start" ]; then
            echo "$row: no seed start in $lock" >&2
            exit 2
        fi
        printf '%s %04d\n' "${start% *}" "${start#* }"
    done | sort -u | comm -12 - "$work/logs/records.txt" > "$work/logs/targets.txt"
else
    cp "$work/logs/records.txt" "$work/logs/targets.txt"
fi
echo "$(wc -l < "$work/logs/targets.txt") seed records to re-cut"

# The plan's cut line of each target, once.
awk 'FILENAME == ARGV[1] { want[$1 " " $2] = 1; next }
    $1 == "cut" { key = $4 " " sprintf("%04d", $5); if (key in want && !(key in seen)) { seen[key] = 1; print } }' \
    "$work/logs/targets.txt" "$plan" > "$work/logs/plan.txt"
missing=$(awk 'FILENAME == ARGV[1] { seen[$4 " " sprintf("%04d", $5)] = 1; next } !($0 in seen)' \
    "$work/logs/plan.txt" "$work/logs/targets.txt")
if [ -n "$missing" ]; then
    echo "no cut line in $plan for:" >&2
    echo "$missing" >&2
    exit 1
fi

begun=$SECONDS
bash "$root/tools/oracle/relock_cut.sh" "$work/logs/plan.txt" "$work/seeds" "$work/logs" cut
bash "$root/tools/oracle/relock_cut.sh" "$work/logs/plan.txt" "$work/seeds" "$work/logs" realign
echo "cut in $((SECONDS - begun))s"

# Whether lsnes states OLD and NEW (zip archives) hold the same members, equal
# but for rrdata.
same_state() {
    local scratch=$work/logs/state
    rm -rf "$scratch"
    mkdir -p "$scratch/old" "$scratch/new"
    unzip -q "$1" -d "$scratch/old" && unzip -q "$2" -d "$scratch/new" &&
        diff -rq --exclude=rrdata "$scratch/old" "$scratch/new" > /dev/null
}

failures=0
report=$work/logs/compare.txt
: > "$report"
while read -r capture index; do
    was=$old/seeds/$capture/$index
    new=$work/seeds/$capture/$index
    if [ ! -f "$new.record" ] || [ ! -f "$new.lss" ]; then
        echo "FAIL $capture:$index not cut" >> "$report"
        failures=$((failures + 1))
        continue
    fi
    size=$(wc -c < "$was.record")
    if [ "$(wc -c < "$new.record")" -ne $((size + ppu_part)) ]; then
        echo "FAIL $capture:$index record of $(wc -c < "$new.record") bytes, old $size + $ppu_part" >> "$report"
        failures=$((failures + 1))
    elif ! cmp -s "$was.record" <(head -c "$size" "$new.record"); then
        echo "FAIL $capture:$index record differs from the old within its $size bytes" >> "$report"
        failures=$((failures + 1))
    elif [ "$(tail -c +$((size + 1)) "$new.record" | head -c 5 | od -An -tx1 | tr -d ' \n')" != 4f48533104 ]; then
        echo "FAIL $capture:$index no PPU part after the memories" >> "$report"
        failures=$((failures + 1))
    elif ! same_state "$was.lss" "$new.lss"; then
        echo "FAIL $capture:$index state differs from the old outside rrdata" >> "$report"
        failures=$((failures + 1))
    else
        echo "ok   $capture:$index old record + PPU part; state equal outside rrdata" >> "$report"
    fi
done < "$work/logs/targets.txt"
echo "ppu re-cut check: $(wc -l < "$work/logs/targets.txt") records, $failures failing" | tee -a "$report"
if [ "$failures" -ne 0 ]; then
    grep '^FAIL' "$report" >&2
    exit 1
fi

# The new tree: OLD with the re-cut records, the old states kept.
rm -rf "$work/relock-v2-lsnes"
cp -r "$old" "$work/relock-v2-lsnes"
while read -r capture index; do
    cp "$work/seeds/$capture/$index.record" "$work/relock-v2-lsnes/seeds/$capture/$index.record"
done < "$work/logs/targets.txt"

# Deterministic, as tools/ci/pack_lockstep_cache.sh; through stdout, since GNU
# tar reads "C:/..." as a remote host.
plain=$work/relock-v2-lsnes.tar.gz
(cd "$work" && tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner \
    --mode=u+rw,go-w,a+rX -cf - relock-v2-lsnes) | gzip -n -9 > "$plain"
name=relock-v2-lsnes-$(sha256sum < "$plain" | cut -c1-16).tar.gz.enc
SM_SEED_PASSPHRASE=$({ printf 'sm-decomp-seeds-v1'; cat "$ROM"; } | sha256sum | cut -d' ' -f1)
export SM_SEED_PASSPHRASE
openssl enc -aes-256-cbc -salt -pbkdf2 -iter 200000 -md sha256 -pass env:SM_SEED_PASSPHRASE \
    -in "$plain" -out "$work/$name"
openssl enc -d -aes-256-cbc -pbkdf2 -iter 200000 -md sha256 -pass env:SM_SEED_PASSPHRASE \
    -in "$work/$name" | cmp -s - "$plain" || { echo "$name: does not decrypt to the tar" >&2; exit 1; }
sha=$(sha256sum < "$work/$name" | cut -d' ' -f1)
echo "name   $name"
echo "sha256 $sha"
echo "path   $work/$name"
echo "pin    bundles = [{ name = \"$name\", sha256 = \"$sha\" }]"
