# How Samus looks, part 1: integration gaps

Where wiring [the spec](samus-looks.md)'s model into the engine finds something that doesn't fit
(MODEL §5). Each gap names what the engine or a neighbouring model lacks and what it would take.
The integrator files them; the observer, or the neighbouring domain's observer, answers them.

## Round 1 in the engine (integrator, 2026-11-10, lane/looks-r1)

The harness runs the model over the importer's clips and compares it on every frame of play in
the domain: 16,210 of 16,210 matched, the null model 471. The engine still runs its own
animation (`game::animation`, and `reset_animation` and `run_super_special` in `game::pose`),
because the model can't replace it with every compared cut exact until the gaps below close. They
are one step: the model's two halves replace the engine's animation together, or not at all, since
the second half also starts the clip of the pose she takes, which today's pose change does.

1. **Samus-motion's outputs the second half reads** (the decode notes' "Requests to
   samus-motion"). The engine's motion step reports a readied wall jump, and her vertical motion
   and dash are in its State, but not as the animation's turn sees them: `dashing` as the clip
   moves on, `dash began` and `dash lost` (a loss in the pose change's speed effects comes after
   the animation, one in a running movement handler before it), and `heading down` after her
   moves. Asked: samus-motion's model reports the three as outputs of its step, as its harness
   then checks them.
2. **The pose a clip's end hands over.** The model reports the end's kind; the pose comes from
   samus-motion's `AnimationEnds`. The engine takes the original's pose from the delay list's
   operand (`animation::finish`) and gives it to the pose change as the super special request,
   whose path also adds the clip's slowdown (`HandOverSlowdown`). Wiring the model replaces that
   request with `handed over`: samus-motion's step must say whether it took the end's pose, and
   the engine's pose id for her must come from that choice and her dress (`pose_id`), never from
   the operand. Until gap 1's outputs exist the engine can't tell a turn's end that gave way from
   one that handed over except by the pad's request, as it does now.
3. **State the engine doesn't keep.** The model's State is her step, the time left and the boost.
   The engine keeps the original's frame offset into the delay list and a 16-bit timer, and no
   boost at all (LEGACY, "Samus's animation timing in the engine"): the bridge converts the frame
   offset through the importer's step map each way, and the engine's State gains the boost, which
   the RAM translation (D6) writes as the speed booster's counter. The engine's `NewPoseFrame` and
   the pose change's animation reset go: `StartingClip` decides where the new clip starts.
4. **The clips in the engine's content.** The importer lives in the harness
   (`verify::harness::samus_looks::import`). The engine needs the same clips: the ROM decoding
   (the delay lists through the frames the marks reach, and the run's paces) goes to `data`, and
   the conversion into the model's clips to the bridge, which the harness then reads too, so there
   is one importer.

The first half alone (`Playhead::end_ahead`, which samus-motion's step reads as
`animation_ends`) could replace `animation::ends_this_frame` today; it is left for the same step,
since it would bring the clips into the engine for one read while the rest of `game::animation`
stays.
