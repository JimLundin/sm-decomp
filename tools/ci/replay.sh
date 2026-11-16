#!/usr/bin/env bash
# Local and CI acceptance use the same complete case index and reviewed baseline.
set -euo pipefail
: "${SM_CACHE:?SM_CACHE is required}" "${SM_ROM:?SM_ROM is required}" "${SM_AUDIO_NATIVE:?SM_AUDIO_NATIVE is required}"
"${SM_PY:-python3}" tools/check_replay_transition.py
"${SM_TOOLS:-target/verify/tools}" replay check scenarios/replay/index.toml --all \
  --baseline scenarios/replay/baseline.toml --cache "$SM_CACHE" --rom "$SM_ROM" \
  --native "$SM_AUDIO_NATIVE" --bundle "${SM_BUNDLE:-}" "$@"
