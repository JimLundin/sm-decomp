#!/usr/bin/env bash
# Fetch the independently recorded adjacent controller, Haze, Pose, ActorUpdate, grounded movement and HUD refresh opportunities.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" adjacent-native-frame-v1 \
  adjacent-native-frame-1d085976ee8f6458.tar.gz.enc \
  1d085976ee8f64584950de900e2c67584d1e7eca68539a7469f365a236dd3f1a 19840 20
