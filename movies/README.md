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
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-speed-booster-settled.inputs --rom rom/sm.smc --route speed-booster-settled
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-save-death-continue.inputs --rom rom/sm.smc --route save-death-continue
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-ice-beam-settled.inputs --rom rom/sm.smc --route ice-beam-settled
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-power-bomb.inputs --rom rom/sm.smc --route power-bomb
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-ice-return-business.inputs --rom rom/sm.smc --route ice-return-business
python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-norfair-reserve.inputs --rom rom/sm.smc --route norfair-reserve
python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml --direct
```

The canonical route has 148,406 frames and SHA256
`bc907fc429252418216b7f575e26be148f7cf863a71c3947b4ffa3c6c6027de6`.
Its controller words match the historical route; canonical casing/newlines change
its byte hash. Use the new filename to preserve existing local inputs.
The settled route appends 120 released-button frames (148,526 total), allowing
the Speed Booster check its full follow-up after message completion at input 148298.
The Ice route similarly preserves the original 161,613 inputs as `ice-beam`;
`ice-beam-settled` appends 120 releases for the message's follow-up at input 161535.
`ice-return-business` retains that settled prefix, then traverses the upper shutter
bypass and collects health drops in Business Center. Its optional segment runs
all 4,426 selected frames; it does not qualify Crocomire or the Power Bomb pickup.

The `power-bomb` route retains the existing 174,992-step room-entry prefix,
then collects the first Power Bomb in A3AE. `pickup-power-bomb` requires a fresh
capacity increase, message completion and 120 follow-up frames; it does not
prove use of a Power Bomb. This authored route is a documented fallback while
published-TAS alignment is blocked at step 9,266 by timed audio-upload NMIs.
It adds gameplay coverage without claiming further published-TAS alignment.

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
The `credits` expectation observes script progress; it does not prove the entire roll completed.

The `wave-gate` authored fallback reuses the checked Speed Booster/farm return,
then crosses ADAD's moving platforms and opens its blue gate. Regenerate with
`python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-wave-gate.inputs --rom rom/sm.smc --route wave-gate`.
The additional `shot-gate-spawn-parameter` regression checks steps 153606..153615:
the re-closing gate must preserve the projectile definition as its spawn parameter.
This route does not yet qualify Wave acquisition or advance the required tally.

The `wave-beam` authored route extends `wave-gate` through ADAD's eastern pit
and overhang into ADDE. Regenerate with
`python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-wave-beam.inputs --rom rom/sm.smc --route wave-beam`.
`item-wave` starts before the fresh award, requires its completed message, and
checks 120 subsequent frames. It does not advance published-TAS timing alignment.

The `crocomire-shot` authored route extends `wave-beam` through the Norfair refill
and Crocomire fight. Regenerate with
`python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-crocomire-shot.inputs --rom rom/sm.smc --route crocomire-shot`.
`crocomire-spit-drop-scratch` observes an onscreen dust bound followed by a spit
pickup: the shared compatibility word must reach the pickup's spawn parameter.
It covers the failing update and three subsequent frames; boss completion remains
pending. Regenerate its setup from boot after the Session schema change.

The `crocomire` authored route continues `crocomire-shot` from the qualified
frame-167285 fight state through both melts, the wall break and skeleton collapse.
Regenerate it with
`python tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/corpus-crocomire.inputs --rom rom/sm.smc --route crocomire`.
`crocomire-melt-graphics-buffer` covers four frames from 169057;
`crocomire-wall-spawn-return` covers four frames from 171090 and all eight piece spawns.
`boss-crocomire` starts with the defeat flag clear at 167285, observes the flag
at 171422, and requires 120 subsequent frames. The recipe ends at 171694.

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
Every file's contents are checked before and after replay. `start_frame` must be
zero, and seeded rows never read or write verifier checkpoints. The boundary index
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
