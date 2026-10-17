# Ordinary projectile pilot

The pilot replaces four actual gameplay entries with the same Rust implementation
used by interactive play: `FireUnchargedBeam`, `HandleProjectile`,
`DrawPlayerExplosions2`, and `DrawBombAndProjectileExplosions`. The admitted domain
is ordinary unupgraded beams and their explosions, without bombs, shinespark
echoes, or Ceres escape Mode7. Input/cooldown scheduling, enemy contact and the
rest of gameplay remain C. `SpawnPLM` is the one explicit foreign service.

The fixed scenario replays the retained `boot-02` inputs from cold reset to the
initial Ceres platform, then fires right, left and up with released intervals.
The cinematic prefix remains C-owned. Rust is activated at the first completed
normal Ceres gameplay step, before any scenario shot, and prefix counters remain
separate from the required Rust interval. The left shot waits for the turn
animation; the upward shot first holds aim to establish its direction.
The required outcome includes two individually tracked shots that move, draw,
hit terrain, draw an explosion and clear, plus the upward shot's cleanup. The
scenario rejects overlapping shots, missing material work, CPU fallback, or any
refusal/C execution at the four required Rust entries. Entry counts alone cannot
pass it. No saved state is injected and no adaptive controller chooses inputs.

## Commands

The retained binaries are `out/pilot/bin/sm-pilot` and `out/pilot/bin/sm-restart`.
The checked native library is:

```sh
PILOT_LIBRARY=out/pilot/native/bdbd7df86cbe390017a02c5b35d2392322bb44898a911dae9f00f75b293a6d4a/libsm_native.so
./out/pilot/bin/sm-pilot check \
  --library "$PILOT_LIBRARY" --rom /home/jim/sm-decomp/rom/sm.sfc \
  --reference out/pilot/runs/reference-03 \
  --out "out/pilot/runs/check-$(date +%Y%m%d-%H%M%S)"
```

The retained `dev` command rebuilds the candidate. The primary `target/` is retained
as a reusable development cache; worker targets are removed at handoff. If that
cache is removed later, the next build is cold. Retained `check` needs no build.

Run from this `restart/` directory. The ROM/archive paths below name existing
private local assets. Cargo dependencies and toolchain must already be installed.

```sh
cargo build --locked --offline -j1
mkdir -p out/pilot/runs
./target/debug/sm-restart prepare \
  --reference-archive /home/jim/sm-decomp/deps/downloads/snesrev-sm.tar.gz \
  --out out/pilot/native
```

Set `PILOT_LIBRARY` to the library path printed by `prepare`, and use fresh output
directory names for these commands:

```sh
./target/debug/sm-pilot reference \
  --library "$PILOT_LIBRARY" --rom /home/jim/sm-decomp/rom/sm.sfc \
  --prefix out/runs/boot-02/inputs.txt --out out/pilot/runs/my-reference

./target/debug/sm-pilot dev \
  --library "$PILOT_LIBRARY" --rom /home/jim/sm-decomp/rom/sm.sfc \
  --reference out/pilot/runs/my-reference --out out/pilot/runs/my-candidate
```

`reference` runs C once and records an authenticated compact trace. `dev` builds
only the candidate executable with Cargo's incremental build, validates inputs
and runs one candidate against the saved trace. When the executable, native
library, ROM, input and reference bytes are unchanged, it reuses the authenticated
last passing result and launches no game process. A README edit therefore creates
no gameplay work. No repository-wide selection or old evidence publication occurs.
`check` skips the build/cache-reuse step and always performs a fresh comparison.

The C reference identity includes pinned C revision, ROM, native library
(including host/observer implementation), input bytes, scenario and observation
schema. Candidate Rust source is excluded. Native host changes conservatively
require preparation and a new reference; candidate edits do not. Preparation
authenticates extracted sources and the compiled library. A previous result's
reuse remains evidence for its recorded source and scenario, not a new run.

