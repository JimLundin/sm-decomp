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

1. **Run it once — done.** Put the ROM in `rom/sm.smc`, run
   `cargo run --release -- rom/sm.smc` from the repo root, play a minute and
   confirm no `Verify failure!` lines. Linux or WSL first. If anything
   differs from what snesrev's C build did, that is a transpile bug: fix it
   before anything else.
   Interactive play and WSL are unavailable on the current Windows machine;
   Cargo's build script is denied execution (OS error 5). Use the automated
   3,600-frame smoke run followed by any% in Linux GitHub Actions instead.
   Defender event 1121 confirms IT-managed ASR rule
   `01443614-CD74-433A-B99E-2ECDC07BFC25` blocks Cargo's generated
   `build_script_build.exe`. Rust compilation works; execution of the new
   executable is denied. `sm/build.rs` only prints linker directives and
   launches no helper. The user confirms DLL loading through Python is
   permitted; the verifier now supports this locally without a build-script
   executable or SDL. From the repo root:
   `python tools/verify_dll.py rom/sm.smc movies/any.inputs`.
   Python is installed at
   `C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/python-3.13.7/python.exe`
   but is not on PATH. The script uses the pinned Rust compiler and bundled
   lld directly, then loads the DLL with ctypes. Build once; add `--no-build`
   for parallel replays in separate Python processes (one replay per process).
   Do not rebuild the DLL while another process has it loaded.
   The private archive supplied the ROM, restored locally to gitignored
   `rom/sm.smc`; expected SHA-1 confirmed. Private Actions could not start
   because GitHub reported a billing/spending limit. The archive was restored
   to archived status. Manual `.github/workflows/verify.yml` runs publicly
   using gzip/base64 ROM chunks in encrypted Actions secrets. Only comparison
   diagnostics are printed; no ROM, save state or image is uploaded.
   The previous session stopped after one hour at an invalid LoROM read
   `$96:2000` during frame-0 decompression. Resumed diagnosis found incorrect
   operand evaluation order in the CPU's BRA/BRL: their targets used PC before
   reading the displacement. Reading it first fixes boot. Next, file-select
   BG2 copied 1,024 words through a 1,024-word pause-map field at index 768;
   the ROM writes `$7E:3600..3DFF`. Using that backing RAM range fixes the
   bounds panic without changing RAM layout.
   The first completed smoke replay then reported 2,727 failing frames out
   of 3,600, first at frame 200: RAM `$7E:00D6` and VRAM. The ROM's VRAM queue
   stores its destination at entry+5 with a 7-byte stride; `repr(C)` put it at
   +6. The 18 duplicate `VramWriteEntry` definitions now use `repr(C, packed)`.
   After these fixes, the 3,600-frame smoke passes with zero mismatches.
   Full any%, low%, and 100% run concurrently with the smoke in independent
   Actions matrix jobs (`fail-fast: false`), per the user's parallelism request.
   Full any% then reached 90,000 frames with zero mismatches before further
   menu-overlay bounds panics in file-copy confirmation and file clear.
   All three row-clear writes now use the full backing menu tilemap.
   Local full any% now passes: 129,712 frames, zero failures, 181.67 seconds.
   Linux full any% also passes: 129,712 frames, zero failures, 216.62 seconds.
   Final-source smoke and any% passed; low% and 100% repeats are still running:
   https://github.com/JimLundin/sm-decomp/actions/runs/39060756294.
2. **Headless verify mode — done.** Add `sm verify <rom> <inputs> [--frames N]`:
   no window, no audio, unthrottled, `RM_BOTH`. It feeds one recorded
   controller word per game frame through the same path as `RtlRunFrame`,
   counts failing frames, prints the first one with the differing RAM
   addresses (named via `ram_map`), and exits non-zero on any failure.
   Implemented in `sm/src/verify.rs`, also shared by the Windows DLL host.
   Local DLL smoke: 3,600 frames, zero mismatches, 4.69 seconds.
3. **Inputs — done.** Restore movie import (`.lsmv` and `.bk2` to one `u16` per
   frame) from `crates/movie` in the archive branch, as a module or a small
   `tools/` binary. The TAS will not stay in sync with snesrev's frame loop
   (lag frames differ), and that is fine: the comparison is still valid on
   whatever the inputs make the game do. Sync only affects coverage. Fix sync
   later if coverage stalls.
   Implemented in `tools/movie_inputs.py`; all three movies imported.
4. **Baseline — done.** Record the failing-frame count for the any%, low% and 100%
   movies on `main` in this file. Target: zero, apart from known snesrev
   mismatches, which get listed here with their frame and cause.
   Baselines on main (2026-10-17):

   | movie | frames | failing frames | local DLL time |
   | --- | ---: | ---: | ---: |
   | any% | 129,712 | 0 | 181.67s |
   | low% | 167,797 | 0 | 240.59s |
   | 100% | 222,788 | 0 | 319.10s |

   Any% uses `f2569067` (Linux and local); low% and 100% used the DLL at `554e5727` before
   the final any%-triggered file-clear bounds fix. Linux also confirmed low%
   and 100% at `21c8a165` (618.18s / 373.15s). The final Actions matrix above
   reruns all three on `f2569067`; its full any% passed in 216.62s.
   No known mismatches in these replays.
