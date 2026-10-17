# Ticket18 item instruction implementation handoff

Owner: `/root/material`; integration owner: `/root`; native seam owner:
`/root/native`. This submission owns only `restart/item-instructions/` in
`/home/jim/sm-decomp-restart-item-rust`, branch `restart/item-instructions-rust`.
Baseline: `2f6ef65fd3f764ef999583f48c424de2c1c7d152`.

Source commits:

- `5b095b9625d1efe5dc4f2a3363f458b778515c7a`: thirteen sealed runtime bodies.
- `00919a0a7800913d3449784655bb300c9bb9abfb`: shared fixtures, six tests, README.

The public `execute`, `Entry`, `Output`, `Service`, and `Services` API follows
the frozen item interface. `fixture-support` exposes public module `fixtures`
with exactly 25 body cases and 14 refusal cases. `Expectation` supports equality;
`Fixture::run` safely rejects invalid raw entries. Direct-write expectations
exclude native service effects and are not an actual-C oracle.

## Source and input dependencies

Frozen semantics contract:
`/home/jim/sm-decomp-restart/restart/out/item-instructions/planning/semantics-contract.md`
SHA256 `07f9978238a4e7ae3b0a6d8f8635a4f76dbe46fa2a852da628727feb0b6e3e95`.

Native seam contract SHA256:
`05bf02441cc269907d43ff6fec8f50267ecd33275be2bf89e6534b6800a87c5e`.

Recovered C pin: `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, retained beneath
`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`.
Source hashes:

| File | SHA256 |
| --- | --- |
| sm_84.c | `7a7da7a5c1a35033c8cac6d608cc9a3c043c52347504ab0f8609ac2c6acc095a` |
| sm_80.c | `171d73158e0e7323118b4bb142d4e7c79f3d74cdc4129a667aa8873d7eb785b7` |
| sm_82.c | `20ae568cc21260a577e441bfcd25ac3fdc33e00fce3eb5c82fa85c7f9ccabebe` |
| sm_85.c | `01035a449979bdb7c50d26f4733b2cda9d04f86a5d3d2596446216a3a096d2c2` |
| variables.h | `e2ce28d07ceff91129e93417099f33f20f161c37e95e745c575d0e30b3fd5585` |

Tests require the private ROM `/home/jim/sm-decomp/rom/sm.sfc` and frozen seed
`/home/jim/sm-decomp-restart/restart/out/first-missile/author-05/missile-revealed.wram`.
Seed SHA256: `7adc5568e4239e479fcef325fef05769da8f6399e5909174d5f6f00c2f436459`.
Environment overrides are `SM_ITEM_FIXTURE_ROM` and `SM_ITEM_FIXTURE_SEED`;
overrides do not bypass the seed hash or script seals. No captures are committed.

## Verification and remaining integration

Source inspection, `rustfmt --edition 2024`, and `git diff --check` passed.
Per the integration owner's producer constraint, this worker ran no build,
unit tests, actual-C controls, or gameplay. The six unit tests are authored,
not claimed passing. Root owns workspace/lockfile registration and the next
integrated command, `cargo test -p sm-item-instructions --locked --offline -j 1`,
from `/home/jim/sm-decomp-restart` after merging and registering this crate.

Root owns the actual-C 25-body/14-refusal comparison, native transport controls,
and fixed 24832-pad C/Rust live pair. The live pair covers normal lifecycle,
message completion, and actual room-load clear-item composition. Compact
native fixtures cover already-collected room-load behavior. Gameplay acceptance,
original-ROM parity, and inventory stage publication are not granted here.

The adapter must preserve service order and full pre/post service state and
must treat service failure after effects as fatal without C fallback. Ordinary
instruction admission must not require stored IP to equal operand minus two;
the native scheduler updates stored IP after a multi-instruction loop.

## Cleanup

This worker started no verification processes and created no build/run outputs;
local bytes reclaimed: 0. Available filesystem bytes at handoff inspection:
103317594112. No process owned by this worker is writing a run directory.
Root retains ownership of integrated verification cleanup, required evidence
dependencies, shared inventory publication, and final implementer worktree
retirement. Do not retire this worktree until its source is integrated and any
evidence-referenced source has been preserved.
