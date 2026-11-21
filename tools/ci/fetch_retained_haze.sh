#!/usr/bin/env bash
# Fetch the independently recorded adjacent controller, Haze, Pose, ActorUpdate, grounded movement, HUD refresh and countdown opportunities.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" adjacent-native-frame-v1 \
  adjacent-native-frame-2c28d0699ff68ae9.tar.gz.enc \
  2c28d0699ff68ae93f9aff1f2dd1c820b169392a5eb2020b4b3e4edcfb00ecb3 20592 21
