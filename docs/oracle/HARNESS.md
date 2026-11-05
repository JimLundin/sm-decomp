# The oracle harness

The oracle harness runs the original, unmodified ROM on an emulator and records
what the new engine is compared against. It is a new crate, `crates/sm_oracle`,
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

The first backend is **upstream LakeSnes** (angelo-wf/LakeSnes `9db90b8`, MIT),
fetched by pinned SHA-256 and compiled with the Zig toolchain the project
already uses. Upstream sources are not edited: the harness's own C bridge
(`crates/sm_oracle/native/bridge.c`) interposes at link level (the CPU's
read/write callbacks, and `-D` renames of `ppu_runLine`, `ppu_write`,
`ppu_handleVblank` and the DMA B-bus calls) to observe the machine. The old
Rust copies of the CPU/SPC/PPU are not reused: they are entangled with the
old Game and snesrev's renderer.

## Crate layout

```text
crates/sm_oracle/
  build.rs                fetch check + zig compile of LakeSnes and the bridge
  native/bridge.c         observation hooks and the per-instruction step loop
  src/lib.rs              the Oracle trait, Start, FrameRecord, Tiers
  src/record.rs           FrameRecord and its parts (machine-domain types)
  src/ppu_registers.rs    the architectural PPU/CPU-IO register struct
  src/digest.rs           per-stream running digests (sm_data::digest fold)
  src/lock.rs             reading scenarios/lock.toml entries, comparing streams
  src/inputs.rs           .inputs files and seed directories
  src/lakesnes.rs         the LakeSnes backend (FFI)
  src/bin/sm_oracle.rs    CLI: run a row, record or check its lock, measure speed
tools/oracle/fetch_lakesnes.sh   pinned download of the upstream sources
```

Dependencies: `sm_data` for the shared digest only; no crates.io dependency.

## The backend trait

```rust
pub trait Oracle: Sized {
    /// Who produced the record: emulator, version, patch set (always empty).
    fn identity(&self) -> OracleId;
    /// Power on with the cartridge, or resume a seed.
    fn open(rom: &[u8], start: Start) -> Result<Self, OracleError>;
    /// Run to the next commit cut with `pad` on controller 1.
    fn step(&mut self, pad: Pad) -> &FrameRecord;
    /// The machine at the last cut, as a resumable seed for this oracle.
    fn seed(&self) -> Seed;
    /// Select the optional observation tiers.
    fn tiers(&mut self, tiers: Tiers);
}
```

A second backend (Mesen, #250) implements the same trait; a disagreement
report steps both with the same pads and diffs records stream by stream.

## The frame record (the architecture lane's oracle contract v1)

- **Frame boundary, the commit cut:** the RTI that ends an NMI in which the main
  thread had requested the frame. On this emulator the bridge sees the NMI
  vector fetch, the interrupted PC and the stack depth, and cuts after the
  matching RTI. An NMI whose interrupted PC is not a known wait site is
  reported as an overrun; an NMI that does not read the joypad is a lag NMI
  and is folded into the record, not hidden.
- **Raw state at the cut:** CPU registers, WRAM 128K, VRAM 64K, CGRAM 512B,
  OAM 544B, SRAM, the architectural PPU registers (raw register values, no
  latches or renderer scratch) and the DMA channel registers.
- **The picture scanned out after NMI k:** 256x224 (or 239 with overscan), in
  PPU-domain 15-bit BGR after colour math, with per-line brightness and
  forced blank. The bridge renders each line at brightness 15, so LakeSnes's
  8-bit expansion is exactly invertible, and records the real brightness.
- **APU CPU side:** ordered port writes and reads `(port, value)` since the
  previous cut, from both the CPU and DMA.
- **Facts:** frame index, video frames elapsed, NMI kinds with interrupted PCs,
  pad word auto-read, overscan.
- **Optional tiers:** SPC RAM + DSP registers + SPC700 registers at the cut,
  audio samples, per-line register writes.

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
