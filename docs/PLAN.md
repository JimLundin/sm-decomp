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

## Workspace discipline

Work directly on `main`, one focused, verified commit at a time. Finish with
a clean working tree; do not leave speculative branches, worktrees, stashes
or uncommitted edits. Create a working branch only when the user requests it,
and remove it when its work is integrated or explicitly retired.

Keep the C implementation out of the checkout, including ignored copies and
old build directories. `sm/src` is the maintained implementation; the
transpile notes are historical provenance, not a regeneration workflow.
Keep only current build outputs in `target/` and the ignored ROM/movie inputs
needed for verification. Remove obsolete scratch and restart artifacts.

Workspace audit (2026-10-17): `main` was clean and synchronized with origin;
no C game source, extra worktrees or stashes were present. The retired local
`restart/playable-baseline` branch had history outside the pre-transpile
archive, so its tip is preserved as the tag
`archive/restart-playable-baseline-2026-10-17` before removing the branch.
Old scratch/restart artifacts, draft refactors and dependency source caches
were moved out of the checkout into a temporary directory (recursive deletion
was rejected by automatic approval review). Only the current verifier DLL
outputs remain in `target/`; there are zero C-family source files in the
checkout. The documented archive branch stays intact. This audit changes no
Rust source; residue counters are unchanged.

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
4. **Globals: done.** Gather
   `static mut` game state behind one `Game` struct passed by
   `&mut`, keeping `g_ram` as the 128 KiB array it is (RAM layout must stay
   byte-identical for the oracle comparison and for save states).
   Immutable-table precursor applied: 755 read-only gameplay statics now
   immutable `static`, preserving storage identity, types and initializers.
   Eight mutable scratch fields and the outer demo pointer table excluded.
   Any% 129,712 frames, zero failures (185.67s); format and frontend/core
   metadata builds pass. Progress counters unchanged (they count unsafe
   functions/RAM offsets, not mutable statics). Stable local Cargo blocked.
   Demo-table precursor applied: outer pointer table is immutable static
   slice references to unchanged child arrays; `.as_ptr()` retains the
   existing unchecked entry offset. Any% 129,712 frames, zero failures;
   format and library/frontend metadata builds pass. Rust lines -1; other
   progress counters unchanged.
   Explicit ownership applied: all 49 mutable gameplay/runtime/oracle fields
   now live in heap-allocated `game_state::Game`, passed through 6,294 Rust
   functions and their callback tables by `&mut`. `g_ram` remains exactly
   `[u8; 131072]`; hardware and save-state operations retain backing pointers
   and byte ranges. The frontend supplies a mutex-protected `GameOwner` via
   SDL audio userdata, borrows only around individual operations, and closes
   the audio device before releasing it. Existing audio-lock hooks remain.
   Any% completes 129,712 frames with zero failures; final format and real
   frontend/core metadata builds pass. Local Cargo remains denied by ASR;
   Linux stable format/release build and all four replays passed on
   `73579a81` (any% 129,712 frames, zero failures, 460.59s):
   https://github.com/JimLundin/sm-decomp/actions/runs/39069476088.
   Raw RAM-offset lines 23,718 -> 23,710; no_mangle 6 -> 5; other ABI/unsafe
   counters unchanged. c_int cast lines 89,830 -> 89,884 and Rust lines
   249,212 -> 259,878 because explicit context expands calls and rustfmt
   wraps existing expressions. No casts or gameplay algorithms were added.
   Remaining mutable core statics are read-only hardware/music/debug tables,
   now immutable in the following separate mechanical change.
   Core-table batch: 35 read-only hardware/music/debug tables made immutable;
   pointer tables use a transparent wrapper retaining layout and storage
   identity. Initializers and all function bodies audited unchanged except
   local `static mut` declarations. No mutable core state remains outside
   `Game` (CRT `stderr` declarations are external FFI); frontend/config/image
   loader state stays at the platform boundary. Format and core/frontend
   metadata builds pass. Unsafe functions +1 and Rust lines +31 provide the
   immutable string-pointer constructor and its documented safety contract;
   all other residue counters unchanged. Final-source any%: 129,712 frames,
   zero failures (183.28s). Linux stable format/release build, smoke and
   full any% passed on `4f305830` (129712 frames: 0 failing frames (463.11s)).
   Supplemental full 100% is still running:
   https://github.com/JimLundin/sm-decomp/actions/runs/39070035812.

## Phase 3: readable gameplay, one subsystem at a time

