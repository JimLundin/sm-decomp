Status: resolved
Blocked by: None
Owner: `/root/ridley_captures`
Triage: ready-for-agent

Correct the native Ridley PPU evidence contract in isolated candidate source.
The old authenticated inventory and natural fixture supplement remain retained
under `ridley-natural-recovery` with their explicit active-PPU limitation.

## Contract

Forty exact bytes: byte 0 active discriminator (0 = `my_ppu`, 1 = `snes_ppu`);
bytes 1–19 hold `my_ppu` (eight signed little-endian Mode7 words, previous latch,
PPU1 and PPU2 open-bus bytes); bytes 20–38 hold `snes_ppu` in the same order;
byte 39 holds SNES open bus. Restore writes both views and selects the exact
active pointer. Unknown active pointers or discriminators reject.

`source/crates/pipeline/src/ridley_ppu_contract.h` supplies identical capture
and restore helpers to inventory and fixture adapter; the new adapter contract
is `rust-ridley-subsystem-v2`. Complete WRAM/host/service contracts are preserved.
All root maintained sources stay untouched; local candidate main only adds
the captured manifest command needed for isolated verification.

## Completion evidence

Fresh full native inventory:
`inventory/1791860689806824782-105188/verdict.toml`, status
`diagnostic-complete`, 129,712 inputs, 897 captures, 483 counter keys, 59,217
boundary entries, 118,434 boundary events, 239,580 ordered effects, eight captured
actual-C service calls and zero CPU fallback. Both scoped cleanup passes succeed
with zero bytes; `inventory-after-cleanup-authentication.toml` authenticates the
retained native evidence and grants no gameplay admission.

Candidate actual-C families:

- `controlled/1791860698541096-105766-0/verdict.toml`: `offline-verified`, 6,457
  pairs across 33 entries, seven ABI controls, 305 measured actual-C service calls,
  zero CPU fallback.
- `natural/1791861212110218-129925-0/verdict.toml`: `offline-verified`, 670
  genuine unpatched pairs across 20 entries, seven ABI controls, eight measured
  actual-C service calls, zero CPU fallback. Generation authenticated the refreshed
  inventory and retains all seven companion assets per selected capture.

The std-only `checker.rs` independently matches every selected full WRAM output,
all eight host words, raw normalized returns, ordered effects, complete native
before-service state and continuation, both PPU views and the active discriminator.
`public-support-parity.toml` and `.tsv` retain its 670 results, all header `e13f`
and area 6. Public admission assessed separately against the pinned original ROM
still admits 574; 92 direct fixture-only states and four getaway boundaries refuse.
The padded/patched native ROM view executes the fixture domain and is separately
authenticated. Gameplay outside the captured Ceres domain and original-ROM parity
remain untested.

Twelve isolated pipeline subsystem tests, three independent checker tests and a
native regression probe pass. The Rust `ppu-regression.rs` orchestration compiles
the pinned actual `snes/ppu.c` with the v2 helper and checks active mutation,
unchanged alternate state, signed multiplication/read values, exact both-view
restore and unknown discriminator refusal. Source Clippy passes using documented
baseline exemptions for untouched pipeline modules; owned Rust format checks pass.
`source-evidence.sha256` and `source-after-cleanup-authentication.log` authenticate
the candidate sources and executable after removing build artifacts.

Retained executable: `bin/sm-pipeline-hardware-v2`, SHA256
`a1201f0267837c680df85c773f01b682617b1e780d8c1bcaebbb391f9fb73282`.
Shared helper SHA256:
`e6d3e4088036779ec5ef128e3e534d7a39f96b6c5d3227d1c20f657236f36c91`.
The executable remains available for retained authentication after root integration.
`cargo clean` removes 2,096 build files (reported 1.8 GiB); direct checker/test and
native probe executables were removed. No Git worktree was created. The temporary
`.git` link used for repository root discovery was removed; retained source
authentication requires no Git metadata. The deps, ROM and movies links are
retained evidence input paths into the main checkout.

Controlled first cleanup removes zero bytes; controlled deduplication reclaims
1,667,891,200 bytes and checks 19,371 snapshots. Natural first cleanup removes
zero bytes; natural deduplication reclaims 292,814,848 bytes and checks 3,350
snapshots. `natural-after-cleanup-authentication.toml` reauthenticates all 670
pairs after cleanup. `controlled-after-cleanup-authentication.toml`
reauthenticates all 6,457 pairs after cleanup; `inventory-retained-context-authentication.toml`
authenticates the full native inventory after `.git` removal. Final verdict/source
hash checks pass. No cleanup action remains. Last measured free space:
227,796,942,848 bytes; retained source snapshots are 10,844,368 bytes and the
frozen executable is 184,085,848 bytes. Retain all sealed family/capture/input
dependencies and shared immutable pipeline/evidence objects.
No root maintained source, registry or manifest was edited. Integrate only the
candidate header, inventory producer/template, captured generator, adapter/host
and one-line family header-source seal. Candidate `main.rs` supplies isolated
orchestration only; root owns final shared CLI/stage publication and live ownership.

## Read-only live probe diagnosis

After the hardware handoff, the integration owner's separate 9,000-frame r5 probe
failed at frame 8,296 before public support on `ProcessEnemyInstructions`.
`/tmp/sm-ridley-whole-live-probe-9000-r5/families.rs` requires runtime ROM bytes
to equal its compiled execution view. The retained `ridley-error-runtime.rom`
differs at exactly one byte: file offset `0x2965a`, `0x8f` to `0x8b`; all other
4,194,303 bytes match. This is the pinned native bug-fix-counter transformation
of the joypad source operand. The existing integration template already permits
that exact transformation, but the r5 generated/compiled snapshot predates it.
The observed current enemy header `e23f` is outside Ridley, so the global ROM
check errors before gameplay admission can correctly refuse the unrelated call.
`live-probe-rom-diff.log` and `live-probe-diagnostics.sha256` retain the read-only
finding; Rust orchestration source is `rom-diff.rs`, and its build output is
removed. The integration owner was given the precise evidence and owns rebuilding
the current template and the unknown-ROM-mutation controls. No live source was
edited by this lane.

`live-rom-guard-check.rs` reproduces that exact retained guard failure with the
pristine r5 generated source and native error image. Its strict comparison test
exits 101 before any support call; `live-rom-guard-pristine-red.stderr` retains
the expected failure. The same Rust orchestration extracts the exact current
template function and checks that the actual image with the one permitted native
patch passes, while unrelated-byte, alternate-operand and shortened-image controls
refuse. `live-rom-guard-probe.log` records that green result. All direct binaries
were removed after the checks. This proves the admission-image guard correction;
it grants no live replay acceptance.
