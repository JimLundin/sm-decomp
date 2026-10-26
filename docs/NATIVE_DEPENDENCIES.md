# Native dependencies

Read this when installing or changing the retained native backend or desktop
adapters. The finished engine's destination remains the
[accepted architecture](target-architecture/README.md); these dependencies support
the current runtime and oracle. Maintained adapters, installers and callers count
as project code; fetched dependency code is counted separately.

## Installation

Use the shared installer with an existing Zig compiler and matching SDL2 headers
and libraries. Keep a separate cache and native-library directory for each target:

```text
python tools/install_native_dependencies.py --target x86_64-windows-gnu --zig <zig.exe> --prefix <source-cache> --sdl-include <SDL-include/SDL2> --library-directory <native-lib>
```

Keep `<native-lib>` in `LIBRARY_PATH` for Cargo and direct verifier builds.
The installers verify pinned source/license hashes. Static archives remain
unbundled in Rust rlibs and resolve at final link; a missing archive fails linking.
Install [PNG decoding](PNG_DEPENDENCY.md) separately for the frontend.
Distribute [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) with native archives
and binaries; retain upstream licenses with cached source.

For Linux x86-64, select `--target x86_64-linux-gnu`. The shared installer supplies
PIC builds and omits desktop: that bridge requires Windows' 32-bit `long` and
does not support Linux LP64. A Linux configuration/shader archive does not establish
a working Linux frontend. Build and replay for the selected target; installation
and historical Windows comparisons alone do not validate it.

For a scoped rebuild, invoke the corresponding installer with the same target,
Zig, library-directory and optional prefix arguments:

| Component | Installer | Archive | Additional argument |
| --- | --- | --- | --- |
| CPU | `tools/install_lakesnes.py cpu` | `liblakesnes_cpu.a` | |
| PPU/DSP | `tools/install_lakesnes.py backend` | `liblakesnes_backend.a` | |
| SPC700 | `tools/install_lakesnes.py spc` | `liblakesnes_spc.a` | Explicit `--library-directory` required |
| Native music | `tools/install_native_music.py` | `libsm_music.a` | `--emit-only` inspects generated source/stamps |
| Configuration, utilities, shaders | `tools/install_native_config.py` | `libsm_config.a` | `--sdl-include` |
| Desktop | `tools/install_native_desktop.py` | `libsm_desktop.a` | `--sdl-include`; Windows GNU x64 only |

Rebuild an archive when its maintained bridge, shared layout or installer inputs
change. Embedded-source checks reject stale archives before use. Shader and desktop
checks include shared installer/OpenGL/callback inputs, so their changes can require
rebuilding both configuration and desktop archives. The headless verifier links
utilities, not the configuration-input, shader or desktop implementations.

## Source pins

The installers own the per-file SHA256 tables. The different LakeSnes revisions
are deliberate compatibility choices; do not unify or update them without renewed
behavior comparisons.

