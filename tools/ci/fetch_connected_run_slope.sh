#!/usr/bin/env bash
# Reviewed connected Run and floor aim opportunities.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" connected-run-slope-v1 \
  connected-run-slope-6ae54520fbb3b30f.tar.gz.enc \
  6ae54520fbb3b30f359130e75315e0356d786bce989ac2ef0ca2d630bfe74761 4222096 146
echo 'CONNECTED RUN SLOPE FIXTURE PIN OK'
