#!/usr/bin/env bash
# Reviewed RTI-prefetch shoot-frame donor and separate Music receipt.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" shot-frame-origin-v1 \
  m0-shot-frame-origin-a50d462c020250cd.tar.gz.enc \
  a50d462c020250cd107792d781bed17cacef7f6126824c6ea198cd17f3265c61 99232 6
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" shot-frame-resume-v1 \
  m0-shot-frame-resume-2bc041b807b76b62.tar.gz.enc \
  2bc041b807b76b6201fb2ed8b40cfaf850a26e82a1a4668ec1ee2708369e3cda 77232 5
echo 'SHOT FRAME ORIGIN FIXTURE PIN OK'
