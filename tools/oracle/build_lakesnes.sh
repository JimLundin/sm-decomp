#!/usr/bin/env bash
# Build liboracle_lakesnes.a, the native half of crates/oracle: the pinned
# upstream LakeSnes sources, unmodified, and the crate's bridge, compiled with
# the project's pinned Zig 0.15.2 (docs/NATIVE_DEPENDENCIES.md).
#
# Usage: tools/oracle/build_lakesnes.sh [--target x86_64-windows-gnu|x86_64-linux-gnu]
# Environment: ZIG (default: the sm-decomp-tools Zig, else zig on PATH),
# SM_LAKESNES_CACHE (default: sm-decomp-tools/lakesnes-oracle-9db90b8).
# It prints the library directory; put it on LIBRARY_PATH for Cargo.
set -euo pipefail

revision=9db90b86e46a377609305e298dd92d71cd1d4c8a
root=$(cd "$(dirname "$0")/../.." && pwd)
target=x86_64-windows-gnu
if [ "${1:-}" = --target ]; then target=$2; fi
if [ -n "${LOCALAPPDATA:-}" ]; then
  tools="$LOCALAPPDATA/sm-decomp-tools"
else
  tools="${XDG_CACHE_HOME:-$HOME/.cache}/sm-decomp-tools"
fi
cache=${SM_LAKESNES_CACHE:-"$tools/lakesnes-oracle-${revision:0:7}"}
zig=${ZIG:-"$tools/zig-x86_64-windows-0.15.2/zig.exe"}
if [ ! -x "$zig" ]; then zig=zig; fi

pins="27568d73a4ce17ced1bf37e86bfee6fcb9b0f6a27de517ddbc16c946c5c509fd LICENSE.txt
f9dabfaf4e1b154bf7496117f7d0d5d6bbe7c19aa9d898da14985f69ed20ca7e snes/apu.c
9b4794d7b8225f78524d7509afb0eaca9487cc2594eff8b1e34884cccf72c3b7 snes/apu.h
b2f1a7cf4af1feba1a99cfbfda44aafdad84e6951d59c40e38922389567927d8 snes/cart.c
e5fc61e436efa1f40b1c958a9341f3aede47ebb0dbe6a682e7e3ba74fb52c74e snes/cart.h
dc1aac884ab84e16d50e4e4d863ab6f292ef8cbcfba25a905a05cf426d951c5d snes/cpu.c
251f08c0144cff5d084525cb1d5d1f55c5cf2c2bbda4fb2b1d4a027ef635b50c snes/cpu.h
ede1325fd8c6a48fc41dd93769d101561ae07d2dcf8a7a3d5bdf95787108a4ac snes/dma.c
a27ea6294ebd22180060da85d5d74d30c4abd2a16c2a0679978c4af081df40da snes/dma.h
b9ac27103a7aba446c05bd7c824ca7b488832460987b8e90e25ee44c7b175d0c snes/dsp.c
1a9025deb9c6da3bd47d90c04381953e4ec8d43911b3d092b9f692e799ddc073 snes/dsp.h
07b2d61e8d8fed3702ff096361a7af3431cc54823ee840ac8185d0e4fdd396ac snes/input.c
07ecfb59d84ad788c6abcf28977152ac67813f7d6b5e0e2231e10f5ff16f8d44 snes/input.h
e4235b0e02305a3b7d5f6e05218640242f1e8f0b24d6791bcc60140be0470da8 snes/ppu.c
501990773c1a9ef20cd6f716d94cf847a6702a7a3d3d444b7a5c770c5f7c27f9 snes/ppu.h
bdfeb3cfba0db3c9cb014dbf777162a8c35e60f6a730e1e21abf717f3afd2ab2 snes/snes.c
02537b972aa0af32c2fa4d8dac96f51526b8213de6fd89f7d3bcc2e115450e78 snes/snes.h
ba52833b6cdf7804d77f272304af1670f2dcb0f6b423c6d006a2b13f213f7ac4 snes/snes_other.c
0905b8ccb8bb99c3883ae502746870eac4126385bbb7bd7c78aad9e5adefeab4 snes/spc.c
4f67323c2577592fdcf50780eca54357b774cfe79a79cb47a5a4d3e188043314 snes/spc.h
20d0e362dc200ea79d6fcba62868683921976327e3114b8c369d9799f484b8b2 snes/statehandler.c
6e1b3affd60dd242bfa2dd5b25c67edfb8663d28f6c2b89e8db1a15b7b7d7249 snes/statehandler.h"

source="$cache/source"
mkdir -p "$source/snes"
while read -r sha256 name; do
  path="$source/$name"
  if [ ! -f "$path" ]; then
    curl -fsSL --retry 3 -o "$path.part" \
      "https://raw.githubusercontent.com/angelo-wf/LakeSnes/$revision/$name"
    mv "$path.part" "$path"
  fi
  echo "$sha256  $path"
done <<< "$pins" | sha256sum --check --quiet

# Named emulator fixes (crates/oracle/native/fixes/NNNN-*.patch) apply in order to
# a copy of the pinned sources; the upstream cache stays untouched.
build="$cache/build-$target"
rm -rf "$build"
mkdir -p "$build"
cp -r "$source/snes" "$build/snes"
for fix in "$root"/crates/oracle/native/fixes/*.patch; do
  [ -e "$fix" ] || continue
  (cd "$build" && git apply "$fix")
done

library="$cache/lib-$target"
objects="$library/objects"
mkdir -p "$objects"
flags=(-target "$target" -O2 -std=c11 -fno-sanitize=undefined -ffp-contract=off -fno-strict-aliasing
  -I "$build/snes")
if [ "$target" = x86_64-linux-gnu ]; then flags+=(-fPIC -D_DEFAULT_SOURCE); fi
for unit in cpu apu spc dsp ppu cart input statehandler snes_other; do
  "$zig" cc "${flags[@]}" -c "$build/snes/$unit.c" -o "$objects/lakesnes_$unit.o"
done
# DMA's B-bus accesses go through the bridge, which logs the APU ports.
"$zig" cc "${flags[@]}" -Dsnes_readBBus=oh_dma_readBBus -Dsnes_writeBBus=oh_dma_writeBBus \
  -c "$build/snes/dma.c" -o "$objects/lakesnes_dma.o"
"$zig" cc "${flags[@]}" -c "$root/crates/oracle/native/bridge.c" -o "$objects/oracle_bridge.o"
rm -f "$library/liboracle_lakesnes.a"
"$zig" ar rcs "$library/liboracle_lakesnes.a" "$objects"/*.o
echo "$library"
