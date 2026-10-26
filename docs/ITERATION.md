# Build and run

## Getting started

Install [rustup](https://rustup.rs/), Python 3.11+ and SDL2 development libraries
(`apt install libsdl2-dev` on Debian/Ubuntu). The project uses Rust edition 2024;
[rust-toolchain.toml](../rust-toolchain.toml) selects the compiler.

Build the pinned native libraries using the relevant guides:
[CPU, PPU and DSP](CPU_DEPENDENCY.md), [SPC and music](SPC_DEPENDENCY.md),
[configuration and utilities](CONFIG_DEPENDENCY.md), [shaders](SHADER_DEPENDENCY.md),
[desktop host](DESKTOP_DEPENDENCY.md), and [PNG decoding](PNG_DEPENDENCY.md).
Keep their archive directory and SDL2 on `LIBRARY_PATH`. Upstream versions and
licenses are indexed in [third_party](../third_party/README.md).

Supply an unheadered Super Metroid (Japan, USA) (En,Ja) ROM at `rom/sm.smc`
(SHA-1 `da957f0d63d14cb441d215462904c4fa8519c613`). No ROM or game data is included.
From the repository root:

```sh
cargo run --release -- rom/sm.smc
```

Settings and controls are in [`sm.ini`](../sm.ini): arrows move; X/Z/S/A map to
SNES A/B/X/Y; C/V are L/R; Enter is Start; right Shift is Select.
`Verify failure!` means Rust gameplay disagreed with the ROM oracle.
See [Verify gameplay](../movies/README.md) for headless replay.

## Windows shell setup

Use the `stable-x86_64-pc-windows-gnu` Rust toolchain and a matching SDL2 MinGW SDK.
If tools are missing from PATH, point this shell at their installed locations:

```powershell
$iterationRust = "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-gnu\bin"
$iterationTools = "$env:LOCALAPPDATA\sm-decomp-tools"
$env:PATH = "$iterationRust;$iterationTools\python-3.13.7;$env:LOCALAPPDATA\Programs\Git\bin;$env:PATH"
Get-Command cargo,rustc,python,sh
```

Adjust the Python path for your installation. `LIBRARY_PATH` must contain the
required native archives as well as SDL2. Rebuild stale archives with their
pinned installers; never edit source stamps to make an old archive appear current.
The DLL/ctypes path supports local verification when executable launches are
restricted. Follow existing host restrictions without changing security settings.
Project work order and resource limits are maintained in [PLAN](PLAN.md).

## Capture and replay reference

The normal workflow is [selected corpus rows](../movies/README.md#run-a-scenario).
These options serve direct replays and data maintenance:

| Need | Command or option |
| --- | --- |
| Check formatting and library | `cargo fmt --check`; `cargo check --no-default-features --lib` |
| Build verifier only | `python tools/verify_dll.py --build-only` |
| Include frontend validation | `python tools/verify_dll.py --build-only --frontend-check` |
| Existing unit tests, no ROM | `python tools/verify_dll.py --unit-tests --direct` |
| Explicit build path | `--cargo` uses Cargo/SDL2; `--direct` uses the standalone headless path |
| Reuse matching build | `--no-build`; source and native-library checks still run |
| Bound a direct replay | `--frames N` |
| Require observations | Repeat `--expect CONDITION` or an existing `--require-hit LABEL` |
| Follow-up window | `--after-expectations N` or `--after-hits N` |

Missing observations or incomplete follow-up windows fail the run. Corpus rows
supply these options from TOML; do not combine `--corpus` with direct-replay
frames, checkpoints or expectations. Use existing checks; the current Fast loop
in [PLAN](PLAN.md) governs which checks an implementation change needs.

A direct replay can save and resume a new local checkpoint:

```sh
python tools/verify_dll.py rom/sm.smc movies/any.inputs --frames 300 --checkpoint-out target/any-300.smvchk
python tools/verify_dll.py rom/sm.smc movies/any.inputs --checkpoint-in target/any-300.smvchk --frames 300
```

The second frame budget starts at the checkpoint. Successful runs alone produce
checkpoints, and output paths must be new. ROM, canonical input prefix, runtime
codec, audio and state guards remain enforced. A reused prefix is not fresh
coverage of that history. Do not rebuild a DLL while another process has it loaded.

### Record original-emulator seeds

Use official [lsnes rr2-beta23](https://files.tasvideos.org/emulatordownloads/lsnes/lsnes-rr2-beta23.7z),
its bsnes v085 Compatibility core and [`lsnes_lag_log.lua`](../tools/lsnes_lag_log.lua).
Choose a new, empty capture directory and run from the repository root:

```powershell
New-Item -ItemType Directory target/seeds/my-capture | Out-Null
$env:SM_LSNES_LOG = Join-Path (Get-Location).Path 'target/seeds/my-capture/lag.csv'
$env:SM_LSNES_SEEDS = Join-Path (Get-Location).Path 'target/seeds/my-capture'
$env:SM_LSNES_BOUNDARIES = '1'
Start-Process -FilePath 'C:\path\to\lsnes-rr2-beta23\lsnes-bsnes.exe' -WindowStyle Hidden -WorkingDirectory (Get-Location).Path -ArgumentList @('--rom-a=rom/sm.smc', 'movies/super-metroid-any-3653.lsmv', '--lua=tools/lsnes_lag_log.lua', '--unpause') -Wait
```

Use the positional movie and `--rom-a=`: this lsnes release ignores `--load=`
and rejects this ROM through its movie-loading `--rom=` path. The logger observes
without altering input or memory. An `.error.txt` companion means capture failed,
even if the emulator exits zero. Retain all log, poll, event, register and memory
companions; `rooms.csv` identifies each captured WRAM seed and its hashes.

| Optional capture setting | Effect |
| --- | --- |
| `SM_LSNES_FRAMES` | Finite video-frame limit; unset for the full declared movie |
| `SM_LSNES_MB_PHASE_SEEDS=1` | Additional Mother Brain phase observations; requires seed capture |
| `SM_LSNES_EOF_CONTINUATION=1` | Observe read-only playback beyond EOF; requires a finite limit beyond EOF |
| `SM_LSNES_CAPTURE_FROM` | Start a narrow log at this frame; values above 1 cannot combine with seed capture |

Remove optional environment variables before an unrelated capture. Movie frames
are one-based; input indices are zero-based. Flat NMI counters alone do not identify
lag, and `.polls.csv` fields labelled `live` are physical input, not movie playback.

Import observed game steps using the captured WRAM filename and a measured count:

```sh
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/my-seed.inputs --rom rom/sm.smc --lag-log target/seeds/my-capture/lag.csv --seed target/seeds/my-capture/0001-DF45-wram.bin --frames 600
python tools/verify_dll.py rom/sm.smc movies/my-seed.inputs --seed target/seeds/my-capture/0001-DF45-wram.bin --frames 600 --expect room:DF8D
```

Choose the actual seed and expectation from your capture. The importer validates
controller phases and waits; never repair a rejected capture with guessed offsets
or input padding. An original seed is not an exact verifier checkpoint: qualify
its episode against the oracle and require its intended events and full tail.
Use the corpus runner for its supported seeded checkpoint reuse.

### Special captures and distribution

Credits beyond movie EOF need recorded continuation metadata. Add
`--eof-capture PATH --eof-capture-sha256 SHA256` to the lag-log import, retaining
its matching movie, seed and `--frames` count. Capture tools must preserve the
observed final input and later zero reads; do not synthesize padding.

For the restored original credits capture:

```sh
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-credits.inputs --rom rom/sm.smc --lag-log target/seeds/100-credits/lag.csv --seed target/seeds/100-credits/0336-91F8-wram.bin --frames 11900 --eof-capture target/seeds/100-credits/lag.csv.eof.json --eof-capture-sha256 fd304868560fd08f823666b2ca16922597fb34b086a375c898b65166d1a757b8
```

Golden Torizo uses the original five-port Snes9x recording, snapshot and pinned
live/history proofs. [`convert_snapshot.py`](../tools/convert_snapshot.py) checks
them before conversion; its command is:

```sh
python tools/convert_snapshot.py target/seeds/golden-input-phase/snes9x.state target/seeds/golden-reconverted --live target/seeds/golden-input-phase/snes9x.live.json --history target/seeds/golden-input-phase/snes9x.history.json --rom rom/sm.smc
python tools/movie_inputs.py 'JXQ - Super Metroid 100% TAS v2.smv' movies/seed-golden-torizo.inputs --rom rom/sm.smc --snes9x-capture target/seeds/golden-input-phase/snes9x.json --snes9x-capture-sha256 77069a8768cd1519e321163253a7804bdecd691f48f2ea33ce4eaf0f466715bd --seed target/seeds/golden-reconverted/0001-B283-wram.bin --frames 1207
```

Keep all capture material under ignored local directories. To distribute verified
corpus data, use `python tools/ci_seed_data.py pack target/new-bundle.enc --rom rom/sm.smc`.
Use `--segment` to pack a subset. Restore into an isolated checkout using the
printed ciphertext SHA-256 and the same selections before publishing ciphertext.
The packer uses AES-256-CBC/PBKDF2 with a ROM-derived password; do not publish its
plaintext, the ROM or password. Restoration instructions are in the
[verification guide](../movies/README.md#get-replay-data).
