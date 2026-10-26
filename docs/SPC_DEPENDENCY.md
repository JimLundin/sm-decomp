# Pinned SPC700 dependency

The SPC700 interpreter uses the unmodified MIT-licensed implementation from
[elzo-d/LakeSnes at 1da16a2](https://github.com/elzo-d/LakeSnes/tree/1da16a22db4d0f2e7f6960a9846ef4e1f3acbd29).
This historical revision matches the current engine's instruction behavior.
Later revisions change instruction semantics and timing; updating this pin
requires repeating the comparisons against maintained Rust.

Only `spc.c` is compiled. Its nine transitive project headers are also pinned:
**2,104 external C/header lines** in total. They are independently published
code, not project implementation moved outside the count. Keep
[the 2021 MIT license](../third_party/lakesnes/LICENSE-SPC-2021.txt) with builds.
The CPU dependency continues using its separate, existing revision and license.

The Rust `Spc` layout, allocation, reset read order, 15-byte legacy save span,
APU bus, timers, DSP, native sound driver and snapshots remain owned here.
Compile-time offset checks guard the foreign layout. The dependency treats
its `Apu` pointer as opaque and calls the two exported Rust bus functions;
it never accesses the foreign header's different `Apu` layout. `C-unwind`
preserves Rust failure behavior through those callbacks.

On Windows, install using
`python tools/install_lakesnes.py spc --target x86_64-windows-gnu --zig <zig.exe> --library-directory <SDL-lib>`.
The installer verifies every downloaded source and license with SHA256,
then creates the uniquely named `liblakesnes_spc.a`. Keep its directory in
`LIBRARY_PATH` together with the existing CPU, codec and SDL dependencies.
The `spc` component requires an explicit `--library-directory`.

For Linux x86-64, select `--target x86_64-linux-gnu` and its own native-library
directory. The installer adds PIC flags. Use `--prefix` to select the source
cache; build and replay for that target before treating it as validated.

Validation compares actual old/new instructions, the entire initialized
APU/DSP memory and register state, cycles and ordered bus events. All opcode
values and flag combinations cover arithmetic limits, zero division, direct-page
and PC wrapping, stack edges, timers, ports, DSP writes, stopped execution and
sequential self-modifying code. Invalid DSP-history cases also compare panic
and preceding state. These are constructed checks, separate from movie coverage.

The final 528,395 cases and 2,028,312 ordered bus events pass, including reset
and legacy saves. All 84 embedded baseline library sources, the actual old/new
interpreter and test source were checked. Compiler layout checks and published
source/archive hashes also pass. Normal release library/frontend/verifier builds
and a probed/clean physical-audio boot segment pass: inputs 0..2, three frames,
zero differences, 11,572 SPC instructions and 1,602 DSP advances. Other instruction
branches have component coverage; gameplay and credits are not exercised.

The isolated fresh Cargo build failed to launch the `khronos_api` dependency
build script (OS error 5). Direct optimized builds using existing GL/SDL artifacts
passed, followed by the normal shared-checkout Cargo release build. No blocked
script was executed and no security setting changed. The earlier combined
1,000-frame run failed its absent PPU-read probe; it is not a coverage pass.
The short segment requires the observed SPC/audio probes; PPU reads are checked
separately by the PPU change. Local reproduction inputs and logs are under
`Local/sm-decomp-experiments/20261021/complete-backend/spc`.

## Native music sequencer

The native music engine uses selected routines from
[snesrev/zelda3 at 45a149d8b4bb8098f53d1009f76e76e7d8b79bbf](https://github.com/snesrev/zelda3/tree/45a149d8b4bb8098f53d1009f76e76e7d8b79bbf).
Keep the [Zelda3 MIT notice](../third_party/zelda3/LICENSE.txt) with distributed
binaries. This dependency is separate from the SPC interpreter above: native
music behavior remains fixed independently of uploaded SPC instruction bytes.

On Windows, install the required `libsm_music.a` with Python and Zig:

```powershell
python tools/install_native_music.py --target x86_64-windows-gnu --zig <zig.exe> --library-directory <SDL-lib>
```

Keep the output directory in `LIBRARY_PATH` for both Cargo and the verifier.
The default source cache is `%LOCALAPPDATA%/sm-decomp-tools/native-music-45a149d8`;
`--prefix` overrides it. Reinstall after changing the C adapter, layout header
or compatibility installer. This Python installer was executed successfully.
It verifies every upstream source/license checksum, selects named music
routines, applies explicit compatibility changes and compiles the maintained
adapter. The verifier checks the installed archive's C/header/installer stamp
as well as the usual compiled Rust sources. No generated implementation is
stored outside the count: selection and patch code are maintained in the installer.

`sm/src/native_music.h` follows Rust's `NativePlayerState` layout, which is the
first field of `SpcPlayer`. Its RAM pointer refers to a separate `ApuRam` owner;
snapshots copy bytes without copying that owner. The DSP uses the same allocation.
Rebuild the native archive after this layout change; old frozen Rust libraries
still require the matching archive with inline player RAM. The replacement
keeps SFX, scheduling, allocation, upload, snapshots and native shared primitives
in Rust. Compatibility changes preserve native tables, pitch envelopes, tremolo,
fade completion, song commands, busy-channel ordering and the `cutk` flag.
The invalid-effect lookup and unbounded keyoff scan still fail through their
Rust implementations, including preceding state changes. C is compiled with
exception unwinding, wrapping signed arithmetic and strict aliasing disabled.
Raw FIR/table accesses that were unchecked in Rust remain unchecked; malformed
inputs reaching undefined raw-memory behavior are outside the comparison claims.

The complete change removes 678 lines from the sequencer module and adds ten
other Rust lines, 37 C lines, 101 header lines, 154 installer lines and ten
verifier-launcher lines: **366 fewer maintained lines**. The complete fetched
dependency is **1,659 external C/header lines**, including 1,447 sequencer,
103 types and 109 DSP-register lines. Only selected music routines compile;
the generated selection is 553 lines, with project modifications charged above.

Validation compares 346 actual old/new cases over full player/RAM state and
416 ordered DSP writes, including two matching failures. All 154 shared-layout
sizes and field offsets match. A synthetic continuous track exercises the
installed archive, native scheduler and DSP for 48 buffers: full state, 5,590
ordered writes and 25,632 stereo samples match, with 38,434 nonzero sample values.
The initial synthetic sample was too quiet after integer volume scaling; its
silent run is excluded from active-synthesis coverage.

Normal release library/frontend/verifier builds pass. Probed and clean native
boot each pass 179 frames, inputs 0..178. Notes and effects first execute at 173;
nonzero PCM is observed from 174 through four follow-up frames. Inputs 0..172
are setup. Echo initialization occurs outside the measured movie-frame window
and has component coverage. The first 2,000-frame run missed its echo probe and
is not a coverage pass; the bounded rerun used the observed note/effect/PCM
probes. An initial verifier import failure was fixed for isolated Python mode
before either successful replay. These checks establish no gameplay, physical
SPC or credits coverage. Local comparisons are under
`Local/sm-decomp-experiments/20261021/patched-native-music`.
Current source stamps pass; altered installer, C and header inputs are rejected.
The tested and final installed archives have identical code, data, relocations
and non-debug symbols; only discardable CodeView records differ.

For Linux x86-64, use the same music installer with `--target x86_64-linux-gnu`.
It preserves the source selection and compatibility changes and supplies PIC
flags. `--emit-only` is available for inspecting the generated sources and stamps.
The Windows comparisons above do not establish Linux runtime behavior.
