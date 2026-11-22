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
bash "$(dirname "$0")/fetch_retained_haze.sh" "$1" "$2"
bash "$(dirname "$0")/fetch_random_draw_observations.sh" "$1" "$2"
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" adjacent-animation-v1 \
  adjacent-animation-b3423893b9b86aac.tar.gz.enc \
  b3423893b9b86aacc0d2dc734f9ac1362820ac6a3c9f88fadeacec2c8b4980ec 912 2
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" adjacent-return-v1 \
  adjacent-return-a82792870e279a85.tar.gz.enc \
  a82792870e279a85c91076ed4a9636928a17e9c3824e85a20c627a4a2c9cd495 5712 5
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" adjacent-drawing-v1 \
  adjacent-drawing-v1-61f918a4e78d15c2.tar.gz.enc \
  61f918a4e78d15c2ae0d37d6f2a41dbecad2e2d7384ce7795338abfbf4ecb783 320 2
