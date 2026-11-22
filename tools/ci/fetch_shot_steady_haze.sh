#!/usr/bin/env bash
# Install the independently qualified passive and same-duration Steady inputs.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" shot-steady-haze-v1 \
  m0-shot-steady-haze-d97c8f1ac6f69a3c.tar.gz.enc \
  d97c8f1ac6f69a3c0eaa4bcb413b95700ca802bc4ecdd7d3b5478888649470be 4176 5
echo 'SHOT STEADY HAZE FIXTURE PIN OK'
