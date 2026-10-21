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
`tools/install_lakesnes_spc.ps1 -Zig <zig.exe> -LibraryDirectory <SDL-lib>`.
The installer verifies every downloaded source and license with SHA256,
then creates the uniquely named `liblakesnes_spc.a`. Keep its directory in
`LIBRARY_PATH` together with the existing CPU, codec and SDL dependencies.
No existing library is replaced. The installer adds 44 maintained lines.

PowerShell script execution is disabled on the current machine. The installer
was syntax-checked but not executed; the archive was compiled and installed
with ordinary Zig commands using the same checksum-verified sources:

```powershell
$spcSource = 'C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261021/complete-backend/spc/upstream'
$spcOutput = 'C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261021/complete-backend/spc'
$spcZig = 'C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/zig-x86_64-windows-0.15.2/zig.exe'
& $spcZig cc -target x86_64-windows-gnu -O2 -std=c11 -c "$spcSource/spc.c" -o "$spcOutput/spc.o"
& $spcZig ar rcs "$spcOutput/liblakesnes_spc.a" "$spcOutput/spc.o"
```

After checking both commands succeeded, copy that archive into the existing
SDL native-library directory. The installed archive must match the tested one.
On Unix, compile the same pinned source with `cc -O2 -std=c11 -fPIC`, archive
it with `ar`, and put the result on the native library search path. Unix builds
have not been tested.

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
