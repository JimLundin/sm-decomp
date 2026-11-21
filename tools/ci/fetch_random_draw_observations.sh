#!/usr/bin/env bash
# Fetch independent comparisons for both live Random draws.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" random-draw-observations-v1 \
  random-draw-observations-5049c208227f00a7.tar.gz.enc \
  5049c208227f00a740a2fe107472a35c5c990cf69c4db7cb761aaa8b5641b7d4 336 1
