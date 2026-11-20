#!/usr/bin/env bash
# Retained native frame origin, physical inputs and independent return observations.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" retained-frame-return-v2 \
  retained-frame-return-bb6f0aa71c795a34.tar.gz.enc \
  bb6f0aa71c795a3438377247da37fc1aa3b3e5ed603aca44df20a7424475842b 207440 73
echo 'RETAINED FRAME RETURN FIXTURE PIN OK'