5. **Speed — done.** A full any% verify should finish in a few minutes. If it is
   slower, profile the snapshot and compare path before anything else.
   Local full any% takes about three minutes; all movies run concurrently.

## Phase 2: one Rust program instead of 52 translation units

Mechanical refactors, each verified with the phase 1 run. Order matters:
every step makes the next one easier.

1. **Shared types — done.** Repeated C structs and typedefs now have one
   definition in `types.rs`; copies are public re-exports.
   Preparation while phase 1 runs: the first mechanical batch can consolidate
   `LongPtr` (22 identical copies), `OamEnt` (7), and `VramWriteEntry` (18)
   into `sm/src/types.rs`, registered in `lib.rs`. Replace definitions with
   explicit `pub use crate::types::{...}` so existing public paths survive.
   First batch applied and verified: any% 129,712 frames, zero failures
   (180.81s). Layout assertions and normal library/frontend metadata builds
   pass; local Cargo linking remains blocked by the documented ASR policy.
   Rust source lines fell from 279,256 to 279,020; other counters unchanged.
   Alias batch applied: 1,113 local typedefs now re-export shared definitions,
   preserving resolved widths, signedness, pointers and C callback ABIs.
   Any% passes 129,712 frames with zero failures (182.10s); format and direct
   library/frontend metadata builds pass. Rust lines now 278,552; other
   counters unchanged. Local Cargo build still encounters the ASR block.
   Gameplay data batch applied: 205 repeated struct/union definitions and
   three collision-result aliases consolidated without layout changes.
   Any%: 129,712 frames, zero failures; format and normal library/frontend
   metadata builds pass. Rust lines now 275,862; other counters unchanged.
   Hardware/music batch applied: 229 repeated definitions now share 22
   types, including `ApuHistory` (retaining original local public aliases).
   Any%: 129,712 frames, zero failures; layout audit, format and normal
   library/frontend metadata builds pass. Rust lines now 271,490; other
   counters unchanged. Windows frontend linking additionally needs the
   existing DLL host's CRT adaptations (`stderr`, `__assert_fail`); defer
   that distinct platform fix to item 3.
   Runtime/platform batch applied: 30 repeated structs, seven `FILE` aliases
   and four opaque dependencies now share definitions. Any%: 129,712 frames,
   zero failures (182.87s); layout audit, format and library/frontend metadata
   builds pass. Rust lines now 271,083. Extern blocks rise 53 -> 54 because
   shared opaque types need a declaration block; existing FFI blocks remain.
   Source consolidation complete: final inventory finds zero repeated named
   structs/unions/typedefs, opaque declarations or anonymous enum concepts.
   Eight repeated anonymous aliases now re-export `KeyAction`, `OutputMethod`,
   `GamepadButton` and `SaveLoadOperation`, preserving all constants and
   signedness. Unique subsystem types and distinct anonymous concepts stay
   local. Final any%: 129,712 frames, zero failures (187.25s); format and
   library/frontend metadata builds pass. Rust lines 271,083 -> 271,090:
   seven extra lines retain old public paths alongside four canonical names.
   All other counters unchanged. Linux Cargo build passed on the preceding
   runtime batch. Final-source Linux format/release build and all four
   replays passed on `d6097771`: any% 129,712 frames, zero failures (462.85s).
   https://github.com/JimLundin/sm-decomp/actions/runs/39065322648.
   This touches `rtl.rs` and game banks 80, 81, 82, 84, 86, 88, 8b, 8f, 90,
   91, 94, 9b, a0, a2, a3, a4, a5, a6, a7, a8, a9, b2 (only the types each
   already defines). Keep all fields, `Copy`/`Clone`, and representation exact:
   `LongPtr` is `repr(C, packed)`, size 3/alignment 1; `OamEnt` is `repr(C)`,
   size 4/alignment 1; `VramWriteEntry` is `repr(C, packed)`, size 7/alignment 1,
   with `src` at byte 2 and `vram_dst` at byte 5, matching the oracle's queue.
   Shared fields need only `uint8 = u8`, `uint16 = u16`, and
   `VoidP = uint16`; leave other local integer and function-pointer aliases
   for a separate batch, preserving their current widths and C ABI.
   After normalizing line endings, duplicate named structs/unions match
   except `Apu`'s numbered anonymous-union dependency and `C2RustUnnamed`:
   that name denotes an APU history union in hardware/runtime modules but
   a packed menu struct in bank 82. Do not consolidate by name alone; resolve
   these dependencies explicitly in a later hardware-type batch. Remaining
   batches: aliases, gameplay data, hardware/music closure, runtime/platform
   types, then a duplicate inventory before marking this item done.
