#!/usr/bin/env bash
# Invented records and exports prove completed scans, record snapshots and independent exclusions.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
tmp=$(mktemp -d)
pid=
cleanup() {
  if [ -n "$pid" ]; then kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; fi
  rm -rf -- "$tmp"
}
trap cleanup EXIT
export CLEANROOM_PRIVATE="$tmp/private"
mkdir "$CLEANROOM_PRIVATE"
token=invented_canary_token
fingerprint='This invented forbidden sentence belongs only to the fixture.'
printf '%s\tfixture-token\tfixture-time\n' "$token" >"$CLEANROOM_PRIVATE/tokens.tsv"
printf '%s\n' "$fingerprint" >"$CLEANROOM_PRIVATE/fingerprints.txt"
printf '%s\tfixture-blob\tfixture-fingerprint\n' "$fingerprint" >"$CLEANROOM_PRIVATE/fingerprints.tsv"
printf 'ordinary clean transcript\n' >"$tmp/clean"
printf '%s\n' "$token" >"$tmp/leak"
printf '%s\n' "$fingerprint" >"$tmp/fingerprint"
fail() { echo "FAIL $*" >&2; exit 1; }
expect() { # STATUS LABEL COMMAND...
  local want=$1 label=$2 status=0
  shift 2
  "$@" >"$tmp/out" 2>&1 || status=$?
  [ "$status" -eq "$want" ] || fail "$label: status $status, expected $want"
  if [ "$want" -ne 0 ] && command grep -q 'CANARY CLEAN' "$tmp/out"; then fail "$label returned CLEAN"; fi
}
canary() { bash "$here/canary.sh" check "$@"; }
expect 0 clean canary "$tmp/clean"
command grep -q '^CANARY CLEAN' "$tmp/out" || fail 'clean verdict absent'
expect 1 token canary "$tmp/leak"
expect 1 fingerprint canary "$tmp/fingerprint"
expect 2 missing-input canary "$tmp/missing"
mkexport() {
  mkdir "$1"
  printf '%s\n' "$2" >"$1/spec.md"
  git -c core.autocrlf=false -C "$1" init -q -b main
  git -c core.autocrlf=false -C "$1" add .
  git -C "$1" -c user.name=fixture -c user.email=fixture@invalid commit -q -m 'Invented export'
}
mkexport "$tmp/given" "$fingerprint"
mkexport "$tmp/other" 'Ordinary invented spec'
expect 0 shared-fingerprint canary "$tmp/given" "$tmp/fingerprint"
expect 1 excluded-elsewhere canary "$tmp/other" "$tmp/fingerprint"
mkexport "$tmp/history" 'Ordinary invented history spec'
git -C "$tmp/history" -c user.name=fixture -c user.email=fixture@invalid commit -q --allow-empty -m "$token"
expect 1 history-token canary "$tmp/history"
# Inject errors at each scanner/producer; no error may yield a clean verdict.
export CANARY_FAULT
CANARY_FAULT=read
function grep() {
  if [ "$CANARY_FAULT" = read ] && [ "${1:-}" = -rhoIF ]; then return 2; fi
  if [ "$CANARY_FAULT" = history-read ] && [ "${1:-}" = -hoIF ]; then return 2; fi
  command grep "$@"
}
function git() {
  case "$CANARY_FAULT:$*" in
    root:*rev-list*|given:*grep*|history:*log*) return 128 ;;
  esac
  command git "$@"
}
function sort() { if [ "$CANARY_FAULT" = sort ]; then return 2; fi; command sort "$@"; }
function cp() { if [ "$CANARY_FAULT" = snapshot ]; then return 2; fi; command cp "$@"; }
export -f grep git sort cp
expect 2 read-error-leak canary "$tmp/leak"
expect 2 read-error-clean canary "$tmp/clean"
CANARY_FAULT=history-read
expect 2 history-read-error canary "$tmp/history"
CANARY_FAULT=root
expect 128 root-error canary "$tmp/given"
CANARY_FAULT=given
expect 2 exclusion-error canary "$tmp/given"
CANARY_FAULT=history
expect 128 history-error canary "$tmp/history"
CANARY_FAULT=sort
expect 2 sort-error canary "$tmp/clean"
CANARY_FAULT=snapshot
expect 2 snapshot-error canary "$tmp/clean"
unset -f grep git sort cp
unset CANARY_FAULT
# Stop one check after exclusions, run another against the same record, then release it.
# Run both verdict orders: a paused clean check and a paused breach check.
export CANARY_BARRIER="$tmp/barrier"
function grep() {
  if [ "${1:-}" = -rhoIF ] && [ "${CANARY_PAUSE:-}" = yes ]; then
    : >"$CANARY_BARRIER/ready"
    local deadline=$((SECONDS + 10))
    while [ "$SECONDS" -lt "$deadline" ]; do
      [ ! -e "$CANARY_BARRIER/go" ] || break
      sleep 0.05
    done
    [ -e "$CANARY_BARRIER/go" ] || return 2
  fi
  command grep "$@"
}
export -f grep
for order in clean breach; do
  mkdir "$CANARY_BARRIER"
  if [ "$order" = clean ]; then first=given; second=other; a=0; b=1
  else first=other; second=given; a=1; b=0; fi
  (status=0; CANARY_PAUSE=yes canary "$tmp/$first" "$tmp/fingerprint" >"$tmp/first.out" 2>&1 || status=$?; echo "$status" >"$tmp/first.status") &
  pid=$!
  deadline=$((SECONDS + 10))
