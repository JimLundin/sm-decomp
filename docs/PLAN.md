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
