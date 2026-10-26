# Build and run

[PLAN](PLAN.md) owns the work order, Fast loop, resource budgets and toolchain
restrictions; [REVIEW](REVIEW.md) owns its milestone overrides. This file only
provides setup and launch commands. Use [movies/README](../movies/README.md) for named
segments, checkpoint compatibility, capture and replay commands.

## Getting started

Install rustup and SDL2 (`apt install libsdl2-dev` on Debian/Ubuntu). The project
uses Rust edition 2024; the currently usable toolchain is listed in [PLAN](PLAN.md).
Build the pinned native libraries using their dependency guides:
[CPU, PPU and DSP](CPU_DEPENDENCY.md), [SPC and music](SPC_DEPENDENCY.md),
[configuration and utilities](CONFIG_DEPENDENCY.md), [shaders](SHADER_DEPENDENCY.md),
[desktop host](DESKTOP_DEPENDENCY.md), and [PNG decoding](PNG_DEPENDENCY.md).
Include the archive directory in `LIBRARY_PATH` when building or running.

Supply an unheadered Super Metroid (Japan, USA) (En,Ja) ROM at `rom/sm.smc`
(SHA-1 `da957f0d63d14cb441d215462904c4fa8519c613`). The directory is gitignored;
ROMs and game data are not distributed with this project. From the repository root:

```sh
cargo run --release -- rom/sm.smc
```

Settings and key bindings are in `sm.ini`. Defaults are arrows for movement,
X/Z/S/A for SNES A/B/X/Y, C/V for L/R, Enter for Start, and right Shift for Select.
`Verify failure!` reports a frame where Rust gameplay disagrees with the ROM oracle.
Use the [verification guide](../movies/README.md) for headless replay and coverage.

## Windows shell setup

On this workstation the tools may be absent from a fresh PowerShell PATH, and
`python` may resolve to the inactive Store alias. These are the existing local
installations; the commands affect this shell only:

```powershell
$iterationRust = "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-gnu\bin"
$iterationTools = "$env:LOCALAPPDATA\sm-decomp-tools"
$env:PATH = "$iterationRust;$iterationTools\python-3.13.7;$env:LOCALAPPDATA\Programs\Git\bin;$env:PATH"
$env:CARGO_BUILD_JOBS = "1"
Get-Command cargo,rustc,python,sh
```

Set `LIBRARY_PATH` to the qualified SDK directory containing SDL2 **and** the
native archives required by your target. SDL2 alone is insufficient. The
dependency guides above provide each pinned installer and source checks.
Preserve a working SDK path when one is already set.
Before building a verifier DLL, preflight **all** of its native inputs, including
backend/music and config/util/shader/desktop inputs as applicable. Checking only
the configuration stamps misses other stale archives. Preserve exact source bytes;
rebuild mismatching native archives from checked inputs, never edit their stamps.

From the repository root, run the exact current Fast loop in PLAN; do not add
frontend builds or placeholder replay windows from an older recipe. Stable is
the currently permitted Windows compiler; DLL/ctypes is the permitted test path.
Nightly and other launches blocked by Defender ASR remain blocked. Do not change
security settings, spoof hosted environments or claim an unrun frontend pass.

Keep source fixed during builds/checks and do not rebuild a loaded DLL. Reuse
artifacts only when source, compiler/options and native identity checks agree.
Use isolated snapshots for concurrent edits; share dependencies, not mutable
whole target trees. Resource admission, disk reserves and Git coordination are
controlled by PLAN: keep at least 50GiB disk free, begin cleanup below 75GiB,
restore 100GiB, and preserve the 2GiB RAM floor after admitted-job growth.
Check the current PLAN before each batch in case those limits have changed.
A denied or unrun check is not a pass.