First subsystem: Waver (bank A3). Its four movement/animation functions now
use explicit 8/16-bit wrapping arithmetic and direct ROM access. Preserve
the original `SignExtend8(0x8000)` initialization and the middle-byte velocity
reversal; every possible middle word (65,536) produces identical reversal
bytes. Any%: 129,712 frames, zero failures (184.53s); format and core/frontend
metadata builds pass. Linux stable format/release build, smoke and full
any% pass on `136f3f19` (129712 frames: 0 failing frames (225.05s)).
https://github.com/JimLundin/sm-decomp/actions/runs/39070442060. Raw RAM offsets -6, c_int cast lines -34, Rust lines -41;
all other counters unchanged. Next: name Waver's RAM fields and accessors,
then remove its raw-pointer access without holding RAM borrows across calls.
Waver RAM view applied: fields named for velocity, animation and sine phase;
reads/writes use little-endian bytes and offsets from the unchanged struct
layout. Current enemy index has a named accessor matching ram_map $0E54.
No typed RAM pointers remain in Waver; borrows end before movement/dispatch
calls, preserving the original slot even when the current index changes.
The instruction callback is safe. Layout audit confirms size 64 and all
field offsets; format, core/frontend metadata and full any% pass (129,712
frames, zero failures, 186.03s). Unsafe functions -2, raw RAM offsets -1,
c_int cast lines -1; Rust lines +102 provide safe field accessors and explicit
borrow scopes. Linux stable validation is pending on this commit.
Next: move Waver into a readable subsystem module and finish its ROM-read
boundary, then choose the next enemy/subsystem.
Waver module move applied: `game/waver.rs` owns its slot layout, RAM view
and four entry points; bank A3 re-exports preserve all caller paths. The ROM
animation-word read is isolated with its loaded-allocation precondition;
no ROM-derived table is embedded. Mechanical audit confirms unchanged
layout, RAM view and entry bodies except extraction of that read. Format,
core DLL build and full any% pass (129,712 frames, zero failures, 185.11s).
Rust lines +22 and unsafe functions +1 document/isolate the ROM boundary;
all other counters unchanged. Local Cargo remains ASR-denied. The prior
temporary SDL metadata dependencies disappeared and the standard DLL output
was denied overwrite; a fresh DLL output linked/loaded and verified this
source. Linux stable format/release build, smoke and full any% passed on
`876c21e8` (129712 frames: 0 failing frames (420.96s)):
https://github.com/JimLundin/sm-decomp/actions/runs/39071332693.
Next: make Waver's movement/ROM dependencies an explicit safe boundary and
choose the next readable gameplay subsystem.
Checked ROM boundary applied: successful cartridge load registers a private
pointer/length view in Game. The safe LoROM word reader checks allocation
bounds while preserving linear table offsets and observes the live patched
ROM. Registration documents lifetime/serialization requirements; reload and
free remain unsafe legacy operations. Waver initialization and animation
update are now safe Rust. Waver_Main remains unsafe because shared movement
and sine helpers still require valid legacy runtime state; no safe wrapper
conceals that precondition. All 65,536 animation indexes retain identical
mapped offsets. Optimized DLL build, format and any% pass: 129,712 frames,
zero failures (181.81s). Unsafe functions -2; Rust lines +33 establish the
registered allocation and checked reader; remaining counters unchanged.
Local Cargo remains ASR-denied; Linux stable frontend validation is pending.
Next: deepen the shared enemy movement/sine boundary starting with the sine
helper, then convert collision helpers without changing quirks.
Shared sine/cosine helpers converted: all five helpers are safe Rust, using
checked live-ROM byte reads, 8-bit amplitude truncation, phase wrapping and
independent integer/fraction negation (no carry between halves). Exhaustive
131,072 sample/amplitude/sign combinations and 131,072 phase shifts agree
with the original arithmetic. Format, optimized DLL build and any% pass:
129,712 frames, zero failures (183.97s). Unsafe functions -5, c_int cast
lines -17, Rust lines -6; other counters unchanged. Local Cargo remains
ASR-denied; Linux stable build/any% validation is pending on this commit.
Next: convert horizontal/vertical enemy collision arithmetic, retaining live
enemy fields after callbacks and the original wrapping edge calculations.
Horizontal/vertical collision arithmetic converted: explicit wrapping 16/32-bit
edge, span, block-offset and boundary calculations replace C cast chains.
Room width uses its ram_map name/accessor. Original callback order and live
reads remain: horizontal success applies callback-adjusted displacement to
live coordinates; vertical success stores its precomputed position. Boundary
and randomized checks cover 100,100 coordinate/size pairs and 1,001,000 block
offsets. Format, optimized DLL build and full any% pass: 129,712 frames, zero
failures (184.60s). Raw RAM offsets -3, c_int cast lines -64, Rust lines -29;
other counters unchanged. Local Cargo remains ASR-denied; Linux stable
frontend validation is pending on this commit.
Next: replace typed enemy coordinate pointers with byte-backed access, then
convert collision dispatch/callbacks without holding RAM borrows across them.
Byte-backed enemy coordinates applied: horizontal/vertical movement reads
and writes position/subposition/size through `game/enemy_ram.rs`, using
EnemyData offsets and little-endian bytes. Each borrow ends before collision
callbacks; reads after them retain the original slot and observe live state.
Horizontal success performs a wrapping 16.16 addition and writes subposition
before position as before. Vertical success retains its precomputed position.
Layout audit confirms offsets 2/4/6/8/10/12; optimized DLL build, format and
any% pass: 129,712 frames, zero failures (184.15s). Rust lines +82 implement
safe coordinate access and explicit live reads; other counters unchanged.
Local Cargo remains ASR-denied; Linux stable validation is pending.
Next: convert collision type dispatch and callback context to Rust references
and named RAM access, then remove unsafe from movement as dependencies permit.
Collision dispatch/context applied: horizontal/vertical tables are direct
matches on the live level-data block type. All 16 selections in both tables
match the originals, including extension handlers and high-bit retries.
Collision context and its handlers use `&mut EnemyBlockCollInfo`; raw context
pointers are gone. Named cur_block_index and live level-data byte reads retain
RAM layout. Format, optimized DLL build and any% pass: 129,712 frames, zero
failures (182.93s). Raw offsets -6, unsafe functions -2, c_int cast lines -17,
Rust lines -48; other counters unchanged. Local Cargo remains ASR-denied;
Linux stable frontend validation is pending on this commit.
Next: convert extension/spike handler RAM/ROM reads, then slope handlers,
so the shared movement boundary can become safe.
Extension/spike reads converted: BTS uses the live named byte accessor;
spike PLMs use checked live-ROM words. Extension offsets preserve signed
bytes, full room width, wrapping block indexes and high-bit retry results.
All 16,777,216 byte/width products match the original arithmetic. Format,
optimized DLL build and full any% pass: 129,712 frames, zero failures
(185.28s). Raw offsets -11, unsafe functions -2, c_int cast lines -20,
Rust lines -35; remaining counters unchanged. Local Cargo remains ASR-denied.
Supplemental low% and 100% run concurrently against the same built DLL;
Linux stable validation is pending. Next: slope handlers. In parallel,
inspect actual replay progression: zero mismatches alone does not establish
that the movies reach the intended gameplay paths.
Coverage inspection found a real gap: the imported any% movie visits only
the six Ceres rooms (DF45, DF8D, DFD7, E021, E06B, E0B5), with enemy
definitions E13F/E1FF/E23F. It never reaches Zebes or Waver (D63F).
Its full replay still passes 129,712 frames with zero failures (248.94s
under parallel load). The verify run now prints gameplay-frame, room and
enemy-presence counts, final state/room, state frequencies and longest
unchanged state/room interval, with coverage counts at existing progress
checkpoints. These are presence measurements, not function-call coverage.
Format and optimized verifier-host build pass; final-source 10,000-frame
smoke has zero failures. Rust lines +67 add this observation to the verify
run; other counters unchanged. Local Cargo remains ASR-denied.
Before continuing further gameplay cleanup, improve actual input coverage:
RtlRunFrame clears opposite directions used by these TASes, and its forced
game-yield/NMI timing differs from bsnes movie frame timing. First preserve
the recorded pads in verify mode, then measure reach and retime inputs if
needed. Repeating unchanged movies cannot establish broader coverage.
Square slope/dispatch conversion completed while coverage runs: named live
BTS access, byte-backed coordinates and explicit near/far quadrant branches
replace typed pointers and translated jumps. Horizontal dispatch still
zero-extends the full BTS word at $1E77. Independent comparison covers
245,760 edge/orientation/span combinations; both nonsquare bodies remain
unchanged. Format, optimized core/DLL build and any% pass: 129,712 frames,
zero failures (251.31s under parallel load), with the same six Ceres rooms.
Raw offsets -17, unsafe functions -2, c_int cast lines -93, Rust lines -133;
remaining counters unchanged. Local Cargo remains ASR-denied. Low%/100%
continue in parallel. Defer nonsquare work until input coverage improves.
All three square-source replays finished with zero failures: any% 129,712
(251.31s), low% 167,797 (326.24s), 100% 222,788 (417.91s), concurrently.
All visit the same six Ceres rooms; low%/100% see only E13F/E23F.
Verify now preserves simultaneous opposite-direction buttons through
RtlRunFrame instead of applying the UI input filter. All 4,096 pad values
retain the previous UI result and preserve the recorded verify bits.
Format and optimized core/DLL build pass; full any% passes 129,712 frames,
zero failures (205.42s). Rust lines +3 add the verify exception; other
counters unchanged. Local Cargo remains ASR-denied. Low%/100% run in
parallel. Coverage remains six Ceres rooms, so input timing still needs work.
Trace found first corridor entry at frame 11,627, when the original movie
already presses left and immediately returns to the shaft. Try a simple
controller sequence aligned to this frame loop before adding scheduling code.
Final exact-pad replays all pass, concurrently: any% 129,712/0 (205.42s),
low% 167,797/0 (263.94s), 100% 222,788/0 (340.77s). All remain in Ceres;
none reaches the successful escape states 32/33 or Waver.
Bounded input-only trials preserved the boot/shaft prefix to frame 11,627
and replaced corridor inputs with coarse run/jump/shoot patterns. They
pass the oracle but reach only three rooms, blocking at X219 in DFD7.
Longer and standing jumps and continuous firing did not clear that obstacle;
these trials do not expand coverage and are retired. Next: inspect the
DFD7 obstacle and retime the original corridor segment against room/Samus
coordinates. Do not resume broad gameplay cleanup on assumed TAS coverage.
Parallel work now separates replay processes, coverage inspection, slope
implementation and independent arithmetic review; source changes remain
one focused verified commit at a time. Latest Linux stable build/replays
are pending at https://github.com/JimLundin/sm-decomp/actions/runs/39074190882.
Optional verifier tracing added: set `SM_VERIFY_TRACE=1` for console output
on state/room transitions and every 100 gameplay frames, including Samus
position, pose and actual timer_status ($0943). It uses the existing CLI/DLL
path and writes no artifacts. Format, optimized host build, trace-enabled
600/10,000-frame checks and full any% pass: 129,712 frames, zero failures
(202.33s). Rust lines +15 add diagnostics; other counters unchanged. Local
Cargo remains ASR-denied; Linux validation is pending.
Live geometry resolves DFD7: it is a 256x512 shaft, with an upper-right
wall at X224 and the outbound exit lower right at Y352..415. Moving left
off the upper shelf then right reaches E021, E06B and Ridley E0B5 by frame
13,758. The measured return route reaches DFD7 at frame 17,578, with
timer_status 8006; the remaining step is ascending its shelves and then
the elevator shaft. A useful ignored controller input remains at
target/geometry-probe/boss-return.inputs. No wider gameplay coverage yet.
Boot investigation also confirms any% starts the intro before its later
menu inputs; the intended moonwalk setting is missed. Retiming only the
startup reaches five rooms and is insufficient. Align room transitions,
using the optional trace, rather than applying a constant movie offset.
Concurrent coverage trials now ascend DFD7 to X131/Y139 by frame 17,900:
turn right at frame 17,685 and hold jump through the apex; early release
truncates the ascent. Upper-exit/elevator trials are running against the
unchanged trace DLL. Six suffix-compression trials found no wider reach,
with zero oracle failures; no general scheduling correction is proven.
Temporary cleanup mistakenly removed concurrent input trials from the
shared target/geometry-probe directory. The supervisor reconstructed the
successful ledge input from its creation history; cleanup is stopped and
active trial files/DLLs are retained. Tracked source was unaffected.
Upper-exit trial 17865 now proves the return ascent: leaves DFD7 at frame
17,950 and enters DF8D gameplay at frame 18,084 (X472/Y139). Subsequent
right input sends it back into DFD7; the other two upper-exit trials also
finish there. All three pass 18,595/18,630/18,655 frames with zero failures.
Next controller correction: keep moving left through DF8D after frame
18,084, then climb DF45 and meet its elevator escape predicate. This pass
stops input tuning at the item's hour limit with a measured successful
DFD7 ascent; concurrent supervisor trials retain their files and continue
independently. Linux exact-pad format/release build and all four replays
passed on ede7dbcd: https://github.com/JimLundin/sm-decomp/actions/runs/39074190882.
Ceres escape coverage achieved with recorded controller inputs, without
changing gameplay: target/elevator-route/escape-20.inputs passes the
standard trace verifier for 20,989 frames, zero failures (29.31s). Escape
states 32/33/34 occur at frames 18,935/18,995/19,010; Zebes landing site
91F8 enters gameplay at frame 20,923. Coverage expands to seven rooms and
five enemy definitions (D07F/D0BF/E13F/E1FF/E23F); Waver is still absent.
The elevator route uses fresh jump presses after each landing, holds jump
through the apex, and approaches alternating shelves from their open side.
Successful shelf positions: X48/Y571, X127/Y475, X92/Y379, X98/Y267,
X72/Y171, then X118/Y75 triggers escape. Trial inputs remain ignored;
none is committed. Source/progress counters unchanged. Linux format,
release build and all four replays also pass on f4637ede:
https://github.com/JimLundin/sm-decomp/actions/runs/39075031032.
Next: extend the verified controller route from the Zebes landing site to
rooms containing Waver and then other gameplay, measuring actual reach.
Retain the useful inputs and active builds; independent workers use
separate target directories and never clean another worker's files.
Zebes route extended: target/zebes-route/bottom-right.inputs passes the
standard trace DLL for 25,350 frames, zero failures (35.20s). Ten rooms
are reached, including Parlor 92FD, old Mother Brain shaft 96BA and 975C;
six enemy definitions observed. Final position is X171/Y187 in 975C.
The ship landing sequence remains scripted after gameplay state 8 starts;
wait for control before moving west. Parlor descent follows its staggered
platforms; its bottom is a blue hatch, not an elevator. Jump then hold
Down+jump+shoot (04C0) to open it, then fall through. A fresh sideways jump
clears the arrival platform in 96BA; alternating movement descends the shaft.
Nearest Waver routes both need bombs: landing site's upper-left exit to
92B3 is sealed at X496..623/Y592..655; Parlor's upper-left exit to 990D
is sealed at X240..271/Y112..159. Their room graphs alone did not prove
physical accessibility. No gameplay or gate was changed to reach them.
Next normal route: 975C -> 97B5, fresh Down on the actual elevator ->
9E9F (Morph Ball X1104/Y656, equipment 0004), right through 9F11 ->
9F64 bottom-left arrival, first missiles X736/Y656. Return to Parlor's
middle-right exit -> 9879, open the red door with missiles -> 9804
(bombs X192/Y160, equipment 1000). Then return to a Waver room.
Extra ignored target/zebes-route/corridor-jump.inputs progresses to
X299/Y187 in 975C (25,755 frames, zero failures, probe host); validate
that continuation with the standard verifier next. This pass stops at
the item's hour limit. Source/progress counters unchanged; useful inputs
and build outputs retained, working tree clean after this plan update.
Optional replay trace now also reports equipped/collected item words and
current/maximum missiles, using ram_map $09A2/$09A4/$09C6/$09C8. This
allows pickup coverage to be proved rather than inferred from room entry.
Format and optimized verifier-host build pass; trace-enabled 10,000-frame
check and full any% pass: 129,712 frames, zero failures (193.75s).
Rust lines +4 add diagnostics; other counters unchanged. Local Cargo remains
ASR-denied; Linux validation is pending on this source change.
Morph Ball pickup coverage achieved: target/morph-route/collect-jump.inputs
passes the current standard verifier for 28,301 frames, zero failures
(39.77s). Twelve rooms and nine enemy definitions are reached. Equipped
and collected item bits both become 0004; the standard trace confirms
this by frame 27,600 and finishes at X1103/Y651 in 9E9F with the message
closed. Source/progress counters unchanged; controller inputs stay ignored.
The 97B5 elevator requires standing near X128 on its pad and a fresh Down
press. In 9E9F, the elevator arrival remains scripted before control resumes;
the Morph ledges need staged jumps. A right jump then left reversal lands
on X1269/Y619, a fresh left jump clears the main wall, and another fresh
jump reaches the pedestal. Contact, not shooting, collects the item.
Next: extend collect-jump.inputs right through 9F11 into 9F64 and collect
the first missiles at X736/Y656; then return toward the bomb room. Preserve
the verified route and use separate worker directories for further trials.
Parallel coverage work now has six independent assignments: first missiles,
return to Parlor, an alternate Crateria/Parlor branch, room-aligned TAS input
timing, bomb-route geometry, and Waver/boss access. Replay workers own
separate target directories and cap their trial processes at two each;
the parent validates useful prefixes with the standard oracle.
The Morph return passage is a normal shootable block at X1216/Y704
(BTS 04): crouch fire opens it, Morph rolls underneath, and a fresh jump
clears the floor lip. target/missile-route/door-hop.inputs now passes the
current standard verifier: 30,465 frames, zero failures (58.49s), thirteen
rooms, nine enemy definitions. Final room is 9F11; missiles remain 0/0.
Its first-missile exit is lower left, not upper right: shoot the shaft's
floor blocks and descend to the door at Y352..415 into 9F64. The upper-right
door is red and leads elsewhere. Return will require climbing this shaft.
Linux format/release build and all four replays pass on the current source
90889612: https://github.com/JimLundin/sm-decomp/actions/runs/39077183059.
Direct rustfmt checks pass; local Cargo is still denied by the managed
Windows policy. Progress counters are unchanged; source has not changed.
First-missile input tuning stops at the item's hour limit. Shooting down
through 9F11's floor reaches Y315; target/missile-route/hall-left.inputs
passes its probe for 31,430 frames, zero failures, ending X133/Y395 near
the lower-left exit. A full-height left jump lands too high, X85/Y315
(lower-door.inputs, 31,635/0). Next attempt needs a short left jump with
early release to clear the door lip without landing on the upper ledge.
No missile pickup is proved yet; active route inputs are retained.
Room-aligned original TAS trials also stop without wider coverage: landing
inputs at original 16038 reach Parlor but return; Parlor inputs 16458/16482
stall at X1077/Y171 even with aim taps removed and the first jump extended;
post-Morph inputs 18563/18700 leave the wrong side or stop at X1211/Y699.
All eight trials pass with zero failures (29,462..33,238 frames), but room
alignment alone does not reproduce the original movement setup. Useful
inputs and logs remain in target/movie-sync; further unchanged splices are
deferred. Independent normal-input room and return-route work continues.
Independent room coverage achieved: target/independent-coverage/map-room.inputs
branches from the Parlor descent with a fresh left jump at frame 22,708,
then left/shoot through its lower-left blue door. The standard verifier
passes 23,463 frames, zero failures (35.89s, independently repeated);
9A44 enters gameplay at frame 22,986, final X456/Y139. This replay visits
nine rooms and seven enemy definitions; the union with the verified Morph/
9F11 route is fourteen rooms and nine enemy definitions. Room entry is
proved; map-terminal activation is not yet proved. Inputs remain ignored.
Normal elevator return is also verified: target/return-parlor/elevator-center.inputs
passes the standard verifier for 31,606 frames, zero failures (68.86s),
returning to 97B5 with Morph equipped. The worker settles at X1405 and
uses a fresh Up press; arrival control resumes after the scripted movement.
Further west input reaches 975C's door at X37/Y139, but shooting has not
opened it. Read-only probe reports killed/quota 0/0 in this room state, so
an enemy-quota explanation is unproved. Door behavior needs inspection.
The nine-screen 96BA ascent is now a separate input worker using a known
bottom-floor prefix while the return worker resolves the west door.
The west-door inspection resolves that return obstacle: current 975C PLM
C848 has preinstruction BE1C (PlmPreInstr_PlayDudSound), not an enemy-quota
handler. Both fresh shots and stepping back before firing leave it closed;
quota is 0/0. Stop the pre-missile return attempt here. Reuse the verified
elevator suffix after first missiles, then measure the resulting room state
before trying this door again. No gameplay gate was changed.
The independent 9A44 branch also stops at a measured Morph requirement:
standing Samus is blocked at X437/Y139 by the X400..431 column; its lower
passage is only 16 pixels high. map-contact.inputs passes 24,278 frames,
zero failures (37.60s), without another room. Continue this branch after
a normal Morph return route exists. No map-terminal pickup was observed.
Independent 96BA ascent preparation verifies its first ledge:
target/mother-shaft-ascent/center-right.inputs passes the standard verifier
for 25,058 frames, zero failures (35.35s, independently repeated), ending
standing at X411/Y2131. It extends the known bottom-floor prefix at X396/
Y2187 with neutral 5, jump 25, right/jump 12, left/jump 6, jump/shoot 55,
neutral 15. The second hop returns to this ledge at X422; next attempt
needs more right travel to reach the X432/Y2080 shelf. Stop this preparation
pass here and reuse its suffix after the first-missile return is available.
All coverage workers in this pass have stopped with their useful ignored
inputs retained. No gameplay source changed, and progress counters stay
unchanged. Next focused item remains the short left jump from 9F11
X133/Y395 into 9F64, followed by a proved first-missile pickup.
Next coverage pass corrects that route using actual replay: 9F11's lower
left leads to A107, the first-missile Chozo room; its upper right leads to
9F64. Earlier door audits inferred source placement from list order, but
DoorDef coordinates describe the destination. Match source doors using
live level BTS and confirm with room transitions before relying on a graph.
The lower passage needs Morph, not a short jump: two fresh Down presses
at X133/Y395 yield ball center Y409 and roll beneath the Y352..399 wall.
Unmorph left of the column and shoot the blue cap to enter A107.
target/missile-pickup/door-shoot.inputs passes the current standard oracle:
31,814 frames, zero failures (46.55s), fourteen rooms/nine enemy definitions.
Together with the independent 9A44 route, coverage is fifteen rooms.
A107 contains missile-orb PLM EF2F at X64/Y112; shoot and then contact it.
Pickup is not yet verified. No source/progress counter changes.
The awake 975C return state is now decoded: Morph plus nonzero missile
capacity selects state 9787; its gray door then uses enemy-quota handler
BE01 and requires five kills (four walking pirates, one wall pirate).
Each has 20 HP and takes two basic-beam hits. Preserve missiles for bombs.
Independent shaft ascent also advances: shelf3-left.inputs under
target/mother-shaft-ascent2 passes the standard oracle for 25,313 frames,
zero failures (36.74s), ending standing at X402/Y1955 in 96BA. Its reusable
fresh jumps extend the previously verified first-ledge prefix.
First-missile pickup now verified: target/missile-pickup/a107-left.inputs
passes the standard oracle for 32,591 frames, zero failures (47.97s).
Current and maximum missiles are both 5/5, Morph equipped/collected 0004,
and the message is closed; final A107 position X85/Y139, gameplay state 8.
The successful continuation from frame 31,780 holds left/shoot 90,
neutral 600, then a fresh jump press and neutral 120. The ordinary beam
opens the Chozo orb and proximity collects its exposed item.
The next normal route is back through A107's right exit to 9F11 lower-left,
Morph right beneath the same wall, then staged jumps up the shaft to its
upper-left Morph-room exit. Earlier claims that this return bypasses the
shaft climb described the wrong room and are superseded by this replay.
Keep all five missiles for the bomb-room red door. The return and awake
pirate quota are the next coverage item; shaft preparation continues in
parallel. No gameplay source changed; residue counters remain unchanged.
Large-shaft ascent is now reusable across its repeated geometry:
target/mother-shaft-ascent2/repeat-exact-run.inputs passes the standard
oracle for 28,638 frames, zero failures (41.66s), ending X401/Y419 in 96BA.
The 559-frame suffix from shelf7-right.inputs (X401/Y1699) to
shelf11-right32.inputs (X401/Y1443) repeats four more times exactly,
covering the remaining 256-pixel cycles. The unique upper shelves and
ceiling hatch still need input verification, then the suffix can be
grafted onto the actual post-missile return. Progress counters unchanged.
Full nine-screen shaft ascent now verified: target/mother-shaft-ascent2/
parlor-hatch.inputs passes the standard oracle for 29,687 frames, zero
failures (41.80s), ending in Parlor 92FD at X370/Y1211, standing.
The distinct upper shelves reach X334/Y379, X383/Y267, X335/Y211,
X327/Y139, then the central island X370/Y91. Up/shoot 60, neutral 5,
Up/jump/shoot 80 and neutral 200 open and cross the ceiling hatch.
The reusable ascent-from-396-2187.inputs suffix has 4,747 frames; its
original mother-drop-left.inputs prefix is exactly 24,940 frames.
Normalize the actual post-missile arrival to standing X396/Y2187 before
appending it, then verify the combined route. This proof uses a separate
pre-Morph prefix and does not yet prove the full return with items.
The post-missile connector climb has reached X119/Y235 in 9F11:
center-edge-hold.inputs under target/post-missile-return passes the standard
oracle for 33,336 frames, zero failures (47.17s), with missiles still 5/5.
Upper shootable blocks constrain the next jump; the worker is inspecting
live block changes and testing a neutral jump through the shot column
before steering right. No source or progress counter changes.
Normal post-missile elevator return now verified: target/post-missile-return/
elevator-up-final.inputs passes the standard oracle for 35,180 frames,
zero failures (48.67s), ending 97B5 at X128/Y136 with Morph 0004 and
current/maximum missiles 5/5. It includes the entire boot, Morph pickup,
first missiles, A107 return, 9F11 ascent and elevator Up; no items were
injected or gates changed. The pickup message is closed.
The difficult upper 9F11 step uses upward shots from its central Y235
platform, then a vertical jump and downward-left beam shots near the apex
to clear shootable blocks and land at X125/Y139. A fresh left jump crosses
the destroyed floor and exits to 9E9F. Running west stops at X1477/Y699;
a fresh left jump reaches X1424/Y683, then left 8, neutral 20, fresh Up 8
and neutral 800 return through the elevator. Useful inputs are retained.
Next focused coverage item: allow elevator control to settle, enter the
awake 975C state, defeat its five pirates with the basic beam and open
the gray west door. Preserve all five missiles. Normalize arrival in 96BA
to X396/Y2187 and append the verified 4,747-frame ascent suffix; verify
that combined return before proceeding through Parlor toward bombs.
This pass's workers have finished; tracked source and residue counters are
unchanged. Linux format/release build and all four original replays remain
passing on the unchanged source 90889612; local Cargo remains ASR-denied.
Awake pirate coverage now verified: target/awake-return/west-entry.inputs
passes the standard oracle for 35,560 frames, zero failures (56.55s),
with Morph and missiles 5/5. Entering 975C after the pickups loads F353
wall pirate and F653 walking pirates; this replay sees eleven enemy
definitions, two more than the preceding route. Coverage across retained
routes remains fifteen rooms. Read-only probe reports one of five kills;
an upward-diagonal shooting-jump continuation kills the wall pirate too.
Three walkers remain; clearing the quota and opening the west door are
not yet verified. Source/progress counters unchanged.
Parlor's bomb-route entrance has a measured Morph requirement: the X832
shaft is roofed at Y160 and entered from the left through Y176..191,
ball center near184. Dropping from the upper hall cannot reach it.
Four pre-Morph alignment trials pass but add no rooms and are stopped.
After the central ascent, use the actual item-bearing prefix for this
side entrance. Source BTS3 at X1008/Y608..671 selects the 9879 exit;
9879's right door selects 9804. At the red cap near X736/Y96, select
missiles with a fresh Select press and use five fresh shoot presses
roughly30 frames apart; held shoot does not repeat missile fire.
The independent Parlor climb is preparing its reusable ascent meanwhile.
Awake gray gate cleared with normal beam combat: target/awake-return/
last-pirate.inputs passes the standard oracle for 36,572 frames, zero
failures (52.96s), preserving missiles 5/5 and Morph 0004. The probe
observes quota 5/5, and the standard trace enters 96BA at frame36,567;
final X462/Y2187. Upward-diagonal shooting jumps finish the wall pirate;
fresh left shooting jumps across the corridor pillars finish the walkers.
No missile shots or gate edits were used. This replay reaches fourteen
rooms and eleven enemy definitions; the retained route union has fifteen
rooms. The worker is normalizing the floor position for the saved ascent.
Independent Parlor preparation has a verified first ledge at X463/Y1180
(29,980/0 standard) and a later right ledge at X473/Y1067
(30,314/0 standard). Upper ceiling islands require approaching each shelf
from outside its underside. These pre-Morph prefixes remain preparation;
the item-bearing combined route is still the next required proof.
Independent Parlor ascent reaches its upper hall: target/parlor-ascent/
upper-align.inputs passes the standard oracle for 32,106 frames, zero
failures (44.75s), final X433/Y170. Further right movement reaches
X795/Y171 at the Morph-passage wall in upper-walk.inputs (32,326/0 probe).
The reusable ascent-from-370-1211.inputs suffix has 2,639 frames and
requires standing X370/Y1211 facing left (pose0002); it includes the
right movement to that wall. It still needs an item-bearing combined
replay. Run movement before fresh jumps and short vertical starts avoid
ceiling islands; centering inside X368..399 clears the last upper roof.
The actual awake 96BA return exposed why the empty-room ascent cannot
be assumed transferable: eleven wall pirates are present, and one near
X464/Y1511 hits Samus before the saved beam sequence kills it, knocking
her off the cycle's horizontal position. Repeated unchanged grafts pass
the oracle but stall below Parlor. Read-only motion/HP tracing identified
the contact; normal combat and position recovery are being verified.
A recovered cycle starting X401/Y1443 now reaches X401/Y1187 without
another hit (38,742/0 probe). The combined exit remains unproved.
No gameplay source or residue counters changed; useful inputs are retained.

