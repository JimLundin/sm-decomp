# Super Metroid restart

This independent Rust workspace runs the pinned recovered-C game through one
native Game interface, with headless scenarios and interactive SDL2 presentation.
The Rust gameplay consumers and measured development commands are documented
in [the projectile pilot](PILOT.md), [room scrolling](SCROLLING.md),
[Ceres doors](DOORS.md), [room transitions](TRANSITIONS.md),
[directional door motion](DIRECTIONAL.md), [normal camera](CAMERA.md) and
[projectile updates during escape](PROJECTILE_ESCAPE.md). The ordinary Chozo
missile [instruction family](ITEM_INSTRUCTIONS.md) now also runs in Rust,
alongside ordinary Morph Ball graphics, animation and equipment pickup.
Independent [player, persistence and actor ownership options](OWNERSHIP.md)
cover vertical motion, the SRAM codec, Waver logic and both Ripper patrol variants.
The native baseline has booted, responded to controller input, escaped Ceres
through states `0x20 -> 0x21 -> 0x22`, and arrived in normal area 0 gameplay.
The retained successful run is `out/runs/ceres-06`: 16,991 host steps and 16,068
completed updates. See [ticket 04](../.scratch/restart-pipeline/issues/04-runnable-baseline.md)
for the current evidence and limitations.

The [Parlor checkpoint](PARLOR.md) extends that exact route with1,081 ordinary
input steps through Landing Site's west door and verifies resumed movement.
The authored route and saved-input replay have identical observations and final
state/image. The subsequent [Morph Ball checkpoint](MORPH.md) reaches the item,
observes acquisition and message completion, then proves resumed movement.
Its saved-input replay matches both observation logs, snapshots and final
state/image. Both are mixed C/Rust progression milestones.

Ordinary Rio can be selected independently with `--rio rust` (`--rio c` is the
default, including with `--gameplay rust`). Run results record the selected owner,
installed hook and separate root/native/Rust/refusal/service counters. This new
selection has source and syntax evidence only: full native build, compiled-C
comparison, admission/fatal controls, composition and a natural live encounter
remain pending in ticket 29. See [Rio](rio/README.md).

Default baseline gameplay uses recovered C. The optional `--gameplay rust` mode
adds ordinary projectiles, room scrolling, Ceres doors, room transitions,
directional door motion, normal camera and the ordinary Chozo missile instruction
and Morph Ball instruction families; other gameplay remains C. Rust
also owns preparation, input scenarios, presentation and supervision. The first
[original-ROM fixture](ORIGINAL.md) matches one firing call on its declared
441-byte output projection. Whole replay equivalence remains untested; the
recovered runtime uses a patched ROM view.

The workspace has its own lockfile and pinned Rust toolchain. It imports no
crates from the old workspace and requires no Python. Run Cargo from `restart/`
so its toolchain pin applies. Keep ROMs, archives and local output outside Git;
`private/`, `target/` and `out/` are ignored.

## Run the retained local build

The checked executable and prepared library are already available on this
machine. This starts an interactive session without rebuilding:

```sh
cd /home/jim/sm-decomp-restart/restart
export SM_RESTART_SDL2=/home/jim/sm-decomp/deps/mesen-runtime/usr/lib/x86_64-linux-gnu/libSDL2-2.0.so.0.10.0
export LD_LIBRARY_PATH=/home/jim/sm-decomp/deps/mesen-runtime/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
./out/bin/sm-restart play \
  --rom /home/jim/sm-decomp/rom/sm.sfc \
  --library out/native/93e9551dc34256b102e6b0466bbca4a225240a3c208593b713df5f227082904e/libsm_native.so \
  --scenario none --out "out/runs/play-$(date +%Y%m%d-%H%M%S)"
```

Arrows move, Z jumps, X shoots, C runs, Enter starts/pauses, and Escape exits.
The final recorded replay is `out/runs/ceres-recorded-final`; its complete
observations, final WRAM, inputs, PNG and result match `ceres-06` byte for byte.

## Build and prepare

Install the pinned Rust toolchain and locked dependencies before offline builds;
`rustup show` and `cargo fetch --locked` provision them with network access.
Native preparation additionally needs a GNU-compatible `cc`. `CC` can name an
alternative compiler executable.

```sh
cd /home/jim/sm-decomp-restart/restart
cargo build --locked --offline --jobs 1 -p sm-restart
mkdir -p out/bin out/runs
cp target/debug/sm-restart out/bin/sm-restart

RESTART_ROM=/home/jim/sm-decomp/rom/sm.sfc
RESTART_ARCHIVE=/home/jim/sm-decomp/deps/downloads/snesrev-sm.tar.gz

./out/bin/sm-restart doctor \
  --rom "$RESTART_ROM" --reference-archive "$RESTART_ARCHIVE"

./out/bin/sm-restart prepare \
  --reference-archive "$RESTART_ARCHIVE" --out out/native
```

`doctor` checks input hashes read-only. `prepare` authenticates the archive,
extracts private sources, applies explicit patches, and compiles the native
library. It prints the resulting library path. Set `RESTART_NATIVE` to that
printed path for the following commands:

```sh
RESTART_NATIVE='out/native/<printed-key>/libsm_native.so'
```

Replace `<printed-key>` before executing that assignment. The key depends on
the host recipe and compiler. Keep the entire prepared directory: its
`library.json`, retained source and library are verified together on reuse and
boot. See [native-host/README.md](native-host/README.md) for boundary details.

