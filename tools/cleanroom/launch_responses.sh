#!/usr/bin/env bash
# launch_responses.sh EXPORT BRIEF TRANSCRIPT PRIVATE_LANE
# Fresh closed-tool Responses session. The provider's credential stays in the controller;
# model-written tests run only after code_check and the capability clippy pass; the import and
# canary barriers follow.
# --prepare-only writes the trusted record without inference.
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
linux=0
case $(uname -s) in
  Linux) linux=1; python=${CLEANROOM_PY:-$(command -v python3.13 || command -v python3)} ;;
  MINGW*|MSYS*|CYGWIN*)
    python=${CLEANROOM_PY:-${LOCALAPPDATA:-$USERPROFILE/AppData/Local}/sm-decomp-tools/python-3.13.7/python.exe}
    for tool in BASH GIT CARGO AWK; do
      name=CLEANROOM_$tool
      export "$name=$(cygpath -m "${!name:-$(command -v "${tool,,}")}")"
    done ;;
  *) echo 'unsupported Responses launcher platform' >&2; exit 1 ;;
esac
umask 077
config=$("$python" -I "$here/responses_controller.py" "${probe[@]}" --prepare "$1" "$4" "$2" "$3")
if [ "$prepare_only" = 1 ]; then printf '%s\n' "$config"; exit 0; fi
status=0
if [ "$linux" = 1 ]; then
  "$python" -I "$here/responses_controller.py" "$config" &
  controller_pid=$!
  stop_controller() {
    trap '' HUP INT TERM
    kill -TERM "$controller_pid" 2>/dev/null || true
    wait "$controller_pid" 2>/dev/null || true
    status=$1
  }
  trap 'stop_controller 129' HUP
  trap 'stop_controller 130' INT
  trap 'stop_controller 143' TERM
  child_status=0
  wait "$controller_pid" || child_status=$?
  trap - HUP INT TERM
  [ "$status" -ne 0 ] || status=$child_status
else
  "$python" -I "$here/responses_controller.py" "$config" || status=$?
fi
# Even an API or tool failure is checked. No launch success precedes the canary gate.
bash "$here/canary.sh" check "$3" "$1" "$4" || exit 1
[ "$status" -eq 0 ] || exit "$status"
bash "$here/code_check.sh" --import "$1"/crates/clean/* || exit 1
echo "LAUNCH OK Responses $4"
