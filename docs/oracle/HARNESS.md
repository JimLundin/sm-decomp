# The oracle harness

The oracle harness runs the original, unmodified ROM on an emulator and records
what the new engine is compared against. It is a new crate, `crates/oracle`,
with no dependency on the old engine (`sm`, `sm_host`, today's `sm_game`).
REVIEW's "Rebuild on a new foundation (Jim, 2026-11-05)" sets the frame; the
architecture lane's oracle contract sets the record's shape.

## How the oracle runs today, and what that teaches

Today's oracle (`sm/src/cpu_infra.rs`, `crates/sm_host`) is snesrev's frame
stepper, not an emulator:

- `run_one_frame_of_game_emulated` runs the 65816 until the PC reaches the
  wait-for-NMI loop (`$80:8343`), raises NMI, runs until the handler returns
  (`$80:834A`), then `draw_frame_to_ppu` replays the frame's scanlines (HDMA,
  IRQ handlers) afterwards. There is no running beam while the CPU executes.
- Because there is no beam, the ROM needs 45 "kept" patches even in faithful
  mode (`oracle_patch_policy::KEPT_*`): beam spin-wait removals (`$80:843C`,
  `$80:8475`, `$80:8525`, `$80:945C`, `$88:82A1`), the native-audio handshake
  (`$80:8028` becomes RTS), the deferred message box (`$85:9670`...), the
  host-owned soft reset (`$81:F000`) and dispatch trampolines.
- There is no SPC700 running the game's sound driver: a native player
  (`spc_player.rs`) replaces it, so the CPU's APU handshakes are faked.
- The CPU core is upstream LakeSnes C behind an `Execution` context that
  carries the old engine's Game, its RAM image and lease checks; the PPU is
  snesrev's renderer with brightness tables, driven from `sm_runtime`.
- Seeds are lsnes (bsnes) captures at a frame boundary: WRAM, VRAM, CGRAM, OAM,
  SRAM, SPC RAM and a register CSV. They hold no SPC700, DSP or timer state.

