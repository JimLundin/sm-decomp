# Rio native controls

Build from `restart/` on a host permitting the pinned toolchain's producers:

```sh
cargo build --locked --offline -j 1 -p sm-native-host --example rio-host-control
target/debug/examples/rio-host-control PREPARED_LIBRARY ROM NEW_OUTPUT
```

Use the Rio recipe's authenticated prepared library, not an older library. The
output must be new. The caller must supervise the native process with a deadline,
inspect its exit status and descendants, and run required project cleanup after
completion. This example is not an unattended supervisor.

The runner declares 24 actual native C/Rust pairs: all ten typed roots in air
and solid-wall terrain, plus State3 horizontal/vertical extensions and spike
allocation with free/full PLM pools. It compares full canonical WRAM, returned continuation,
ordered native effects, ownership/service counter deltas and unchanged SRAM.
Twelve admission checks cover wrong explicit/current indices, zero room width,
zero actor radius, wrong dynamic bank, bad continuation and unsupported state.
They also cover extension cycles, out-of-range targets, extension chains and
unsupported spike indices and distinct explicit/current movement actors. Native
slope helpers read the current actor, so movement admission requires that actor
to match the explicit slot. Admission must leave complete WRAM and counters unchanged. A final success record
is written only after all cases pass. Failed output remains diagnostic evidence.

These synthetic cases are distinct from the larger scripted actor-body corpus.
Nineteen isolated fatal children cover malformed buffer/size/token/result/selector/
slot/amount/order, binding and commit guards, recursive and sibling roots, unissued
services and invalid callback completion. Each has a 20-second deadline and must
exit with code 2 and its declared fatal message. The outer caller still supervises
the entire runner and checks for unexpected descendants.

The runner retains source and executable/library hashes. It has not compiled or
executed on the current Windows host because dependency build scripts are blocked.
These are declared cases, not passing verification evidence.

Two additional valid air fixtures invoke ordinary Main AI with a movement vector
just beyond the Rust admission limit. They compare C against installed Rust
refusal followed by C, including full WRAM/effects and exact ownership counters.
The malformed diagnostic admissions are never invoked as native gameplay.

The focused package exposes `prepare_sources` for a private authenticated recovered
source copy. It executes the exact production source patching sequence without
archive/compiler/Game dependencies. Callers seal inputs, outputs and patch
receipts; this seam alone grants no native authentication or gameplay acceptance.

`restart/examples/rio-runtime-control.rs` declares four full-state pairs through
public production installers with all twelve owners registered. Build with
`cargo build --locked --offline -j 1 -p sm-restart --example rio-runtime-control`.
Its arguments are `LIB ROM JET_INPUT RIPPER_INPUT WAVER_INPUT RIO_INPUT FRESH_OUT`.
Use retained Ripper/Waver air fixtures and the new native runner's `Main-air/input.wram`.
It checks exact target counter deltas, balanced services, unchanged sibling counters
and SRAM, and ordinary CallEnemyAi dispatch. This runner also remains uncompiled
and unexecuted. A natural encounter and affected baseline replay remain required.
