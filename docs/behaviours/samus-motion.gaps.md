# Samus's motion: integration gaps

Where wiring [the spec](samus-motion.md)'s model into the engine found something that doesn't
fit (MODEL §5). Each gap names the frame, the field, the expected and actual values and the
suspected entry. The integrator files them; the observer answers them in the spec.

| # | Frame | Field | Expected (today's engine) | Model | Suspected entry |
| --- | --- | --- | --- | --- | --- |
| G8 | none recorded | `position.y` sub at a wall-jump probe | the solid-enemy check of the probe zeroes the Y subpixel when she already touches the enemy (`EnemyTouchZeroesYSubpixel`) | `WallHit` carries no touch, so the bridge can't pass it | `probe_wall` (§6), `EnemyTouchZeroesYSubpixel` |
| G9 | none recorded | `position` | the arm-pump check ran for any movement type, the knockback, grapple, damage boost and shinespark types among them | `Locomotion` has no variant for them, so neither `step` nor `arm_pump_step` runs | `Locomotion` (§2), `ArmPumpStep` |
| G10 | samus-walljump-16:58, -22:64, -28:70; samus-grapple-walljump:115 | `base_speed`, `mode` | 0001.6000, Decelerating (wall jump, type 14, pose 83, to spin jump pose 81 with command 6) | 0000.0000, Accelerating (`KillXSpeed`) | `PoseCommandWrites` (`KillXSpeed`), `PoseWriteOrder`: the spin start after a wall jump may replace the pose and skip the command |
| G11 | samus-walljump-16/22/28:44 | `position.y` | 044C.37FF | 044B.37FF (spin jump, pose 82, left held, jump pressed) | `WallJumpCheck`, `VerticalStep`: 1 px |
| G12 | chain-0014-96BA:11, samus-input-poses:125, chain-0232-D5A7:24 | `vertical`, `vertical_speed` | None, 0 (turn on the ground, type E, to a spin pose 19/82 with command 2) | Up, the jump speed (`SpinStart`) | `PoseStart` for a turn-to-spin: is it `SpinStart`? |
| G13 | samus-input-poses:116, chain-0057-9FBA:9, 499, chain-0224-D0B9:188, chain-0232-D5A7:363, chain-0106-AD5E:167 | `vertical`, `vertical_speed`, `base_speed`, `mode` | Down with a small speed (spin pose to normal-jump pose 6B/6C/13, type 3 → 2) | None, 0 | the decode at N (`vertical` None in a spin jump) or `JumpOrFall`: not read yet |

Not model gaps, still open in the harness (decode notes, "Open"): the arm-pump step's 1 px move on
a slope (chain-0097-ACF0, chain-0012-91F8, samus-input-poses:476-520; x subpixel), and the crouch
take-off's 10 px lift on a pose-fit frame the radius test misses (item-morph-ball:1311,
samus-shinespark-*:184/197).

The wiring (lane wire-motion) left the free-run gate, the localiser's first differing frames and
the lockstep ratchet where they were; G8 and G9 are cases no recorded frame reaches.

## Answered

Each answer is in the spec; the gap's row is gone.

- **G1** (the arm-pump step's order): spec §8 item 7, `ArmPumpStep` (a second entry point,
  `arm_pump_step`, after pose selection has applied `vertical_result`) and the `pose_request` input.
- **G2** (a zero horizontal move): `HorizontalStep`; §6's heading. V26-V26d.
- **G3** (the Y subpixel): the new quirk `EnemyTouchZeroesYSubpixel`; §6's `Stop { blocker: Enemy }` and
  `EnemyHit`. V26, V26d, V27.
- **G4** (the elevator's step): the new quirk `ElevatorHeadsUp`; §6's heading.
- **G5** (root decision): the rows for Crouching, Transition, Moonwalking, the spring ball and
  RanIntoWall stay. They come from the original (spec §4's rows);
  the oracle judges any change they cause.
- **G6** (root decision): one motion State persists across every movement, as in the
  original: spec §1.1, with pose selection's writes in §7.
- **G7** (the speed booster's jump): settled by the original and kept deferred. `$90:9919`: with
  the speed booster equipped, the jump adds `extra_run`'s subpixel word to the vertical subspeed
  and half its pixel word (`LSR`) to the pixel word, the subpixel's carry dropped; spec §10 states
  it. Rows do equip it (the shinesparks, chain-0106, chain-0220 to 0232); the harness skips those
  frames as `Deferred::SpeedBooster` (decode notes) until the speed booster is in scope.