The complete item-bearing return is now verified by the standard oracle:
target/awake-return/parlor-hatch-current.inputs passes 42,651 frames,
zero failures (61.66s, independently repeated), ending in Parlor at
X371/Y1211, facing left, with Morph 0004 and missiles 5/5. The recovered
awake shaft cycles and unique upper shelves reach the ceiling hatch normally;
this supersedes the earlier unproved grafts. Fourteen rooms and fourteen
enemy definitions are present on this route; retained-route union is fifteen
rooms. No gameplay state was injected.
The actual Parlor ascent is also verified: target/awake-return/
item-shelf443-right.inputs passes the standard oracle for 46,133 frames,
zero failures (66.76s), ending X795/Y171 with Morph and missiles 5/5.
The empty-room suffix alone stalls after enemy contact on the first ledge.
Recovery from frame 42,940 moves left/shoot 8, shoot 30, then resumes the
prepared Parlor suffix at offset 293. At frame 44,650, a fresh left press
and neutral 20 precede its offset-1307 running jump; beam fire through this
middle section reaches X444/Y443. At frame 45,210, right/shoot 14 and
shoot 120 settle at X459/Y443; the original suffix from offset 1850 then
reaches the upper hall. Beam fire is unselected throughout; missiles stay 5/5.
Morph entry toward bombs now passes the standard oracle as well:
target/awake-return/bomb-morph-entry.inputs, 46,435 frames, zero failures
(65.99s), ending X883/Y409 in Parlor. From the upper-hall prefix, fresh
Down, neutral 8, fresh Down, neutral 12, right/shoot 80, down/shoot 80,
neutral 120 enter the low passage. Continuing right reaches X1003/Y494
in ball form (bomb-descent.inputs, 46,755/0 standard). The alternative
bomb-left-drop.inputs route under the same directory reaches X964/Y537 in
ball form (46,760/0 standard). Unmorphing there and moving right reaches
X987/Y523 (bomb-left-unmorph.inputs, 47,031/0 standard, 67.75s).
The left/shoot 25, shoot 100, right/shoot 80 continuation passes its probe
for 47,356 frames with zero failures but returns to X987/Y523. Shooting
sideways alone does not resolve the lower passage. Inspect the live floor
and try downward shots or a leftward Morph descent before repeating it.
No additional room or Bomb pickup is proved yet. Next: descend the remaining
Parlor passage to its lower-right door, confirm 9879 by replay, then proceed
toward the bomb-room red cap using the five preserved missiles.
Full any% regression passes 129,712 frames, zero failures (186.68s).
Direct rustfmt checks pass; Cargo fmt/build remain ASR-denied on this host.
Tracked source and all progress counters are unchanged; the unchanged source
90889612 retains its passing Linux stable build and four original replays.

