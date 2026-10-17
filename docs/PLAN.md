# Plan

Goal: Super Metroid as a readable, maintained Rust program that plays from
boot to credits, with emulated SNES hardware for video and audio. Map Rando
support comes after vanilla.

## Where we are (2026-10-17)

`sm/` is a complete Rust version of the game. It was transpiled once from
snesrev/sm with c2rust; there is no C left in this repository and we do not go
back to it. See `tools/transpile/README.md` for exactly how.

The code is correct but ugly: raw pointer offsets into `g_ram`, C integer
casts everywhere, `unsafe` on every function, and modules that find each other
through `extern "C"` declarations instead of `use`. The work from here is to
turn it into good Rust without changing what it does.

Everything that existed before is in the branch `archive/pre-transpile-2026-10-17`.

## The oracle

The binary contains two complete implementations of each frame:

- **theirs**: the original ROM code, run on the emulated 65816 in `sm/src/snes`.
- **mine**: the Rust game logic in `sm/src/game`.

In the default run mode (`RM_BOTH` in `sm/src/cpu_infra.rs`), every frame runs
theirs, snapshots RAM and hardware state, rewinds, runs mine, snapshots again
and compares (`RunOneFrameOfGame_Both`, `VerifySnapshotsEq`). On a mismatch it
prints `Verify failure!`, writes a bug snapshot, adopts theirs and keeps
going. So a single replay counts every bad frame.

This is the same method snesrev used to build the C version, and it is the only
correctness check this project needs. It does not depend on C, lsnes, Mesen or
any external emulator.

## Definition of done for any change

1. `cargo build --release` and `cargo fmt --check` pass.
2. The verify run (phase 1) over the any% movie has no new failing frames
   compared with `main`.
3. `sh tools/progress.sh` numbers did not go up, or the commit says why.

A commit with these three is finished. No evidence files, receipts, ticket
directories or acceptance documents. The commit message is the record.

## Layout

```text
Cargo.toml            workspace (one member: sm)
rust-toolchain.toml   pinned nightly (needed for extern_types until phase 2)
sm.ini                game config, read from the working directory
sm/src/lib.rs         module tree
sm/src/main.rs        SDL front end, input, frame loop
sm/src/game/sm_XX.rs  gameplay, one file per original ROM bank
sm/src/snes/          emulated SNES hardware (CPU, PPU, APU, DMA): runs "theirs"
sm/src/cpu_infra.rs   per-frame comparison, ROM patches, snapshots
sm/src/rtl.rs         runtime helpers used by gameplay (RAM, save states, replay)
sm/src/spc_player.rs  music driver
sm/src/platform/      OpenGL output
sm/src/ram_map.rs     names for g_ram offsets (from snesrev's variables.h)
movies/manifest.toml  TAS movies used for verification (downloaded, not committed)
rom/                  your ROM goes here, gitignored
tools/progress.sh     residue counters
tools/transpile/      record of how sm/src was generated
docs/PLAN.md          this file
```

Keep it this small. Add a crate only when the game code itself needs one.

## Phase 1: make the oracle a one-command check

Nothing else should start until this works, because it is how every later
change is judged.

1. **Run it once.** Put the ROM in `rom/sm.smc`, run
   `cargo run --release -- rom/sm.smc` from the repo root, play a minute and
   confirm no `Verify failure!` lines. Linux or WSL first. If anything
   differs from what snesrev's C build did, that is a transpile bug: fix it
   before anything else.
   Interactive play and WSL are unavailable on the current Windows machine;
   Cargo's build script is denied execution (OS error 5). Use the automated
   3,600-frame smoke run followed by any% in Linux GitHub Actions instead.
   The private archive supplied the ROM, restored locally to gitignored
   `rom/sm.smc`; expected SHA-1 confirmed. Private Actions could not start
   because GitHub reported a billing/spending limit. The archive was restored
   to archived status. Manual `.github/workflows/verify.yml` runs publicly
   using gzip/base64 ROM chunks in encrypted Actions secrets. Only comparison
   diagnostics are printed; no ROM, save state or image is uploaded.
   Stopped after one hour of investigation on 2026-10-17, per AGENTS.md.
   Linux release build and formatting pass. Real-ROM smoke crashes on frame
   0 before comparison completes: invalid LoROM read `$96:2000` in the
   decompressor's `LDA $0000,X` at `$80:B185` (PC advanced to `$80:B188`).
   Registers: A=$0080, X=$2000, Y=$C84D, SP=$1FF1, DP=$0000, DB=$96;
   direct-page $44..$50 is FB 09 FF 00 20 00 00 00 00 00 7F 00 00.
   `snes_loadRom` already resets hardware; missing reset is ruled out.
   Absolute-X address arithmetic looks correct; source scratch pointer
   $47..$48 is already $2000. Root cause remains unresolved. Next diagnostic
   step: trace decompressor source setup and writes to $47..$49 during boot.
   Latest run: https://github.com/JimLundin/sm-decomp/actions/runs/39058480660.
   No baseline count exists and the full any% replay has not run.
2. **Headless verify mode.** Add `sm verify <rom> <inputs> [--frames N]`:
   no window, no audio, unthrottled, `RM_BOTH`. It feeds one recorded
   controller word per game frame through the same path as `RtlRunFrame`,
   counts failing frames, prints the first one with the differing RAM
   addresses (named via `ram_map`), and exits non-zero on any failure.
3. **Inputs.** Restore movie import (`.lsmv` and `.bk2` to one `u16` per
   frame) from `crates/movie` in the archive branch, as a module or a small
   `tools/` binary. The TAS will not stay in sync with snesrev's frame loop
   (lag frames differ), and that is fine: the comparison is still valid on
   whatever the inputs make the game do. Sync only affects coverage. Fix sync
   later if coverage stalls.
