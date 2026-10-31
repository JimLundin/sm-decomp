# Verify gameplay

The replay runner compares Rust gameplay with the ROM oracle frame by frame.
Each frame starts from the same state and input; the original ROM runs on the
emulated 65816 and the Rust game runs on the same emulated hardware. WRAM,
SRAM, VRAM, CGRAM, OAM, PPU registers and port state, and the rendered image
must match. The [README](../README.md#verification) lists what this does not
cover.

[segments.toml](segments.toml) defines the scenarios (rows), their inputs and
required events: 129 rows, 219,592 selected frames. Follow the
[build guide](../docs/ITERATION.md) first; commands below run from the
repository root with Python 3.11+ (`python3` on systems without `python`).
You need your own unheadered ROM at `rom/sm.smc`. The verifier builds only on
Windows x86-64 GNU.

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
Invoke-WebRequest 'https://github.com/JimLundin/sm-decomp/releases/download/ci-data/layout150-f158-full22.tar.gz.enc' -OutFile target/checkpoints.enc
python tools/ci_seed_data.py restore target/checkpoints.enc --rom rom/sm.smc --sha256 723ccef6ac6df45f31d5179ba5d9569184de0fb5c678266365d3eaf62d3c10b9 --segment boss-crocomire
```

Fourteen rows start from checkpoints: that bundle holds ten of them and
`layout150-checkpoints-147` the other four. Original-emulator room seeds use
separate bundles. The current asset names, hashes and exact row-to-bundle
mapping are in `seed_bundle` in [plan_corpus.py](../.github/scripts/plan_corpus.py).
Restore each bundle with only its matching `--segment` selections. Restoration
checks hashes and refuses to replace different existing files.
Add `--runtime-only` when you only need to replay existing rows; it restores
selected inputs, starts and seed companions. Keep the default full restore when
regenerating inputs, because it also restores the original capture logs.

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

## Checks in CI

CI holds the ROM in repository secrets and keeps every ROM-derived file
(inputs, checkpoints, logs) private to the runner.

| Workflow | Runs | What it does |
| --- | --- | --- |
| [ci.yml](../.github/workflows/ci.yml) | Every pull request and push to main | Format, progress reports, dependency direction, target-crate tests, the ROM import check below, the Windows native and desktop build, verifier unit tests and detached composition checks |
| [segments.yml](../.github/workflows/segments.yml) | Every 3 hours, on dispatch, and after a push to main that deletes a legacy bank file | The corpus: rows grouped by seed bundle or input into at most 6 parallel shards; checkpoint rows have a 60-minute limit |
| [full-route.yml](../.github/workflows/full-route.yml) | Nightly and on dispatch | The whole 100% TAS |
| [coverage.yml](../.github/workflows/coverage.yml) | Manual | The corpus with coverage instrumentation |

To run selected corpus rows on a branch, dispatch `segments.yml` with
space-separated row IDs; unknown IDs fail. An empty selection runs the full
corpus. A change lands after PR CI and a full corpus pass on its exact head.

### ROM import check

`ci.yml` restores the ROM and runs `cargo test -p sm_data --test rom_import`
with `SM_ROM` set. The test imports every `sm_data` record from the real
cartridge. A record that rejects it would leave the game on its old ROM reads,
so replays would pass without exercising it. The log is filtered to pass/fail,
the rejected record names and error kinds, and the failing assertion's source
location, because it can hold ROM-derived values. When adding a record to
`sm_data`, add it to [rom_import.rs](../crates/sm_data/tests/rom_import.rs);
run it locally as shown in the [build guide](../docs/ITERATION.md#target-crates-on-any-platform).

### Full route

`full-route.yml` imports `movies/100.inputs` from the 100% movie in
[manifest.toml](manifest.toml) (222,788 frames) and checks its hash. A
`boundaries` job replays the original from boot and saves a checkpoint every
20,000 frames; these are cached per source. Each chunk then replays in lockstep
from its boundary with the same per-frame comparison as the corpus, at most 6
in parallel. Dispatch with `regenerate` to rebuild the boundary checkpoints.
[full_route.py](../.github/scripts/full_route.py) holds the steps.

### Coverage

`coverage.yml` builds the verifier with `-C instrument-coverage`, runs the
selected rows (all by default), merges the profiles and summarises line and
function coverage of `crates/sm_game` and `sm/src/game` with
[coverage_summary.py](../tools/coverage_summary.py). The `coverage` artifact
lists files and the largest functions the corpus never executes; no
ROM-derived data is recorded. It is manual and does not gate landing.

### Mother Brain by phase

The room-to-room chain does not leave Mother Brain's room, so her fight is
covered by phase rows instead. Each `mother-brain-*` row starts from one of the
`100-mb-phases` observations (captured when her form word `$7E:7800` changes)
and requires the probe at the next form change, with 120 frames after it:

| Row | Seed | Probe |
| --- | --- | --- |
| `mother-brain-glass` | 324 | `mother-brain.form-1`: the glass phase ends |
| `mother-brain-fake-death` | 325 | `mother-brain.form-2`: she rises for phase 2 |
| `mother-brain-phase2` | 326 | `mother-brain.form-3`: the baby Metroid latches on |
| `mother-brain-baby-metroid-drain` | 327 | `mother-brain.drained`: she is drained grey |
| `mother-brain-baby-metroid-sacrifice` | 328 | `baby-metroid.fatal-blow`, `mother-brain.form-4` |

`boss-mother-brain` covers the defeat from seed 329, and
`mother-brain-escape-door` continues from the same seed through the escape
timer and the door to `$DE4D`. Each row's input is the lag-aligned import
from its seed:

```sh
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-mother-brain-glass.inputs --rom rom/sm.smc --lag-log target/seeds/100-mb-phases/lag.csv --seed target/seeds/100-mb-phases/0324-DD58-wram.bin --frames 760
```

with seeds 0325, 0326, 0327, 0328 and 0329 and frames 1360, 2940, 1480,
2830 and 1500 for the following rows. The inputs agree with the chain's
room-324 input: each later seed's input is the chain's from native step
602, 1804, 4584, 5910 and 8582.

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

Rows that test menus or other play the movies never reach use authored
controller inputs. [authored_inputs.py](../tools/authored_inputs.py) holds their
recipes in a small input language (frame-exact holds, presses, repeats and
named moves) and writes each input; a seeded recipe keeps the movie word
latched at its seed as its first word. Its sweeps write one input per
parameter set and print the matching rows; `check` keeps the pinned recipes.

A row's `require_hit` names the probes it must reach, and the row fails if
any never runs. Routines a row exists to cover carry a probe named
`module.routine`, for example `draygon_grab.tail_whip`; add one with
`crate::probe::hit` (or `crate::verify_probe::hit` in `sm`) inside the
branch that does the work.

A scenario seed is a recorded room seed with declared edits to its work RAM:
energy, ammunition, items, beams, the selected item, Samus's position or raw
words such as a boss phase. [scenario_seeds.py](../tools/scenario_seeds.py)
holds the recipes; `write NAME` creates `target/seeds/scenario-NAME/` with
fresh region hashes and prints the row's `seed` and `seed_sha256`. Both
engines load the edited seed, so the original game still decides what
follows. Scenario seeds stay local and go to CI in encrypted bundles.

Keep ROMs, movies, input files, memory dumps, checkpoints and capture logs local
and ignored by Git. Publish only permitted metadata or encrypted bundles;
include no ROM, derived game data or decryption password in commits or logs.
