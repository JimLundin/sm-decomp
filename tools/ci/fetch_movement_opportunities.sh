#!/usr/bin/env bash
# Immutable reviewed passive movement opportunity capture; decoded by replay admission.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
rom=$1
destination=$2
asset=movement-opportunities-84754af24c91d7bf.tar.gz.enc
sha256=84754af24c91d7bf7339b2f6cf3dbc425758a48c5e514697ba833ad3e1e5bc4b
test "$(sha256sum <"$rom" | cut -d' ' -f1)" = 12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72
test -d "$destination"
test ! -e "$destination/movement-opportunities-v1"
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
test "$(wc -c <"$scratch/capture.enc")" -eq 7532576
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
    if len(members) != 284 or any(
        not member.isfile() or pathlib.PurePosixPath(member.name).parts[0] != "movement-opportunities-v1"
        or ".." in pathlib.PurePosixPath(member.name).parts
        or pathlib.PurePosixPath(member.name).is_absolute()
        for member in members
    ):
        raise SystemExit("unexpected opportunity archive members")
    archive.extractall(sys.argv[2], filter="data")
PY
echo 'MOVEMENT OPPORTUNITY FIXTURE PIN OK'
