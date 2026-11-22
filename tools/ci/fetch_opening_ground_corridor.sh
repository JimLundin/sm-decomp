#!/usr/bin/env bash
# Install the scoped opening corridor inputs and independent comparisons.
set -euo pipefail
fetch="$(dirname "$0")/fetch_native_fixture.sh"
bash "$fetch" "$1" "$2" opening-clock-cut8-10-work-v1 \
  opening-clock-cut8-10-work-v1-b92726026605f6ba.tar.gz.enc \
  b92726026605f6ba2d69caf28679c37387128cc9ff3db3ff3de0ddbd4790b31f 5584 5
bash "$fetch" "$1" "$2" opening-clock-cut9-10-branches-v1 \
  opening-clock-cut9-10-branches-v1-410508d5f8fa82ee.tar.gz.enc \
  410508d5f8fa82ee1d0a79f98f5966132316a1f6d5538457158197e9b15196d5 5280 5
bash "$fetch" "$1" "$2" opening-clock-through-cut10-expected-v1 \
  opening-clock-through-cut10-expected-v1-86974ff1d9e52aaf.tar.gz.enc \
  86974ff1d9e52aaf53f14652a96e7435ec0e8eb3cff8c63921e515c256dfebe8 1696 2
bash "$fetch" "$1" "$2" opening-clock-mixed-drawing-v1 \
  opening-clock-mixed-drawing-v1-8e5f2ebc3e0bc872.tar.gz.enc \
  8e5f2ebc3e0bc87280505854b039dbe144d1655c69b341cc962e9735f8ea7dca 1776 3
bash "$fetch" "$1" "$2" opening-clock-mixed-drawing-expected-v1 \
  opening-clock-mixed-drawing-expected-v1-55f03d2f79170318.tar.gz.enc \
  55f03d2f7917031842a78e2eeff9dd334d718769db99fb06a5e2c031a23a5ac3 1168 2
bash "$fetch" "$1" "$2" opening-clock-cut10-aim-camera-v1 \
  opening-clock-cut10-aim-camera-v1-f155c2a15268e077.tar.gz.enc \
  f155c2a15268e077eed8111f8abf030c8baded59e365283fd763720776376878 2384 3
bash "$fetch" "$1" "$2" opening-clock-cut10-aim-camera-expected-v2 \
  opening-clock-cut10-aim-camera-expected-v2-08844336f5acea7a.tar.gz.enc \
  08844336f5acea7a9bc34d22adef5702fe3a55761aae0701230817facafddcfc 448 2
bash "$fetch" "$1" "$2" opening-clock-cut11-15-work-v1 \
  opening-clock-cut11-15-work-v1-1a71e4746a979e14.tar.gz.enc \
  1a71e4746a979e14bcb4b52dc56225eb86f443d40431419b1c6bfe0247a82c5a 7888 8
bash "$fetch" "$1" "$2" opening-clock-through-cut15-expected-v1 \
  opening-clock-through-cut15-expected-v1-97c348954a636782.tar.gz.enc \
  97c348954a6367827f26455e09e4f6616da8aafa3b88d5467a2d64228238936d 2416 2
