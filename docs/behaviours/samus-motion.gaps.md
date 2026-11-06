# Samus on the ground: integration gaps

Where wiring [the spec](samus-motion.md)'s model into the engine found something that doesn't
fit (MODEL §5). Each gap names the frame, the field, the expected and actual values and the
suspected entry. The integrator files them; the observer answers them in the spec.

| # | Frame | Field | Expected (today's engine) | Model | Suspected entry |
| --- | --- | --- | --- | --- | --- |

No gap is open.

## Answered

Each answer is in the spec; the gap's row is gone.

- **G1** (the arm-pump step's order): spec §6 item 6, `ArmPumpStep` (a second entry point,
  `arm_pump_step`, after pose selection has applied `vertical`) and the `pose_request` input.
- **G2** (a zero horizontal move): `HorizontalStep`; §5's heading. V26-V26d.
- **G3** (the Y subpixel): the new quirk `EnemyTouchZeroesYSubpixel`; §5's `Blocker::Enemy` and
  `EnemyHit`. V26, V26d, V27.
- **G4** (the elevator's step): the new quirk `ElevatorHeadsUp`; §5's heading.
- **G5** (root decision): the rows for Crouching, Transition, Moonwalking, the spring ball and
  RanIntoWall stay. They come from the original (checked again: §3, the note under the table);
  the oracle judges any change they cause. Spec §7 item 4.
- **G6** (root decision): one ground-motion State persists across every movement, as in the
  original. §1.1 now says so and lists pose selection's writes. Spec §7 item 5.
