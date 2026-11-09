#!/usr/bin/env bash
# Packs the lockstep record cache (A2) for the ci-data release, where the
# `verify / lockstep-ratchet` job fetches it.
#
#   tools/ci/pack_lockstep_cache.sh <cache-dir> <rom> [out-dir]
#
# <cache-dir> holds one directory per row (index.txt, records.bin, and hits.txt
# when it was recorded with watches). The rows are those
# scenarios/locks/lockstep.ratchet.txt names, or those ROWS lists. The tar is
# deterministic, so its name is a hash of its contents: the same cache packs to
# the same name. xz -9e because the records are snapshots of nearly the same
# state: 627 MB pack to about 1 MB, and the runner unpacks them in seconds.
# Encryption matches the seed bundles (docs/ORACLE.md): AES-256-CBC,
# PBKDF2 with 200000 iterations and SHA-256, passphrase
# sha256("sm-decomp-seeds-v1" + ROM). Prints the asset name and its ciphertext
# SHA-256; the root uploads it and pins both in .github/workflows/verify.yml.
set -euo pipefail

if [ $# -lt 2 ] || [ $# -gt 3 ]; then
  echo "usage: $0 <cache-dir> <rom> [out-dir]" >&2
  exit 2
fi
cache=$1
rom=$2
out=${3:-${TEMP:-/tmp}/collab/ci-cache}
here=$(cd "$(dirname "$0")/../.." && pwd)
ratchet=$here/scenarios/locks/lockstep.ratchet.txt

rom_sha256=12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72
if [ "$(sha256sum <"$rom" | cut -d' ' -f1)" != "$rom_sha256" ]; then
  echo "$rom: not the expected ROM" >&2
  exit 1
fi

# The ratchet's `row ROW ticks N equal N` lines name the rows, unless ROWS lists others
# (a harness cache: every row the harnesses read).
if [ -n "${ROWS:-}" ]; then
  read -ra rows <<<"$ROWS"
else
  mapfile -t rows < <(sed -nE 's/^row ([a-z0-9-]+) .*/\1/p' "$ratchet" | sort -u)
fi
for row in "${rows[@]}"; do
  for file in index.txt records.bin; do
    if [ ! -f "$cache/$row/$file" ]; then
      echo "$cache/$row/$file: missing" >&2
      exit 1
    fi
  done
done

mkdir -p "$out"
plain=$(mktemp "$out/lockstep-cache.XXXXXX")
trap 'rm -f "$plain" "$plain.xz"' EXIT
# Written through stdout: GNU tar reads "C:/..." as a remote host.
(cd "$cache" && tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner \
  --mode=u+rw,go-w,a+rX -cf - "${rows[@]}") >"$plain"
name=lockstep-cache-$(sha256sum <"$plain" | cut -c1-16).tar.xz.enc

xz -T0 -9e -c "$plain" >"$plain.xz"
SM_SEED_PASSPHRASE=$({ printf 'sm-decomp-seeds-v1'; cat "$rom"; } | sha256sum | cut -d' ' -f1)
export SM_SEED_PASSPHRASE
openssl enc -aes-256-cbc -salt -pbkdf2 -iter 200000 -md sha256 -pass env:SM_SEED_PASSPHRASE \
  -in "$plain.xz" -out "$out/$name"

echo "name   $name"
echo "sha256 $(sha256sum <"$out/$name" | cut -d' ' -f1)"
echo "bytes  $(wc -c <"$out/$name" | tr -d ' ')"
echo "path   $out/$name"