| Implementation | Upstream revision | External C/header lines at adoption | License notice |
| --- | --- | ---: | --- |
| CPU and state handler | [dinkc64/LakeSnes `048a0d7`](https://github.com/dinkc64/LakeSnes/tree/048a0d72568668f74e0cdf371dcd9e8efd614965) | 2,836 | [MIT](../THIRD_PARTY_NOTICES.md#lakesnes-cpu) |
| PPU and DSP | [angelo-wf/LakeSnes `28b8fbc`](https://github.com/angelo-wf/LakeSnes/tree/28b8fbc15ad15afff003165ff4f2a8adb1c93d18) | 1,261 PPU; 685 DSP | [MIT](../THIRD_PARTY_NOTICES.md#lakesnes-ppu-dsp-and-spc700) |
| SPC700 | [elzo-d/LakeSnes `1da16a2`](https://github.com/elzo-d/LakeSnes/tree/1da16a22db4d0f2e7f6960a9846ef4e1f3acbd29) | 2,104 | [MIT](../THIRD_PARTY_NOTICES.md#lakesnes-ppu-dsp-and-spc700) |
| Music, config, utilities, shaders, desktop | [snesrev/zelda3 `45a149d8`](https://github.com/snesrev/zelda3/tree/45a149d8b4bb8098f53d1009f76e76e7d8b79bbf) | Shared headers; see historical accounting below | [MIT](../THIRD_PARTY_NOTICES.md#snesrevzelda3) |

LakeSnes inputs are unmodified; the CPU uses four pinned files without behavioral
C glue, and SPC compiles `spc.c` with nine transitive pinned project headers.
Zelda3 routines are selected and explicitly patched by maintained installers;
current Rust behavior defines those compatibility changes. Original Super Metroid
C is not a regeneration source.

## Hardware compatibility

### CPU

Rust retains the CPU layout, legacy saves, hardware bus, oracle hooks and outer
interrupt sequencing. Foreign raw registers are copied per instruction; no foreign
state survives a snapshot. The real bus does not observe `cyclesUsed` during an
instruction: its final value and return match, but intermediate callback-cycle
byte identity is not promised.

### PPU and DSP

Rust retains native rendering, sprite evaluation, color math, scanlines, DMA,
scheduling, buses, snapshots and CPU hooks. Shared native/oracle consumers use the
dependency; this does not retire the backend. Rebuild the backend archive after
editing `sm/src/snes/backend.c`; its compiled bridge hash is checked.

PPU register calls temporarily supply beam position/open bus and restore the SNES
pointer. Preserve line-zero mosaic setup, complete native routing bytes and snapshot
layout. Legacy pixel queries adapt one packed sprite cell, then restore both obsolete
sprite-buffer bytes; no callback observes that temporary view. The actual loop uses
x in `0..256`; invalid modes fail in Rust before foreign indexing. Native decoding
retains its distinct VRAM wrapping, layer priorities and palette rules.

DSP calls temporarily borrow the native sequencer's or emulated APU's RAM and restore
the pointer. Preserve immediate KON/KOFF effects, alternating-cycle state and sample
writes stopping at 534. Rust retains register reads, allocation, snapshot callbacks
and resampling. Invalid envelope/FIR indices fail after the same partial voice
updates; bridge checks precede foreign indexing. Actual callers own disjoint DSP
and APU-RAM allocations; aliasing between them is not supported.

### SPC700 and native music

The SPC pin preserves instruction semantics/timing. Rust retains `Spc` layout,
allocation, reset read order, 15-byte legacy save span, APU bus, timers, DSP, native
sound driver and snapshots. Offset checks guard the layout. Foreign `Apu` is opaque:
SPC calls two Rust bus exports and never reads the differing foreign header layout.
`C-unwind` preserves callback failure behavior.

Native music is independent of uploaded SPC bytes. `sm/src/native_music.h` matches
`NativePlayerState`, the first `SpcPlayer` field. Its RAM pointer refers to the separate
`ApuRam` owner shared with DSP; snapshots copy bytes, not ownership. Old frozen Rust
libraries need their matching archive with inline player RAM. Rust retains SFX,
scheduling, allocation, upload, snapshots and shared primitives.

Preserve native tables, pitch envelopes, tremolo, fade completion, song commands,
busy-channel ordering and `cutk`. Invalid-effect lookup and unbounded keyoff scans
still fail through Rust, with preceding state changes intact. Compile C with exception
unwinding, signed wrapping and strict aliasing disabled. Previously unchecked raw
FIR/table behavior remains outside malformed-input comparison claims.

## Frontend compatibility

### Configuration and utilities

Keep SM command numbers, defaults, filenames, accepted options and the shared
`g_config` layout. Unknown Zelda options stay unknown; unsupported OpenGL ES selects
SDL. Duplicate keyboard entries consume slots; equally specific gamepad entries
prefer newest first. Preserve allocation thresholds, file-buffer/pointer lifetimes,
partial allocation state, failure order and aliased string replacement. Rust checks
invalid gamepad indices. `Die` may return or unwind: foreign declarations must permit
both, and builds retain exceptions and signed wrapping. Explicit casts preserve
native `c_ulong` file-read results and `c_long` comment-span lengths.

Recorder buffers now use owned `recording.rs` storage; the four former ByteArray
functions/declarations/layout checks are retired. Logical shrink retains initialized
bytes, including short-read and replay behavior. Do not restore that obsolete native
selection while updating configuration or music.

### Desktop

Rust retains `GameOwner`, game/SNES layouts, both pixel buffers, verifier entry,
diagnostics and the OpenGL presenter. Input, frame, presentation and audio operations
retain their owner gate; audio takes it before the SDL audio mutex. Internal callbacks
allow unwinding; SDL audio entry does not. Renderer callbacks omit the unused Game
argument while their caller still holds the gate.

Preserve startup/shutdown and early-failure order, SM filenames/commands/settings,
forced 44,100 Hz stereo request, obtained-frequency block sizing, native pixel-copy
dimensions, turbo cadence, bug-title updates, ordinary PauseDimmed toggle, live config
reads and Windows pointer-difference narrowing. Rust supplies saturating FPS conversion
and bounds/null-callback failures. Disable floating-point contraction. OpenGL allocation,
viewport and cleanup algorithms retain their behavior.

### Shaders

The existing GL loader supplies 42 pointers without extra SDL queries; fallback order
and reload follow generated bindings. Rust retains source vectors, PNG loading,
temporary pixel buffers, CString allocation/release, saturating dimensions and bounds
failures. Preserve presenter allocation, drawing and cleanup behavior.

Keep comments before version directives, live config reads, unsigned parameter locations,
the one-based 20-slot boundary, zeroed status/log outputs, assertions and GL-call order.
Recursive file/path cleanup uses C scope cleanup with `-fexceptions`. Failed creation
retains raw-root ownership; a Rust guard separates GL destruction from owned-memory
release. Disable floating-point contraction. Representation changes alter allocation
sizes/counts; allocator event identity is not promised. The non-unwinding GL ABI does
not promise recoverable missing-function panics or native GL unwinding.

## Historical verification and accounting

These are adoption results, not a claim that today's tree passed these experiments.
Current work follows [PLAN](PLAN.md) and the named [replay rows](../movies/README.md).
The source-stamped comparisons include failure prefixes and full relevant state;
their coverage limits remain important when changing an adapter.

| Component | Recorded comparisons | Integration and limits |
| --- | --- | --- |
| CPU | 524,288 instructions; 4,096 lifecycle cases | Boot any% and Speed Booster replays; neither reached credits. |
| PPU pixels | 85,814: 81,920 pixel queries, 2,048 native tiles, 1,600 scanlines, 246 invalid-mode failures | Full state/output and restored sprite bytes, including native VRAM spill. Legacy-renderer boot inputs 0..2 passed probed/clean; no gameplay/full-movie claim. |
| DSP | 66,590 full-state, RAM, failure and PCM comparisons | Registers, BRR, echo, envelopes, key events and sequential synthesis; native/physical-audio boot inputs 0..2 passed probed/clean. No gameplay/music claim from those boot windows. |
| SPC700 | 528,395 cases; 2,028,312 ordered bus events | Opcode/flag combinations, wrapping, timers, self-modifying code, failures, reset/saves. Three boot frames: 11,572 SPC instructions, 1,602 DSP advances; no gameplay/credits. |
| Music | 346 cases, 416 DSP writes, two matching failures; 154 layout checks | Synthetic track: 48 buffers, 5,590 writes, 25,632 stereo samples (38,434 nonzero values). Boot inputs 0..178: notes/effects at 173, PCM 174..178; echo component coverage, no gameplay/physical-SPC/credits. |
| Config/utilities | 31 scenarios; 5,463 Boolean cases; 263,438 input lookups; 133 utility cases; 32 layout and 35 stamp checks; five bounds and three allocation/limit failures | Real small files plus controlled Windows read narrowing; large comment span only source/width checked. Unchecked invalid allocation behavior not exercised as valid input. No movie/GPU/physical-controller claim. |
| Desktop | 125 full old/new host scenarios; 33 installed-archive checks | Real owner gate with controlled SDL/filesystem/game boundaries; single-threaded scope checks, no movie/device/concurrency stress claim. |
| Shader | 38 scenarios; 46 GL-loader cases; four allocation failures; 22 source checks and five normal-library shader comparisons | GL/SDL controlled. Candidate destructor fault checks heap guard only; no physical GPU, native GL-unwind, concurrency or gameplay claim. |

Failed/underpowered trials are excluded: SPC's 1,000-frame trial missed its PPU-read
probe; music's initial quiet synthetic sample and 2,000-frame trial missing echo were
not coverage passes. The successful bounded music replay used note/effect/PCM probes.
Historical local SPC/music comparisons used
`Local/sm-decomp-experiments/20261021/complete-backend/spc` and
`Local/sm-decomp-experiments/20261021/patched-native-music`.

Adoption net maintained-line reductions were PPU pixels 256, DSP 425, music 366,
config/utilities 541, desktop 697 and shaders 282. These include their then-current
adapters/installers/callers and are not additive claims about later work. Zelda3
inputs then comprised music 1,659 lines (1,447 sequencer, 103 types, 109 DSP registers),
configuration 529 C + 110 header + 280 utility C, desktop 882 C, shader 659 C + 99 header;
shared types count once. Generated selections were music 553 and desktop 720 lines.
The full experiment narratives and exact historical accounting remain in
[Git history at 299cfceb](https://github.com/JimLundin/sm-decomp/tree/299cfcebe7b77a7d42e175d5e2706defdf7de5a2/docs).
