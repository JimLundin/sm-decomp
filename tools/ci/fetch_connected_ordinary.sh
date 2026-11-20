#!/usr/bin/env bash
# Reviewed ordinary movement, appearance and camera opportunities.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" connected-ordinary-v1 \
  connected-ordinary-8fc71a3b9c87b393.tar.gz.enc \
  8fc71a3b9c87b3933d080053a7d7ff393f1e48c5cb54edf6afec733e8431227e 2393040 83
echo 'CONNECTED ORDINARY FIXTURE PIN OK'
