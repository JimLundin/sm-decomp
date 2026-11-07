# Samus's motion: integration gaps

Where wiring [the spec](samus-motion.md)'s model into the engine found something that doesn't
fit (MODEL §5). Each gap names the frame, the field, the expected and actual values and the
suspected entry. The integrator files them; the observer answers them in the spec.

No gap is open.

Not model gaps, still open in the harness (decode notes, "Open"): the arm-pump step's 1 px move on
a slope (chain-0097-ACF0, chain-0012-91F8, samus-input-poses:476-520; x subpixel), and the crouch
take-off's 10 px lift on a pose-fit frame the radius test misses (item-morph-ball:1311,
samus-shinespark-*:184/197).

Not a model gap, open in the engine's bridge (`collision/world.rs`): `Moved.seat` (G11,
`SlopeSeat`) is always 0 there, because today's block collision has no slope alignment after a
horizontal move (the old engine left it open too); it comes with the collision domain (spec §10).
No locked row reaches it; the engine misses the 1 px down at samus-walljump-16/22/28:44.

The bridge (lane bridge-gaps) answers `WallHit.touching` from the probe's solid-enemy check, and
the animation's hand-over passes `AnimationEnd` and `Forced`; the free-run gate, the localiser's
first differing frames and the lockstep ratchet stayed where they were.

The wiring (lane wire-motion) left the free-run gate, the localiser's first differing frames and
the lockstep ratchet where they were; G8 and G9 are cases no recorded frame reaches.

## Answered

Each answer is in the spec; the gap's row is gone.

- **G10** (wall jump to spin, `KillXSpeed` applied): a decode gap, fixed in the harness. With the
  screw attack the spin start substitutes pose 81 for the proposed 19 (`$91:F6E3`), so
  `$91:F404` reports the pose replaced and the command is skipped. The harness now marks a pose
  replaced whenever it differs from the proposal. Spec §7 "Order" states it; P13. Fixes
  samus-walljump-16:58, -22:64, -28:70 and samus-grapple-walljump:115 (all match now).
- **G12** (turn to spin jumps): the turn's animation ends with "transition to pose", taken as the
  super special proposal (`$91:EBAF`). That runs the pose's initialisation (`$91:F433`) but not
  the jump transition (`$91:FBBB`, which makes the jump), and its command is "animation finished"
  (bounce = None, `$91:F36E`), not the prospective one. Spec §7 "How the pose arrives",
  `PoseCommand::AnimationEnd`, `AnimationEndPose`; `VerticalStep` (the moonfall that follows).
  P14, P14b, A24. Fixes chain-0014-96BA:11, samus-input-poses:125, chain-0232-D5A7:24 (all match
  now; decode notes).
- **G13** (spin to normal-jump pose over the floor): a bridge gap, fixed in the harness. The new
  pose's Y radius is larger, so pose fit (`$91:FDAE`) pushes her up in the same frame; the
  record's radius shows it a frame later, and the bridge read the push as a landing. The
  harness skips the frame as `PoseFit` (decode notes). samus-input-poses:116, chain-0057-9FBA:9,
  331, 499, chain-0224-D0B9:188, chain-0232-D5A7:363 and chain-0106-AD5E:167 are now skipped.
- **G11** (1 px down in a spin jump): the horizontal move ends with the slope seat
  (`$90:9376`, `$94:87F4`), which moves her down under a sloped ceiling (`0DBA` = 1 at
  record 44). Spec §6 `Moved.seat`, `SlopeSeat`; A25. Fixes samus-walljump-16/22/28:44 (all
  match now; decode notes, "The slope seat").
- **G8** (the probe's enemy touch): the wall probe's solid-enemy test (`$90:9D80`, `$90:9DF8`)
  is the same test as a move's, so a touch zeroes y's subpixel. Spec §6 `WallHit.touching`,
  `EnemyTouchZeroesYSubpixel`; A26 (constructed: no record reaches it).
- **G9** (the arm-pump step for other movement types): those types belong to deferred behaviours
  (spec §10): knockback (0A), grapple (16), damage boost (19), shinespark (1B). The harness now
  skips them by name (`Deferred::Knockback`, `Grapple`, `DamageBoost`, `Shinespark`); their
  arm-pump step comes with them.

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
