#!/usr/bin/env bash
# Fetch root-pinned native fixtures and the independent continuation seed.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <new-fixture-dir>" >&2
  exit 2
fi
: "${AUDIO_NATIVE_ASSET:?AUDIO_NATIVE_ASSET is required}" "${AUDIO_NATIVE_SHA256:?AUDIO_NATIVE_SHA256 is required}"
[[ "$AUDIO_NATIVE_ASSET" =~ ^audio-native-[0-9a-f]{16}\.tar\.xz\.enc$ ]]
[[ "$AUDIO_NATIVE_SHA256" =~ ^[0-9a-f]{64}$ ]]
rom=$1
destination=$2
rom_sha256=12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72
test "$(sha256sum <"$rom" | cut -d' ' -f1)" = "$rom_sha256"
scratch=$(mktemp -d)
scratch_resolved=$(cd "$scratch" && pwd -P)
cleanup() {
  "${SM_PY:-python3}" - "$scratch" "$scratch_resolved" <<'PY'
import pathlib
import sys

scratch = pathlib.Path(sys.argv[1])
expected = pathlib.Path(sys.argv[2]).absolute()
if scratch.is_symlink() or scratch.resolve(strict=True) != expected or not scratch.is_dir():
    raise SystemExit("audio fixture scratch path changed")
for name in ("pair.enc", "pair.tar.xz"):
    path = scratch / name
    if path.is_symlink():
        raise SystemExit("audio fixture scratch file is a symlink")
    path.unlink(missing_ok=True)
scratch.rmdir()
PY
}
trap cleanup EXIT
curl --fail --silent --show-error --location \
  --retry 8 --retry-delay 15 --retry-all-errors --retry-max-time 600 \
  --output "$scratch/pair.enc" \
  "https://github.com/JimLundin/sm-decomp/releases/download/ci-data/$AUDIO_NATIVE_ASSET"
echo "$AUDIO_NATIVE_SHA256  $scratch/pair.enc" | sha256sum --check
SM_SEED_PASSPHRASE=$({ printf 'sm-decomp-seeds-v1'; cat "$rom"; } | sha256sum | cut -d' ' -f1)
export SM_SEED_PASSPHRASE
openssl enc -d -aes-256-cbc -pbkdf2 -iter 200000 -md sha256 -pass env:SM_SEED_PASSPHRASE \
  -in "$scratch/pair.enc" -out "$scratch/pair.tar.xz"
test "$(tar -tJf "$scratch/pair.tar.xz" | LC_ALL=C sort)" = "$(printf '%s\n' control.local.toml expire-post64-seed.local.toml expire.local.toml native.sha256)"
mkdir "$destination"
tar -xJf "$scratch/pair.tar.xz" -C "$destination"
(cd "$destination" && sha256sum --check --strict native.sha256)
echo 'NATIVE AUDIO FIXTURE PIN OK'
bash "$(dirname "$0")/fetch_native_fixture.sh" "$rom" "$destination" opening-ceres-audio-v1 \
  m0-opening-ceres-audio-origin-d7a17dcad0bb037c.tar.gz.enc \
  d7a17dcad0bb037c7de70303db59805a2c480b2012d4171360a494b05d390148 560 1
