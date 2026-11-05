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
  src/streams.rs          per-stream digests, legacy and v1, and the running fold
  src/lock.rs             reading scenarios/lock.toml pins and entries
  src/inputs.rs           .inputs files
  src/seed.rs             lsnes room captures (migration seeds)
  src/cli.rs              the command line
  src/bin/oracle.rs       the binary around cli::main
tools/oracle/build_lakesnes.sh   pinned fetch and Zig build of the native archive
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

## Inputs and starts

- `Start::PowerOn` with a row's `.inputs` file (one pad word per tick, as
  today's runner reads them).
- `Start::Seed`: today's lsnes seed directories (`target/seeds/<capture>/`)
  are loaded into the emulator field by field. They lack SPC700/DSP/timer
  state, so a seeded APU starts from a reconstructed state; this is reported,
  not hidden. The durable answer is harness-native seeds: the raw record plus
  the emulator's own save state, cut by playing the movies from power-on.

## Proving it against the lock

Streams whose definition the existing lock shares (`wram` with its frozen
`LOCK_WRAM_MASK`, `sram`, `vram`, `cgram`, `oam`) are compared directly.
Streams the lock defines by old internals (`ppu`: LakeSnes and snesrev struct
fields including renderer scratch; `video`: snesrev's RGB; `apu`: the native
player's pending queue) are emitted in both the old definition (where it can
be computed) and the new one, and reported separately. Every mismatch is
classified (frame-stepper sample point, kept patch, seed without APU state,
emulator difference).

## Speed

Measured as ticks/s on the same machine against today's oracle path, with and
without picture capture. The bridge's step loop is C; Rust sees one call per
tick.

## Out of scope here

The oracle RAM to engine State decoder (the architecture lane's schema), CI
workflow changes (proposed in the handoff), and Mesen.