Bomb pickup is now verified with normal inputs and the standard oracle:
target/awake-return/bomb-orb-contact.inputs passes 49,789 frames, zero
failures (70.38s), ending in 9804 at X187/Y187, gameplay state 8, equipped
and collected items 1004 (Morph and Bombs), missiles 0/5. This route visits
sixteen rooms and sixteen enemy definitions; union with the retained 9A44
branch is seventeen rooms. The new definitions are D0FF in 9879 and EEFF
in 9804. No gameplay state or gate was edited.
The lower Parlor shaft bends left: from bomb-left-unmorph.inputs at
X987/Y523, left/shoot 100 then neutral 180 reaches X821/Y593
(bomb-left-run.inputs, 47,311/0 standard, 68.31s). Right/shoot 180,
neutral 120 opens and crosses the lower-right blue door into 9879
(bomb-door-right.inputs, 47,611/0 standard, 70.81s). Downward shots from
the earlier ledge add no progress; its visible geometry is solid/sloped.
Across 9879, right/shoot 220, neutral 120 reaches X575/Y187; right 5,
run/right/shoot 200 and neutral 120 reaches the raised lip at X699/Y187
(bomb-red-run.inputs, 48,276/0 standard, 69.75s). Fresh jump/right/shoot
50 after neutral 5 reaches X731/Y139 beside the red cap
(bomb-cap-jump.inputs, 48,451/0 standard, 69.51s).
Fresh Select, neutral 30, then five fresh shoot presses separated by
neutral 30 (neutral 60 after the fifth), right 100, neutral 120 opens
the cap and enters 9804 (bomb-five-missiles.inputs, 48,887/0 standard,
69.81s). Current missiles are confirmed 0/5; the older probe prints only
maximum missiles, so its 5 alone must not be read as remaining ammunition.
Fresh Select again, neutral 10, right/shoot 70, neutral 700, fresh jump
and neutral 120 shoots/contacts the Bomb orb. Item bits become 1004 by
frame 49,000 in the standard trace. The message is closed at the finish.
Post-pickup left/shoot 100, neutral 5, left/jump/shoot 70, neutral 120
reaches X37/Y187 in 9804 (bomb-exit-left.inputs, 50,084/0 probe) but does
not exit. The vertical-jump attempt also passes the standard oracle
(bomb-exit-up.inputs, 50,329/0, 71.80s) and remains blocked at X37 even
at door height. Read-only live-state inspection confirms PLM BAF4 uses
preinstruction BDF2, PlmPreInstr_GotoLinkIfTorizoDead; enemy EEFF is active
with 800 HP, Samus has 31 health and HUD selection 0. The gray door needs
the boss defeat, not another movement adjustment. Bomb Torizo activation
is proved; combat and return through the corridor remain unfinished.
The facing-right stationary-beam continuation (right 1, shoot 1200,
neutral 120) passes 51,650 frames with zero failures but ends in death
state 26. Preserve bomb-orb-contact.inputs; use movement and beam combat
or shorten the post-pickup wait to improve the fight setup. Boss defeat
is not proved. Continue this item-bearing route toward wider enemy coverage
before resuming the deferred slope cleanup.
Full any% passes 129,712 frames, zero failures (188.48s). Direct rustfmt
passes; Cargo fmt/build are still ASR-denied. No source changes; all residue
counters are unchanged, and prior Linux validation applies to source 90889612.

