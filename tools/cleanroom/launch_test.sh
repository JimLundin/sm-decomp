#!/usr/bin/env bash
# Fake CLI subprocesses prove completion without model sessions or credentials; invented
# transcripts prove what a probe session must show (probe_check.py).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
python=${CLEANROOM_PY:-${SM_PY:-python3}}
"$python" -I "$here/launch_stream_test.py"
"$python" -I "$here/responses_controller_test.py"
"$python" -I "$here/box_relay_test.py"
"$python" -I "$here/probe_check_test.py"
echo 'LAUNCH TEST OK'