Add `--mode c` to `check` for a fresh C determinism control. Add `--fault yes`
to a Rust check to deliberately move a real newly fired projectile by one pixel;
the same gameplay hook executes and comparison must reject it. On a mismatch,
the command stops at its first observed host step and retains candidate memory,
external state, effects and a PNG. Repeat C with `--capture-step N` to obtain
the expected raw state at that ordinal without storing all raw passing states.

For interactive mixed play, use the normal SDL environment from README and:

```sh
./target/debug/sm-restart play --gameplay rust --scenario none \
  --library "$PILOT_LIBRARY" --rom /home/jim/sm-decomp/rom/sm.sfc \
  --out out/pilot/runs/my-play
```

Interactive mixed play permits an unchanged-state refusal outside the admitted
domain and records C/Rust counts. A Rust error or foreign request followed by a
refusal terminates the call; it cannot retry in C. The pilot check requires Rust
throughout its declared entries and fails even if a refusal would preserve play.

## Comparison, cost and evidence

At every completed native host step, compare all 128 KiB WRAM, the host's declared
external state, framebuffer/PCM, and ordered hardware/service/scheduled-entry
events. Ownership counters are separate assertions. The native host README lists
the serialized state and effect schema. This is native C/Rust equivalence over
the exercised scenario, not original-hardware timing or universal correctness.

The trace stores three component hashes per step and authenticates its identity,
count, order and checksum before use. A mismatch reports the component and ordinal;
it never skips inputs or searches for a later matching state. Full state is
retained on failure or an explicitly requested diagnostic step. There is one
comparison implementation for reference, candidate and fresh-C controls.

`invocation.toml` records build, validation, child execution and total elapsed
time. `result.toml` records gameplay/comparison time, per-entry ownership and
material lifecycle outcomes. Inputs, lifecycle CSV, transition PNGs and final
WRAM are retained alongside. The supervisor enforces a 90-second game deadline;
the proposed warm edit-to-result budget is 30 seconds. Cold build/preparation
cost is separate. No additional subsystem should be assigned to this approach
until the actual pilot cost and outcomes are recorded.

Original-ROM comparison remains a separate result. Existing original captures
locate the initial Ceres platform, but contain no reusable original shot state
and effect trace. A native step number is not an original input/frame number.
See ticket 05 for the exact original-reference gap, measurements and cleanup.

## Measured result

The actual C reference (`reference-03`) and Rust candidate (`rust-02`, `dev-01`)
match all 8,777 observations. Rust activates at step 8,262, then completes
3 firing, 414 update, 516 beam-drawing and 516 explosion-drawing entries. There
are no refusals, foreign service calls or C-owned entries in that interval.
The cinematic prefix remains C-owned. Three shots spawn and clear; two complete
the individually tracked wall-impact/explosion lifecycle. Normal CLI replay
`cli-rust-01` produces byte-identical final WRAM and PNG through the same installer.

| Retained experiment | Wall time | Meaning |
| --- | ---: | --- |
| C reference capture | 11.87 s | One-time trace for the fixed native identity/scenario |
| Rust comparison | 15.52 s | One candidate against the saved C trace |
| Forced incremental candidate rebuild + comparison | 17.48 s | Includes 0.469 s build and 0.076 s validation; meets the 30 s pilot budget |
| README-only edit | 0.209 s | Reused validated result, zero game processes |
| Deliberate one-pixel projectile fault | 15.27 s | Rejected at the first firing step, 8,390, in WRAM |
| Normal mixed CLI replay | 8.09 s | Same gameplay implementation, without comparison hashing |

These are individual observations on this development host, not a universal
performance guarantee. The initial independent Cargo build took 22.22 s.
The compact C reference is under 1 MB. The first reference attempt exposed a
finite 6,563,600-byte setup effect burst; the fixed host retains it completely
with an 8 MiB allocation and a 64 MiB hard cap. The next scenario attempt exposed
an ignored firing press during turning; released steps fixed the authored route.
Both failures remain available alongside the passed runs.
