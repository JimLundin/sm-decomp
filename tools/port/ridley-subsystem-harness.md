# Whole Ridley fixture adapter integration

Owned adapter: `crates/pipeline/src/ridley_subsystem_adapter.rs`, public
`build`, `case`, `check` and crate-visible `check_instrumented_units`;
dependency contract `rust-ridley-subsystem-v1`.
Fixture generator: `ridley_subsystem_fixtures::generate(checkout, rom, output)`.
Manifest: `tools/port/ridley-subsystem.toml`, callback adapter `ridley-subsystem`.

The root coordinator owns `main.rs`, `family.rs`, batch catalog, public port
exports and live registration. This handoff supplies the required routing while
keeping those shared files untouched in the implementation worktree.

## Shared routing

1. Add `mod ridley_subsystem_adapter; mod ridley_subsystem_fixtures;` to pipeline
   `main.rs`. Expose a bounded fixture generation command using the generator
   above. The manifest is offline-only; live eligibility remains false.
2. In `family::verify`, recognize request adapter `ridley-subsystem`, validate it
   agrees with the manifest, and route build/case to the new adapter. The existing
   `fixture_execution::run` owns dependency scheduling, the shared leases and
   independent preloaded C/Rust helpers. It may use Linux x86_64 fork or fresh
   execution. Every fixture starts Rust before C, and failures block successors.
3. Set `gameplay_translation = Some(true)` and retain actual-C dependency counts
   from `CaseRecord::callbacks`. Record seven rejected ABI controls when the
   build completes and set `remaining_c_gameplay_callbacks` to the checked sum
   of these per-case counts. Do not set remaining C callbacks to zero.
4. In `family::authenticate`, recognize `rust-ridley-subsystem-v1`, require its
   callback adapter `ridley-subsystem`, and check all four sealed binaries:
   `ridley`, `c`, `rust`, `libridley.a`. All three executable paths contain the
   same linked image; `fixture_execution` launches separate helper processes and
   independently forked subject children. Actual C never routes through Rust.
5. Require the new adapter/generator/ABI/host template, canonical headless source,
   family/batch/cleanup, executor/worker, metrics and sampler source seals. The
   embedded worker is part of `libridley.a`; require `fixture-worker.rs` and its
   exact worker-source hash instead of an extra `libfixture_worker.a` artifact.
   Allow this explicit contract at the fork-adapter check.
6. Require the final ABI-control stdout line `7 rejected ABI inputs`, the
   zero-service/no-gameplay invalid-input checks, and the declared execution
   contract. Invoke `ridley_subsystem_adapter::check(directory, fixture)` for
   each retained case and compare all returned metrics, input/output hashes,
   return values, callback counts and `capture_after_sha256` to the verdict.
7. Preserve existing outer source/archive/manifest/ROM/compiler dependency and
   coverage authentication. The new checker authenticates initial fixture bytes
   and host inputs, unpatched captured inputs, predecessor WRAM/host equality,
   full output WRAM, host words, Mode7 state/open buses, ordered effects and full
   service-entry/return snapshots. Artifact inventory protects every record.
8. Require and call `check_instrumented_units(source, run)` during retained
   authentication. It reconstructs the four `ridley-services-sm_*.c` units
   exactly from pinned C: only unique service/effect definitions are renamed,
   while all native public call sites remain intact. This observes same-unit
   nested services and requests as well as calls from other banks. Preserve
   these generated units as sealed source artifacts.

## Compared effects and external services

The ordered log records actual SNES writes/reads, audio/music requests,
projectile requests and typed service invocations. The sidecar `*-services.bin`
contains ordered service entry/return records. Each record has five little-endian
u32 words (phase, service kind, argument A, argument B, returned value), followed
by 131,072 WRAM bytes and the eight external u16 host words. Nested services keep
their actual order. The checker validates balanced entry/return nesting,
phases, service kinds, arguments, return extents and agreement with the ordered
service request stream alongside complete byte equality. All fifteen native
sound channel/queue maximum variants are observed, including service-internal
requests. Complete stdout/stderr is retained even when native C prints a
diagnostic before its final metrics line.

The final metrics line contains return, actual-C service count, CPU-fallback
call count and CPU-fallback instruction count. Both fallback counters must be
zero. A service is measured C ownership, not a Rust translation claim.

The ABI controls first require a supported `Ridley_Shot` invocation to mutate
WRAM, then require each invalid input to leave an explicit private-gameplay
invocation counter unchanged. C controls also require zero service and both
fallback counters. Caller WRAM and all host words must remain unchanged on
each refusal.

The PPU sidecar contains eight Mode7 matrix words, the shared Mode7 latch and
both PPU open buses plus the SNES open bus (20 bytes). Every fixture restores
`ppu-before.bin` before either subject runs. Controlled initial fixtures use
the native reset state; a continuation inherits matching complete predecessor
PPU state. Mode7 multiplication uses actual WriteReg/ReadReg operations in
native order. The native invocation now takes 15 arguments, with the hardware
input path at argv14.

Captured fixtures use the inventory's canonical `NAME-before.wram` path and
seven companions: `NAME-before.host`, `NAME-before.ppu`, `NAME-after.wram`,
`NAME-after.host`, `NAME-after.ppu`, `NAME-effects.log` and `NAME-services.bin`.
Each companion must be in manifest `rust_dependencies`, so the compiler-input
contract hashes it before execution. Each case also retains the exact files
and hashed `capture-inputs.toml`; authentication checks both original and
retained hashes plus full host/hardware/effect/service expectations. The
captured host words in the manifest must match `NAME-before.host` exactly.
Require `expected_after_file` and its hash for every such capture. A captured
fixture is never patched or used to replace a chained input.

## Acceptance boundary

The controlled fixture seam intentionally supports directly chosen native state
and callback inputs beyond the immutable live selector. It diagnoses the whole
implementation without granting live admission to synthetic states. The public
live boundary validates native headers, immutable script/table bytes and complete
reachable control flow before mutation; accepted errors terminate.
Frames that reach `SpawnEnemy` or `EnemyItemDrop` are conservatively refused
until their exact continuation footprints are modeled in admission. Terminal
contact/death services and reviewed Samus handler updates remain explicit
actual-C dependencies. Record this limitation separately from controlled parity.

The coordinator must extend the existing native inventory once at the whole
Ridley lifecycle/AI seam, authenticate a captured natural supplement, then run
one combined four-independent-process acceptance replay. Controlled fixtures
alone grant no original-ROM parity or live exercise. Final source integration,
affected-family checks, run cleanup/deduplication and worktree retirement remain
required before closing the local scope.
