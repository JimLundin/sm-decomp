# TAS movies

[`manifest.toml`](manifest.toml) lists the published Super Metroid TASes used
as verification inputs: any%, low% and 100%, with download links, filenames,
frame counts and SHA-256 hashes. Download a movie into this folder. If the
download wraps the `.lsmv` or `.bk2` in another ZIP, extract the movie first.
Check the movie's SHA-256 against the manifest and use your own
[unheadered ROM](../rom/README.md). Movies and imported inputs are gitignored.

Run these commands from the repository root. Import the any% movie with
Python 3 (`python` on Windows):

```sh
python3 tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/any.inputs --rom rom/sm.smc
```

The importer accepts clean-start LSMV/BK2 recordings and writes one
hexadecimal SNES controller word per frame. LSMV imports also check the ROM's
SHA-256 against the movie. The any% import should report 129,712 frames.

With SDL2 installed, run the headless verifier:

```sh
cargo run --release -- verify rom/sm.smc movies/any.inputs
```

For Windows with the `x86_64-pc-windows-gnu` Rust toolchain, the DLL host runs
the same verifier through Python without SDL2:

```powershell
python tools/verify_dll.py rom/sm.smc movies/any.inputs
```

After a Cargo release build on Windows, `--cargo` validates and reuses its
library, compiling only the DLL host. This mode needs Cargo and the SDL2
MinGW SDK. Set `LIBRARY_PATH` to the SDK's `lib` directory; runtime DLLs are
loaded from that directory or its sibling `bin` directory:

```powershell
$env:LIBRARY_PATH = "C:\path\to\SDL2\x86_64-w64-mingw32\lib"
cargo build --release
python tools/verify_dll.py rom/sm.smc movies/any.inputs --cargo
```

Cargo checks the game and DLL host, including the verifier's included source
files. Unchanged runs reuse both compilations and leave the cached DLL intact.
Build errors stop before replay. Keep the SDL2 environment for subsequent
`--no-build` runs.

The Cargo verifier keeps game and host artifacts in this checkout's `target/`,
including when `CARGO_TARGET_DIR` points elsewhere. Sharing package artifacts
between checkouts can make older source timestamps incorrectly appear fresh.

For a late gameplay change, the verifier can save a local checkpoint and
replay a short suffix with the same oracle, rendered-frame checks and audio
advancement. This is a verifier checkpoint, not a frontend save state.
`--frames` counts frames executed by this invocation; input indices printed
for a resumed segment are absolute, zero-based movie indices.

For example, the local Charge Beam route described in `docs/PLAN.md`
contains the Morph Ball eye-beam sequence at frames 28200..28550:

```powershell
python tools/verify_dll.py rom/sm.smc movies/charge-beam.inputs --cargo --frames 28200 --checkpoint-out target/morph.smvchk
python tools/verify_dll.py rom/sm.smc movies/charge-beam.inputs --cargo --checkpoint-in target/morph.smvchk --frames 350
```

The first command verifies the prefix and creates a new checkpoint. After
editing gameplay, the second command rebuilds and verifies only 350 frames.
Omit `--frames` to replay the entire remaining suffix. Checkpoints reject a
different ROM, preceding controller inputs, audio backend, incompatible
runtime source or corrupt payload. Runtime/state-layout changes require a
new checkpoint from boot. Gameplay changes may reuse one, but the suffix
does not validate the changed program's earlier history. Outputs must be new
paths, and failing runs never produce checkpoints. Keep these ROM-derived
files local and ignored by Git.

To establish that the changed code actually runs, insert an explicit probe
at the routine entry and inside each relevant changed branch, for example:

```rust
crate::verify_probe::hit("morph-fade");
```

Then append `--require-hit morph-fade` to the segment command. Repeat the
option for branch labels. A required probe with zero calls makes the run
exit 2 even if all compared frames match. The report gives call counts and
first/last input-frame indices. Labels count explicit `hit` calls; they do
not search function names or infer execution from room coverage. Keep probes
while iterating and checking the final changed body, then remove temporary
probes and rerun the same segment on the clean build. Extend the segment far
enough to observe the behavior's effects. Omit `--require-hit` after removing
its probes. See the
[validation strategy](../docs/PLAN.md#validation-strategy-2026-10-19) for when
full boot replays are required and the integration cadence.

The DLL script builds before replaying; `--rustc` selects a compiler when it
is not on PATH. Use `--no-build` only to reuse a DLL built from the source you
intend to verify, and do not rebuild it while another process has it loaded.

Either verifier accepts `--frames 3600` for a short smoke check. Omit it for
the full movie. Verifier exit code 0 means no mismatches, 1 means comparison
failures, and 2 means an input or setup error. The DLL launcher also returns
2 on a build error. Review the reported gameplay
rooms and ending/credits frames as well as the failure count: imported TAS
timing differs from this runtime, and the any% input currently visits six
rooms without reaching credits. Every rewritten routine still needs actual
execution in a verification scenario; see the
[definition of done](../docs/PLAN.md#definition-of-done-for-any-change).
