#!/usr/bin/env bash
# The transition check admits all replay workers once, and every worker must pass before merging.
set -euo pipefail
fake_python() {
  case "$*" in
    tools/check_replay_transition.py)
      echo checked >> "$TRANSITION_LOG"
      return "$TRANSITION_STATUS"
      ;;
    "tools/ci/required_witnesses.py ")
      [ "$(cat "$TRANSITION_LOG")" = checked ] || return 88
      echo "witness log"
      return "$WITNESS_STATUS"
      ;;
    *) return 85 ;;
  esac
}
fake_tools() {
  [ "$(cat "$TRANSITION_LOG")" = checked ] || return 88
  [ "$1" = replay ] && [ "$3" = scenarios/replay/index.toml ] && [ "$4" = --all ] || return 81
  case $2 in
    check)
      [ "${15}" = --shard ] && [ "${17}" = --write ] || return 82
      echo "shard ${16} log"
      echo "${16}" > "${18}"
      [ "${16}" != "$FAILING_SHARD" ] || return 17
      ;;
    merge)
      [ "$5" = --baseline ] && [ "$7 $9 ${11}" = "--receipt --receipt --receipt" ] || return 83
      [ "$(cat "$8" "${10}" "${12}")" = "$(printf '0/3\n1/3\n2/3')" ] || return 84
      [ "${13}" = --write ] || return 86
      echo merged > "${14}"
      ;;
    *) return 87 ;;
  esac
}
export -f fake_tools fake_python
export SM_PY=fake_python SM_TOOLS=fake_tools SM_CACHE=recorded-cache SM_ROM=player-rom SM_AUDIO_NATIVE=recorded-native
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
receipt=$scratch/receipt
export TRANSITION_LOG=$scratch/transition
export TRANSITION_STATUS=0 WITNESS_STATUS=0 FAILING_SHARD=none
bash tools/ci/replay_parallel.sh "$receipt" > /dev/null
[ "$(cat "$receipt")" = merged ]
[ "$(cat "$TRANSITION_LOG")" = checked ]

: > "$receipt"
: > "$TRANSITION_LOG"
export FAILING_SHARD=1/3
if log=$(bash tools/ci/replay_parallel.sh "$receipt"); then exit 1; fi
grep -qx '::error::shard-1 failed with exit 17' <<< "$log"
grep -qx 'shard 1/3 log' <<< "$log"
[ ! -s "$receipt" ]

: > "$TRANSITION_LOG"
export WITNESS_STATUS=23 FAILING_SHARD=none
if log=$(bash tools/ci/replay_parallel.sh "$receipt"); then exit 1; fi
grep -qx '::error::witnesses failed with exit 23' <<< "$log"
grep -qx 'witness log' <<< "$log"

: > "$TRANSITION_LOG"
export TRANSITION_STATUS=24 WITNESS_STATUS=0
if log=$(bash tools/ci/replay_parallel.sh "$receipt"); then exit 1; else status=$?; fi
[ "$status" = 24 ]
[ "$(cat "$TRANSITION_LOG")" = checked ]
[ -z "$log" ]
[ ! -s "$receipt" ]
echo 'REPLAY PARALLEL TEST OK'