4. **Baseline.** Record the failing-frame count for the any%, low% and 100%
   movies on `main` in this file. Target: zero, apart from known snesrev
   mismatches, which get listed here with their frame and cause.
5. **Speed.** A full any% verify should finish in a few minutes. If it is
   slower, profile the snapshot and compare path before anything else.

## Phase 2: one Rust program instead of 52 translation units

Mechanical refactors, each verified with the phase 1 run. Order matters:
every step makes the next one easier.

1. Shared types: each file currently redeclares every C struct and typedef it
   uses. Move them into one `types.rs` and delete the copies.
   Preparation while phase 1 runs: the first mechanical batch can consolidate
   `LongPtr` (22 identical copies), `OamEnt` (7), and `VramWriteEntry` (18)
   into `sm/src/types.rs`, registered in `lib.rs`. Replace definitions with
   explicit `pub use crate::types::{...}` so existing public paths survive.
   This touches `rtl.rs` and game banks 80, 81, 82, 84, 86, 88, 8b, 8f, 90,
   91, 94, 9b, a0, a2, a3, a4, a5, a6, a7, a8, a9, b2 (only the types each
   already defines). Keep all fields, `Copy`/`Clone`, and representation exact:
   `LongPtr` is `repr(C, packed)`, size 3/alignment 1; `OamEnt` is `repr(C)`,
   size 4/alignment 1; `VramWriteEntry` is `repr(C)`, size 8/alignment 2, with
   `src` at byte 2 and `vram_dst` at byte 6, including the existing padding.
   Its layout must not be changed to packed despite the SNES queue's 7-byte
   stride. Shared fields need only `uint8 = u8`, `uint16 = u16`, and
   `VoidP = uint16`; leave other local integer and function-pointer aliases
   for a separate batch, preserving their current widths and C ABI.
   After normalizing line endings, duplicate named structs/unions match
   except `Apu`'s numbered anonymous-union dependency and `C2RustUnnamed`:
   that name denotes an APU history union in hardware/runtime modules but
   a packed menu struct in bank 82. Do not consolidate by name alone; resolve
   these dependencies explicitly in a later hardware-type batch. The first
   batch is prepared only; apply and commit it after the baseline verifies.
2. Replace `extern "C" { fn Foo(); }` declarations with `use` of the real
   function. Then drop `#[no_mangle]` and `extern "C"` from Rust-to-Rust
   functions. Keep the C ABI only at the SDL boundary.
   After shared types, pilot the three `snes/input.rs` exports and their sole
   consumer, `snes/snes.rs`; then `game/sm_92.rs` (four exports, consumers in
   `sm_90.rs` and `sm_9b.rs`, no function-pointer tables). Replace declarations
   with imports before removing each export's ABI and `#[no_mangle]`.
   Convert callback aliases, tables and casts together. `main.rs` is a separate
   binary crate: its `Die`, `Warning` and audio-lock hooks cannot be imported
   by the library until ownership moves or explicit callbacks replace them.
   Keep real libc, math, SDL and OpenGL FFI until their replacements exist.
3. Replace the hand-written SDL/OpenGL FFI with the `sdl2` crate (or similar)
   and drop `extern_types`; then move `rust-toolchain.toml` to stable.
4. Globals: gather `static mut` game state behind one `Game` struct passed by
   `&mut`, keeping `g_ram` as the 128 KiB array it is (RAM layout must stay
   byte-identical for the oracle comparison and for save states).

## Phase 3: readable gameplay, one subsystem at a time

Pick a subsystem (Samus movement, an enemy, PLMs, doors, HUD...), make it
idiomatic, run verify, commit. Typical moves:

- Raw `g_ram` offsets → named accessors generated from `ram_map.rs`
  (`ram.samus_x_pos()`, `ram.enemy(k).y_pos`), still backed by `g_ram`.
- `as ::core::ffi::c_int` chains → the actual unsigned/signed 8/16-bit
  arithmetic, with `wrapping_*` where the SNES wraps.
- Function pointer tables and `switch` dispatch → `match` and enums.
- Remove `unsafe` once a function no longer touches raw pointers.

The hand-written Rust in the archive (`restart/`, `crates/port`,
`translations/`) covers projectiles, scrolling, doors, camera, item pickups,
Rio, Waver, Ripper and nine more enemies. Mine it for names and structure
when cleaning the same subsystem. Do not merge it wholesale; the transpiled
code is the source of truth.

## Phase 4: finish ownership

- `sm_a3.rs` still calls the emulated CPU for Bang's AI (`Call(...)`).
  Reimplement it in Rust; then the game runs without the 65816 except as the
  oracle.
- Review snesrev's bug-fix patches (`kPatchedCarrys`, `FixBugHook` in
  `cpu_infra.rs`) and record which ones change behaviour versus the ROM.
- Then Map Rando: its patches are ROM changes, so the oracle still works as
  long as the patched ROM runs on the emulated side.

## Progress

`sh tools/progress.sh` from the repo root. Baseline at transpile time:

| metric | count |
| --- | ---: |
| raw g_ram offsets | 23,718 |
| extern "C" blocks | 52 |
| extern "C" fns | 5,842 |
| #[no_mangle] | 6,808 |
| unsafe fns | 6,886 |
| c_int casts | 90,021 |
| lines of Rust | 278,936 |

Phase 1 adds: failing frames per movie (target 0).

Headless verification adds one unsafe diagnostic function and runner source
lines; gameplay residue counters are unchanged. Baseline blocked by the
frame-0 crash described above. Any%, low% and 100% inputs have been imported
locally (129,712 / 167,797 / 222,788 frames) and checked against manifest
hashes and ROM identity; all movie files remain ignored.
