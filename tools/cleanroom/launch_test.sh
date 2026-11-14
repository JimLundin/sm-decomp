#!/usr/bin/env bash
# Fake CLI subprocesses prove completion without model sessions or credentials.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
python=${CLEANROOM_PY:-${SM_PY:-python3}}
"$python" -I "$here/launch_stream_test.py"
"$python" -I "$here/responses_controller_test.py"
echo 'LAUNCH TEST OK'
