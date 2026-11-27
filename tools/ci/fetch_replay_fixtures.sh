#!/usr/bin/env bash
# Install the pinned native fixtures for replay and the fault workers.
set -euo pipefail
if [[ $# != 2 ]]; then echo "usage: fetch_replay_fixtures.sh ROM DEST" >&2; exit 2; fi
here=$(cd "$(dirname "$0")" && pwd)
for fixture in audio_native sender_offers movement_touchdown connected_ordinary \
    connected_run_slope movement_early_map shot_contact native_search_packets health_tail; do
  echo "Fetching $fixture"
  bash "$here/fetch_$fixture.sh" "$1" "$2"
done
