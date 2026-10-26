# Verify gameplay

The replay runner compares Rust gameplay with the ROM oracle frame by frame.
[segments.toml](segments.toml) defines the scenarios, inputs and required events.
Follow the [build guide](../docs/ITERATION.md) first; commands below run from the
repository root with Python 3.11+ (`python3` on systems without `python`).
You need your own unheadered ROM at `rom/sm.smc`.

## Run a scenario

With its inputs and starting state installed:

```sh
python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml --segment boss-crocomire
```

Repeat `--segment ID` to select more rows. Omit it to run the full corpus once
all required data is installed. The runner builds once and reuses compatible
checkpoints. `--no-build` reuses a matching build while retaining source checks.
A selected run reports partial coverage; it proves only the behavior it executes.

## Get replay data

Private inputs and starting states are distributed as encrypted assets in the
[ci-data release](https://github.com/JimLundin/sm-decomp/releases/tag/ci-data).
OpenSSL and the matching ROM are required to restore them. For Crocomire:

```powershell
New-Item -ItemType Directory -Force target | Out-Null
Invoke-WebRequest 'https://github.com/JimLundin/sm-decomp/releases/download/ci-data/checkpoints-v1-final-fdf74bcab62e16fc.tar.gz.enc' -OutFile target/checkpoints.enc
python tools/ci_seed_data.py restore target/checkpoints.enc --rom rom/sm.smc --sha256 fdf74bcab62e16fccdd7590bf9b112846c50d8aa00a6773f200d27ba6883eed1 --segment boss-crocomire
```

That bundle contains all thirteen checkpoint-backed rows. Original-emulator
room seeds use separate bundles; the current asset names, hashes and exact
row-to-bundle mapping are in [`seed_bundle` in the workflow](../.github/workflows/segments.yml).
Restore each bundle with only its matching `--segment` selections. Restoration
checks hashes and refuses to replace different existing files.

The intro uses the remaining authored route. Download the any% movie listed in
[manifest.toml](manifest.toml), extract the movie from its download ZIP, check
its SHA-256, then generate the input:

```sh
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-speed-booster.inputs --rom rom/sm.smc --route speed-booster
```

## Read the result

| Output | Meaning |
| --- | --- |
| Selected frames | Frames checked for this row, including its required events and tail |
| Setup frames | Earlier frames compared to reach the selected window |
| Skipped prefix | Previously verified history reused from a compatible checkpoint |
| Exit 0 / 1 / 2 | Passed / comparison or corpus failure / setup, input or build error |

Loaded item or boss flags do not satisfy fresh-event expectations. Checkpoint
compatibility includes ROM, inputs and runtime state; a rejected checkpoint may
regenerate from boot. A failed comparison or missing required event stays a failure.
Both save rows retain the full Kago approach before their selected save windows.

## Import or capture other inputs

[manifest.toml](manifest.toml) lists original movies, download links and hashes.
A direct LSMV/BK2 import produces one controller word per video frame:

```sh
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/any.inputs --rom rom/sm.smc
python tools/verify_dll.py rom/sm.smc movies/any.inputs --frames 3600
```

Raw movie timing can differ from native game steps. Use the
[capture and checkpoint reference](../docs/ITERATION.md#capture-and-replay-reference)
for observed wait-step inputs, original-emulator seeds and special captures.
A short replay does not establish completion of the published movie.

Keep ROMs, movies, input files, memory dumps, checkpoints and capture logs local
and ignored by Git. Publish only permitted metadata or encrypted bundles;
include no ROM, derived game data or decryption password in commits or logs.
