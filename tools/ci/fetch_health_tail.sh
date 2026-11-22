#!/usr/bin/env bash
# Install the scoped tail origin, independent observations and actor offer.
set -euo pipefail
scripts=$(dirname "$0")
bash "$scripts/fetch_native_fixture.sh" "$1" "$2" ceres-health-tail-v1 \
  ceres-health-tail-v1-65879fe0ba492cfd.tar.gz.enc \
  65879fe0ba492cfdc8a714b4d23db9724cce9a5094e204f9c3866585beb79e28 880 2
bash "$scripts/fetch_native_fixture.sh" "$1" "$2" ceres-tail-offer-v1 \
  ceres-tail-offer-v1-efe971792b8edb25.tar.gz.enc \
  efe971792b8edb2577ecdca2b2664d21a6924af4377327318122861f86b75fb7 272 1
bash "$scripts/fetch_native_fixture.sh" "$1" "$2" ceres-tail-origin-v1 \
  ceres-tail-origin-v1-770f77970b7eb51c.tar.gz.enc \
  770f77970b7eb51cd6d8d825626d2f5f1eb27022ae7472a41c96d9612b648b33 40992 1
bash "$scripts/fetch_native_fixture.sh" "$1" "$2" ceres-health-hud-operands-v1 \
  ceres-health-hud-operands-v1-c7dbf41614d13f5e3.tar.gz.enc \
  c7dbf41614d13f5e304c7c6904412ec7831058dedd437f5e7f3bcd4144fc2ab0 576 1