The mistakes not to repeat: an oracle that needs ROM patches to run, an
emulator core welded to the engine it is checking, a sample point that is not
a hardware instant (main loop + NMI + the *next* frame's IRQs), and
observations defined by one emulator's internal structs.

## Decision: a real, beam-scheduled emulator

The harness runs a complete SNES emulator: 65816 with the master-clock beam,
DMA/HDMA, NMI/IRQ timing, auto-joypad, the PPU rendering per scanline, and the
SPC700 + DSP running the game's own sound driver. With a running beam none of
the kept patches is needed: the ROM runs unmodified, and `rom_patches.rs`,
`oracle_patch_policy` and `tools/snesrev_patches.py` have no counterpart.
`OracleId::patches` is empty.

The first backend is **upstream LakeSnes** (angelo-wf/LakeSnes `9db90b8`, MIT).
Upstream sources are not edited: the crate's own C bridge
(`crates/oracle/native/bridge.c`) includes `snes.c` with `ppu_write`,
`ppu_runLine` and `ppu_handleVblank` renamed to hooks, replaces the CPU's
read/write callbacks, and `dma.c` is compiled with its B-bus calls renamed, so
the bridge sees every register write, APU port access, NMI entry and line.
The step loop is C; Rust makes one call per video frame. The old Rust copies of
the CPU/SPC/PPU are not reused: they are entangled with the old Game and
snesrev's renderer.

## Toolchain

`tools/oracle/build_lakesnes.sh` fetches the pinned upstream files (curl, each
checked against its SHA-256), compiles them and the bridge with **Zig 0.15.2**
and writes `liboracle_lakesnes.a`; Cargo links it from `LIBRARY_PATH`
(`#[link(kind = "static", modifiers = "-bundle")]`), as every native archive
in docs/NATIVE_DEPENDENCIES.md is linked.

- Why Zig: it is already the project's pinned C toolchain. Every native
  archive (the LakeSnes CPU and PPU, the configuration and desktop bridges) is
  built with `zig cc` by the installers, and CI's `build` job downloads Zig
  0.15.2 for them. The `cc` crate is not used anywhere, and there is no MSVC or
  MinGW gcc on the Windows hosts.
- Why no `build.rs`: on the locked-down Windows host, Defender blocks some
  freshly built executables at random, build scripts included. An archive built
  by a script is how the other native archives work too.
- Targets: `--target x86_64-windows-gnu` (default) and `--target x86_64-linux-gnu`
  (adds `-fPIC -D_DEFAULT_SOURCE`). CI would run the same script after its
  existing Zig step and put the printed directory on `LIBRARY_PATH`.

## Crate layout

```text
crates/oracle/
  native/bridge.c         observation hooks and the per-frame step loop
  src/lib.rs              the Oracle trait, Start, Tiers, OracleId
  src/record.rs           FrameRecord and its parts (machine-domain types)
  src/game.rs             the game's wait-for-NMI sites (the only game knowledge)
  src/lakesnes.rs         the LakeSnes backend (FFI)
  src/lsnes.rs            the lsnes backend: lsnes-bsnes.exe as a separate process
  src/streams.rs          per-stream digests, legacy and v1, and the running fold
  src/lock.rs             reading scenarios/lock.toml pins and entries
  src/inputs.rs           .inputs files
  src/seed.rs             lsnes room captures (migration seeds)
  src/corpus.rs           running lock rows, comparing and recording lock entries
  src/measure.rs          the architecture lane's measurements (MEASUREMENTS.md)
  native/fixes/           named emulator fixes, applied by the build script
  src/cli.rs              the command line
  src/bin/oracle.rs       the binary around cli::main
tools/oracle/build_lakesnes.sh   pinned fetch and Zig build of the native archive
tools/oracle/lsnes_server.lua    the Lua server lsnes runs for the lsnes backend
```

Dependencies: `sm_data` for the shared digest only; no crates.io dependency.

## The backend trait

```rust
pub trait Oracle: Sized {
    fn identity(&self) -> OracleId;
    fn open(rom: &[u8], start: Start) -> Result<Self, OracleError>;
    /// One video frame with `pad` on controller 1, to the frame's cut.
    fn step(&mut self, pad: Pad) -> &FrameRecord;
    fn save_state(&self) -> Vec<u8>;
    fn set_tiers(&mut self, tiers: Tiers);
    fn reset(&mut self);
    /// Beam position and four WRAM bytes whenever the CPU reaches an address.
    fn set_watch(&mut self, addresses: &[u32], peek: u32);
}
```

A second backend implements the same trait; a disagreement report steps both
with the same pads and diffs records stream by stream.

### GPL emulators as a second reference

Mesen2 and bsnes are GPL-3; LakeSnes is MIT. Linking either into this crate
would put the harness under the GPL. Both can instead run as a **separate
process** behind the `Oracle` trait: a small adapter process, GPL-licensed
and built from their sources, speaks a pipe protocol (pad word in; the frame
record's memories, registers, port log and picture out), and this crate's
`Oracle` implementation talks to it without linking it. The record is about
200 KB per frame, so a pipe costs well under a millisecond per frame.

- **Mesen2** (GPL-3, C++): cycle-accurate, with a scriptable core and
  save states. Roughly 1,000-2,000 frames/s headless on one core for SNES.
- **bsnes** (GPL-3, C++, as in lsnes, which recorded the movies): bsnes v085's
  compatibility core is what the movies and seeds match frame for frame.
  Roughly 300-600 frames/s for the accuracy profile, more for performance.

These speeds are published orders of magnitude, not measurements here.

The lsnes backend (`src/lsnes.rs`, `--oracle lsnes`) runs lsnes rr2-beta23,
the bsnes v085 core that recorded the movies, as a separate GPL process: no
emulator code is linked or copied. lsnes runs `tools/oracle/lsnes_server.lua`;
requests go to its standard input, and each step's record goes to a
file it overwrites, announced on its standard output. The cut is the
interrupted instruction reached again with the stack restored (the NMI's RTI
has completed); a frame without an NMI is cut at lsnes's frame boundary
(line 241). An NMI that runs past that boundary is cut at its RTI in the
next lsnes frame, which takes the next pad the client previewed, since lsnes
fixes a frame's input at its start (MEASUREMENTS.md (i)); without a preview
it is cut at the boundary with the NMI running. It records the memories, the CPU, the cut, the joypad reads and the
wait entry. It does not record the picture, the APU port log, the SPC state
(lsnes shows SPC700 and DSP only as raw core structs), the profile, the write
log, watched-PC hits, `PpuRegisters` or `overscan`. Its state is lsnes's own
savestate at the frame boundary that ends the step it was requested before
(inside that step's NMI if it runs on), before the next step's NMI.
The script loads a starting state itself (`OH_START`, `load-state`), because
lsnes refuses a state given on its command line with this ROM and hangs.

**lsnes crashes (fixed).** The server's cut callback used to unregister
itself while lsnes dispatched it. Now and then lsnes then called a freed entry:
lsnes.log filled with `Error in Lua memory callback: attempt to call a nil
value` and lsnes died, which was the "exit at step 18068".

| Run | Result |
| --- | --- |
| Power-on to 19000, no saves | clean |
| From seed 16, to 18300 | clean |
| With saves | 1180 errors and a hang by 19829 |
| Cut callback wrapped in `xpcall` | crashed with `0xC0000028` after 154 frames |

Never unregistering, or unregistering from the next wait-entry callback or the
frame callback, is clean but runs at 7 frames/s instead of 44: the wait loops
spin through the cut's address.

The cut is now a callback that stays on the NMI handler's own RTI. The server
finds it once from the ROM: the vector's `REP #$30; JML`, then the handler's
one `PLY PLX PLA PLD PLB RTI`, at `$80:9601`. It takes the cut when S equals
the NMI's S and derives the registers after the RTI: S+4, and P, PC and PB
from the stack, with X and Y cut to 8 bits when that P has x set. PC is the
interrupted instruction's address, the one the RTI returns to, as LakeSnes
records it. It used to be one past it, as an exec callback at that
instruction reports it (`$80:8344` and `$80:8347`, inside `LDA $05B4` and
`BNE` of the wait loop); the cross-oracle report found it (MEASUREMENTS.md
(f)). Records and ladder rungs cut before the fix differ in those two bytes.

| Check | Result |
| --- | --- |
| Records over 1000 frames from power-on, any% pads | byte-identical to the old script's |
| From seed 16, 17829 to 19829, `--native-seeds` (3 saves) | clean, twice; no `nil value` in lsnes.log |
| Speed, all memories | 46 frames/s from power-on (old 44.7), 44 from seed 16 |

Each run keeps lsnes's standard error, the server's Lua error log and lsnes.log
in its run directory, and reports them if lsnes exits.

## The frame record (the oracle contract v1)

One record per video frame, lag or not. Record k is the closed interval
(cut k-1, cut k] plus the state at cut k:

- **The cut:** the RTI that ends the NMI serviced in vblank k (`Cut::NmiReturn`).
  With NMI disabled the cut is vblank start (`Cut::VblankWithoutNmi`); an NMI
  handler still running when vblank ends is `Cut::NmiStillRunning`.
- **NMI k:** its kind (`Requested` when the handler read the joypad, `Lag`
  otherwise), its interrupted PC and `overrun` when that PC is not one of the
  game's wait sites (`game::WAIT_SITES`).
- **Facts:** the pad auto-read at vblank k, joypad register reads, the
  overscan flag, the beam position where the main thread entered a wait.
- **Raw state at cut k:** CPU registers, WRAM 128K, VRAM 64K, CGRAM 512B,
  OAM 544B, SRAM, and the architectural registers (`PpuRegisters`: INIDISP to
  SETINI, NMITIMEN, HTIME/VTIME, HDMAEN, MEMSEL, the 8 DMA channels' $43x0-$43xA),
  rebuilt from the emulator's decoded state so they survive a state load.
- **The picture scanned out in the interval,** i.e. the one NMI k-1
  committed: 224 or 239 lines of 256 BGR555 dots after colour math, with each
  line's brightness and forced blank. The bridge renders each line at
  brightness 15, so LakeSnes's 8-bit expansion inverts exactly.
- **APU port log of the interval:** CPU and DMA writes and reads of
  $2140-$2143, in order.
- **Tier b (on request):** SPC700 registers, the four ports each way, SPC RAM
  64K and the DSP's 128 registers at the cut.
- **Tier a (on request):** every PPU register write of the interval, with
  its line, dot and source (main thread, NMI, IRQ, DMA, HDMA), and every
  watched instruction address reached. The APU log also carries the SPC700's
  own port writes and an SPC700 cycle stamp on each access, which the
  latency measurement uses. These extras are not digested.
  Up to 32 watched addresses and 16384 hits a step; with a watch set, each
  `$420B` write is a hit too (`0x01000000 | MDMAEN`, the first channel's DMAP,
  B-bus address and size in the peek). `oracle movie --trace FROM TO PC,...`
  prints them; on lsnes the server logs the same PCs to `OH_TRACE_LOG` when
  `OH_TRACE` is set (MEASUREMENTS.md (k)).
- **Profile (on request):** instructions executed per program bank.

## Inputs and starts

- `Start::PowerOn { wram_fill }`: power on with WRAM filled; the movies were
  recorded on bsnes v085, which fills 0x55, and that fill is what makes the
  `$7F:E800+` block match the lsnes captures. Pads are one per video frame.
- `Start::State(blob)`: this emulator's own save state (`Oracle::save_state`).
  This is what harness-native seeds will be.
- `Start::Capture`: an lsnes room capture (`target/seeds/<capture>/NNNN-ROOM-*`),
  for migration only (`seed.rs`, `LEGACY(seed)`). Memories are copied, and the
  PPU, DMA and CPU I/O registers are replayed as register writes. The CPU
  resumes at the capture's boundary hook (`seed_boundary_pc`, `$80:8348`). The
  SPC700 has no captured state, so it resumes at the sound driver's idle loop.

The corpus runner (`corpus.rs`) maps the existing lock's Ticks onto video frames:

- a Tick is a requested frame, one whose NMI interrupted a wait site;
- the runner presents pad word k+1 until that NMI has run;
- the lock's streams fold on Ticks only.

## Ladders

A long emulator run is the exception. Each movie keeps a ladder of lsnes
states, a rung every N frames (10000 by default), so that a seed cut or a
measurement resumes from the nearest rung and runs for minutes
(`crates/oracle/src/ladder.rs`).

- `oracle movie ROM movies/NAME.inputs --oracle lsnes --ladder N DIR` saves
  rung F, for each multiple F of N the run passes, as `DIR/NAME/FFFFFF.lss`
  (the state before step F, as for native seeds) and `FFFFFF.record` (the raw
  record of step F-1). `DIR/NAME/ladder.txt` lists each rung's frame and the
  SHA-256 of its record. It works with `--from` and `--no-memories`; a rung's
  record always carries the memories, and so does a native seed's (so it can
  be compared with its lsnes room capture).
- `oracle ladder-check ROM INPUTS DIR [--rungs A..B]` resumes each rung whose
  frame lies in A..B, runs to the next rung and prints `equal` or the parts
  that differ, with byte counts. It exits 1 if any rung differs or fails.
- Reproduces means the record of the step before the next rung is
  byte-identical to the stored one: the cut (kind, interrupted PC, joypad
  reads, wait entry), the frame counter, the pad, the CPU and all five
  memories. The `.lss` bytes are not compared, because two runs that reach the
  same machine state can save different state bytes.
- `movie ... --native-seeds CSV DIR --from-ladder LADDERS` starts at the
  latest rung at or before the earliest seed's step. For any other frame,
  `oracle ladder-rung INPUTS LADDERS FRAME` prints `RUNG PATH`, for
  `movie --from PATH RUNG`.
- The nightly check runs each movie whole with `--ladder` into a fresh
  directory, compares its `ladder.txt` with the stored one, and runs
  `ladder-check` over the stored ladder.

## Movie sync

`oracle movie ROM movies/any.inputs --lag-log CSV` compares each frame with
the lsnes lag log (`any-boundary/lag.csv`): the frame counter `$05B6`, the NMI
counter `$05B8`, the lag flag, the game state, the room and Samus's position.

| Emulator | Exact through | State, room and position through | First difference |
| --- | --- | --- | --- |
| LakeSnes 9db90b8 | frame 352 | frame 352 | boot APU upload ends 12 lines early, frame 353 not lag |
| + fix 0001 (APU clock) | frame 395 | frame 8566 | one-frame NMI shifts around NMI re-enable in loads (396, 8314); Ceres desync from 8567 |
| + fix 0002 (bsnes v085 SPC rate, exact) | frame 395 | frame 17984 | one more lag frame on the door out of room 97B5 (17985); one frame behind from 18108, desync from 18587 |
| + fix 0004 (no HDMA overhead after every channel terminated) | frame 395 | frame 35199 | one frame late out of the door transition into room 9CB3 (35200); desync from 35202 |

Fixes live in `crates/oracle/native/fixes/NNNN-name.patch`. They are applied
to a copy of the pinned sources by `tools/oracle/build_lakesnes.sh` and named
in `OracleId::fixes`. They are emulator fixes, not ROM patches.

- `0001-apu-clock-real-master-rate`: the SPC at 32040 Hz against the real
  NTSC master clock (21 477 272 Hz), not 1364 · 262 · 60 dots.
- `0002-apu-clock-bsnes-v085-exact`: the SPC at bsnes v085's 32040.5 Hz, the
  rate the movies were recorded at, stepped as an exact integer ratio
  (MEASUREMENTS (h)).
- `0004-hdma-overhead-only-while-a-channel-runs`: no HDMA alignment or
  overhead on a line where every enabled channel has terminated for the
  frame, as anomie's timing doc and bsnes v085 have it (MEASUREMENTS (k),
  (l)). Upstream charged 12-18 clocks on every display line HDMAEN was set.

Tried and not applied: `native/candidates/0003-apu-ports-in-time-order.patch`
orders the CPU-SPC port accesses in time, as bsnes v085 does. It changes no
lag-log word in any% steps 0-18999 (MEASUREMENTS (j)), so it is not in
`fixes/`.
No `0005` either: none of the HDMA variants for the 35175 lag (line overhead
18, the request at dot 278, the alignment rules, the H/V-IRQ point per anomie)
zeroes the HUD-line pairs against lsnes, and the request at dot 278, the one
that turns 35175 into `Requested`, moves the boot from step 472
(MEASUREMENTS (n)).

## Proving it against the lock

`oracle corpus ROM scenarios/lock.toml DATA` runs every faithful entry whose
data is restored under DATA (one directory per ci-data bundle). It compares the
streams whose definition the lock shares, which `streams::legacy` computes:

- `wram` with the frozen `LOCK_WRAM_MASK`;
- `sram`, `vram`, `cgram` and `oam`;
- `ppu` from the same LakeSnes PPU fields, rebuilding snesrev's register
  caches and leaving its renderer scratch at zero.

`video` (snesrev's RGB) and `apu` (the native player's pending queue) cannot
be computed: they are redefined by the contract and emitted only in their v1
definition. `--record FILE` writes the new entries. The counts are in
docs/handoff/oracle-harness.md.

Expected, explainable mismatches:

- **Kept patches.** `$A0:896F` (the debug enemy-set names, WRAM
  `$7E:D552-D5F1` on every room load) and the native-audio handshake
  `$82:8A59/8A72/8A80` (immediate APU acknowledgement) change WRAM.
- **Door timing.** The door-transition spin removals (`$82:E400`... `$82:E540`)
  and the deferred message boxes (`$85:9670`, `$A2:AB22`, `$84:EFDC`) change the
  frame on which things happen.
- **The migration seeds' missing APU state.** In some rows the resumed driver
  answers no request, so the CPU's sound state differs, and uploads and door
  transitions stall.
- **`ppu`.** The old struct includes renderer scratch and latch fields.

### lsnes streams

lsnes records the picture, the CPU's APU port accesses and `PpuRegisters`
when asked (`Lsnes::record_streams`, the server's `T` request, before the
first step); otherwise its records are unchanged. The picture is lsnes's
screenshot of the video frame the step ends in (taken in `on_frame`, once
lsnes has handed that frame to its screen buffer), sent as lsnes's own PNG
(`save_png()`, base64) and mapped back from its 8-bit palette to BGR555:
exact at brightness 15, without per-line brightness or forced blank.
The port log has the CPU side only. The PPU registers are rebuilt from bsnes's
decoded fields in `memory.getregisters()`, without `nmitimen`, `htime`,
`vtime`, `hdmaen` and `memsel`. Against LakeSnes on any% 0-1000:
MEASUREMENTS.md (o), (r).

### The picture compared

Two pictures are equal when every dot shows the same 8-bit colour
(`Picture::shown`, the `picture` part of `oracle cross`):

- a dot's 5-bit components `c` at its line's master brightness `b` (0-15)
  show as `round((b + 1) * c * 255 / 496)` each (`record::shown_channel`,
  lsnes's lrgb palette for bsnes's dots);
- a dot on a forced-blank line shows black.

At brightness 15 this map is one-to-one, so equality is BGR555 equality.
Below 15 it is not: lsnes's screenshot cannot tell some components apart,
nor brightness 0 from 1 (level 1's values are all level 2's). The stored
BGR555 dots and brightness are kept as each backend has them (LakeSnes:
exact, per line; lsnes: the brightest level that produces the screenshot,
the smallest component for each value), and both show the same 8-bit
colours as the screenshot, so the comparison is exact where the picture is
observable and blind only where no screenshot could tell. The BGR555 dots
still differ there; `cross` prints both counts.

## Speed

One process with the picture on runs 236-366 frames/s (about 300) on the
seeded rows of this machine. Ten processes in parallel run about 95 frames/s
each, about 950 in total. For comparison, today's check-lock (engine replay
against the lock, single process, GitHub runner) is 133-156 ticks/s
(docs/PERFORMANCE.md). The old oracle could not be built on this
locked-down host, so there is no same-machine figure for it. The bridge's
step loop is C, with one Rust call per frame. Per-frame copies of the
memories (about 200 KB) and the tiers are the remaining overhead.

## Out of scope here

- The oracle RAM to engine State decoder (the architecture lane's schema).
- CI workflow changes (proposed in the handoff).
- Mesen and bsnes, which are GPL-3 and would run out of process.
