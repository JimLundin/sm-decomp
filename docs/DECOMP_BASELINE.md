# First executable C baseline

The preparation increment now has a headless executable reference. Run
`just decomp-baseline 9000` to verify the frozen source archive and extracted
files, compile the actual C game, replay the existing any% inputs twice in fresh
processes, and write an independent `decomp-c-baseline-v1` verdict beneath
`out/decomp/baseline/`. Build/replay errors and timeouts preserve their logs and
fail the command. ROM-derived states and fixtures remain ignored.

## Source decision

Use snesrev/sm at `578f90b3cc49557bb70060ad033bb90b8cf8ac50` for the
initial integration reference and decompression oracle. A fresh upstream ref
check on 2026-10-10 confirmed that revision still heads main. The driver checks
the archive SHA-256 from `tools/dependencies.json` and every extracted file;
the source remains unmodified. Each run records compiler, source, host, binary,
ROM and input digests and retains the MIT/third-party notice. It builds the bank
sources, runtime, SNES hardware and real SPC player with GCC 12.2.0, GNU C11,
`-O2 -fno-strict-aliasing`; unused GUI code is discarded at link time. SDL
development headers are not needed.

Reviewed PR #50's retained diff: its behavioral correction initializes Kraid
explosion scratch state; it does not change decompression. Do not import that
fix implicitly. The rewrite fork and C# port remain supplementary references.

The inspected C# revision is `216f3a1d38c2e9d82ec9c0e626a35415b74d47c9`;
fresh upstream HEAD has advanced to `41bd9068973836538f2ebecc3df19723266b0176`.
This increment does not assert that the newer code was reviewed or tested.
The inspected replay supports LSMV parsing as well as SMV, but conversion
requires native traces of completed main-loop dispatches, NMI continuations,
controller reads, message-box boundaries and APU waits. It preserves input
edges and intentional waits, retaining a source-frame map for removed hardware
stalls. That approach is applicable as a design for our movie adapter; the
existing video-frame WRAM capture is insufficient to reproduce it directly.
No .NET SDK is installed (`net10.0` is required), no matching update-aligned
capture has been generated for our Sniq any% movie, and no project copying
permission was located at the inspected pin. No C# implementation was imported.
These concrete limits favor the locally runnable MIT C reference for this
bounded increment.

## What the replay establishes

The retained run `20261010T175516.246893Z` completed two 9,000-frame processes.
Progress logs, fallback/decompression logs and complete terminal WRAM/SRAM
matched. Terminal C game state was 8 (main gameplay), room `$DF45`, Samus at
`(211, 171)`, RNG 52931. It observed 23 RAM-decompression calls and zero
C-to-CPU fallback instructions. All 46 before/after WRAM fixtures also matched
between processes. The verdict records their SHA-256 digests. A missing-input
negative control (`20261010T175542.826745Z`) returned a failing error verdict;
it could not become a successful replay.

The host selects upstream `RM_MINE`, never its normalized dual-run comparator.
No reference snapshots are restored between frames. The only input adaptation
is reversing the 16 pad bits to match the upstream host convention. No lag frames
are removed and no corrective gameplay state is injected. Initial SRAM is zero;
upstream reset and ordinary SRAM writes run normally in a run-local directory.
The real PPU receives a pixel buffer, fixing the first-frame crash caused by
omitting the GUI's `PpuBeginDrawing` initialization. Audio queuing/upload and
SPC initialization remain real; no audio output thread runs. Playback/audio
parity and save-load lifecycle are not validated here.

The original-ROM side is **not compared** in this command. Upstream `SnesInit`
patches its in-memory ROM, and the C scheduler removes original waits. Raw
video-frame inputs need a verified gameplay-update adapter before this can
claim adapted TAS progress. Completing 9,000 host iterations proves a runnable,
deterministic candidate, not 9,000 original-equivalent updates or movie completion.
Rust ownership remains zero.

Linker wrappers report `Call` fallback entries, fallback CPU instructions, and
cross-translation-unit `DecompressToMem` calls. The existing fallback entry is
`Call` -> `RunAsmCode`; wrapping `Call` matters because the call to `RunAsmCode`
is within the same C translation unit. Zero observed calls in this prefix does
not establish that later game paths are translated. Same-unit decompression
calls are outside this wrapper's inventory. Fixture snapshots report routine
boundary state; they do not yet observe transient writes.

## Replacement seam and next increment

Keep the headless C scheduler and hardware host. The first live seam will replace
`DecompressToMem` through the existing linker wrapper, calling a Rust static
library with an explicit C ABI. C owns the single `g_ram` byte array; Rust borrows
a bounded destination/WRAM view for the duration of the call and receives a
validated immutable ROM view. The adapter must return explicit errors, measure
Rust hits, and propagate errors without executing the original routine afterward.
Offline C/Rust equivalence must pass before enabling that live seam.

Next implement the decompression job's safe ROM/numeric interface, routine
manifest and differential fixture command. Compile the actual pinned C routine,
compare full destination plus surrounding state, cover all compression modes and
bank crossing with independent synthetic expectations, replay captured real-ROM
calls, and prove wrong-output, transient-write and unexecuted replacements fail.
Only then connect the live Rust replacement and compare independent C/mixed
replays. The new baseline is preparation evidence, not an offline translation
acceptance gate or the launch of a parallel translation queue.
