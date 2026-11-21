#!/usr/bin/env bash
# Independently admitted causal inputs for the complete scoped movement search.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" native-search-inputs-v1 \
  native-search-inputs-bfd255492baaa64c.tar.gz.enc \
  bfd255492baaa64cdd774aadfece5068a4a413980bb9ce2de8ec293835dfe8ab 5922352 161 --rom "$1"
