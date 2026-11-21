#!/usr/bin/env bash
# Fetch the independently recorded adjacent controller and Haze opportunities.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" adjacent-native-frame-v1 \
  adjacent-native-frame-bdde8a2f3f55ef57.tar.gz.enc \
  bdde8a2f3f55ef576b51d7ae797b6102629cebdd0d06ff86d8a88d873389477b 15600 11
