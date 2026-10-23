# Composed room / player checks

Draft harness for the actual integrated candidate, adapted from existing
`sm/src/verification/player_detached`. Install these files over that directory in
an isolated candidate containing all room-session production replacements.
The relative imports deliberately resolve actual candidate `sm/src/game` files.
Do not run this folder against unintegrated main or substitute declaration stubs.

One `World.player: game::session::Session` owns all player position, inventory,
projectiles, room terrain and mechanism state. Both Hosts return that same value.
Animation and palette retain their independent existing owners. There is no RAM,
Game, encoding, refresh or second mutable inventory/position in this harness.
Content is zeroed synthetic storage with handwritten records. Unsupported effects
and aliases panic explicitly. Message display yields are prepared Host responses;
this is not coverage of the actual message-box renderer/interpreter or runtime codec.

Actual algorithm cases:

- 1040 missile pickup commits before visual/music, then real missile spends it.
- 1041 beam pickup immediately changes real shot kind and damage definition;
  music effect mutates the captured message operand to verify its late read.
- 1042 missile station queues/refills before player unlock; real missile spends it.
- 1043 real plant hold changes coordinates used by real firing; contact
  invincibility stays distinct from projectile protection.
- 1044 actual scheduler executes pickup, yields, retains actual missile spend
  while suspended, resumes, then feeds a second actual missile spend.

Existing firing/collision, replay, reset/palette, animation/palette and unsupported
operation checks remain. Existing collision mutants remain, with new real-source
beam-mask and plant-position mutants. The runner distinguishes behavior assertion
kills, survived mutants, compile errors and execution errors. Mutation builds
retain the existing GitHub-CI-only restriction.

Integrator validation (not run by this worker):

```powershell
rustc --edition=2024 -O sm/src/verification/player_detached/main.rs -o <isolated-output>/composed.exe
<isolated-output>/composed.exe
# CI only:
python sm/src/verification/player_detached/mutations.py
```

Status: source syntax formatted with rustfmt skip_children; mutation Python parsed.
No Rust compilation, baseline execution or mutation execution performed. Source-only
compatibility review against candidate-e216f0ef found all imported paths present
and both Host implementations matched (the optional projectile probe uses its
production default). Updated room_host.rs adds the v2.1 alias/map methods and lazy
generic graphics transfer signature. Re-overlay that file into candidate-e216f0ef
before compiling. Gameplay and mutation verdicts remain unrun.
