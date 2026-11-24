# How Samus looks, part 1: integration gaps

Where wiring [the spec](samus-appearance.md)'s model into the engine finds something that doesn't fit
(MODEL §5). Each gap names what the engine or a neighbouring model lacks and what it would take.
The integrator files them; the observer, or the neighbouring domain's observer, answers them.

## The engine wiring

The engine runs the model (`game::samus_appearance_bridge`) in place of its old animation, which is
gone. Her clips and the run's paces are one importer's: `data` decodes the delay lists, through
the frames the marks reach (`data::pose::animation::reached`), and the paces
(`data::pose::animation::RunPaces`); the bridge turns them into the model's clips, each with its
step map, and the harness reads the same clips from the engine's content. The engine's
`Samus::animation` retains the complete clean State through `canonical::model::Model`, with its shape pinned for checkpoints. The RAM translation (D6, `verify::decode`)
reads `0A96` through the step map of her pose's clip, `0A94` as is and `0B3E` as the stage and
the laps; a frame on no step, a pose outside the domain, a time left past 255 or a stage past 4
is a decode error. Each frame:

- before samus-movement's step, the end her clip reaches and whether her spin has turned far are
  read from the State the frame starts with (`Ahead::of`);
- samus-movement's step carries its whole `Outcome` (`samus_movement_bridge::step::Stepped`); on a hand-over
  the pose change takes samus-movement's pose in the sprite her dress shows, as the handed-over
  pose (`samus_movement_bridge::step::next`, `pose::Asked`);
- after the pose change, the update reads samus-movement's outputs (dashing after her moves, dash
  began and lost, handed over, heading down, vertical speed after her moves, the standing
  animation's restart, the readied wall jump; `samus_appearance_bridge::Moved`, which the harness reads
  too), her energy at the frame's start and her items;

Her sprite (`game::samus_drawing`, part 2, not yet a model) is drawn by the original's frame
number, which the bridge gives from her step through the same step map until part 2 lands.

## What is left

1. **Poses outside the domain.** The model keeps a step only in the domain's poses; in another
   pose (the shinespark's, the grapple's, the x-ray scope's, knockback, the drained fall) her
   animation stands still and has no frame to draw, and a record in one doesn't decode (LEGACY,
   "Samus's animation timing in the engine"). The engine reaches none of them: samus-movement's
   model chooses only the domain's poses, and the engine runs no other pose's writer. They come
   with the domains that own those poses (spec §7).

## What other domains own

Each reaches the model as an input those domains give, or leaves it as an output they read. The
engine has none of them yet, so the bridge gives the input a domain would (LEGACY, "Samus's
animation timing in the engine"):

- **The liquids:** the step slowdown, the start slowdown and whether she is submerged: 0, 0 and not
  submerged.
- **The sound domain:** the sounds the model asks for leave as ordered typed producer requests, and no sound waits (`BoostingLap`): every boosting lap on record had 0
  waiting.
- **The boost's readers:** the shinespark, the echoes, her glow, the blocks a boost breaks and the
  enemies she touches read the boost stage the engine now keeps; none of them reads it yet
  (LEGACY, "The blocks' askers the engine doesn't have yet").

## What is compared

Lockstep compares her step, the time left, the boost stage and the laps left
(`samus.animation`). `0A9A` and the super special request (`0A2C`, `0A32`) are not State: every
frame's start clears them before a rule reads them (ARCHITECTURE §11.6). item-morph-ball, a
gate row, hands over at 687, 1265, 1310 and 1312 and opens its message box at 816; its ratchet
holds as it did before the wiring.

The harness compares the 2,023 frames the harness note adds: the Ceres escape's handler, the
frames a message box opens and the frames a door starts. Left uncompared: the frames in poses
outside the domain (1,688), and the masked boost frame (terrain-quicksand 156, the shinespark's).


## Sound opportunities and bounded native admission

The spec's §9 supplies FootContact, generic SpinStarts, normal-drawing SpinEnds and
first-passage AtTheDoor. Imported contact-step content and actual ordinary Ceres movement, selection and drawing
opportunities produce ordered typed requests through `TickOutput`. No indexed case witnesses
the first-entry AtTheDoor request until the played run carries the sound queue, and the spinning
passage arms still need their callers.
WallJumpBranch's and BoostStages' sounds join the frame's one list of sounds (spec §9.1, §9.2)
without another animation update. Normal drawing during Quieting must run SpinEnds with remembered
movement and frozen animation. The first-entry passage request precedes holding sounds.

Cinematic, boss and special-colour suppression need typed owning inputs or native-seed
validated inactive capabilities; no default-false bridge admission is evidence. Charge,
retained charging resume, echo cancellation, debug and time-up are separate unsupported
engine drawing neighbours. The native M0 observations establish their absence over0..183;
production still validates seed/restoration and preserves that capability. Active restored
owners must be supplied before complete drawing work is admitted. This capability limitation
never changes the clean model's total SpinEnds rule for supplied movements.

Ceres unboosted foot contacts have no graphical slot/OAM effect. Part2 still owns boosted
dust and liquid effects; sound suppression may not drop the graphical contact opportunity.
The library3 family at157 is CeresDoorOpening, produced by the existing Ceres door owner.
Audio replies are U1;
source opportunities alone do not establish native prefix sound-State equality.


Normal drawing's kept action is appearance State (spec §9.1), separate from `Samus.previous`
pose-change history. The engine carries it with the rest of the appearance State, tells the
model which frames prepare her work, and keeps it through paused drawing that bypasses
preparation. Frozen animation alone is not a refresh guard. Production keeps only the
animation part of the appearance State today, so the kept action, the cannon delay and the
colour sequences need the whole State carried before drawing, item controls or the palette
opportunity get an owner.

The ordinary producer capability is separately admitted from independently decoded native
seed facts: no cinematic, boss, special colour, charge/resume, echo, debug, time-up or liquid
surface. The ordinary owner preserves this certificate; Session reopening checks Ceres,
equipped items, drawing and boost quantities along with the ordinary producer/actor path.
Other gameplay has no appearance opportunity admission. Audio queues and replies are not
reseeded, made ready or compared by this producer proof; U1 remains required.

Movement publishes typed `touchdown_requests`. The scoped native soft landings at cuts 57
and 124 use those requests to offer Land to Contact, independently of Appearance's contact
graphics. The timed Frame still needs the native floor and body-fit handoff at that opportunity.
CeresDoorOpening at cut 157 comes separately from the slot-one B8 run. The shared carried shot
path offers it after the actual W1 fire opportunity.
