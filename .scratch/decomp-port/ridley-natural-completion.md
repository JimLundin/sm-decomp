# Natural whole Ridley captured supplement

Status: resolved
Blocked by: None
Owner: `/root/ridley_recovery/natural_supplement`

The maintained Rust generator
`crates/pipeline/src/ridley_subsystem_captured.rs` exposes
`generate(checkout, inventory_verdict, output) -> Result<Manifest>`.
It authenticates the complete native inventory, derives supported signatures
from the existing 33-entry controlled manifest and pinned native declarations,
then independently selects complete raw event pairs. Generic `EnemyInstr_Sleep`
uses the existing native callback signature and the pinned A6 alias. Native
pointer kinds preserve zero, bank-relative and ROM address normalization.

Every fixture keeps the unpatched 131,072-byte native WRAM input and output,
eight host words, 20 hardware bytes, ordered effects and service entry/return
snapshots. All eight capture files, including the seven companions, are declared
dependencies. Captures never replace chained inputs. The manifest uses the
original pinned 3 MiB ROM; native `SnesInit` derives the same patched/padded
4 MiB view used by the inventory.

## Completion evidence

- Manifest: `tools/port/ridley-subsystem-captured.toml`, 670 fixture pairs
  across 20 entries: AI 95, touch 4, shot 4, direct 104, state 92, script 230,
  opcode 75 and tail 66. The 75 opcodes include 16 generic sleep calls.
- Authenticated source inventory:
  `out/port/ridley-subsystem/recovery-inventory/1791858734336387661-2/verdict.toml`.
  It retains 897 captures across the 129,712-frame pinned replay. The supplement
  explicitly excludes 141 internal baby instruction, 80 baby-state and six
  helper direct calls whose standalone signatures are outside this fixture
  contract. Their containing lifecycle captures remain in the supplement.
- A dedicated Rust checker imported the maintained generator and family model,
  invoking maintained inventory authentication before generation. Its native
  pointer-contract test passes and strict Clippy passes. No Python was used.
- Formal actual-C command:
  `/home/jim/sm-decomp/target/debug/sm-pipeline family-verify --checkout /home/jim/sm-decomp --source /home/jim/sm-decomp/deps/snesrev-sm --manifest /home/jim/sm-decomp/tools/port/ridley-subsystem-captured.toml --adapter ridley-subsystem --output /home/jim/sm-decomp/out/port/ridley-subsystem/recovery-captured --root /home/jim/sm-decomp/out/pipeline --case-jobs 2`.
  All 670 pairs pass with seven rejected ABI controls, zero CPU fallback calls
  and instructions, and eight explicit measured actual-C service calls.
- Formal verdict:
  `out/port/ridley-subsystem/recovery-captured/1791859892114324-2-0/verdict.toml`.
  Retained family authentication passes before and after cleanup. Final receipt:
  `.scratch/decomp-port/ridley-natural-final-authentication.toml`.
- Scoped `sm-pipeline cleanup --root <formal run> --apply` succeeds and reclaims
  zero disposable bytes. Scoped `cleanup --root <formal run> --deduplicate
  --apply` succeeds, checks 3,350 snapshots and reclaims 292,814,848 bytes.
  `cargo clean --manifest-path /tmp/sm-ridley-captured-checker/Cargo.toml` removes
  783 files, 237.8 MiB. Remaining filesystem space: 224 GiB. No Git worktree was
  created. Retain all sealed run files, inventory captures/source, manifests,
  ROM and shared pipeline/evidence stores. Owned formatting and diff checks pass.

This is natural offline parity. Public admission continues to refuse lifecycle
frames reaching foreign `SpawnEnemy` or item-drop continuation footprints;
controlled and captured fixture acceptance does not grant public live admission.
The verdict records `live_exercised = false` and original-ROM parity as
`not-tested`. CLI registration, combined live acceptance and shared translation
stage publication remain the integration owner's work. No commit was made.

## Hardware scope correction from the continuation audit

The retained 20-byte hardware sidecar saves and restores `g_snes->my_ppu`.
The native headless path uses `g_snes->ppu == g_snes->snes_ppu` for register
reads and writes. Consequently this family proves the retained inactive
renderer state and ordered effects, but does not establish active final PPU
state parity. The existing immutable verdict and files remain unchanged.
The independent follow-up documents 102 captures with Mode7 activity and this
gap in `ridley-natural-recovery/report.md`. An isolated v2 hardware contract
now captures the active discriminator, both PPU engine views and SNES open bus
under `ridley-hardware-recovery/`; refreshed native and differential evidence
is required before reporting complete hardware parity or live acceptance.
