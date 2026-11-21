#!/usr/bin/env bash
# Install the qualified packets in the order used by replay and fault workers.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
scripts=$(dirname "$0")
bash "$scripts/fetch_native_search.sh" "$@"
bash "$scripts/fetch_native_search_order.sh" "$@"
bash "$scripts/fetch_native_search_work.sh" "$@"
