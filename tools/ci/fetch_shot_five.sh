#!/usr/bin/env bash
# Install the one-origin causal shot package and separately pinned comparisons.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" shot-contact-five-v1 \
  shot-contact-five-v1-22ed9d3c8bdf4e78.tar.gz.enc \
  22ed9d3c8bdf4e7885edcc26cfa5e0208df51bb9e222af140f5fd514db73931f 51392 3