Bomb Torizo defeat now passes the standard oracle: target/awake-return/
fight-right-tap.inputs, 52,820 frames, zero failures (81.22s). The read-only
probe observes EEFF health falling to zero by frame 51,500, its slot clearing
by 51,900, and BAF4 changing from boss-death preinstruction BDF2 to the normal
shot handler BD0F. Samus survives with 23 health, Morph/Bombs 1004 and
missiles 0/5; final room 9804, gameplay state 8, X135/Y187.
The successful combat prefix retains bomb-exit-up.inputs, turns right for
one frame and holds beam fire until frame 50,400. Run/right/jump/shoot 65,
left/shoot 1 and shoot 230 cross the approaching boss and face it from
X118/Y187. The next crossing starts at 50,696; retain its first four frames
through frame 50,700, then shoot 1 / neutral 9 repeated 200 times, neutral
120. This stops the second crossing early and settles facing left near
X116, shooting Torizo at the left wall. All actions are normal controller
inputs. Held Bomb fire did no damage; repeated full-room crossings and
stationary beam fire died. Six-frame beam taps also defeat the boss from
the frame-50,500 prefix (52,720/0 probe), but the standard-verified ten-frame
route remains the canonical continuation.
The post-defeat door needs a beam shot, then a separate jump after its
opening animation. torizo-exit.inputs passes the standard oracle for
53,110 frames, zero failures (78.44s), removes BAF4 and ends X37/Y187 below
the door. Neutral 5, left/jump/shoot 90 and neutral 120 then crosses into
9879 (torizo-exit-hop.inputs, 53,325/0 standard, 76.23s), X728/Y139.
Left/shoot 250, neutral 5, left/jump/shoot 70 and neutral 120 reaches the
corridor's west cap at X37/Y139. A second fresh jump (neutral 5,
left/jump/shoot 90, neutral 140) returns to Parlor, X975/Y651:
torizo-return-door.inputs passes the standard oracle for 54,005 frames,
zero failures (79.69s). The route preserves Morph/Bombs
1004 and missiles 0/5. Next: climb the eastern Parlor passage, return to
the upper hall and use Bombs to reach a Waver room. No source changes;
direct rustfmt and progress checks pass with all counters unchanged.
Full any% passes 129,712 frames, zero failures (190.40s). Cargo fmt/build
remain ASR-denied; prior Linux validation applies to source 90889612.

Post-Bomb Parlor climb preparation now reaches X842/Y459 in normal gameplay:
target/awake-return/parlor-return-run.inputs passes the standard oracle for
54,686 frames, zero failures (76.67s). From torizo-return-door.inputs, neutral 5,
left/jump/shoot 35, neutral 120 reaches X933/Y641; neutral 5,
left/jump/shoot 60, neutral 120 reaches X837/Y619; neutral 5,
right/jump/shoot 55, neutral 120 reaches X915/Y539. The next ledge needs
run/left/shoot 6 before run/left/jump/shoot 30 and neutral 120, landing
X842/Y459. Ordinary diagonal jumps and a vertical start fall back to the
Y539 shelf; the running approach clears the overhang. Morph/Bombs remain
1004 and missiles 0/5. Upper-hall return and Waver presence remain unproved;
continue from this prefix before the deferred gameplay cleanup.

The post-Bomb climb now reaches X933/Y267 with the standard oracle:
target/awake-return/parlor-return-left352.inputs, 55,153 frames, zero
failures (80.26s). From parlor-return-run.inputs, run/right/shoot 6,
run/right/jump/shoot 32 and neutral 120 reaches X915/Y395; another
run/right/shoot 6, run/right/jump/shoot 25 and neutral 120 reaches
X1003/Y336. Run/left/shoot 6, run/left/jump/shoot 32 and neutral 120 then
lands on the higher shelf at X933/Y267. Jumping left directly from Y395
falls back to Y459; the right-hand detour is required for this setup.
A vertical jump from Y267 reaches Y175 but sideways movement stops at
X933 on the left or X955 on the right. Read-only five-frame tracing shows
the standing body blocked by the low roof. Early airborne Morph cancels
the ascent; later Morph works but left movement still hits the column.
Right movement during the Down presses retains a spin/turn pose instead
of Morph. A delayed-right Morph continuation is being tested; upper-hall
return and Waver remain unproved. All inputs are ordinary controller words.
Full any% passes 129,712 frames, zero failures (190.43s). Direct rustfmt
and progress pass; source/counters unchanged, Cargo remains ASR-denied.

Bomb-block opening on the return is now standard-verified:
target/awake-return/parlor-return-real-ground-bomb.inputs, 56,014 frames,
zero failures (81.21s), final X987/Y185 in ball form, items 1004, missiles
0/5. From parlor-return-upper-right.inputs at X955/Y267, neutral 5,
jump/shoot 28, Down/jump 1, jump 2, Down/jump 1, jump 10,
right/jump 15, jump/shoot 1, shoot 140 and neutral 120 reaches X971/Y185
in ball form. Keep Jump held through both Down presses: releasing it cuts
the ascent short. This prefix is parlor-return-ball185.inputs; exclude
the later Up press that unmorphs into crouch before firing another Bomb.
Fresh shoot 1, neutral 90, right/shoot 40, neutral 60, Up 1, neutral 20,
neutral 5, right/jump/shoot 50 and neutral 120 produces the verified finish.
Read-only block checks show X976/Y176 and X992/Y176 changing from F2D8
to air; the next two Bomb blocks stay intact. A 12-frame left roll settles
X957/Y185; unmorphing reaches standing Y171. Its vertical jump clips the
ledge, while a Bomb lift reaches ball Y151. Longer left rolls overshoot and
fall back to the Y267 shelf. Centering the vertical opening is next;
upper-hall traversal and Waver presence are still unproved. No source or
counter changes; any% and format results above apply.

