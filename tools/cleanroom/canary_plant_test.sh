#!/usr/bin/env bash
# Invented text and binary sources, with private records and a minimal repository.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
script=${CANARY_SCRIPT:-$here/canary.sh}
tmp=$(mktemp -d)
tmp=$(cd "$tmp" && pwd -P)
trap 'rm -rf -- "$tmp"' EXIT
export CLEANROOM_PRIVATE="$tmp/private"
export TEMP="$tmp/source-temp"
export CANARY_ALSO=
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
mkdir -p "$TEMP/c4a/logs" "$tmp/repo"
cd "$tmp/repo"
printf 'An invented ordinary source for the fingerprint fixture.\n' >readme.md
git init -q --template= -b main
git add readme.md
git -c user.name=fixture -c user.email=fixture@invalid commit -qm 'Invented source'
sources="$TEMP/c4a/logs"
awk 'BEGIN { for (i = 1; i <= 850; i++) print "Invented text source line " i }' >"$sources/text.md"
printf 'An invented short state file.\n' >"$sources/state.txt"
{ printf 'MZ\000'; awk 'BEGIN { for (i = 1; i <= 850; i++) print "Invented binary payload" }'; } >"$sources/payload.dll"
{ awk 'BEGIN { for (i = 1; i <= 30000; i++) print "Invented long text prefix" }'; printf '\000binary tail\n'; } >"$sources/late.data"
for name in text.md state.txt payload.dll late.data; do cp "$sources/$name" "$tmp/$name.original"; done
fail() { echo "FAIL $*" >&2; exit 1; }
expect_status() {
  local want=$1 status=0
  shift
  bash "$script" "$@" >"$tmp/output" 2>&1 || status=$?
  [ "$status" -eq "$want" ] || fail "${CANARY_FAULT:-nominal} $*: status $status, expected $want"
}
unchanged() { cmp -s "$1" "$2" || fail "bytes changed: $(basename "$1")"; }
expect_status 0 plant
for name in state.txt payload.dll late.data; do unchanged "$sources/$name" "$tmp/$name.original"; done
if cmp -s "$sources/text.md" "$tmp/text.md.original"; then fail 'text source was not planted'; fi
[ "$(wc -l <"$sources/text.md")" -eq 854 ] || fail 'text markers are missing'
expect_status 0 unplant
for name in text.md state.txt payload.dll late.data; do unchanged "$sources/$name" "$tmp/$name.original"; done
# A stale active entry may point at a binary. Even a matching token must not rewrite it.
printf 'invented_binary_token\n' >>"$sources/payload.dll"
cp "$sources/payload.dll" "$tmp/binary-active.original"
printf 'invented_binary_token\tfile\t%s\n' "$sources/payload.dll" >"$CLEANROOM_PRIVATE/active.tsv"
expect_status 0 unplant
unchanged "$sources/payload.dll" "$tmp/binary-active.original"
# Real scanner/reader failures must stop before replacing source bytes.
export CANARY_FAULT
function tr() { if [ "$CANARY_FAULT" = classifier ]; then return 2; fi; command tr "$@"; }
function wc() { if [ "$CANARY_FAULT" = line-count ] && [ "${1:-}" = -l ]; then return 2; fi; command wc "$@"; }
function grep() { if [ "$CANARY_FAULT" = strip ] && [ "${1:-}" = -vF ]; then return 2; fi; command grep "$@"; }
function awk() { if [ "$CANARY_FAULT" = plant-read ] && [ "${1:-}" = -v ] && [[ "${2:-}" = c=* ]]; then return 2; fi; command awk "$@"; }
export -f tr wc grep awk
CANARY_FAULT=classifier
expect_status 2 plant
unchanged "$sources/text.md" "$tmp/text.md.original"
CANARY_FAULT=line-count
expect_status 2 plant
unchanged "$sources/text.md" "$tmp/text.md.original"
CANARY_FAULT=plant-read
expect_status 2 plant
unchanged "$sources/text.md" "$tmp/text.md.original"
CANARY_FAULT=none
expect_status 0 plant
cp "$sources/text.md" "$tmp/planted.original"
CANARY_FAULT=classifier
expect_status 2 unplant
unchanged "$sources/text.md" "$tmp/planted.original"
CANARY_FAULT=strip
expect_status 2 unplant
unchanged "$sources/text.md" "$tmp/planted.original"
[ -s "$CLEANROOM_PRIVATE/active.tsv" ] || fail 'failed unplant discarded active records'
unset -f tr wc grep awk
unset CANARY_FAULT
expect_status 0 unplant
unchanged "$sources/text.md" "$tmp/text.md.original"
unchanged "$sources/payload.dll" "$tmp/binary-active.original"
echo 'CANARY PLANT OK: text markers, binary bytes, restoration and read failures'
