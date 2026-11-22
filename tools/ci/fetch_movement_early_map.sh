#!/usr/bin/env bash
# Reviewed current Movement and mini-map observations.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" movement-early-map-v1 \
  movement-early-map-v1-a7afd7d542b5dd82.tar.gz.enc \
  a7afd7d542b5dd821f031763e9a26f20479af8dcd3976f2d44fb611b0ecce0ed 976 2