2. **Rust linkage — done.** Replace `extern "C" { fn Foo(); }` declarations with `use` of the real
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
   Input pilot applied: `input_init`, `input_free`, `input_reset` use ordinary
   Rust imports/ABI. Any% 129,712 frames, zero failures; format and normal
   library/frontend metadata builds pass. C exports/no_mangle each fall by
   three, Rust lines 271,090 -> 271,086; other counters unchanged.
   Bank 92 pilot applied: four exports use Rust imports/ABI in banks 90/9b.
   Any% 129,712 frames, zero failures (181.35s); format, unchanged-body audit
   and library/frontend metadata builds pass. C exports/no_mangle each fall
   by four, Rust lines 271,086 -> 271,083; other counters unchanged.
   Host ownership seam applied: five formerly binary-owned globals live in
   the library; frontend/DLL explicitly register Rust error/warning/audio-lock
   callbacks before startup. Original callback bodies/defaults remain exact.
   Any% 129,712 frames, zero failures (181.59s); format and library/frontend
   metadata builds pass. C exports -6, no_mangle -18. Unsafe functions +6 and
   Rust lines +33 are callback dispatch/registration required to remove the
   binary symbol hooks, not gameplay changes; other counters unchanged.
   Import batch applied: 2,437 internal function/static declarations replaced
   by direct Rust imports; zero signature conflicts. Bodies and 210 true FFI
   declarations unchanged. Any% 129,712 frames, zero failures (182.99s);
   format and library/frontend metadata builds pass. Extern blocks 54 -> 40,
   Rust lines 271,116 -> 269,432; other counters unchanged.
   ABI batch applied: 6,870 Rust function definitions and 1,587 callback/type
   occurrences now use Rust ABI; 6,790 unnecessary `no_mangle` removed.
   Callback tables/casts converted together, three callback transmutes now
   direct calls. Real SDL/OpenGL/libc/DLL boundaries remain C ABI; unused
   variadic `StrFmt` remains until stable cleanup. Any% 129,712 frames, zero
   failures (186.61s); format and library/frontend metadata builds pass.
   C ABI export counter 5,833 -> 3; no_mangle 6,795 -> 5; Rust lines 269,432
   -> 261,547; other counters unchanged. Final-source Linux validation
   passed on `bd036cd8`: format, release build, smoke and full any% all pass.
   https://github.com/JimLundin/sm-decomp/actions/runs/39066700833.
3. **Crate bindings and stable — done.** Handwritten SDL/OpenGL FFI replaced
   with crate bindings; `extern_types` dropped and toolchain moved to stable.
   SDL batch applied: handwritten functions/events now use `sdl2-sys` 0.38;
   integer config domains retain exact values, SDL enums adapted at boundary.
   Optional default `frontend` feature keeps the direct core DLL independent
   of SDL dependencies. Any% 129,712 frames, zero failures (190.18s); format
   and real-binding library/frontend metadata builds pass. Source lines
   261,547 -> 261,137; c_int casts 90,009 -> 90,001; other counters unchanged.
   Local Cargo dependency build scripts also meet the managed ASR block.
   OpenGL batch applied: renderer/shader use `gl` 0.14 calls and binding
   constants; handwritten loader module removed. All 121 call sites preserve
   function selection/arguments; core 3.3 requirement checked through GL
   version integers. Any% 129,712 frames, zero failures (184.90s); format and
   real-binding library/frontend metadata builds pass. Extern blocks 40 ->39,
   unsafe functions 6,901 ->6,863, c_int casts 90,001 ->89,831, Rust lines
   261,137 ->249,227; remaining counters unchanged.
   Stable cleanup applied: opaque pointer targets are stable Rust types;
   unused variadic `StrFmt` removed, feature attributes gone. Toolchain now
   stable; DLL linker uses shipped `ld.lld` with supported options (no `-Z`).
   Any% 129,712 frames, zero failures (184.76s), using permitted pinned
   compiler locally. Metadata builds pass without nightly features. Actual
   stable cargo/compiler execution is also denied on this Windows host;
   Linux must validate stable. Blocks -1, exports/no_mangle/unsafe/c_int
   each -1, Rust lines 249,227 ->249,179; other counters unchanged.
   Windows frontend CRT fix applied separately: initialize `stderr` through
   MinGW's `__acrt_iob_func`, supply an assertion diagnostic/abort bridge.
   Real-dependency optimized Windows frontend linking now passes. Any%
   129,712 frames, zero failures (188.07s); format passes with permitted
   compiler. Blocks +1, C export/unsafe +1, no_mangle +2, lines +34 are
   required Windows CRT compatibility symbols; gameplay counters unchanged.
   Linux stable format/release build, smoke and full any% passed on
   `aaa80120`; no failing frames. Final Windows-only CRT source rerun:
   https://github.com/JimLundin/sm-decomp/actions/runs/39067855970.
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
lines; gameplay residue counters are unchanged. Movie baselines are zero.
Any%, low% and 100% inputs have been imported
locally (129,712 / 167,797 / 222,788 frames) and checked against manifest
hashes and ROM identity; all movie files remain ignored.
The DLL host adds one extern block, four counted C ABI exports, twelve
`no_mangle` symbols, eight unsafe functions and source lines. These are host
glue for the same oracle, not additional gameplay residue. The phase-1 fixes
reduce c_int casts from 90,021 to 90,009; all raw RAM offset counts stay fixed.
