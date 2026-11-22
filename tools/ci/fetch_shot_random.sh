#!/usr/bin/env bash
# Install causal Random inputs and independent device and result comparisons.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" shot-random-v1 \
  m0-shot-random-0c0248f72ef838dd.tar.gz.enc \
  0c0248f72ef838dd99adf56d8e27cb6249c4c71e1e2234d91a9b8707fe3b6c94 768 3
echo 'SHOT RANDOM FIXTURE PIN OK'
