#!/usr/bin/env bash
# Fetch the independently recorded adjacent controller, Haze, Pose and ActorUpdate opportunities.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" adjacent-native-frame-v1 \
  adjacent-native-frame-84de90e28d930e60.tar.gz.enc \
  84de90e28d930e60430865b3965f485ea0fe8a4a157deb36d6b28fc1cdce0d9a 17792 12
