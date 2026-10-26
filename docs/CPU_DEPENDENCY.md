# Pinned external hardware libraries

## PPU operations and DSP synthesis

PPU registers, legacy pixel queries and DSP synthesis use unmodified
[angelo-wf/LakeSnes revision 28b8fbc](https://github.com/angelo-wf/LakeSnes/tree/28b8fbc15ad15afff003165ff4f2a8adb1c93d18).
Its PPU register state matches the existing interface.
The dependency adds 1,261 PPU and 685 DSP C/header lines, counted separately.
The maintained C bridge, Rust adapters,
installer and verifier changes are counted as project implementation.
Native PPU rendering, sprite evaluation, color math, scanline output, DMA,
scheduling, buses, snapshots and CPU hooks stay in Rust. This dependency replaces
shared native/oracle consumers; it does not replace the entire hardware backend.

Install with
`python tools/install_lakesnes.py backend --target x86_64-windows-gnu --zig <path-to-zig.exe> --library-directory <SDL-lib>`.
The installer checks every downloaded file against its pinned SHA256 and builds
`liblakesnes_backend.a`. Add its directory to `LIBRARY_PATH` alongside the CPU,
SPC, PNG and SDL libraries. Rebuild this archive after editing `sm/src/snes/backend.c`;
the verifier rejects an archive whose compiled bridge hash is stale. Include
[the pinned revision's MIT license](../THIRD_PARTY_NOTICES.md#lakesnes-ppu-dsp-and-spc700)
with distributed binaries.

PPU register calls use a temporary beam-position/open-bus view and restore the original
SNES pointer. The Rust adapter preserves line-zero mosaic setup and the native
renderer's complete routing bytes.
The shared state remains byte-compatible with existing snapshots.

Legacy pixel queries adapt one cell from the packed sprite buffer, then restore
both obsolete sprite-buffer bytes before returning. No callback observes that
temporary view. The actual pixel loop supplies x in 0..256; invalid modes fail
in Rust before foreign indexing. Native tile decoding preserves its different
VRAM wrapping, layer priorities and palette rules.

This pixel replacement removes 269 Rust lines and adds 13 C bridge lines:
**256 maintained lines removed**, with no added dependency source. It passes
85,814 actual old/new comparisons: 81,920 legacy pixel queries, 2,048 native tile
queries, 1,600 complete scanlines and 246 invalid-mode failures. Full PPU state
and output buffers match, including restored sprite bytes and native VRAM spill
cases. All scanline fixtures complete successfully. Other retained functions
match exactly; the native caller only drops constant arguments.

Probed and clean boot checks pass at 3/0, inputs 0..2, with the legacy renderer
selected in a local verifier host. That host uses the actual verifier/checkpoint
sources with module paths adjusted and sets `g_new_ppu = false` after `Game::new()`.
All comparisons, rendering and audio advancement remain enabled. Required pixel
and visible-output probes first execute at frame zero; two follow-up frames
observe continued rendering. These checks establish boot integration; broader
modes, color math, mosaics and unusual states have component coverage. No gameplay
or full-movie coverage is claimed. Normal release library/frontend/verifier builds
and embedded-source/native-bridge checks pass on the clean source.

DSP calls use a temporary view of the native sequencer's or emulated APU's RAM
and restore the original pointer before returning. The Rust adapter preserves
immediate KON/KOFF effects, alternating-cycle state and stopping sample writes
at 534. Register reads, allocation, snapshot callbacks and output resampling
retain their Rust operations. Invalid envelope/FIR indices still fail in Rust
after the same partial voice updates; the C bridge checks those indices before
the dependency could access them. Actual callers own disjoint DSP and APU RAM
allocations; no aliasing between them is introduced or supported.

The complete DSP replacement removes 478 Rust lines and adds 50 C bridge and
three installer lines: **425 maintained lines removed**. The final code passes
66,590 old/new full-state, APU-RAM, failure and PCM comparisons, including all
register/value pairs, BRR filters/ranges, echo wrapping, envelope states, key
events and sequential sample generation. Native and physical-audio boot segments
each pass probed and clean at 3/0, inputs 0..2. Required cycle, full-buffer,
resampling and consumer probes first execute at frame zero. These short boot
segments establish integration, not gameplay/music coverage; active synthesis
and unusual states have component coverage. Normal release library, frontend
and verifier builds pass.

For Linux x86-64, use the same installer with `--target x86_64-linux-gnu`.
The installer supplies PIC flags and the bridge's source stamp. Choose a separate
native-library directory for each target; installation alone does not verify runtime behavior.

The separate SPC interpreter dependency is documented in
[SPC_DEPENDENCY.md](SPC_DEPENDENCY.md).

## CPU instructions

The CPU adapter links the unmodified MIT-licensed CPU and state handler from
[dinkc64/LakeSnes](https://github.com/dinkc64/LakeSnes/tree/048a0d72568668f74e0cdf371dcd9e8efd614965),
revision `048a0d72568668f74e0cdf371dcd9e8efd614965`. This fork last changed in
April 2025; its CPU last changed in November 2023. It is independently published
external code, not the project's CPU moved outside the maintained source count.

Install a normal Zig C toolchain and run
`python tools/install_lakesnes.py cpu --target x86_64-windows-gnu --zig <path-to-zig.exe> --library-directory <SDL-lib>`.
The script fetches four unmodified pinned files, verifies SHA256 and creates
`liblakesnes_cpu.a`. No C
behavioral glue or modified external source is used. The external closure has
2,836 C/header lines, disclosed separately from every maintained Rust adapter
and installer line. Include [the upstream MIT license](../THIRD_PARTY_NOTICES.md#lakesnes-cpu)
with distributed builds.

Install the uniquely named archive alongside SDL in its configured native-library
directory, then use the normal Cargo release build with the existing SDL
`LIBRARY_PATH`. Rust leaves this external static archive unbundled, and the final
linker resolves it through that library path. For a separate prefix, include its
`lib` directory in `LIBRARY_PATH`. No Cargo build script or new Rust dependency
is required. A missing library must fail the ordinary link.

For Linux x86-64, use the same checked sources through the Python installer:

```sh
python tools/install_lakesnes.py cpu --target x86_64-linux-gnu --zig /path/to/zig --library-directory /path/to/native/lib
```

The installer's `SOURCES` table records every upstream file checksum. Use
`--prefix` to choose a source cache; retain the license notice with the archive.
Build and replay for the selected target before treating it as validated.

The existing Rust CPU layout, legacy saves, hardware bus, oracle hooks and
outer interrupt sequencing remain. Foreign raw registers are copied per
instruction; no foreign state survives a snapshot. The adopted source passes
524,288 instruction and 4,096 lifecycle comparisons, plus full boot any% and
Speed Booster replays with zero failing frames. Neither route reaches credits.

The real CPU bus does not observe `cyclesUsed` during an instruction. Its final
value and return match; intermediate callback-cycle byte identity is not promised.
