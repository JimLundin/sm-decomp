#!/usr/bin/env bash
# Check the reviewed baseline. An explicit --case selects one diagnostic case.
replay_check() {
  local selection=(--all)
  local checker=("${SM_TOOLS:-target/verify/tools}")
  if [[ ${1:-} == --case ]]; then
    selection=(--case "${2:?--case requires an indexed case ID}")
    printf 'Reproduce on %s with bash tools/ci/replay.sh --case %q\n' "$(git rev-parse HEAD)" "$2"
    if [[ -z ${SM_TOOLS:-} ]]; then
      checker=("${CARGO:-cargo}" run --quiet --locked --profile verify -p tools --bin tools --)
    fi
    shift 2
  else
    : "${SM_CACHE:?SM_CACHE is required}" "${SM_ROM:?SM_ROM is required}" "${SM_AUDIO_NATIVE:?SM_AUDIO_NATIVE is required}"
  fi
  "${checker[@]}" replay check scenarios/replay/index.toml "${selection[@]}" \
    --baseline scenarios/replay/baseline.toml --cache "${SM_CACHE:-}" --rom "${SM_ROM:-rom/sm.smc}" \
    --native "${SM_AUDIO_NATIVE:-}" --bundle "${SM_BUNDLE:-}" "$@"
}

if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  set -euo pipefail
  "${SM_PY:-python3}" tools/check_replay_transition.py
  replay_check "$@"
fi
