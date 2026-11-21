#!/usr/bin/env bash
# Fetch an encrypted, pinned native replay fixture into the native source directory.
set -euo pipefail
if [ "$#" -ne 7 ] && [ "$#" -ne 9 ]; then
  echo "usage: $0 <rom> <destination> <folder> <asset> <sha256> <bytes> <members> [--rom <rom> | --parent <parent>]" >&2
  exit 2
fi
rom=$1
destination=$2
folder=$3
asset=$4
sha256=$5
bytes=$6
members=$7
shift 7
test "$(sha256sum <"$rom" | cut -d' ' -f1)" = 12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72
test -d "$destination"
if [ "$#" -eq 0 ]; then
  test ! -e "$destination/$folder"
  test ! -L "$destination/$folder"
fi
scratch=$(mktemp -d)
cleanup() {
  rm -f -- "$scratch/capture.enc" "$scratch/capture.tar.gz"
  rmdir -- "$scratch"
}
trap cleanup EXIT
curl --fail --silent --show-error --location \
  --retry 8 --retry-delay 15 --retry-all-errors --retry-max-time 600 \
  --output "$scratch/capture.enc" \
  "https://github.com/JimLundin/sm-decomp/releases/download/ci-data/$asset"
echo "$sha256  $scratch/capture.enc" | sha256sum --check
test "$(wc -c <"$scratch/capture.enc")" -eq "$bytes"
SM_SEED_PASSPHRASE=$({ printf 'sm-decomp-seeds-v1'; cat "$rom"; } | sha256sum | cut -d' ' -f1)
export SM_SEED_PASSPHRASE
openssl enc -d -aes-256-cbc -pbkdf2 -iter 200000 -md sha256 -pass env:SM_SEED_PASSPHRASE \
  -in "$scratch/capture.enc" -out "$scratch/capture.tar.gz"
"${SM_PY:-python3}" "$(dirname "$0")/publish_native_fixture.py" \
  "$scratch/capture.tar.gz" "$destination" "$folder" "$members" "$@"
