#!/usr/bin/env bash
# Derived enemy-stop ordering checked against the ROM.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" native-search-order-v1 \
  native-search-order-346746ce94e8dcb0.tar.gz.enc \
  346746ce94e8dcb090be3880ad8d1ecd2d4aa2fe05f63ed172b9d90d8fa03b76 14173648 418 --rom "$1"
