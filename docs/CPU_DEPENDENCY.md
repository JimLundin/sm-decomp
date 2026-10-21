# Pinned external hardware libraries

## PPU registers and DSP synthesis

The PPU register and DSP synthesis engines use unmodified
[angelo-wf/LakeSnes revision 28b8fbc](https://github.com/angelo-wf/LakeSnes/tree/28b8fbc15ad15afff003165ff4f2a8adb1c93d18).
Its PPU register state matches the existing interface.
The dependency adds 1,261 PPU and 685 DSP C/header lines, counted separately.
The maintained C bridge, Rust adapters,
installer and verifier changes are counted as project implementation.
The custom PPU renderer, DMA, scheduling, buses, snapshots and CPU hooks stay
in Rust. This dependency replaces shared native/oracle consumers; it does not
replace the entire hardware backend.

Install with
`tools/install_lakesnes_backend.ps1 -Zig <path-to-zig.exe> -LibraryDirectory <SDL-lib>`.
The installer checks every downloaded file against its pinned SHA256 and builds
`liblakesnes_backend.a`. Add its directory to `LIBRARY_PATH` alongside the CPU,
SPC, PNG and SDL libraries. Rebuild this archive after editing `sm/src/snes/backend.c`;
the verifier rejects an archive whose compiled bridge hash is stale. Include
[the pinned revision's MIT license](../third_party/lakesnes/LICENSE-2022.txt)
with distributed binaries.

The installer was syntax-checked. This machine's PowerShell script policy
requires running its checksum-verified Zig compilation and archive commands
directly; that is how the tested archive was built.

PPU calls use a temporary beam-position/open-bus view and restore the original
SNES pointer. The Rust adapter preserves line-zero mosaic setup and the native
renderer's complete routing bytes.
The shared state remains byte-compatible with existing snapshots.

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

On another 64-bit host, verify the installer’s source hashes and compile
`sm/src/snes/backend.c` with that source directory on the include path, C11,
exception unwind support and `SM_BACKEND_SOURCE_HASH` defined as the quoted
SHA256 of the bridge. Archive the object as `liblakesnes_backend.a`. The Windows
GNU build is validated here; this host-native recipe has not been tested on Linux.

The separate SPC interpreter dependency is documented in
[SPC_DEPENDENCY.md](SPC_DEPENDENCY.md).

## CPU instructions

The CPU adapter links the unmodified MIT-licensed CPU and state handler from
[dinkc64/LakeSnes](https://github.com/dinkc64/LakeSnes/tree/048a0d72568668f74e0cdf371dcd9e8efd614965),
revision `048a0d72568668f74e0cdf371dcd9e8efd614965`. This fork last changed in
April 2025; its CPU last changed in November 2023. It is independently published
external code, not the project's CPU moved outside the maintained source count.

Install a normal Zig C toolchain and run
`tools/install_lakesnes_cpu.ps1 -Zig <path-to-zig.exe> -LibraryDirectory <SDL-lib>`.
The script fetches four unmodified pinned files, verifies SHA256 and creates
`liblakesnes_cpu.a`. No C
behavioral glue or modified external source is used. The external closure has
2,836 C/header lines, disclosed separately from every maintained Rust adapter
and installer line. Include [the upstream MIT license](../third_party/lakesnes/LICENSE.txt)
with distributed builds.

Install the uniquely named archive alongside SDL in its configured native-library
directory, then use the normal Cargo release build with the existing SDL
`LIBRARY_PATH`. Rust leaves this external static archive unbundled, and the final
linker resolves it through that library path. For a separate prefix, include its
`lib` directory in `LIBRARY_PATH`. No Cargo build script or new Rust dependency
is required. A missing library must fail the ordinary link.

On the current Windows machine, invoking the `.ps1` installer failed because
PowerShell script execution is disabled. That result remains a failed installer
attempt. The separately authorized direct Zig compiler/archive commands below
succeeded; they do not run or rehost the script or change any execution policy.

The four files already downloaded under `cpuSourceDirectory` below are unmodified
files from the pinned revision above. Their verified SHA256 values are:

| File | SHA256 |
|---|---|
| cpu.c | `45200e56488392a25f2664caedb4edb7782ac78197a28972ccf5612b0088a470` |
| cpu.h | `4dcce50ce511cf44e115c9384bcb5cf0e057d2158581b69f490c12a613941b19` |
| statehandler.c | `2bf94ba80e0c7ca8363ab2a1f7d3e7ecb076278c9339990a73141641b772f1c0` |
| statehandler.h | `6e1b3affd60dd242bfa2dd5b25c67edfb8663d28f6c2b89e8db1a15b7b7d7249` |

Run these ordinary commands from an interactive PowerShell session. The source,
build and SDK directories shown already exist on this machine. On another machine,
use the corresponding directories and the same pinned, checksum-verified files.

```powershell
$cpuZig = 'C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/zig-x86_64-windows-0.15.2/zig.exe'
$cpuSourceDirectory = 'C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261021/lakesnes-cpu-dependency/upstream/snes'
$cpuBuildDirectory = 'C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/lakesnes-cpu-048a0d7/lib'
$cpuSdlLibraries = 'C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/SDL2-2.32.10/SDL2-2.32.10/x86_64-w64-mingw32/lib'
& $cpuZig cc -target x86_64-windows-gnu -O2 -std=c11 -c "$cpuSourceDirectory/cpu.c" -o "$cpuBuildDirectory/cpu.o"
& $cpuZig cc -target x86_64-windows-gnu -O2 -std=c11 -c "$cpuSourceDirectory/statehandler.c" -o "$cpuBuildDirectory/statehandler.o"
& $cpuZig ar rcs "$cpuBuildDirectory/liblakesnes_cpu.a" "$cpuBuildDirectory/cpu.o" "$cpuBuildDirectory/statehandler.o"
Copy-Item -LiteralPath "$cpuBuildDirectory/liblakesnes_cpu.a" -Destination "$cpuSdlLibraries/liblakesnes_cpu.a"
$env:LIBRARY_PATH = $cpuSdlLibraries
cargo build --release
```

Check that each compiler/archive command exits successfully before continuing.
The archive has its own name; no SDL library is replaced. Keep the bundled MIT
license with distributed binaries, including those built by this direct recipe.

For Debian/Ubuntu, build the same pinned files with the host C toolchain. Obtain
`cpu.c`, `cpu.h`, `statehandler.c` and `statehandler.h` from
`https://raw.githubusercontent.com/dinkc64/LakeSnes/048a0d72568668f74e0cdf371dcd9e8efd614965/snes/`
and verify the table above. Run from this repository's root, with `cpu_source`
pointing to that verified directory. Keep the existing SDL development dependency.
This host-native recipe is provided for Unix installation; Linux compilation and
runtime were not tested in this experiment.

```sh
cpu_source=/path/to/checksum-verified/LakeSnes/snes
cpu_prefix="$HOME/.local"
cpu_objects="$cpu_prefix/lib/lakesnes-cpu-048a0d7"
mkdir -p "$cpu_objects"
cc -O2 -std=c11 -fPIC -c "$cpu_source/cpu.c" -o "$cpu_objects/cpu.o"
cc -O2 -std=c11 -fPIC -c "$cpu_source/statehandler.c" -o "$cpu_objects/statehandler.o"
ar rcs "$cpu_prefix/lib/liblakesnes_cpu.a" "$cpu_objects/cpu.o" "$cpu_objects/statehandler.o"
install -Dm644 third_party/lakesnes/LICENSE.txt "$cpu_prefix/share/licenses/lakesnes-cpu/LICENSE.txt"
LIBRARY_PATH="$cpu_prefix/lib${LIBRARY_PATH:+:$LIBRARY_PATH}" cargo build --release
```

The existing Rust CPU layout, legacy saves, hardware bus, oracle hooks and
outer interrupt sequencing remain. Foreign raw registers are copied per
instruction; no foreign state survives a snapshot. The adopted source passes
524,288 instruction and 4,096 lifecycle comparisons, plus full boot any% and
Speed Booster replays with zero failing frames. Neither route reaches credits.

The real CPU bus does not observe `cyclesUsed` during an instruction. Its final
value and return match; intermediate callback-cycle byte identity is not promised.
