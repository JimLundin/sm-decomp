#!/usr/bin/env bash
# launch_responses.sh EXPORT BRIEF TRANSCRIPT PRIVATE_LANE
# Fresh closed-tool Responses session. The provider's credential stays in the controller;
# model-written source executes behind the existing code_check/import/canary barriers.
# Windows launcher. --prepare-only writes the trusted record without inference.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
probe=()
prepare_only=0
while [[ "${1:-}" = --* ]]; do
  case "$1" in
    --probe) probe+=(--probe); shift ;;
    --mode) [ $# -ge 2 ] || exit 1; probe+=(--mode "$2"); shift 2 ;;
    --prepare-only) prepare_only=1; shift ;;
    *) echo 'unknown launcher option' >&2; exit 1 ;;
  esac
done
[ $# -eq 4 ] || { echo 'usage: launch_responses.sh EXPORT BRIEF TRANSCRIPT PRIVATE_LANE' >&2; exit 1; }
python=${CLEANROOM_PY:-${LOCALAPPDATA:-$USERPROFILE/AppData/Local}/sm-decomp-tools/python-3.13.7/python.exe}
export CLEANROOM_BASH=$(cygpath -m "${CLEANROOM_BASH:-$(command -v bash)}")
export CLEANROOM_GIT=$(cygpath -m "${CLEANROOM_GIT:-$(command -v git)}")
export CLEANROOM_CARGO=$(cygpath -m "${CLEANROOM_CARGO:-$(command -v cargo)}")
config=$("$python" -I "$here/responses_controller.py" "${probe[@]}" --prepare "$1" "$4" "$2" "$3")
if [ "$prepare_only" = 1 ]; then printf '%s\n' "$config"; exit 0; fi
status=0
"$python" -I "$here/responses_controller.py" "$config" || status=$?
# Even an API or tool failure is checked. No launch success precedes the canary gate.
bash "$here/canary.sh" check "$3" "$1" "$4" || exit 1
[ "$status" -eq 0 ] || exit "$status"
bash "$here/code_check.sh" --import "$1"/crates/clean/* || exit 1
echo "LAUNCH OK Responses $4"
