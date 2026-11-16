#!/usr/bin/env bash
# The entrypoint must invoke the shared checker once and propagate execution failure.
set -euo pipefail
fake_python() {
  [ "$#" = 1 ] && [ "$1" = tools/check_replay_transition.py ] || return 85
  return "$REPLAY_METADATA_STATUS"
}
fake_tools() {
  [ "$#" = 14 ] || return 80
  [ "$1" = replay ] && [ "$2" = check ] && [ "$3" = scenarios/replay/index.toml ] || return 81
  [ "$4" = --all ] && [ "$5" = --baseline ] && [ "$6" = scenarios/replay/baseline.toml ] || return 82
  [ "$7" = --cache ] && [ "$8" = recorded-cache ] && [ "$9" = --rom ] && [ "${10}" = player-rom ] || return 83
  [ "${11}" = --native ] && [ "${12}" = recorded-native ] && [ "${13}" = --bundle ] && [ -z "${14}" ] || return 84
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
echo 'REPLAY ENTRYPOINT TEST OK'
