# Samus's motion: integration gaps

Where wiring [the spec](samus-motion.md)'s model into the engine found something that doesn't
fit (MODEL §5). Each gap names the frame, the field, the expected and actual values and the
suspected entry. The integrator files them; the observer answers them in the spec.

| # | Frame | Field | Expected (today's engine) | Model | Suspected entry |
| --- | --- | --- | --- | --- | --- |

No gap is open. The domain harness (item-morph-ball, area-brinstar) found none in the model: every
mismatch it showed was the decode or the bridge (see the decode notes).

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
