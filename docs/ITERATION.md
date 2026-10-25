# Windows command reference

[PLAN](PLAN.md) owns the work order, Fast loop, resource budgets and toolchain
restrictions; [REVIEW](REVIEW.md) owns its milestone overrides. This file only
provides workstation setup. Use [movies/README](../movies/README.md) for named
segments, checkpoint compatibility, capture and replay commands.

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
[repository build instructions](../README.md#run) link each dependency's pinned
installer and source checks. Preserve a working SDK path when one is already set.
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