Post-Bomb upper-hall entry now passes the standard oracle:
target/awake-return/parlor-return-hall-west.inputs, 56,577 frames, zero
failures (82.88s), standing X1077/Y171 with Morph/Bombs 1004, missiles 0/5.
The rightward low passage opens its second Bomb-block pair with fresh fire:
from parlor-return-real-ground-bomb.inputs, shoot 1, neutral 90, right 20,
then right/shoot 1 and right 89 repeated three times. Retain only through
frame 56,330 of this continuation (parlor-return-right-stop56330.inputs),
then Up 1, neutral 20, run/left/shoot 6, run/left/jump/shoot 100, neutral 120.
The longer rightward route dies near X1243; the retained prefix turns back
before that contact. Earlier turn-back at 56,200 falls into the lower shaft.
At X1077 the westward jump hits the central hill's side; Morph with fresh
Bombs reopens the low passage but prolonged left rolling falls back to Y267.
Next trial starts vertically before steering left over the hill. The main
upper hall and Waver remain unproved; no gameplay state was injected.
Source/counters unchanged; any% and formatting results above apply.

The vertical-start hill crossing is now standard-verified:
target/awake-return/parlor-return-hill-top.inputs, 56,786 frames, zero
failures (92.15s), standing X965/Y91, items 1004 and missiles 0/5.
From parlor-return-hall-west.inputs, neutral 5, jump/shoot 24,
left/jump/shoot 60 and neutral 120 clears the eastern side of the hill.
Run/left/shoot 300 and neutral 120 crosses most of the main hall but drops
through the central hatch near X391, ending X277/Y289 (57,206/0 probe).
Retaining that run only through frame 56,920 and appending a running left
jump clears the hatch geometrically but dies (57,105/0 probe, state 24).
Keep the safe hill-top prefix; further westward travel must account for
Samus's low health and enemy contacts. The Bomb-sealed western exit and
Waver presence remain unproved. No source changes; counters and full any%
remain unchanged from this pass's checks.

Waver coverage is now reached with normal inputs and the standard oracle:
target/awake-return/west-door-contact.inputs, 58,109 frames, zero failures
(86.55s), ending 990D X1480/Y139 with Morph/Bombs 1004, missiles 0/5.
Room 990D loads D63F Wavers; this route now sees seventeen rooms and
seventeen enemy definitions, eighteen rooms in union with the retained 9A44
branch. The previous fatal westward jump hits DCFF near X286/Y105 with
Samus at 5 health. From parlor-return-before-hatch.inputs, run/left/jump/shoot
38, right/shoot 10, shoot 80, neutral 120 brakes and lands X397/Y235 alive
(west-brake.inputs). Run/left/shoot 6, run/left/jump/shoot 55, neutral 120
then reaches X298/Y150 (west-from-island.inputs, 57,349/0 probe).
Fresh Down, neutral 8, fresh Down, neutral 12, left 10, shoot 1,
neutral 90, left/shoot 30, neutral 90, shoot 1, neutral 90, neutral 120
reaches X239/Y153 in ball form through the Bomb wall (west-bomb-wall.inputs,
57,803/0 probe). Left/shoot 45, neutral 20, Up 1, neutral 20,
left/shoot 100 and neutral 120 crosses the blue door into 990D.
Samus preserves 5 health through the Parlor exit. No gameplay state was
injected. Baseline any% passes 129,712 frames, zero failures (187.87s).
This closes the Waver-presence gap; resume deferred nonsquare slope work.

Horizontal nonsquare slope cleanup uses named saved BTS access, checked
live-ROM words and wrapping multiplication. Negative displacement retains
the original signed-16-bit truncation after shifting; positive displacement
retains all shifted bits. Independent arithmetic comparison covers all
16,777,216 shifted displacements with seven multiplier boundaries
(117,440,512 combinations), plus all 65,536 flags/table offsets.
Stable optimized core/DLL build and direct format pass; Cargo fmt/build
remain ASR-denied. Updated-source any% passes 129,712 frames, zero failures
(187.68s); Waver-route passes 58,109 frames, zero failures (86.27s).
Raw offsets -1, unsafe functions -1, cast lines -20, Rust lines -12.
Next: vertical nonsquare slope handler, preserving its edge tests and writes.

Vertical nonsquare slope cleanup now uses byte-backed enemy coordinates,
named block/BTS access and checked live-ROM bytes. Upward/downward paths
share their offset and correction calculation; the original downward
equality case and negative correction test are equivalent to a nonpositive
correction. Position writes still precede subposition writes: upward sets
subposition 0000, downward FFFF. The low byte of room width still controls
modulus, including the original unchanged dividend on a zero divisor.
Both the handler and vertical slope dispatcher are safe Rust; no RAM borrow
crosses a call that can mutate coordinates. Independent comparison passes
4,194,304 orientation/offset/contact cases with boundary positions, 589,824
edge pairs and 16,777,216 modulus cases. Stable optimized core/DLL build and
direct formatting pass; Cargo fmt/build remain ASR-denied.
Updated Waver-route passes 58,109 frames, zero failures (88.44s); any%
passes 129,712 frames, zero failures (189.29s). Raw offsets -8, unsafe
functions -2, cast lines -70, Rust lines -90.
Next: audit remaining unsafe shared movement dependencies, starting with
horizontal slope dispatch and the spike/PLM boundary, without concealing
legacy runtime preconditions behind safe wrappers.

Movement safety audit: horizontal slope dispatch only calls safe handlers
and now is safe Rust. Spike collision still calls SpawnPLM, whose setup
dispatch requires initialized legacy runtime state; movement cannot yet be
made safe through that boundary. SpawnPLM slot preparation now uses
little-endian RAM bytes and checked live-ROM words instead of typed pointers.
The descending 40-slot allocation, zero result when full, all write ordering
and callback result remain unchanged. Instruction is read after header/pre
setup, and the setup pointer is read after the final current-slot write.
SpawnPLM stays unsafe with its callback precondition documented.
Stable optimized core/DLL build and direct format pass; Cargo fmt/build
remain ASR-denied. Updated Waver route passes 58,109 frames, zero failures
(83.57s); any% passes 129,712 frames, zero failures (183.94s).
Raw offsets -12, unsafe functions -1, cast lines
-21, Rust lines -21. Next: byte-backed PLM setup handlers reached by enemy
spikes, retaining their callback effects before removing movement's unsafe.

Enemy-breakable spike setup now uses safe little-endian RAM access, retaining
both word-index roundings, the low twelve block bits and its zero return.
The bank A0 log confirms the spike table has sixteen entries, only BTS 0F
spawning D094 (setup B3D4). The collision code still reads the live ROM with
the original 7F mask; out-of-table BTS values and patched setup pointers are
not restricted. SpawnPLM and movement remain unsafe because arbitrary setup
dispatch still reaches legacy callbacks. Stable optimized core/DLL build and
direct format pass; Cargo fmt/build remain ASR-denied. Updated Waver route:
58,109 frames, zero failures (88.18s); any%: 129,712 frames, zero failures
(190.15s). Raw offsets -2, unsafe functions -1, cast lines -6, Rust lines -3.
Next: deepen the PLM setup boundary incrementally, starting with block
deactivation and carry-only setups; preserve live-ROM callback dispatch.

PLM deactivation and both carry-only setups are now safe Rust. Deactivation
shares byte-backed block masking with the enemy-breakable setup, preserving
the original 8FFF mask and both rounded word indexes. Carry returns remain
zero and one; no callback dispatch changes. Stable optimized core/DLL build
and direct format pass; Cargo fmt/build remain ASR-denied. Updated Waver
route passes 58,109 frames, zero failures (87.70s); any% passes 129,712
frames, zero failures (189.33s). Raw offsets -2, unsafe functions -3,
cast lines -6, Rust lines -6. Next: the shared block-type/BTS writer and PLM
deletion used by extension setups, preserving their byte write order.

The shared level-data type/BTS writer, PLM deletion and four directional
extension setups are now safe byte-backed Rust. The writer retains the exact
level-data byte offset (including odd offsets), rounds BTS indexing down,
and writes level data before BTS. Extension setups retain 50FF/5001/D0FF/D001,
then clear the original PLM header and return zero. Stable optimized core/DLL
build and direct format pass; Cargo fmt/build remain ASR-denied. Waver route:
58,109 frames, zero failures (85.18s); any%: 129,712 frames, zero failures
(185.73s). Raw offsets -7, unsafe functions -6, cast lines -17, Rust lines -36.
Next: air/solid scroll setups and their wrapping instruction advance,
preserving level-data, BTS, trigger and instruction write order.

