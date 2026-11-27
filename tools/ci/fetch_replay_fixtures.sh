#!/usr/bin/env bash
# Install the pinned native fixtures for replay and the fault workers.
set -euo pipefail
if [[ $# != 2 && $# != 3 ]]; then
  echo "usage: fetch_replay_fixtures.sh ROM DEST [FOLDER]" >&2
  exit 2
fi
here=$(cd "$(dirname "$0")" && pwd)
exec "${SM_PY:-python3}" "$here/fetch_replay_fixtures.py" "$@"
