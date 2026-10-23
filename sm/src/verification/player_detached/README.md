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
  `43:0000` to `47:1000`, then solid contact and impact at `53:3000`.
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