## Headless boot and Ceres

Each run starts at ordinary cold reset with zero SRAM. The `--out` directory
must not already exist; choose a new name for each experiment. Its parent must
exist, as created above. A supervisor enforces the wall-clock deadline even if
a native call hangs, independently of the host-step limit.

```sh
./out/bin/sm-restart run \
  --rom "$RESTART_ROM" --library "$RESTART_NATIVE" \
  --scenario boot --steps 12000 --deadline-seconds 60 \
  --out out/runs/boot-demo

./out/bin/sm-restart run \
  --rom "$RESTART_ROM" --library "$RESTART_NATIVE" \
  --scenario ceres --steps 22000 --deadline-seconds 90 \
  --out out/runs/ceres-demo
```

Without `--inputs`, these scenarios choose native buttons from the observed
state. `boot` requires material controller response. `ceres` additionally
requires the ordered escape transitions, matching loading state, cleared timer
and normal area 0 arrival. `none` simply runs to its step limit; it is useful
for a bounded presentation experiment and makes no named milestone claim.

Every run records the exact native buttons in `inputs.txt`. Replay the retained
successful Ceres sequence with:

```sh
./out/bin/sm-restart run \
  --rom "$RESTART_ROM" --library "$RESTART_NATIVE" \
  --scenario ceres --inputs out/runs/ceres-06/inputs.txt \
  --steps 22000 --deadline-seconds 90 \
  --out out/runs/ceres-replay-demo
```

These inputs are one 12-bit hexadecimal pad mask per native host step, in bit
order B, Y, Select, Start, Up, Down, Left, Right, A, X, L, R. They are not imported
original movie frames. Exhausting an input file before the milestone/step limit
fails; the final input is never repeated to fill a gap. On another checkout,
retain `ceres-06/inputs.txt` or use a new successful run's input file.

## Play and watch a replay

Presentation needs SDL2, a display and an audio device. The development machine
already has this runtime and its adjacent dependencies:

```sh
export SM_RESTART_SDL2=/home/jim/sm-decomp/deps/mesen-runtime/usr/lib/x86_64-linux-gnu/libSDL2-2.0.so.0.10.0
export LD_LIBRARY_PATH=/home/jim/sm-decomp/deps/mesen-runtime/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
export DISPLAY=:0

./out/bin/sm-restart play \
  --rom "$RESTART_ROM" --library "$RESTART_NATIVE" \
  --scenario none --steps 200000 --deadline-seconds 3600 \
  --out out/runs/play-demo
```

Use arrows to move, **Z** to jump, **X** to shoot, **C** to run, **D** to cancel
the selected item, **A/S** to aim down/up, **Shift** to select an item, and
**Enter** for Start/pause. **Escape** or closing the window ends the run. Focus
loss releases held buttons. Closing the window is recorded as `window-closed`,
not as a passed gameplay milestone.

To watch the saved Ceres input sequence at the native audio cadence:

```sh
./out/bin/sm-restart run \
  --rom "$RESTART_ROM" --library "$RESTART_NATIVE" \
  --scenario ceres --inputs out/runs/ceres-06/inputs.txt \
  --present replay --steps 22000 --deadline-seconds 360 \
  --out out/runs/ceres-watch-demo
```

Replay mode consumes saved input; the keyboard can close the window. `play`
uses keyboard input. `--sdl-library PATH` overrides the SDL environment variable.
For deliberately silent/headless presentation smoke, set `SDL_AUDIODRIVER=dummy`
and, if needed, `SDL_VIDEODRIVER=dummy`. Audio advancement remains inside Game
in every mode; SDL only consumes completed pixels and PCM. The presentation
cadence is `32000/534` native steps per second, not verified SNES timing.

## Results and checks

Inspect `result.toml` for the scenario verdict and distinct counters, and
`invocation.toml` for command, executable/library identities and deadline result.
Runs retain `observations.csv`, `inputs.txt`, stdout/stderr logs, source snapshots,
periodic PNGs, `final.png` and `final.wram`. `--capture-every N` changes the
periodic screenshot interval; lifecycle/room changes are also captured. Failures
retain completed observations for diagnosis. Host steps, actual input polls and
completed updates are separate counters. Zero CPU fallback does not imply Rust
gameplay ownership: this baseline still runs C gameplay.

```sh
cargo test --workspace --locked --offline --jobs 1
cargo fmt --all --check
cargo clippy --workspace --locked --offline --all-targets --jobs 1 -- -D warnings
```

Ordinary Cargo checks need neither private ROM nor recovered-C inputs. The
explicit native-integrity and real SDL checks have additional prerequisites,
documented in their crate READMEs. Examples are named `native-smoke` and
`presentation-smoke`; the main CLI owns the complete gameplay scenarios.

## Pinned inputs

| Input | Expected identity | Origin in the old repository |
| --- | --- | --- |
| ROM | 3,145,728 bytes; SHA-256 `12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72` | `movies/manifest.toml` |
| Recovered C archive | SHA-256 `69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193` | `tools/dependencies.toml` |

The archive names snesrev/sm revision
`578f90b3cc49557bb70060ad033bb90b8cf8ac50`, distributed at
<https://codeload.github.com/snesrev/sm/tar.gz/578f90b3cc49557bb70060ad033bb90b8cf8ac50>.
The source notices remain with prepared artifacts. No old gameplay acceptance
or original-ROM comparison transfers to this workspace.
