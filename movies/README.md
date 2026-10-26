# TAS movies

[`manifest.toml`](manifest.toml) lists the published Super Metroid TASes used
as verification inputs: any%, low%, 100% and Spore Spawn, with download links,
filenames, frame counts and SHA-256 hashes. Download a movie into this folder. If the
download wraps the `.lsmv` or `.bk2` in another ZIP, extract the movie first.
Check the movie's SHA-256 against the manifest and use your own
[unheadered ROM](../README.md#run). Movies and imported inputs are gitignored.

Run these commands from the repository root. Import the any% movie with
Python 3 (`python` on Windows):

```sh
python3 tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/any.inputs --rom rom/sm.smc
```

The importer accepts clean-start LSMV/BK2 recordings and writes one
hexadecimal SNES controller word per frame. LSMV imports also check the ROM's
SHA-256 against the movie. The any% import should report 129,712 frames.

To measure the original movie's timing, use the official Windows
[lsnes rr2-beta23](https://files.tasvideos.org/emulatordownloads/lsnes/lsnes-rr2-beta23.7z)
with its bsnes v085 Compatibility core and [`lsnes_lag_log.lua`](../tools/lsnes_lag_log.lua).
From the repository root, with the extracted executable path substituted:

```powershell
New-Item -ItemType Directory -Force target | Out-Null
$env:SM_LSNES_LOG = Join-Path (Get-Location).Path "target/any-bsnes085-lag.csv"
Start-Process -FilePath "C:\path\to\lsnes-rr2-beta23\lsnes-bsnes.exe" -WindowStyle Hidden -WorkingDirectory (Get-Location).Path -ArgumentList @("--rom-a=rom/sm.smc", "movies/super-metroid-any-3653.lsmv", "--lua=tools/lsnes_lag_log.lua", "--unpause") -Wait
```

Use the positional movie and `--rom-a=` exactly: this release's advertised
`--load=` option is ignored, and its `--rom=` movie-loading path rejects this ROM.
The script observes playback without changing input or memory and quits at the
movie's end. For a 300-frame smoke, set `$env:SM_LSNES_FRAMES = "300"`; remove that
variable before a full capture. Use a new output name for each run.

The CSV records raw NMI counters, emulator lag, room/state and the game's latched
joypad word. Its `.polls.csv` companion preserves every input callback and the
recorded first subframe; the fields labelled `live` describe physical input,
not movie playback. Movie frames are one-based; `input_frame` is zero-based.
An `.error.txt` companion means capture failed even if the emulator exits zero.
Keep these files under ignored `target/`; commit only a completed capture's hash.
Flat counters can also mean boot, reset or coroutine waits. These observations
do not yet authorize dropping frames, and a zero-difference native replay does
not establish that the published TAS reaches credits.

For the measured options-menu regression, set `SM_LSNES_BOUNDARIES=1`,
`SM_LSNES_FRAMES=12500` and a fresh `SM_LSNES_LOG` path before that invocation.
The optional `.events.csv` records waits, controller reads and door loading/IRQ events.
The ROM reads input during NMI for its next update; native code reads at the
update's start. The importer therefore uses the input observed at wait entry,
not the next input latched at wait exit:

```powershell
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/any-wait-prefix.inputs --rom rom/sm.smc --lag-log target/any-boundaries.csv --frames 8250
python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml --segment options-menu-return-parameter
```

Substitute the actual capture path for `target/any-boundaries.csv`. The importer
checks the captured input against the published movie and rejects missing waits,
subframes or unaccounted NMI advances. The original 8,250-step recipe remains available; the loading-boundary recipe
below extends through the first four Ceres doors only.
The regression checks steps 329..333: step 330 previously lost the highlighted
tile value passed to the returning options-menu border. This is an additional
regression, not another completed required coverage item. Keep all capture files
local; the manifest binds the reference capture hashes.

For the door-loading regression, capture 10,000 movie frames with
`SM_LSNES_BOUNDARIES=1`, `SM_LSNES_FRAMES=10000` and a new output path, then run:

```powershell
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/any-door-prefix.inputs --rom rom/sm.smc --lag-log target/any-door-boundaries.csv --frames 9100
python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml --segment door-loading-lag-nmi --segment tas-ceres-four-doors
```

The importer uses the latch before each recorded controller read and preserves
one loading suspension per observed placement. It checks that acknowledged NMI,
input and position stay unchanged while the total NMI counter advances once
between IRQ setup and the first scrolling IRQ. The vblank wait itself does not
advance that counter. Missing boundaries or a different observed ordering fail
instead of introducing offsets. IRQ, decode and DMA events retain ordering evidence.
The five-step regression starts before the first suspension; the longer row covers
four doors and subsequent gameplay. These are additional checks, not completion
of the required corpus or full-game/credits alignment. Capture hashes are in the manifest.

The next prefix uses the same capture procedure with `SM_LSNES_FRAMES=23000`.
The logger also records door direction for the loading-order comparison. Import
and check the measured cross-video controller read with:

```powershell
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/any-cross-frame-prefix.inputs --rom rom/sm.smc --lag-log target/any-extended-boundaries.csv --frames 9266
python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml --segment tas-ceres-cross-frame-read
```

The read at native step 9216 starts in movie frame 10021 and ends in 10022.
Its boundary events are consecutive and neither NMI counter changes. The importer
accepts that uninterrupted read while rejecting missing boundaries, counter changes,
and larger frame gaps. The first 9,100 imported words remain byte-identical.
The new row reaches E0B5 during door transition; it does not prove gameplay resumes
there. Step 9266 remains unmapped: a controller read follows the door's `more_exit`
without an observed normal wait entry. Do not extend past it using offsets.

With SDL2 installed, run the headless verifier:

```sh
cargo run --release -- verify rom/sm.smc movies/any.inputs
```

For Windows with the `x86_64-pc-windows-gnu` Rust toolchain, the DLL host runs
the same verifier through Python without SDL2:

```powershell
python tools/verify_dll.py rom/sm.smc movies/any.inputs
```

Run the verifier's existing unit tests without a ROM or standalone test executable:

```powershell
python tools/verify_dll.py --unit-tests --direct
```

This executes the same test bodies discovered by Cargo, reports each result,
and exits nonzero on failure. Keep the named case list in `coverage_checks::run`
in sync when adding tests. Unit tests do not count as gameplay coverage.

With the installed SDL2 SDK on `LIBRARY_PATH`, the launcher automatically uses
Cargo's normal release artifacts. `--cargo` selects it explicitly. This mode needs Cargo and the SDL2
MinGW SDK. Set `LIBRARY_PATH` to the SDK's `lib` directory; runtime DLLs are
loaded from that directory or its sibling `bin` directory:

```powershell
$env:LIBRARY_PATH = "C:\path\to\SDL2\x86_64-w64-mingw32\lib"
cargo build --release
python tools/verify_dll.py rom/sm.smc movies/any.inputs --cargo
```

For final release library, frontend, verifier and loaded-source checks, one
invocation builds the necessary targets with the same flags and no replay:

```powershell
python tools/verify_dll.py --build-only --frontend-check
```

The next selected replay reuses those artifacts. `--build-only` alone checks
the verifier/library without adding frontend compilation. Build logs are streamed
to `target/verify-dll/build.log`; the launcher reports which targets compiled.
Normal Cargo builds and launcher builds use identical compiler options, so
switching between them no longer recompiles an unchanged host.

Cargo checks the game and DLL host, including the verifier's included source
files. Unchanged runs reuse both compilations and leave the cached DLL intact.
Build errors stop before replay. Keep the SDL2 environment for subsequent
`--no-build` runs.

The Cargo verifier keeps game and host artifacts in this checkout's `target/`,
including when `CARGO_TARGET_DIR` points elsewhere. Sharing package artifacts
between checkouts can make older source timestamps incorrectly appear fresh.

Without the SDL2 environment, the standalone rustc path remains available;
`--direct` selects it explicitly. It uses separate artifacts and incremental
compilation, and skips unchanged builds only when source, compiler/options,
native archives and output bytes match. Compiled-source checks still run.
Use one path throughout an iteration to keep its cache useful.

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
not search function names or infer execution from room coverage.

To stop based on execution, add `--after-hits N`. For example, this checks
through 60 additional frames after both existing camera probes first run,
with a 35,000-frame budget:

```powershell
python tools/verify_dll.py rom/sm.smc movies/any.inputs --cargo --frames 35000 --require-hit camera.blocks --require-hit camera.auto_x --after-hits 60
```

Every frame still receives the normal oracle/render/audio checks. Repeated
hits do not restart the window. Choose N for the change's downstream effects;
60 is an example, not a universal sufficient tail. Zero checks through the
hit frame itself. A missing probe or a movie/frame limit that cuts off the
requested tail fails the run and cannot save a checkpoint. Saved checkpoints
use the actual stopping frame. The same options work with `--checkpoint-in`
to skip a justified prefix; `--frames` remains a budget from that checkpoint.
This selects the endpoint from explicit probes, not automatic Git/call-graph
analysis. Include probes for every changed path and later effect that matters.

Keep probes while iterating and checking the final changed body, then remove temporary
probes and rerun the same segment on the clean build. Extend the segment far
enough to observe the behavior's effects. After removing probes, omit both
`--require-hit` and `--after-hits` and use the measured `--frames` count. See the
[Fast loop](../docs/PLAN.md#fast-loop-reviewer-2026-10-25-replaces-the-2026-10-21-validation-strategy) for selecting
exercised segments and justifying any full replay through changed-code coverage.

The DLL script builds before replaying; `--rustc` selects a compiler when it
is not on PATH. `--no-build` checks the DLL's embedded source stamp before
reusing it. Changed Rust sources, manifests, lockfile, toolchain configuration,
local Cargo configuration or launcher require a rebuild; unstamped older DLLs
also require one. The launcher rejects source changes during build/replay.
It also compares source bytes embedded by the game library's own compilation.
If Cargo reuses stale library contents (including edits with restored old
timestamps), `--cargo` rebuilds the sm package, then checks again before replay.
New library source files must be added to `sm/src/verify_source.rs`; omitted
files fail the check. It prints the source SHA-256, build/check time and total
time. These checks establish source identity; required probes and oracle
comparisons establish which behavior was exercised. Capturing a source file
does not prove its functions ran, and the DLL replay does not exercise the UI.
Keep artifacts local to the checkout as described above, and do not rebuild
a DLL while another process has it loaded.

Either verifier accepts `--frames 3600` for a short smoke check. Omit it for
the full movie. Verifier exit code 0 means no mismatches, 1 means comparison
failures, and 2 means an input or setup error. The DLL launcher also returns
2 on a build error. Review the reported gameplay
rooms and ending/credits frames as well as the failure count: imported TAS
timing differs from this runtime, and the any% input currently visits six
rooms without reaching credits. Every rewritten routine still needs actual
execution in a verification scenario; see the
[definition of done](../docs/PLAN.md#definition-of-done-for-any-change).

## Checkpoint corpus

[segments.toml](segments.toml) defines required scenarios and explicit gaps. Its
runnable recipes are candidates until the current verifier reports their results;
adding a recipe does not establish coverage. Full mode exits nonzero while any
required scenario is pending, missing or fails.

Generate the shared authored route from the any% movie above, then run the corpus:

```powershell
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-speed-booster.inputs --rom rom/sm.smc --route speed-booster
python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml --direct
```

The canonical route has 148,406 frames and SHA256
`bc907fc429252418216b7f575e26be148f7cf863a71c3947b4ffa3c6c6027de6`.
Its controller words match the historical route; canonical casing/newlines change
its byte hash. Use the new filename to preserve existing local inputs.
The unchanged bundled Ice inputs traverse the upper shutter bypass and collect
health drops in Business Center; `ice-return-business` retains all 4,426 selected
frames. The bundled `power-bomb-room-entry` inputs retain their separate 8,910-frame
navigation window. Their generation recipes are retired after exact input distribution.

For focused work, add repeatable `--segment ID`, for example
`--segment item-morph-ball`. Selection reports PARTIAL coverage and does not
establish the complete required corpus. The runner prints each segment's result
and the existing room/coverage output. It builds once through the selected local
path and runs each segment in a fresh Python process; `--no-build` still checks sources.

Checkpoint setup is separate from segment coverage. The runner advances earlier
same-input checkpoints, checks source/ROM/input identity and all existing checkpoint
invariants, and creates new output paths under `target/verify-corpus`. An incompatible
checkpoint can regenerate from boot; gameplay or expectation failures never cause
a retry. Input paths resolve relative to the TOML file, so an isolated source snapshot
can use the main checkout's absolute TOML path and its local movies. Keep ROMs,
movies, inputs and checkpoints uncommitted.

Thirteen rows can start from distributed, previously verified native-plus-oracle
checkpoints. These are separate from the 68 original-emulator room-seeded rows;
the intro remains a zero-setup boot row. The checkpoint format is unchanged.
The first seven starts are listed here; six further starts appear below.

| Row | Checkpoint frame | Compared setup to original start | Original ceiling |
| --- | ---: | ---: | ---: |
| `options-menu-return-parameter` | 329 | 0 | 5 |
| `demo-attract-setup` | 2619 | 0 | 189 |
| `door-transition` | 13600 | 0 | 290 |
| `save-memory` | 55650 | 14350 | 1800 |
| `save-death-reload` | 55650 | 14350 | 6064 |
| `ice-return-business` | 161656 | 0 | 4426 |
| `power-bomb-room-entry` | 166082 | 0 | 8910 |

Each row names a flat `checkpoint` filename under `target/checkpoints` and its
`checkpoint_sha256`. The full original input file, input SHA256, selected start,
requirements and tails stay unchanged. The corpus validates the asset's hash and
frame before invoking the existing loader, whose runtime, ROM, canonical input
prefix, audio, payload and state round-trip guards remain in force. A rejected
runtime checkpoint can regenerate from boot; required event failures cannot.
The log distinguishes previously verified skipped prefixes from newly compared
setup and selected frames. Skipped prefixes are not fresh coverage for the build.

The explicit asset takes precedence over later local caches. Both save rows
therefore compare the full 55650..70000 approach on each run, retaining the Kago
`E7FF` encounter before the selected save window. The intro row still begins at
zero and retains all 8,780 selected frames. Configured setup across these seven
checkpoint rows falls from 484,286 to 28,700 frames, a reduction of 455,586.

CI restores the unchanged full inputs and checkpoint files from its pinned
encrypted bundle through `ci_seed_data.py`. For a local copy of the same bundle:

```powershell
python tools/ci_seed_data.py restore checkpoint-final.tar.gz.enc --rom rom/sm.smc --sha256 fdf74bcab62e16fccdd7590bf9b112846c50d8aa00a6773f200d27ba6883eed1 --segment save-memory --segment save-death-reload --segment door-transition --segment ice-return-business --segment power-bomb-room-entry --segment options-menu-return-parameter --segment demo-attract-setup
```

The `save-death-continue`, `ice-return-business` and `power-bomb-room-entry`
generators now have no consumers and are removed with their exclusive helpers.
The six further rows below retire the other four specialized generators; only
Speed Booster generation remains for the intro. Full distributed inputs remain private.

Ordinary pickup rows require a newly observed award, actual message completion and
120 follow-up frames. Varia waits for its award and restored player handlers instead.
Kraid requires fresh defeat state plus actual death/background completion probes.
Loaded inventory or boss flags cannot satisfy event expectations. Door/elevator
checks require fresh departure through arrival and, for elevators, unlocked landing.
Ceres requires combat through the escape trigger; Bomb Torizo requires observed
damage, death entry and the later completion flag. These do not cover every attack.
`save-death-continue` requires both named save/restore probes and an ordered fresh
save, death, Game Over, successful selected-slot restore and normal gameplay in the
saved room. Its authored input has 76,064 frames and SHA256
`af79d24c791c1414cd4b1b83f040da0787a2a69bd0d2c4d6b73fc667b7d74467`.
This covers in-memory SRAM restoration; disk persistence remains unverified.
The additional `save-memory` check covers the save/checksum write alone.
Missing bosses/items/areas and credits remain explicit gaps.
The `credits` expectation observes script progress; the separate `credits-complete` expectation requires the roll-ending transition.

The `wave-gate` authored route reuses the checked Speed Booster/farm return,
then crosses ADAD's moving platforms and opens its blue gate. Its unchanged full
input is now restored from the encrypted checkpoint bundle.
The additional `shot-gate-spawn-parameter` regression checks steps 153606..153615:
the re-closing gate must preserve the projectile definition as its spawn parameter.
This route does not yet qualify Wave acquisition or advance the required tally.

The unchanged bundled `crocomire-shot` authored input includes Wave pickup, the
Norfair refill and Crocomire fight.
`crocomire-spit-drop-scratch` observes an onscreen dust bound followed by a spit
pickup: the shared compatibility word must reach the pickup's spawn parameter.
It covers the failing update and three subsequent frames. Boss completion remains
a separate row; the native checkpoint loader retains its compatibility guards.

The `crocomire` authored route continues `crocomire-shot` from the qualified
frame-167285 fight state through both melts, the wall break and skeleton collapse.
Its full original input is restored from the encrypted bundle.
`crocomire-melt-graphics-buffer` covers four frames from 169057;
`crocomire-wall-spawn-return` covers four frames from 171090 and all eight piece spawns.
`boss-crocomire` starts with the defeat flag clear at 167285, observes the flag
at 171422, and requires 120 subsequent frames. The recipe ends at 171694.

Six more authored-route rows retain their original full input files and selected
windows while starting at exact, verified native/oracle checkpoints:

| Row | Checkpoint / selected start | Original ceiling | Compared setup |
| --- | ---: | ---: | ---: |
| `pickup-reserve` | 147273 | 614 | 0 |
| `shot-gate-spawn-parameter` | 153606 | 10 | 0 |
| `crocomire-spit-drop-scratch` | 167281 | 4 | 0 |
| `boss-crocomire` | 167285 | 4409 | 0 |
| `crocomire-melt-graphics-buffer` | 169057 | 4 | 0 |
| `crocomire-wall-spawn-return` | 171090 | 4 | 0 |

These six rows remove 975,592 configured setup frames; their observations and
follow-up tails are unchanged. Both earlier save rows still compare their complete
Kago approach. The Reserve and Crocomire inputs share a verified prefix through
145574 and then diverge: Reserve's later checkpoint cannot initialize Crocomire.
The exact gate and Crocomire starts come from one checked continuation of the
full Crocomire input, whose prefix also matches the gate and spit-regression files.
The bundle preserves all 12 prior assets and adds four full inputs and six starts.

Restore another selected row from the same cumulative bundle, for example:

```powershell
python tools/ci_seed_data.py restore checkpoint-final.tar.gz.enc --rom rom/sm.smc --sha256 fdf74bcab62e16fccdd7590bf9b112846c50d8aa00a6773f200d27ba6883eed1 --segment boss-crocomire
```

Repeat `--segment` for additional rows. The bundle contains all 13 checkpoint rows.

The `norfair-reserve`, `wave-gate`, `crocomire-shot` and `crocomire` generators,
plus their exclusive helpers, are retired: 264 fewer Python support lines. The
remaining Speed Booster generator and all importer guards remain unchanged.

For room-entry memory capture, create an empty `target/seeds/<movie>` directory
and set `SM_LSNES_SEEDS` to its absolute path before the existing lsnes command.
Use a fresh `SM_LSNES_LOG` too. The optional mode captures WRAM, SRAM, VRAM,
CGRAM, OAM and APU RAM after the first completed gameplay frame following each
room change, including revisits. These ROM-derived files remain local.
`rooms.csv` records each region's size and SHA-256. The committed
`room-seeds-any.csv` lists the 197 observed entries; its `seed_sha256` hashes the
concatenation of those six regions in the order above. `movie_frame` is one-based
and completed; `next_input_index` is the following zero-based movie input.
`movie_subframe_index` identifies the completed frame's recorded first subframe.
The original capture is still a hardware observation, not a verifier checkpoint:
the CPU may be inside NMI, and each seed must pass the replay qualification below.

Room-seeded replay (`--seed`) also requires the companion `-registers.csv` from
the updated logger. It records CPU/PPU observations, passive DMA register reads
and observed writes to IRQ/NMI controls. Enable `SM_LSNES_BOUNDARIES=1` for the
input recipe. The importer finds the captured NMI's next completed wait from
the seed's indexed frame, room and NMI counter; it validates the original movie
and polls and applies the existing lag-aware rules starting at that boundary.
It never guesses an input shift or requires earlier rooms to replay successfully.

```powershell
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/seed-ceres.inputs --rom rom/sm.smc --lag-log target/seeds/any-hardware-final/lag.csv --seed target/seeds/any-hardware-final/0001-DF45-wram.bin --frames 600
python tools/verify_dll.py rom/sm.smc movies/seed-ceres.inputs --seed target/seeds/any-hardware-final/0001-DF45-wram.bin --frames 600 --expect room:DF8D
```

The first seed's 600-word input SHA-256 is
`31f0eb3e632cbe77bdc9b34ed2ec8a28cda0d00bfb4798090eda5614b78b6c20`.
The original-emulator capture enters DF8D next. Qualification requires both
zero oracle differences and that route expectation; merely surviving 600 frames
does not qualify a seed. Keep ordinary RAM, rendering and audio checks enabled.
The import completes the recorded CPU phase, reuses the existing load-stack
normalization, and reconstructs Session state through the normal RAM entry path.
Six memory regions and these registers are still not an exact emulator checkpoint:
SPC/DSP execution phase and some PPU write latches are not fully represented.
Qualify each named episode independently; a room-route claim also needs its
next-room expectation. Existing checkpoint guards are unchanged;
seed input cannot combine with checkpoint input/output or SRAM loading. Seeded
replays cannot publish checkpoints until the checkpoint format records seed provenance.

The full captures contain 197 any% and 331 100% room entries in
`room-seeds-any.csv` and `room-seeds-100.csv`. `seed_sha256` binds the six
memory regions. Both completed supplemental indexes also record
`registers_sha256` for the companion CSV and `observation_sha256` for the
six regions followed by that CSV; `cpu_pc` and `nmi` identify the observed
CPU phase. No seed binaries, input files or original movies are committed.

The updated logger captures seeds at the next normal wait's `$808348` opcode
fetch after the first completed gameplay video frame. Video callbacks can stop
halfway through an instruction: the X-ray seed captured an unfinished INX, which
made the imported HDMA loop miss its terminating index. The new capture records
the exact hook address, observed PC, frame, NMI counter and boundary event. The
importer validates that phase, completes PLB/PLP using the existing stack loader,
and selects inputs from the observed completed wait. Message-box no-update waits
retain their measured lag NMI and unchanged input.

Seeded corpus rows use `seed = "any-boundary:50"` to select a room index under
`target/seeds/any-boundary/`. Their `seed_sha256` hashes WRAM, SRAM, VRAM, CGRAM,
OAM, APU RAM and the register CSV in that order (the index's `observation_sha256`).
Every file's contents are checked before and after replay. `start_frame` counts
from the original seed: a nonzero value verifies that many setup frames before
the selected window. Setup has no early stop and contributes no selected hits or
expectations. Local checkpoints retain both machines and bind the ROM, original
seed bytes, input prefix, audio backend and runtime codec; resumed runs require
the same seed. Setup frames remain reported separately. The boundary index
`room-seeds-any-boundary.csv` records all 197 any% entries; the initial 57-entry
prefix is unchanged. `room-seeds-100-boundary.csv` records all 331 entries of the
completed 100% instruction-boundary capture; seed binaries remain local.

Regenerate the X-ray inputs after the any% boundary capture reaches this room:

```powershell
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/seed-xray.inputs --rom rom/sm.smc --lag-log target/seeds/any-boundary/lag.csv --seed target/seeds/any-boundary/0050-A2CE-wram.bin --frames 1400
python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml --segment item-x-ray
```

The X-ray row qualifies the fresh award, completed item message and 120 subsequent
frames. Its 1,400-frame budget stops when these observations and tails complete.
The separate 1,400-frame continuation had zero oracle differences but stayed in
A2CE instead of reaching captured next room A293; that alignment issue remains
open. This row does not claim the whole room route is reproduced.

For other seeded rows, use the same import command with the row's `inputs` name,
its capture directory and indexed WRAM filename, and its input frame count below.

| Row | Capture and entry | Input frames |
| --- | --- | ---: |
| item-plasma | any-boundary:134 | 1400 |
| area-tourian | any-boundary:179 | 300 |
| area-wrecked-ship | any-boundary:160 | 300 |
| item-screw-attack | 100-boundary:131 | 684 |
| item-spring-ball | 100-boundary:239 | 1400 |
| boss-ridley | 100-boundary:118 | 2100 |
| boss-phantoon / exit regression | any-boundary:165 | 2062 |

The Phantoon exit regression replays through frame 1985 and three following
frames. Seeded observations cannot use verifier checkpoints, so its prefix begins
at the room seed. Room setup preserves the incoming accumulator `0xC8C8` when
spawning its projectile, including the shared spawn parameter at `$1993`.

`mother-brain-beam-spawn-parameters` uses `any-boundary:190` and
`seed-mother-brain.inputs` for 4,560 frames from the instruction-boundary seed.
It covers the sound-return, charging-opcode and rainbow-explosion spawn values;
this regression does not qualify Mother Brain defeat. Seeds and inputs stay local.

`boss-draygon` uses `100-boundary:259` and `seed-draygon.inputs`; require a
fresh defeat and 120 following frames. `draygon-bubble-word-counter` (18 frames)
and `space-jump-bubble-word-counter` (440 frames, `100-boundary:260`) retain the
first underwater bubble timing mismatches and three following frames.

`item-space-jump` uses `100-boundary:260` and `seed-space-jump.inputs`.
Require a fresh item award, message completion and their full 120-frame tails;
the qualified 527-frame window ends after the message at frame 406.

The `100-input-phase` capture also records the exact controller state before the
NMI input read at `$809459`. Set `SM_LSNES_BOUNDARIES=1`,
`SM_LSNES_FRAMES=222788`, and fresh log/seed paths under
`target/seeds/100-input-phase` when replaying `super-metroid-100-4010.lsmv`.
`room-seeds-100-input-phase.csv` indexes its 331 observations. The memory regions,
video/poll/boundary logs and pre-existing registers match `100-boundary`; only the
versioned controller supplement changes the observation hashes.

The importer checks the captured read's configuration, frame/NMI/room association,
all five resulting controller words, and the first replay input. At the normalized
`$80834A` boundary it restores only the recorded producer inputs, allowing the
normal controller read to reproduce the original button edges. Old seeds without
this supplement retain their existing behavior. No input words are guessed or shifted.

Seven more rows use original `100-input-phase` observations with unchanged
frame ceilings, required observations and full follow-up windows. Setup frames
are verified before selected observations begin; cached runs may reuse them.

| Row | Observation | Setup | Ceiling | Selected frames | Input words |
| --- | --- | ---: | ---: | ---: | ---: |
| `boss-kraid` | `73`, `$A59F` | 0 | 5560 | 1161 | 2779 |
| `item-bombs` | `27`, `$9804` | 0 | 1400 | 563 | 606 |
| `samus-input-poses` | `1`, `$DF45` | 0 | 440 | 440 | 440 |
| `samus-square-slope-reserve` | `92`, `$AC83` | 0 | 398 | 398 | 398 |
| `elevator-up` | `294`, `$9E9F` | 25 | 840 | 837 | 865 |
| `item-charge` | `55`, `$9D19` | 700 | 1119 | 687 | 1819 |
| `death` | `142`, `$A98D` | 3000 | 1000 | 520 | 4042 |

Input files are named `seed-<row>.inputs`. Regenerate each using the strict
`movie_inputs.py` command below, its observation and the table's input-word count.
The recordings retain every captured controller word and the original seed's
input-phase metadata. Explicit early-stop rows may have fewer inputs than their
ceiling, but still must complete every required observation and tail before EOF.

Charge's setup excludes the earlier missile message: acquisition is observed at
input 879, its own message completes at 1266, and 120 following frames complete.
The elevator finishes at 741 and retains its full 120-frame tail. Death starts
from an alive Crocomire-room observation, verifies 3000 setup frames, reaches the
complete death sequence at 3399 and keeps 120 following frames. This death is
observed in the patched paired-oracle replay, not an original-movie death capture;
save/reload coverage remains separate. The original captured trajectory crosses a square slope within the selected
window before its message pause. These seven rows remove 499,264 configured setup frames, leaving
3,725 on a cold run. At that point all eight authored recipes had other consumers.
The attempted Ice seed did not reach its required doors; that attempted
conversion left the original Ice row and recipe unchanged.

```powershell
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-item-charge.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0055-9D19-wram.bin --frames 1819
```

Six further rows retain their original ceilings and required observations:

| Row | Observation | Verified setup | Ceiling | Selected frames | Input words |
| --- | --- | ---: | ---: | ---: | ---: |
| `tas-ceres-cross-frame-read` | `5`, `$E06B` | 32 | 166 | 166 | 198 |
| `tas-ceres-four-doors` | `1`, `$DF45` | 249 | 861 | 861 | 1110 |
| `door-loading-lag-nmi` | `1`, `$DF45` | 272 | 5 | 5 | 277 |
| `samus-forced-ceres` | `6`, `$E0B5` | 1000 | 1000 | 1000 | 2300 |
| `boss-bomb-torizo` | `27`, `$9804` | 1441 | 2491 | 1664 | 3932 |
| `samus-bomb-jump-vertical` | `27`, `$9804` | 6762 | 800 | 800 | 7562 |

The first four inputs come from strict original-movie imports. Ceres seed 1
starts at native input 7990; its four-door and loading inputs exactly equal the
old route's `[7990:9100]` and `[7990:8267]` slices. Seed 5 starts at 9068;
the cross-frame inputs equal `[9068:9266]`, retaining the read at native 9216
(selected offset 116). The five-frame loading row still includes the recorded
suspension. Forced Ceres uses the existing 2300-word Ridley import, selecting
`[1000:2000]`; its required falling-x call occurs at 1268.

The last two are **authored continuations from an original observation**.
Their inputs are unchanged contiguous slices of `corpus-speed-booster.inputs`
(SHA256 `bc907fc429252418216b7f575e26be148f7cf863a71c3947b4ffa3c6c6027de6`):
`[48888:52820]` for `authored-bomb-torizo.inputs`, and `[48888:56450]`
for `authored-bomb-jump.inputs` (zero-based, end-exclusive). They are not captured
TAS continuations. Seed 27 and its input-phase metadata remain unchanged;
the first input is its recorded `0000`, with no bridge inputs or padding.
Bomb Torizo's fresh defeat is observed at 2984, followed through 3105 with the
full 120-frame tail. Bomb jump retains all 800 selected frames and five required
probe calls at inputs 6791 through 7379. Setup frames are oracle-checked before
selected observations begin, or reused from matching verified checkpoints.

These six rows remove 136,824 configured setup frames (146,580 to 9,756).
All 4,496 selected frames pass. With these conversions, 68 of 82 rows are seeded;
those conversions alone left all eight authored recipes with unseeded consumers.
Reserve and shot-gate trials lacked their required events despite zero
comparison differences; those attempts left the original rows and routes unchanged.

Four more navigation/encounter rows use original `100-input-phase` observations,
with their existing input budgets, required hits, expectations and follow-up windows:

| Row | Observation | Input file | Budget |
| --- | --- | --- | ---: |
| `area-ceres` | `1`, `$DF45` | `seed-area-ceres.inputs` | 300 |
| `area-maridia` | `170`, `$CEFB` | `seed-area-maridia.inputs` | 300 |
| `boss-ceres-ridley` | `6`, `$E0B5` | `seed-boss-ceres-ridley.inputs` | 2300 |
| `elevator-traversal` | `16`, `$97B5` | `seed-elevator-traversal.inputs` | 860 |

Maridia starts within the area and observes the actual door into `$CF54`, followed
by 120 frames; it no longer covers the former approach from Brinstar. The elevator
starts before the episode, observes its completion in `$9E9F`, then follows 120 frames.
Shared authored navigation remains required by other rows.

Eleven pickup rows now use original `100-input-phase` observations. Each retains
its existing input budget, acquisition expectations and follow-up window; rows
with message-completion requirements keep them. Dedicated recipes are removed
only when no remaining row uses them; shared navigation stays.

Area checks for Crateria and Brinstar also start from observations `12` and `17`;
Norfair reuses observation `85` and its Hi-Jump inputs. Their original budgets
and 120-frame follow-up windows remain. Morph Ball shares observation `17` and
runs all 1,200 inputs: acquisition at 590, message completion at 977 and 222
following frames. The previous authored window had 506 post-message frames; the
fixed window is retained, while that relative observation period is shorter.

| Row | Observation | Input file | Recorded inputs |
| --- | --- | --- | ---: |
| `item-morph-ball` | `17`, `$9E9F` | `seed-item-morph-ball.inputs` | 1200 |
| `item-varia` | `74`, `$A6E2` | `seed-item-varia.inputs` | 1581 |
| `pickup-energy` | `80`, `$A4B1` | `seed-pickup-energy.inputs` | 801 |
| `item-hi-jump` | `85`, `$A9E5` | `seed-item-hi-jump.inputs` | 869 |
| `item-spazer` | `173`, `$A447` | `seed-item-spazer.inputs` | 740 |
| `pickup-missile` | `19`, `$A107` | `seed-pickup-missile.inputs` | 1050 |
| `pickup-super` | `37`, `$9BC8` | `seed-pickup-super.inputs` | 1804 |
| `item-speed-booster` | `98`, `$AD1B` | `seed-speed-booster.inputs` | 692 |
| `item-wave` | `104`, `$ADDE` | `seed-wave.inputs` | 612 |
| `item-ice` | `163`, `$A890` | `seed-ice.inputs` | 808 |
| `pickup-power-bomb` | `179`, `$A3AE` | `seed-power-bomb.inputs` | 650 |

Regenerate these input files with:

```powershell
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-item-varia.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0074-A6E2-wram.bin --frames 1581
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-pickup-energy.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0080-A4B1-wram.bin --frames 801
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-item-hi-jump.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0085-A9E5-wram.bin --frames 869
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-item-spazer.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0173-A447-wram.bin --frames 740
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-pickup-missile.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0019-A107-wram.bin --frames 1050
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-pickup-super.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0037-9BC8-wram.bin --frames 1804
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-area-crateria.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0012-91F8-wram.bin --frames 300
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-item-morph-ball.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0017-9E9F-wram.bin --frames 1200
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-boss-ceres-ridley.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0006-E0B5-wram.bin --frames 2300
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-area-ceres.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0001-DF45-wram.bin --frames 300
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-area-maridia.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0170-CEFB-wram.bin --frames 300
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-elevator-traversal.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0016-97B5-wram.bin --frames 860
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-speed-booster.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0098-AD1B-wram.bin --frames 692
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-wave.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0104-ADDE-wram.bin --frames 612
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-ice.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0163-A890-wram.bin --frames 808
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-power-bomb.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0179-A3AE-wram.bin --frames 650
```

`item-gravity` uses `100-input-phase:212` and `seed-gravity.inputs` (737
input frames). It requires a fresh award, completed message and both full
120-frame tails; the qualified window is 693 frames. Regenerate its inputs with:

```powershell
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-gravity.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0212-CE40-wram.bin --frames 737
```

`item-grapple` uses `100-input-phase:148` and `seed-grapple.inputs` (1000
input frames). It requires a fresh award, completed message and both full
120-frame tails; the qualified window is 534 frames. Regenerate its inputs with:

```powershell
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-grapple.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0148-AC2B-wram.bin --frames 1000
```

`boss-botwoon` uses the published low% movie `super-metroid-low-3273.lsmv`
and `low-input-phase:122`. Capture all 167,797 frames with the same controller-phase
logger, `SM_LSNES_BOUNDARIES=1`, and fresh log/seed paths under
`target/seeds/low-input-phase`. `room-seeds-low-input-phase.csv` indexes its
186 observations. The full capture's 167,797 original movie polls, 950,641 boundary
events, 1,116 region hashes and exact controller-read metadata were audited.

The row requires a fresh Botwoon defeat and the full 120-frame tail. Defeat occurs
at input frame 824 and the qualified window ends at 945. The input file contains
1,032 observed frames; a longer import rejects a later door-epilogue boundary.
This row does not qualify that continuation or relax the importer.

```powershell
python tools/movie_inputs.py movies/super-metroid-low-3273.lsmv movies/seed-botwoon.inputs --rom rom/sm.smc --lag-log target/seeds/low-input-phase/lag.csv --seed target/seeds/low-input-phase/0122-D95E-wram.bin --frames 1032
python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml --segment boss-botwoon
```

`mother-brain-purple-breath-parameter` uses `100-input-phase:324` and
`seed-mother-brain-100.inputs`. The 2,756-frame regression covers the first
mismatch at frame 2,752 and three following frames. The original purple-breath
instruction at `$A9:9B6D` preserves its incoming opcode accumulator when spawning
the projectile; native code must pass `0x9B6D` as the shared spawn parameter.

```powershell
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-mother-brain-100.inputs --rom rom/sm.smc --lag-log target/seeds/100-input-phase/lag.csv --seed target/seeds/100-input-phase/0324-DD58-wram.bin --frames 10000
```

This regression does not qualify Mother Brain defeat. The longer 10,000-frame
100% continuation has no oracle differences but does not observe defeat; the
older any% continuation's later invalid LoROM read also remains unresolved.


### Private corpus data in hosted checks

`Segments` restores original-emulator room seeds, their lag/poll/event logs and
captured inputs from the encrypted `ci-data` release asset. The workflow pins the
ciphertext SHA-256 before decryption. The password is the hexadecimal SHA-256 of
`b"sm-decomp-seeds-v1" + ROM bytes`; OpenSSL uses AES-256-CBC with PBKDF2,
SHA-256 and 200,000 iterations. Existing ROM secrets supply the ROM; no seed key
secret is needed. Neither the password nor plaintext data is uploaded.

To replace the bundle, run `python tools/ci_seed_data.py pack <new-file>.enc` from
a checkout containing the current captured data and `rom/sm.smc`. It packs only
captures referenced by the corpus and checks every captured input's recipe hash.
Round-trip into an isolated checkout with `restore <file>.enc --sha256 <digest>`;
then upload only the ciphertext and update the workflow's asset name and digest.
Existing capture files are never overwritten. The verifier still checks each
selected input and seed against `movies/segments.toml` before replay.

Seeded rows share a shard only when their `capture:index` matches. Captured prefix
rows and generated routes retain their existing start frames and full windows;
they still need prefix replay. Pending checks remain explicitly pending.


### Mother Brain phase observations

Set `SM_LSNES_MB_PHASE_SEEDS=1` alongside `SM_LSNES_SEEDS` and
`SM_LSNES_BOUNDARIES=1` to capture additional observations when Mother Brain's
form (`$7E7800`, [bank A9:B507](https://patrickjohnston.org/bank/A9#B507)) changes
inside room `$DD58`. Use a new capture directory; the additional observations
have their own indexes and never replace an existing capture.

These observations use the same normal-wait CPU boundary and exact pre-read
controller metadata as room-entry seeds. The register companion also records
`seed_mother_brain_form` and `seed_mother_brain_body_function` for identifying the
phase. They are original-emulator observations; a new seed still needs strict
replay comparison, a fresh defeat and the complete tail before it qualifies a
boss row. No input or emulated memory is changed by this capture mode.

The original 100% movie produced 336 observations in `100-mb-phases`:
all 331 previous room observations plus five additional phase observations.
All original lag/poll/event logs remain byte-identical. The committed
`room-seeds-100-mb-phases.csv` binds the observations by hash. Final fighting
form 4 is seed 329, captured before the Mother Brain defeat bit is set.

```powershell
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-mother-brain-phase4-complete.inputs --rom rom/sm.smc --lag-log target/seeds/100-mb-phases/lag.csv --seed target/seeds/100-mb-phases/0329-DD58-wram.bin --frames 1380
python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml --segment boss-mother-brain
```

The row retains the fresh-defeat expectation and all 120 following frames,
including the beginning of the escape door transition. Credits remain pending;
this movie does not record the complete credits roll.


### Spore Spawn category movie

[Submission 4481](https://tasvideos.org/4481S) supplies the current 33,342-frame
PowerOn LSMV by nymx and Sniq. The downloaded replacement uses lsnes rr2-beta23,
bsnes v085 Compatibility and the manifest's ROM hash. Historical submission
prose describes an older BizHawk movie; use the current archive hash and frame
count in `manifest.toml`. Its stated goal is the Spore Spawn fight, ending at the
Super Missile pickup.

The unchanged read-only logger captured 39 observations in `spore-input-phase`.
`room-seeds-spore-input-phase.csv` binds all six regions and register metadata;
all 33,342 original input/poll rows and 171,074 boundary events were audited.
Seed 38 enters `$9DC7` before the Spore Spawn defeat bit is set.

```powershell
python tools/movie_inputs.py movies/super-metroid-spore-4481.lsmv movies/seed-spore-spawn.inputs --rom rom/sm.smc --lag-log target/seeds/spore-input-phase/lag.csv --seed target/seeds/spore-input-phase/0038-9DC7-wram.bin --frames 2294
python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml --segment boss-spore-spawn --segment spore-spawn-parameter
```

The 167-frame regression includes the first differing frame, 163, and three
following frames. The original [spawn instruction](https://patrickjohnston.org/bank/86#DC77)
passes the spawner's Y position in A; the translated callback must preserve that
value in the shared spawn parameter. The boss row separately retains a fresh
defeat and all 120 follow-up frames.

### Complete credits from the original movie's EOF continuation

The published 100% movie has 222,788 declared frames. Its original lsnes build
continues in read-only mode after EOF: `movie.cpp::next_input` returns zero and
records the resulting input polls. A read-only capture runs through video frame
234,849, including the complete credits roll at 234,609 and 240 following frames.
This is observed emulator behavior, not synthesized input padding. The first EOF
hardware read still consumes `0x0440` from the final declared input; subsequent
reads are zero. The importer preserves that boundary.

`room-seeds-100-credits.csv` retains observation 336 from the qualified Mother
Brain capture, including unchanged memory and register hashes. A matching
189-frame overlap (1,075 events) joins its original full prefix to the narrow
credits capture. The composed EOF metadata pins all seven capture companions,
the original movie and emulator, and the raw capture metadata. Its SHA256 is
`fd304868560fd08f823666b2ca16922597fb34b086a375c898b65166d1a757b8`.

After restoring the encrypted seed bundle, regenerate the 11,900-word input file:

```sh
python tools/movie_inputs.py movies/super-metroid-100-4010.lsmv movies/seed-credits.inputs --rom rom/sm.smc --lag-log target/seeds/100-credits/lag.csv --seed target/seeds/100-credits/0336-91F8-wram.bin --frames 11900 --eof-capture target/seeds/100-credits/lag.csv.eof.json --eof-capture-sha256 fd304868560fd08f823666b2ca16922597fb34b086a375c898b65166d1a757b8
python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml --segment credits
```

`credits` still requires fresh script advancement. `credits-complete` additionally
requires the roll-ending transition: the active credits object becomes disabled,
its cursor is deleted, and the cinematic function changes from `$DB9D` to `$E0F4`.
The main row keeps 120 native frames after both expectations. Twelve focused
regression rows retain the first differing frame and three following frames for
the cinematic spawn accumulator and post-roll HDMA-table fixes.

New captures opt in with `SM_LSNES_EOF_CONTINUATION=1` and a finite frame limit
beyond EOF. `SM_LSNES_CAPTURE_FROM` can reduce logs for overlap captures, but
cannot be combined with seed capture unless it starts at frame 1. Default movie
capture behavior is unchanged. The original standalone capture logger hash is
`3ae43931e94cf21c8ef6ef5291ef5c324af8ea4b8f74e44cfef413f623f8f8ea`;
the reusable published logger has a different hash because it adds opt-in guards.

### Golden Torizo: original Snes9x observation

The Golden seed comes from frame 186800 of JXQ's Super Metroid 100% TAS v2,
original SMV SHA-256 `a92e393a6b65ffb7b0358f168579c8d7b4ed624cb3a3445a3d4be75469da5e04`.
The v1 movie declares five controller ports and includes frame zero. The importer
retains that layout and checks every captured game input read against port 1 at
its actual movie frame. All 16 observed input bits are preserved, including the
movie's nonzero reserved bits. Other ports are not described as empty.

`room-seeds-golden-input-phase.csv` records each original seed region's size and
SHA-256. The original `sm.006` snapshot has SHA-256
`d034c893a276238becf16c40642c65d545726c0a0586735cbca2cf875ce8af05`;
its actual CPU PC is `$8DC535`. Conversion does not replace that PC with an input
boundary. The existing seed normalizer completes its pending work, and the input
importer starts after the observed first wait exit at frame 186801.

The pinned v17 live observation supplies VRAM read buffer `$2C0F`. An independently
reviewed native instruction trace from frame 186798 through 186800 proves the
CGRAM and Mode 7 byte latches are both zero, including CPU writes, DMA and HDMA.
The converter requires those pinned proofs and checks its complete output hash.
The converted seed uses the original snapshot, not a resumed emulator snapshot.

The accompanying capture covers 1300 contiguous original frames, 186800 through
188099. Its checkpoint at 187900 matches the original snapshot in every block.
The importer validates all observed input reads, then uses only complete video
frames to produce 1207 native work steps. Additional events at 188100 are checked
but are not extended with an invented video observation.

With the encrypted original capture restored under `target/seeds/golden-input-phase`,
regenerate the seed and input sequence with:

```sh
python tools/convert_snapshot.py target/seeds/golden-input-phase/snes9x.state \
  target/seeds/golden-reconverted \
  --live target/seeds/golden-input-phase/snes9x.live.json \
  --history target/seeds/golden-input-phase/snes9x.history.json --rom rom/sm.smc
python tools/movie_inputs.py 'JXQ - Super Metroid 100% TAS v2.smv' \
  movies/seed-golden-torizo.inputs --rom rom/sm.smc \
  --snes9x-capture target/seeds/golden-input-phase/snes9x.json \
  --snes9x-capture-sha256 77069a8768cd1519e321163253a7804bdecd691f48f2ea33ce4eaf0f466715bd \
  --seed target/seeds/golden-reconverted/0001-B283-wram.bin --frames 1207
```

Expected complete seed SHA-256:
`3d65a01ecba9dc550449d70db11d6172211ab4fc728334ae9ccad93a6b5d226d`.
Expected input SHA-256:
`fe331aac2f3650b33fbaa64dfc555ccbbd7c904e9fe574fbe1babb3c8dd04c34`.
