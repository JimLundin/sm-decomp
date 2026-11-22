#!/usr/bin/env bash
# Independently pinned inputs and comparisons for the three-return opening and its controls.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
fetch="$(dirname "$0")/fetch_native_fixture.sh"
bash "$fetch" "$1" "$2" opening-clock-cut2-comparison-v1 \
  opening-clock-cut2-comparison-v1-5911f99547069667.tar.gz.enc \
  5911f9954706966786cb38f345b74125664f903ca583552e34488e6e16256f60 720 2
bash "$fetch" "$1" "$2" opening-clock-cut2-room-v1 \
  opening-clock-cut2-room-v1-b58bf74c679eef4c.tar.gz.enc \
  b58bf74c679eef4c4403dd8a48d7622ca6c220e34d22710742c0ffdceb3e1b48 432 2
bash "$fetch" "$1" "$2" opening-clock-reaim-input-v1 \
  opening-clock-reaim-input-v1-907c808c0e251e1f.tar.gz.enc \
  907c808c0e251e1fb0e30a9685a892c1d27bf5352b2c00767f0c538a5c729e9d 1136 2
bash "$fetch" "$1" "$2" opening-clock-stale-haze-control-v1 \
  opening-clock-stale-haze-control-v1-fc9f80b2f2fca0bf.tar.gz.enc \
  fc9f80b2f2fca0bf6674309ad98f3bdf49edaced1d06a77b6dc0f44b2f1acb07 688 2
bash "$fetch" "$1" "$2" opening-clock-ordinary-right-input-v1 \
  opening-clock-ordinary-right-input-v1-2e94708c2a360fd1.tar.gz.enc \
  2e94708c2a360fd1ca0801184b0666f850bcc9f447aa4aac4eb688654412d72c 1584 3
bash "$fetch" "$1" "$2" opening-clock-ordinary-right-comparison-v1 \
  opening-clock-ordinary-right-comparison-v1-7b79619df9d9ad9b.tar.gz.enc \
  7b79619df9d9ad9b7d46da204f0908259a724f524a2c0752285c73b3204b8036 688 2
bash "$fetch" "$1" "$2" opening-clock-foot-contact-input-v1 \
  opening-clock-foot-contact-input-v1-29d7b8c08284e2eb.tar.gz.enc \
  29d7b8c08284e2ebb8028970bcd3c84030ea928250a7ea98241f67281780b390 560 2
bash "$fetch" "$1" "$2" opening-clock-camera-guard-v1 \
  opening-clock-camera-guard-v1-0c64ff8dc31cb01a.tar.gz.enc \
  0c64ff8dc31cb01a87e6e758d21860cea13e5fafc179cbd4ad33b6e76c314917 448 2
bash "$fetch" "$1" "$2" opening-clock-drawing-shapes-v1 \
  opening-clock-drawing-shapes-v1-71185a1111250906.tar.gz.enc \
  71185a11112509062fad625d22ffe76858e7d65ab9d3286223a628c50d69c04f 7872 2
bash "$fetch" "$1" "$2" opening-clock-contact-match-v1 \
  opening-clock-contact-match-v1-50c66cca36252b4b.tar.gz.enc \
  50c66cca36252b4b66c34fa5fb32fff317d0d020dc89171d4c52c68573763cbe 768 3
bash "$fetch" "$1" "$2" opening-clock-three-return-expected-v1 \
  opening-clock-three-return-expected-v1-5c007ce6bada93eb.tar.gz.enc \
  5c007ce6bada93ebf096c050319c672b81c1361cd1669ad475ac8e84f3e3d3c6 13072 5
bash "$fetch" "$1" "$2" opening-clock-movie-input-v1 \
  opening-clock-movie-input-v1-2dc32eb72af19f0f.tar.gz.enc \
  2dc32eb72af19f0f51401e53168e4e7f8a945759b27c645a867eee9835c6596e 640 2
bash "$fetch" "$1" "$2" opening-clock-reaim-comparison-v2 \
  opening-clock-reaim-comparison-v2-4f9f07ab8cd1f78e.tar.gz.enc \
  4f9f07ab8cd1f78e6b41bfb919fe8ab464a4612d507ee2c2454d35ae4754ace1 928 2
bash "$fetch" "$1" "$2" opening-clock-movie-timing-expected-v1 \
  opening-clock-movie-timing-expected-v1-e7e717d1fd3fc291.tar.gz.enc \
  e7e717d1fd3fc291e4052f5c4cb3fd8ff53a4f98c7cddde3bcc84af2409d1e31 13152 2
