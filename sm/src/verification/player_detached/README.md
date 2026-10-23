# Detached composed check

`main.rs` directly includes the production player-projectile,
palette-effect and animated-tile cores and their safe shared kernels. It declares
no `Game`, RAM codec, backend, CPU, PPU, SDL or native runtime module. CI compiles this entry directly with rustc and runs the resulting executable.

`World` owns the projectile Session, its observations, the palette Session and its
sole current-color array, animation state, target colors and progression. Prepared
cartridge and terrain data remain fixed during each check and replay. Snapshot
comparison includes every mutable World field and compares ordered output lists
separately. Reset tests call the actual projectile reset as well as resetting the
complete independent World for replay.

Checks use synthetic authored content, not ROM-derived data:

- Actual ordinary beam input/placement/initialization and speed launch.
- Two actual flight/terrain/animation updates: rightward movement from
  `41:0000` to `45:1000`, then solid contact and impact at `51:3000`.
  The leading collision edge is center plus radius minus one; wall column 3
  starts at X = 48. Starting at 43 would already impact on the first frame.
- The original center-43 boundary is checked separately: first-frame impact at
  `49:1000`, followed by stationary explosion playback on the next frame.
- Actual impact animation and explosion drawing request, with ordered sound
  capacities and draw coordinates asserted independently.
- Actual player reset synchronously spawning the palette core, followed by an
  actual color write into the single owned color array. Fraction/auxiliary fields
  retain their documented stale values.
- Actual animation interpreter spawning and advancing palette effects, changing
  progression and target colors, and issuing acknowledged graphics uploads.
- Complete World snapshot/reset/replay and explicit unsupported-effect rejection.

Samus simulation, room-mechanism behavior, enemy-projectile behavior, HDMA/power
bomb effects, Samus palette routines, raw aliases and trail output are outside
this prepared scenario. Requests fail explicitly; they are not simulated as
successful gameplay. Sound, drawing and upload requests are synchronous terminal
outputs for this host. This check does not replace actual-old fidelity comparisons
or demonstrate a complete backend-free game.

The module wrappers include the production source files using repository-relative
paths. Gameplay algorithms are compiled directly, without copied implementations.

CI also runs `mutations.py`: an unchanged baseline must pass, then isolated
copies ignoring the positive collision radius or skipping collision retirement
must compile and fail an equality assertion in `player_checks.rs`. Compile
errors, unrelated crashes and surviving mutants fail the job. These two faults
check the collision harness's sensitivity, not comprehensive mutation coverage.
The runner is verification support and executes only in GitHub Actions.