while [ "$SECONDS" -lt "$deadline" ]; do
    [ ! -e "$CANARY_BARRIER/ready" ] || break
    sleep 0.05
  done
  [ -e "$CANARY_BARRIER/ready" ] || fail 'overlap barrier not reached'
  expect "$b" "overlap-$order-second" canary "$tmp/$second" "$tmp/fingerprint"
  : >"$CANARY_BARRIER/go"
  wait "$pid"
pid=
  [ "$(cat "$tmp/first.status")" -eq "$a" ] || fail "overlap-$order-first"
  if [ "$a" -ne 0 ] && command grep -q 'CANARY CLEAN' "$tmp/first.out"; then fail 'overlap breach returned CLEAN'; fi
  rm -rf -- "$CANARY_BARRIER"
done
# The scan and its attribution must use the records captured before either record changes.
mkdir "$CANARY_BARRIER"
(status=0; CANARY_PAUSE=yes canary "$tmp/leak" "$tmp/fingerprint" >"$tmp/snapshot.out" 2>&1 || status=$?; echo "$status" >"$tmp/snapshot.status") &
pid=$!
deadline=$((SECONDS + 10))
while [ "$SECONDS" -lt "$deadline" ]; do
  [ ! -e "$CANARY_BARRIER/ready" ] || break
  sleep 0.05
done
[ -e "$CANARY_BARRIER/ready" ] || fail 'snapshot barrier not reached'
command cp "$CLEANROOM_PRIVATE/tokens.tsv" "$tmp/tokens.saved"
command cp "$CLEANROOM_PRIVATE/fingerprints.tsv" "$tmp/fingerprints.saved"
printf 'replacement_token\treplacement-source\tfixture-time\n' >"$CLEANROOM_PRIVATE/tokens.tsv"
: >"$CLEANROOM_PRIVATE/fingerprints.tsv"
: >"$CANARY_BARRIER/go"
wait "$pid"
pid=
[ "$(cat "$tmp/snapshot.status")" -eq 1 ] || fail 'record mutation changed verdict'
command grep -q '^BREACH token from fixture-token ' "$tmp/snapshot.out" || fail 'token attribution changed'
command grep -q '^BREACH fingerprint from fixture-fingerprint ' "$tmp/snapshot.out" || fail 'fingerprint attribution changed'
command cp "$tmp/tokens.saved" "$CLEANROOM_PRIVATE/tokens.tsv"
command cp "$tmp/fingerprints.saved" "$CLEANROOM_PRIVATE/fingerprints.tsv"
rm -rf -- "$CANARY_BARRIER"
unset -f grep
# A busy record must fail within the bounded wait, without removing another owner's lock.
mkdir "$CLEANROOM_PRIVATE/.record-lock"
expect 2 busy-record canary "$tmp/clean"
[ -d "$CLEANROOM_PRIVATE/.record-lock" ] || fail 'foreign lock removed'
rmdir "$CLEANROOM_PRIVATE/.record-lock"
[ -z "$(find "$CLEANROOM_PRIVATE" -name '.*' -print)" ] || fail 'shared scratch left behind'
echo 'CANARY CHECK OK: clean, leaks, scanner/git errors, snapshots and overlapping exclusions'
bash "$here/canary_plant_test.sh"
