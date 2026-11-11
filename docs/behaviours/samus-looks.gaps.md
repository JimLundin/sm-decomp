# How Samus looks, part 1: integration gaps

Where wiring [the spec](samus-looks.md)'s model into the engine finds something that doesn't fit
(MODEL §5). Each gap names what the engine or a neighbouring model lacks and what it would take.
The integrator files them; the observer, or the neighbouring domain's observer, answers them.

## The engine wiring

The engine runs the model (`game::samus_looks_world`) in place of its old animation, which is
gone. Her clips and the run's paces are one importer's: `data` decodes the delay lists, through
the frames the marks reach (`data::pose::animation::reached`), and the paces
(`data::pose::animation::RunPaces`); the bridge turns them into the model's clips, each with its
step map, and the harness reads the same clips from the engine's content. The engine's
`Samus::animation` holds the State's four quantities. The RAM translation (D6, `verify::decode`)
reads `0A96` through the step map of her pose's clip, `0A94` as is and `0B3E` as the stage and
the laps; a frame on no step, a pose outside the domain, a time left past 255 or a stage past 4
is a decode error. Each frame:

- before samus-motion's step, the end her clip reaches (`Playhead::end_ahead`) and whether her
  spin has turned far are read from the State the frame starts with;
- samus-motion's step carries its whole `Outcome` (`samus_world::step::Stepped`); on a hand-over
  the super special request is the pose she took in her plain sprite with command 3, and the
  pose change takes samus-motion's pose in the sprite her dress shows (`samus_world::step::next`);
- after the pose change, the update reads samus-motion's outputs (dashing after her moves, dash
  began and lost, handed over, heading down, vertical speed after her moves, the standing
  animation's restart, the readied wall jump), her energy at the frame's start and her items;

Her sprite (`game::samus_drawing`, part 2, not yet a model) is drawn by the original's frame
number, which the bridge gives from her step through the same step map until part 2 lands.

## What is left

1. **Turned far as the frame starts.** The model reports `turned_far` at the end of a frame
   (`Report`), but samus-motion's step reads it at the start of the next, and a frame that starts
   from a seed has no frame before it. The bridge reads it again from the State
   (`samus_looks_world::turned_far`: her clip's kick step, and her step at it or past it), which
   restates `SpinTurnedFar`. For the model's next round: give it ahead from the playhead and her
   pose's clip, as `Playhead::end_ahead` gives the end.
2. **Poses outside the domain.** The model keeps a step only in the domain's poses; in another
   pose (the shinespark's, the grapple's, the x-ray scope's, knockback, the drained fall) her
   animation stands still and has no frame to draw, and a record in one doesn't decode (LEGACY,
   "Samus's animation timing in the engine"). The engine reaches none of them: samus-motion's
   model chooses only the domain's poses, and the engine runs no other pose's writer. They come
   with the domains that own those poses (spec §7).

## What other domains own

Each reaches the model as an input those domains give, or leaves it as an output they read. The
engine has none of them yet, so the bridge gives the input a domain would (LEGACY, "Samus's
animation timing in the engine"):

- **The liquids:** the slowdown, the start's slowdown and whether she is submerged: 0, 0 and not
  submerged.
- **The sound domain:** the sounds the model asks for (the wall jump's spin, the speed booster's
  boost) go nowhere, and no sound waits (`BoostingLap`): every boosting lap on record had 0
  waiting.
- **The boost's readers:** the shinespark, the echoes, her glow, the blocks a boost breaks and the
  enemies she touches read the boost stage the engine now keeps; none of them reads it yet
  (LEGACY, "The blocks' askers the engine doesn't have yet").

## What is compared

Lockstep compares her step, the time left, the boost stage and the laps left
(`samus.animation`) and the super special request (`0A2C`, `0A32`). `0A9A` is not State: every
frame's start clears it before a rule reads it (ARCHITECTURE §11.6). item-morph-ball, a
gate row, hands over at 687, 1265, 1310 and 1312 and opens its message box at 816; its ratchet
holds as it did before the wiring.

The harness compares the 2,023 frames the harness note adds: the Ceres escape's handler, the
frames a message box opens and the frames a door starts. Left uncompared: the frames in poses
outside the domain (1,688), and the masked boost frame (terrain-quicksand 156, the shinespark's).