Air/solid scroll setups and the debug-draw instruction advance are now safe
byte-backed Rust. They retain 3046/B046, reset the slot's variable, then read
and advance the live instruction pointer by four with 16-bit wrapping. Slot
word indexing still rounds odd offsets down. Stable optimized core/DLL build
and direct format pass; Cargo fmt/build remain ASR-denied. Waver route passes
58,109 frames, zero failures (88.41s); any% passes 129,712 frames, zero
failures (188.61s). Raw offsets -5, unsafe functions -3, cast lines -9,
Rust lines -10. Next: scroll-block touch, retaining descending slot search,
the no-match infinite loop and the two separate wrapping instruction writes.

Scroll-block touch now uses safe byte-backed RAM access. It clears its own
block index before searching all forty slots from last to first, preserving
zero-index self-matches and the no-match infinite loop. The selected slot's
8000 variable test, full 8000 write, both separate instruction increments
and final timer write remain ordered as before. Stable optimized core/DLL
build and direct format pass; Cargo fmt/build remain ASR-denied. Waver route:
58,109 frames, zero failures (84.91s); any%: 129,712 frames, zero failures
(183.58s). Raw offsets -8, unsafe functions -1, cast lines -18, Rust lines -16.
Next: ice-physics setup and then quicksand setup dependencies, retaining
their exact position tests and physics writes.

Ice-physics setup now uses safe byte-backed Samus position/radius reads and
wrapping bottom-edge arithmetic. All 131,071 possible promoted bottom-edge
sums retain the same low-nibble 7/15 alignment test. The deceleration write
still covers two bytes at B4C, including the following byte, despite ram_map
naming the field uint8. Stable optimized core/DLL build and direct format
pass; Cargo fmt/build remain ASR-denied. Waver route: 58,109 frames, zero
failures (84.27s); any%: 129,712 frames, zero failures (184.85s).
Raw offsets -5, unsafe functions -1, cast lines -15, Rust lines -12.
Next: quicksand surface dispatcher and its three movement handlers,
preserving the overlapping middle-word writes at B2D/B5B.

Quicksand surface setup and all three movement handlers are now safe
byte-backed Rust, with direct direction dispatch replacing the callback
table. Momentum/boost/run-speed resets retain their order, the base subspeed
retains its low fifteen bits, and equipment/reaction-point tests remain live.
Middle-word speed/displacement writes retain B2D/B5B byte ranges and the
unsigned strict speed clamp. Direct comparison against prior Rust source
passes 524,291 RAM/return cases: all 65,536 middle-word speeds, both equipment
choices, four directions and nonzero reaction points. Stable optimized
core/DLL build and direct format pass; Cargo fmt/build remain ASR-denied.
Waver route: 58,109 frames, zero failures (83.92s); any%: 129,712 frames,
zero failures (183.17s). These routes do not prove quicksand coverage; the
direct comparison supplies the handler check. Raw offsets -26, unsafe
functions -4, cast lines -48, Rust lines -56. Next: submerging quicksand and
sandfall setup writes, then the collision-side quicksand context boundary.

Both submerging-quicksand setups and all three sandfall setups are now safe
byte-backed Rust. Inside reactions retain displacement words 2000/4000/C000
followed by 0001; collision-side submerging retains all four speed/acceleration
resets in order. Return values remain zero. Direct comparison with prior
Rust passes 1,280 full-RAM/return cases over 256 fill patterns. Stable
optimized core/DLL build and direct format pass; Cargo fmt/build remain
ASR-denied. Waver route: 58,109 frames, zero failures (83.01s); any%: 129,712
frames, zero failures (182.84s). These routes do not prove sandfall coverage.
Raw offsets -11, unsafe functions -5, cast lines -12, Rust lines -11.
Next: collision-side quicksand RAM/arithmetic cleanup, retaining its unsafe
active-context precondition; the raw displacement pointer is installed and
cleared by both Samus collision dispatchers in bank 94.

Collision-side quicksand now uses byte-backed collision-direction/contact
reads and wrapping quicksand-counter writes. The displacement clamp retains
its unsigned middle-word comparison and both outside bytes. Raw active
displacement access and the original null-context branch remain; the handler
stays unsafe with a documented live/writable pointer requirement. Direct
comparison against prior Rust passes 8,388,608 displacement/RAM/return cases
(all middle words, four directions, sixteen collision flags, two contact
branches), including counter wrap. Stable optimized core/DLL build and direct
format pass; Cargo fmt/build remain ASR-denied. Waver route: 58,109 frames,
zero failures (84.57s); any%: 129,712 frames, zero failures (183.99s).
Raw offsets -7, cast lines -24, Rust lines -21; unsafe count unchanged.
Next: make the Samus collision displacement context explicit across its PLM
callback boundary, preserving retries, callback order and live RAM reads.

PLM setup dispatch now receives the active collision displacement explicitly.
All three spawn paths pass the stored context at the final callback boundary;
quicksand consumes that argument instead of reading the pointer from Game.
The stored pointer remains a bridge from Samus collision dispatch, retaining
its scope across retries and nested game calls. Unsafe contracts document
the live/writable requirement. Direct dispatch comparison against prior Rust
passes 8,388,608 displacement/RAM/return cases with the new Game's stored
pointer deliberately null. Stable optimized core/DLL build and direct format
pass; Cargo fmt/build remain ASR-denied. Waver route: 58,109 frames, zero
failures (85.66s); any%: 129,712 frames, zero failures (183.35s).
Rust lines +20 expose/document the context; all other counters unchanged.
Next: replace Samus collision context parameters with references, preserving
the active displacement's retry-loop scope before removing the stored bridge.

Samus horizontal/vertical collision dispatch entry points now take mutable
context references. Both legacy context/displacement pointers derive from
one borrow and stay raw throughout the retry loop, avoiding a reborrow while
callbacks access displacement through Game. Named live block reads replace
typed RAM pointers; the stored context is still cleared after dispatch.
Direct comparison with prior Rust passes 4,096 synthetic callback cases over
both directions, all block types, 0..15 retries and eight displacement
boundaries; callbacks modify the live block index and displacement, and
final RAM/returns/context cleanup agree. Stable optimized core/DLL build and
direct format pass; Cargo fmt/build remain ASR-denied. Waver route: 58,109
frames, zero failures (83.72s); any%: 129,712 frames, zero failures (181.44s).
Raw offsets -6, cast lines -16, Rust lines -15; unsafe count unchanged.
Next: direct Samus collision type matches, then carry/extension handler
cleanup, keeping raw callback context until the stored bridge is removed.

Samus horizontal/vertical collision type dispatch now uses direct matches.
All thirty-two handler selections retain the original table order and
direction-specific spike-air behavior. Both retry loops, live block reads,
raw callback context and displacement cleanup are unchanged. Comparison of
all 131,072 level-word/direction selections agrees with the prior tables.
Stable optimized core/DLL build and direct format pass; Cargo fmt/build
remain ASR-denied. Waver route: 58,109 frames, zero failures (83.91s); any%:
129,712 frames, zero failures (183.99s). Rust lines -24; other counters
unchanged. Next: carry/extension handlers, retaining signed BTS offsets,
full room width, wrapping block indexes and high-bit retry results.

Samus extension reactions now use named live BTS/block/room-width access and
explicit wrapping arithmetic. Zero BTS still returns zero without writing;
nonzero signed BTS offsets update the block index and return FF. Vertical
extension retains the full sixteen-bit room width. Both clear-carry handlers
and the inert vertical spike-air handler are safe zero-return functions.
Direct comparison with prior Rust passes 33,554,432 extension cases over all
BTS bytes, all room widths and both axes with varied valid backing indexes.
Stable optimized core/DLL build and direct format pass; Cargo fmt/build
remain ASR-denied. Waver route: 58,109 frames, zero failures (83.35s); any%:
129,712 frames, zero failures (182.10s). Raw offsets -11, unsafe functions -5,
cast lines -16, Rust lines -33. Next: Samus special-air/block PLM selection
reads, preserving area-specific tables and callback-adjusted collision state.

Special-air/block collision now shares named live BTS and byte-backed area
reads plus checked live-ROM PLM selection. Air retains the original BTS
across setup and ignores low-BTS carry; special blocks invoke the correct
solid handler after setup, keeping collision reads live. Invalid area-table
windows still call the legacy diagnostic and set g_fail before mapping the
same bytes. Synthetic prior-source comparison passes 2,883,584 selection,
callback-order, return and diagnostic cases over every BTS/return value,
eleven area boundaries and all four handlers. Stable optimized core/DLL
build and direct format pass; Cargo fmt/build remain ASR-denied. Final-source
Waver route: 58,109 frames, zero failures (84.34s); any%: 129,712 frames,
zero failures (184.78s). Raw offsets -12, cast lines -64, Rust lines -131;
unsafe functions +1 for the shared selector retaining legacy diagnostics.
Next: bombable-air/bomb-block PLM selection reads, retaining carry handling
and live collision state after setup.

Bombable-air/bomb-block collision now uses named live BTS and checked ROM
words. Both air directions share the same reaction; high-bit air still skips
spawn and returns zero. High-bit solid blocks collide immediately; low-bit
blocks mask setup carry and invoke the directional solid handler afterward,
retaining live context/RAM reads. Synthetic comparison with prior Rust
passes 262,144 selection/callback/return cases over all BTS bytes, setup
returns and four handlers, with setup mutating BTS/area RAM. Stable optimized
core/DLL build and direct format pass; Cargo fmt/build remain ASR-denied.
Waver route: 58,109 frames, zero failures (83.79s); any%: 129,712 frames,
zero failures (183.97s). Raw offsets -8, cast lines -32, Rust lines -58;
unsafe count unchanged. Next: door collision RAM/ROM reads, preserving the
existing pose-based enemy-door condition and write order.

