#!/usr/bin/env bash
# Pack the final validated pair only. Root uploads and pins the emitted ciphertext.
# Each private fixture contains the frozen source pins and named writer exclusions.
set -euo pipefail
if [ "$#" -ne 3 ]; then
  echo "usage: $0 <validated-fixture-dir> <rom> <out-dir>" >&2
  exit 2
fi
fixtures=$1
rom=$2
out=$3
rom_sha256=12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72
test "$(sha256sum <"$rom" | cut -d' ' -f1)" = "$rom_sha256"
for arm in control expire; do test -f "$fixtures/$arm.local.toml"; done
mkdir -p "$out"
scratch=$(mktemp -d "$out/audio-native.XXXXXX")
scratch_resolved=$(cd "$scratch" && pwd -P)
cleanup() {
  "${SM_PY:-python3}" - "$scratch" "$scratch_resolved" <<'PY'
import pathlib
import sys

scratch = pathlib.Path(sys.argv[1])
expected = pathlib.Path(sys.argv[2]).absolute()
if scratch.is_symlink() or scratch.resolve(strict=True) != expected or not scratch.is_dir():
    raise SystemExit("audio fixture scratch path changed")
for name in ("control.local.toml", "expire.local.toml", "native.sha256", "pair.tar.xz"):
    path = scratch / name
    if path.is_symlink():
        raise SystemExit("audio fixture scratch file is a symlink")
    path.unlink(missing_ok=True)
scratch.rmdir()
PY
}
trap cleanup EXIT
cp -- "$fixtures/control.local.toml" "$fixtures/expire.local.toml" "$scratch/"
(cd "$scratch" && sha256sum control.local.toml expire.local.toml > native.sha256)
(cd "$scratch" && tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner \
  --mode=u+rw,go-w,a+rX -cJf - control.local.toml expire.local.toml native.sha256) > "$scratch/pair.tar.xz"
name=audio-native-$(sha256sum <"$scratch/pair.tar.xz" | cut -c1-16).tar.xz.enc
SM_SEED_PASSPHRASE=$({ printf 'sm-decomp-seeds-v1'; cat "$rom"; } | sha256sum | cut -d' ' -f1)
export SM_SEED_PASSPHRASE
openssl enc -aes-256-cbc -salt -pbkdf2 -iter 200000 -md sha256 -pass env:SM_SEED_PASSPHRASE \
  -in "$scratch/pair.tar.xz" -out "$out/$name"
echo "name   $name"
echo "sha256 $(sha256sum <"$out/$name" | cut -d' ' -f1)"
echo "bytes  $(wc -c <"$out/$name" | tr -d ' ')"
echo "path   $out/$name"
