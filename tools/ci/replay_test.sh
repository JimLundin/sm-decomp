#!/usr/bin/env bash
# The entrypoint must invoke the shared checker once and propagate execution failure.
set -euo pipefail
fake_python() {
  [ "$#" = 1 ] && [ "$1" = tools/check_replay_transition.py ] || return 85
  return "$REPLAY_METADATA_STATUS"
}
fake_tools() {
  [ "$1" = replay ] && [ "$2" = check ] && [ "$3" = scenarios/replay/index.toml ] || return 81
  shift 3
  if [[ -n ${REPLAY_TEST_CASE:-} ]]; then
    [[ $1 == --case && $2 == "$REPLAY_TEST_CASE" ]] || return 80
    shift 2
  else
    [[ $1 == --all ]] || return 80
    shift
  fi
  [ "$#" = 10 ] && [ "$1" = --baseline ] && [ "$2" = scenarios/replay/baseline.toml ] || return 82
  [ "$3" = --cache ] && [ "$4" = recorded-cache ] && [ "$5" = --rom ] && [ "$6" = player-rom ] || return 83
  [ "$7" = --native ] && [ "$8" = recorded-native ] && [ "$9" = --bundle ] && [ -z "${10}" ] || return 84
  return "$REPLAY_TEST_STATUS"
}
export -f fake_tools fake_python
export SM_PY=fake_python SM_TOOLS=fake_tools SM_CACHE=recorded-cache SM_ROM=player-rom SM_AUDIO_NATIVE=recorded-native
export REPLAY_METADATA_STATUS=0
export REPLAY_TEST_STATUS=0
bash tools/ci/replay.sh
export REPLAY_TEST_STATUS=17
if bash tools/ci/replay.sh; then exit 1; else status=$?; fi
[ "$status" = 17 ]
export REPLAY_METADATA_STATUS=23 REPLAY_TEST_STATUS=0
if bash tools/ci/replay.sh; then exit 1; else status=$?; fi
[ "$status" = 23 ]
export REPLAY_METADATA_STATUS=0 REPLAY_TEST_CASE=blocks/missing-requests
bash tools/ci/replay.sh --case "$REPLAY_TEST_CASE"
export REPLAY_TEST_STATUS=19
if bash tools/ci/replay.sh --case "$REPLAY_TEST_CASE"; then exit 1; else status=$?; fi
[ "$status" = 19 ]
echo 'REPLAY ENTRYPOINT TEST OK'
