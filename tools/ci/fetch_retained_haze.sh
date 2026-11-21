#!/usr/bin/env bash
# Fetch the independently recorded adjacent controller, Haze, Pose, ActorUpdate and grounded movement opportunities.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" adjacent-native-frame-v1 \
  adjacent-native-frame-415155682a07309f.tar.gz.enc \
  415155682a07309f500bfa3256b9191c80ed0a1b4f995fdb8af83cdc881882dc 19072 13