Both door collision handlers now use byte-backed RAM and checked live-ROM
words. Door-list indexing retains sixteen-bit wrap before mapping; invalid
list/door windows retain legacy diagnostics and g_fail writes. The initial
E17D function write, pose < 9 enemy-door test, collision fallback, door-pointer
write and state-9 write keep their order. Synthetic comparison against prior
Rust passes 25,088 full-RAM/collision/diagnostic cases over all BTS values,
seven list boundaries, seven poses and both directions. Stable optimized
core/DLL build and direct format pass; Cargo fmt/build remain ASR-denied.
Waver route: 58,109 frames, zero failures (84.25s); any%: 129,712 frames,
zero failures (183.21s). Raw offsets -16, cast lines -36, Rust lines -22;
unsafe count unchanged. Next: solid collision boundary arithmetic and
byte-backed Samus coordinates, preserving live context after PLM callbacks.

Solid horizontal/vertical collision now shares explicit wrapping boundary
arithmetic and reads/writes Samus coordinates through backing RAM bytes.
The difference still truncates to signed sixteen bits before clamping; raw
live context and displacement-before-subposition write order remain. Prior
Rust comparison passes 4,194,304 full-RAM/context/return cases over every
position, eight edge/radius pairs, four displacement values and both axes.
Stable optimized core/DLL build and direct format pass; Cargo fmt/build
remain ASR-denied. Waver route: 58,109 frames, zero failures (84.65s); any%:
129,712 frames, zero failures (186.84s). Raw offsets -12, cast lines -36,
Rust lines -31; unsafe count unchanged. Next: Samus slope dispatch uses
existing live block/BTS accessors, preserving square/nonsquare selection
and the saved BTS write before entering nonsquare handlers.

Both Samus slope dispatchers now use existing live block/BTS accessors.
The low-five-bit < 5 square threshold, full saved BTS word, block arguments
and write-before-nonsquare-call order remain exact. Prior Rust comparison
passes 3,584 full-RAM/context/return/selection cases over every BTS byte,
seven valid backing indexes and both directions; synthetic handlers mutate
the block index/BTS and inspect the saved word. Stable optimized core/DLL
build and direct format pass; Cargo fmt/build remain ASR-denied. Waver route:
58,109 frames, zero failures (85.02s); any%: 129,712 frames, zero failures
(183.61s). Raw offsets -13, cast lines -19, Rust lines -41; unsafe count
unchanged. Next: horizontal nonsquare slope displacement arithmetic and
byte-backed speed reads, retaining signed-shift truncation and multiplication.

Horizontal nonsquare slope collision now uses named saved-BTS access,
byte-backed vertical speed and explicit middle-word displacement arithmetic.
The bit-7/speed early returns, signed right-shift truncation, unsigned
sixteen-bit negation and product sign remain exact. Prior Rust comparison
passes 5,505,024 full-RAM/displacement/return cases: every middle word with
both signs and all slope factors, plus every saved BTS word, five speed
boundaries and four displacement boundaries. Stable optimized core/DLL
build and direct format pass; Cargo fmt/build remain ASR-denied. Waver route:
58,109 frames, zero failures (85.02s); any%: 129,712 frames, zero failures
(185.51s). Raw offsets -4, cast lines -19, Rust lines -13; unsafe count
unchanged. Next: vertical nonsquare slope collision, preserving low-byte
room-width modulus (including zero), direction/BTS alignment and signed
sixteen-bit displacement clamping.

Vertical nonsquare slope collision now shares mirrored direction branches
and uses byte-backed coordinates plus named live block/room-width access.
The low-byte width modulus retains zero-divisor return, BTS orientation and
mirror tests retain table coordinates, and displacement wraps to signed
sixteen bits before the zero clamp. Prior Rust comparison passes 2,097,152
full-RAM/context/return cases over every BTS/table coordinate, signed high
word and room width, with matched/mismatched columns and separate BTS indexes.
Stable optimized core/DLL build and direct format pass; Cargo fmt/build
remain ASR-denied. Waver route: 58,109 frames, zero failures (82.53s); any%:
129,712 frames, zero failures (178.91s). Raw offsets -19, cast lines -73,
Rust lines -89; unsafe count unchanged. Next: square slope collision quadrant
selection and eight-pixel boundary arithmetic, preserving directional flags
and displacement-before-subposition writes.

Both square slope collision handlers now share readable quadrant selection
and eight-pixel boundary arithmetic, with byte-backed coordinate writes.
The first/last sample tests, orientation XOR, signed truncation before clamp
and displacement-before-subposition writes retain behavior; only vertical
nonnegative collision writes the slope-position flag. Prior Rust comparison
passes 3,964,928 full-RAM/context/return cases over all five shapes/BTS bytes,
quadrant sample combinations and every position with six radius/edge pairs,
four displacement boundaries and both axes; a fully occupied shape ensures
the boundary sweep reaches displacement writes. Stable optimized core/DLL
build and direct format pass; Cargo fmt/build remain ASR-denied. Waver route:
58,109 frames, zero failures (84.93s); any%: 129,712 frames, zero failures
(185.58s). Raw offsets -23, cast lines -83, Rust lines -96; unsafe count
unchanged. Next: Samus slope alignment RAM/arithmetic, preserving the second
block lookup after the first position adjustment and strict/non-strict tests.

Samus slope alignment now uses byte-backed coordinates and named live
block/BTS access. Bottom correction retains its strict negative test; top
correction retains its nonpositive test and flag write even at zero. The
second lookup reads the position after the first correction; lookup calls,
wrapping Y writes and flag ordering remain exact. Prior Rust comparison
passes 1,446,912 full-RAM/lookup cases over every bottom/top BTS pair, table
coordinates, radius/position boundaries and block-type/movement gates;
synthetic lookups also mutate coordinates to check live reads. Stable
optimized core/DLL build and direct format pass; Cargo fmt/build remain
ASR-denied. Waver route: 58,109 frames, zero failures (86.01s); any%: 129,712
frames, zero failures (183.26s). Raw offsets -35, cast lines -97, Rust lines
-110; unsafe count unchanged. Next: CalculateBlockAt coordinate arithmetic
and byte-backed block writes, retaining signed range and low-byte multiply.

CalculateBlockAt is now safe Rust with wrapping coordinate sums, explicit
0..4096 range checks, low-byte room-width multiplication and named backing
block-index writes. The original signed tests accept exactly that wrapped
range; invalid coordinates still write FFFF. Prior Rust comparison passes
20,447,232 full-RAM cases over all wrapped coordinate sums, seven offsets,
four opposite-axis boundaries and every room width/valid row combination.
Stable optimized core/DLL build and direct format pass; Cargo fmt/build
remain ASR-denied. Waver route: 58,109 frames, zero failures (86.52s); any%:
129,712 frames, zero failures (186.84s). Raw offsets -3, unsafe functions -1,
cast lines -19, Rust lines -28. Next: block-inside detection type dispatch
and its retry loops, preserving three live position samples and callbacks.

Block-inside detection now shares direct type matches across its three
sample retry loops. All sixteen selections, live block reads and high-bit
retry behavior remain exact; position sampling/reset writes are unchanged.
Prior Rust comparison passes 66,560 full-RAM/callback-log cases over every
level word and all types with 0..15 retries across four sample scenarios;
callbacks change the live index/type, coordinates and collision context.
Stable optimized core/DLL build and direct format pass; Cargo fmt/build
remain ASR-denied. Waver route: 58,109 frames, zero failures (83.60s); any%:
129,712 frames, zero failures (182.27s). Raw offsets -6, cast lines -18,
Rust lines -53. Unsafe functions +1 is the shared raw-context dispatcher
replacing the three table calls. Next: block-inside detection coordinates
and reset writes, preserving live center/top tests after reaction callbacks.

Block-inside detection now uses byte-backed reset/coordinate access and
wrapping bottom/top arithmetic. B4C still clears the following byte, reset
and phase writes retain order, and center/top tests read live coordinates
after callbacks. The unusual top XOR expression is preserved explicitly.
Prior Rust comparison passes 721,920 full-RAM/callback cases over every
initial and callback-adjusted Y value, seven radius boundaries, level words
and retries; contexts/sample phases agree. Stable optimized core/DLL build
and direct format pass; Cargo fmt/build remain ASR-denied. Waver route:
58,109 frames, zero failures (85.51s); any%: 129,712 frames, zero failures
(185.29s). Raw offsets -24, cast lines -49, Rust lines -42; unsafe count
unchanged. Next: inert slope/default-speed inside reactions, then spike-air
reactions, preserving their contact gates and speed-table writes.

The inert slope and three default-speed inside reactions are now safe
functions. Shootable/special reactions share the byte-backed default write;
source audit confirms unchanged A6C..A6D range, 9F55 value and zero returns.
No collision context is dereferenced. Stable optimized core/DLL build and
direct format pass; Cargo fmt/build remain ASR-denied. Waver route: 58,109
frames, zero failures (83.07s); any%: 129,712 frames, zero failures (183.50s).
Raw offsets -3, unsafe functions -4, cast lines -6, Rust lines -5. Next:
spike-air inside reactions, retaining contact gates, damage/knockback writes,
final speed-table write and the existing out-of-range BTS panic behavior.

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
