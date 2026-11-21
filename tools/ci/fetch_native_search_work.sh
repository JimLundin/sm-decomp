#!/usr/bin/env bash
# Independently reviewed movement observations checked against the order packet.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" native-search-work-v2 \
  native-search-work-7fb78a87fe8e52bc.tar.gz.enc \
  7fb78a87fe8e52bccd349d47fc04d3326e291d43d44e7a42aec96553d1146676 130592 14 --parent "$2/native-search-order-v1"
