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
   samus-motion"). Given since samus-motion's round 18: its step reports `dashing` as her moves
   leave it, `dash began`, `dash lost`, `heading down` and `handed over` (`Outcome`), dashing is
   in the engine's State (`Motion::dashing`, which the RAM translation reads and writes as `0B3C`),
   and the engine's adapter returns the step's whole `Outcome` (`samus_world::step::Ran`).
   The harness takes dashing, dash began and lost, handed over and the readied wall jump from
   samus-motion's model wherever that model runs the frame (`SamusMotion::outcome`), and
   reads them off the records elsewhere: 16,210 of 16,210 frames still match. Left for the
   wiring round: the Tick's `samus_world::step::Stepped` carries only her pose and the readied
   wall jump to the animation, so it must carry these outputs too.
2. **The pose a clip's end hands over.** The model reports the end's kind; the pose comes from
   samus-motion's `AnimationEnds`. Given since samus-motion's round 18: the engine gives its step
   the end's kind (`animation::ends_this_frame`, `ClipEnd::EndUnlessJump` for the auto-jump
   transition), samus-motion decides whether an end unless a jump gives way, and its step reports
   `handed over`, the pose then the table's (an aimed crouch or stand-up keeping its aim). Left
   for the wiring round: the engine still takes the original's pose from the delay list's operand
   (`animation::finish`), holding the auto-jump transition back by the pad's request, and gives
   it to the pose change as the super special request, whose path also adds the clip's slowdown
   (`HandOverSlowdown`). Wiring the model replaces that request with `handed over`, the engine's
   pose id for her from samus-motion's choice and her dress (`pose_id`), never from the operand.
   The two agree on every frame the lockstep ratchet and the free-run gate compare.
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

5. **Heading down, and the rising hold's harness cases.** The harness still reads heading down
   off record N+1 (`0B36` = 2), not from samus-motion's model. The two disagree on landings: as
   her moves leave her she is heading down, and record N+1 shows the landing's direction, none.
   Both match all 16,210 frames, since a landing's new clip starts either way, but the harness
   cases of `RisingHold` (boss-kraid-defeated 297 and 315) are two such landings from a straight
   jump: with samus-motion's heading down they exercise no hold, and the rising hold's exercised
   frames go from 30 to 12. Asked (observer): hold frames for the cases, read with samus-motion's
   heading down, so that the harness and the wiring round take it from the model.

6. **Her vertical speed as her moves leave it.** `FastFall` reads her fall's speed at the
   animation's turn (root decision a: as her moves leave it, before the pose change). The
   harness reads it off record N+1, and samus-motion's `Outcome` reports her vertical speed only
   at the frame's end. Asked: samus-motion's step reports her vertical speed as her moves leave
   it, beside `heading down`.

The first half alone (`Playhead::end_ahead`, which samus-motion's step reads as
`animation_ends`) could replace `animation::ends_this_frame` today; it is left for the same step,
since it would bring the clips into the engine for one read while the rest of `game::animation`
stays.
