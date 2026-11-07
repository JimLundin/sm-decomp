# Samus's motion: integration gaps

Where wiring [the spec](samus-motion.md)'s model into the engine found something that doesn't
fit (MODEL §5). Each gap names the frame, the field, the expected and actual values and the
suspected entry. The integrator files them; the observer answers them in the spec.

| # | Frame | Field | Expected (today's engine) | Model | Suspected entry |
| --- | --- | --- | --- | --- | --- |
| G7 | none recorded (no row equips the speed booster) | `vertical_speed` at a jump | the jump adds half the extra run pixel word and its subpixel word, without a carry, when the speed booster is equipped | `Jump` ignores the speed booster (§10 fixes it false) | `Jump`, `SpeedBooster` (§10) |
| G8 | none recorded | `position.y` sub at a wall-jump probe | the solid-enemy check of the probe zeroes the Y subpixel when she already touches the enemy (`EnemyTouchZeroesYSubpixel`) | `WallHit` carries no touch, so the bridge can't pass it | `probe_wall` (§6), `EnemyTouchZeroesYSubpixel` |
| G9 | none recorded | `position` | the arm-pump check ran for any movement type, the knockback, grapple, damage boost and shinespark types among them | `Locomotion` has no variant for them, so neither `step` nor `arm_pump_step` runs | `Locomotion` (§2), `ArmPumpStep` |

The wiring (lane wire-motion) left the free-run gate, the localiser's first differing frames and
the lockstep ratchet where they were; G7 to G9 are cases no recorded frame reaches.

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
