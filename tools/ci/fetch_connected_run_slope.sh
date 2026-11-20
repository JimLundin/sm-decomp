#!/usr/bin/env bash
# Immutable reviewed connected ordinary inputs; decoded by replay admission.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
rom=$1
destination=$2
asset=connected-run-slope-6ae54520fbb3b30f.tar.gz.enc
sha256=6ae54520fbb3b30f359130e75315e0356d786bce989ac2ef0ca2d630bfe74761
test "$(sha256sum <"$rom" | cut -d' ' -f1)" = 12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72
test -d "$destination"
test ! -e "$destination/connected-run-slope-v1"
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
test "$(wc -c <"$scratch/capture.enc")" -eq 4222096
SM_SEED_PASSPHRASE=$({ printf 'sm-decomp-seeds-v1'; cat "$rom"; } | sha256sum | cut -d' ' -f1)
export SM_SEED_PASSPHRASE
openssl enc -d -aes-256-cbc -pbkdf2 -iter 200000 -md sha256 -pass env:SM_SEED_PASSPHRASE \
  -in "$scratch/capture.enc" -out "$scratch/capture.tar.gz"
# The ciphertext authenticates the archive; refuse unexpected extraction paths too.
"${SM_PY:-python3}" - "$scratch/capture.tar.gz" "$destination" <<'PY'
import pathlib
import sys
import tarfile
with tarfile.open(sys.argv[1], "r:gz") as archive:
    members = archive.getmembers()
    if len(members) != 146 or any(
        not member.isfile() or pathlib.PurePosixPath(member.name).parts[0] != "connected-run-slope-v1"
        or ".." in pathlib.PurePosixPath(member.name).parts
        or pathlib.PurePosixPath(member.name).is_absolute()
        for member in members
    ):
        raise SystemExit("unexpected connected ordinary archive members")
    archive.extractall(sys.argv[2], filter="data")
PY
echo 'CONNECTED RUN SLOPE FIXTURE PIN OK'
