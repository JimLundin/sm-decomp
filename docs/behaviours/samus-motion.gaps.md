# Samus moving through the world: integration gaps

Where wiring [the spec](samus-motion.md)'s model into the engine finds something that doesn't fit
(MODEL §5). Each gap names the frame, the field, the expected and actual values and the suspected
entry. The integrator files them; the observer answers them in the spec.

The previous spec's gaps and answers are in git history (`1dc3a12`).

## Filed by the observer: take-offs from a landing pose (adversary 42's sweep, 2026-11-08)

32. **A landing pose takes off with jump held but not newly pressed (`PadChart`, `AnimationEnds`,
    `Jumping`).** Four frames, each the frame after a landing in A5h (landing from a normal jump,
    facing left), with jump held since a press in the air before the landing and no new press:
    - chain-0059-A293 107 → 108: jump pressed at 96 (in 88h, then falling in 2Ah), held through the
      landing at 107 (2Ah → A5h, 00AB.FFFF);
    - chain-0059-A293 174 → 175: pressed at 165 (2Ah), landed at 174;
    - samus-screw-attack 174 → 175 and 606 → 607: pressed at 171 and 603 (52h), landed at 174
      and 606.

    Game: A5h → 4Ch (the take-off), `0B36` 0 → 1, vertical speed 4.E000 up (6.0000 on
    samus-screw-attack, with hi-jump), x and y unchanged. Model: keeps A5h on the ground, vertical
    speed 0, not rising, and moves her 1 px left (chain-0059-A293 108: x 0764.8FFF against
    0765.8FFF). The spec's take-off arm (`Jumping`, `RisingAndFalling`: a take-off → rising) is
    right; the model doesn't choose the take-off: the pad chart's landing rows take off on a jump
    press, and none comes on these frames. Not a listed gap, and no harness mask covers them.

    Suspected: the original's auto-jump. A landing pose's animation runs an instruction
    (`$90:8370`, animation delay instruction 8) that, unless the pose already asked for is a
    take-off or a spin, installs a one-frame pose input handler (`$90:E926`); that handler counts
    jump as newly pressed when the auto-jump timer (`$0AF4`, counted by `$90:EAB3` on each frame
    jump stays held, cleared when it isn't) is 1 to 8, then reads the pose table (`$91:A172` for
    A5h: a jump tap → 4Ch). The records here don't hold `$0AF4`, so the counts are to confirm:
    from the press to the landing's record jump is held 3 frames on both samus-screw-attack
    landings, inside the window, but 11 and 9 on chain-0059-A293's, which the 1-8 window alone
    doesn't admit. Asked of the observer: the auto-jump as a rule (which poses, which window,
    what she remembers), with these four frames as vectors.

    **Observer's note** (adversary 43's check, 2026-11-08; not yet a rule). Where the timer is
    counted: `$90:EAB3` is called only from `$90:EB42`, in `$90:EB35` (draw Samus and
    projectiles), which `$A0:884D` calls at enemy layer 3 from the main gameplay loop
    (`$82:8B98`) and the door transitions, and `$8B:8E37`/`$8B:8E41` in the intro. So in gameplay
    it runs once on every frame, after her pose and movement: on a frame with jump held and held
    the frame before it adds 1, otherwise it clears the timer. Its only other writer is `$90:E93A`
    (the auto-jump handler, which clears it when it fires). The handler `$90:E926` is installed by
    animation delay instruction 8 (`$90:8370`), which ends not only the landings (A4h-A7h,
    E0h-E7h) but every turn (25h/26h, 2Fh/30h, 43h/44h, 87h/88h, 8Bh-9Ah, 9Ch-A3h) and the
    moonwalk's turns (BFh-C4h), and fires on the next frame's input when the timer is 1 to 8.
    Counting with a frame's input being the record it starts from, and the press frame's own
    count clearing the timer:
    - samus-screw-attack 174 and 606: the press frame's count clears it, then 172-173 (604-605)
      add 2: the handler sees 2, admitted.
    - chain-0059-A293 174: pressed at 165, falling in 2Ah throughout, 166-173 add 8:
      it sees 8, admitted (the 1-8 window holds; the gap's 9 counts the press frame).
    - chain-0059-A293 107: pressed at 96, the timer alone would reach 10. But the turn 88h ends
      into 29h at 99 (instruction 8 at 98), so the handler runs at 99, sees 2, injects a press a
      fall ignores and clears the timer; 99-106 bring it back to 8 (or to 1, if the turn 87h's end
      into 2Ah at 106 fires it again at 106). Either way the landing's handler at 107 admits it.

    So the timer explains all four, with what she remembers being the timer (frames jump held
    since its press, reset when a turn's or a landing's end fires the handler). The records don't
    hold `$0AF4`; the counts are the bank log's, to be confirmed with `oracle corpus
    --watch-words 0AF4` on these rows before the rule and its four vectors are written.

## Filed by the observer: the knockback's tail (adversary 43, M6, 2026-11-08)

33. **Defer the knockback's tail (harness; the camera's harness keys off it).** A frame that starts
    with the knockback on (`0A52` ≠ 0) under the normal handler is the knockback's tail: out of
    scope (spec §10, knockback is the damage domain's). Samus-motion's harness defers it, as it
    defers the knockback's handler: nothing on it is compared, x, y, the speeds, the pose, the
    placement and `rising` included. The camera's harness masks y on exactly the frames this
    predicate defers (camera.decode, Masks), so both harnesses use this one predicate; a mask of
    some fields instead of the deferral would leave the camera's mask without its trigger. Seen at
    r05 395-400 (camera C3, decode notes C3: at 400 the knockback's end hands her 52h at
    magnitude 0 where the model lands her in A5h). Asked of the integrator: the deferral, and
    the rows' compared-frame counts after it.

## Round 10 in the engine and the harness (integrator, 2026-11-08, lane/wire-r10)

The bridge (`game::samus_world`) takes round 10's API. On the ground `0B36` down at a magnitude
of 0 (a bomb jump's or a knockback's end) is `Vertical::GroundFalling`, and it writes back as down
at 0; the spin's frame count maps to the pose history as the spin enum did (`Spin::OFF`,
`Spin::CARRIED_ON`); a slope's depth is the level's surface offset as it stands, counted from the
block's open side (offsets past 16 are held as 17). `Outcome::skidding` isn't read: the speed mode
word the write-back derives from her way and facing already says it. Gap 29 is closed: the
harness's mask for falling at 0 on the ground is gone, and samus-bomb-jump-vertical 1036 → 1037
matches with y compared (the row at 286, its floor). Gap 30's three frames (the moonwalk's jump,
chain-0014-96BA 5, chain-0232-D5A7 18, samus-input-poses 119) now match on the mode word, and those
three floors rose by one. No frame newly differs on any row. The proof against main 1efc249a0
holds: the free-run gate's output and its localisation are identical, and the lockstep ratchet is
at 1372 of 1427 (item-morph-ball) and 476 of 476 (area-brinstar). The harness matches 12,639 of
12,782 compared frames (12,636 on main). No gap is filed.

## Answered: the code gate's question (adversary 32; observer, lane/r11-spec)

**Can a standing pose be falling at 0, and does the ground pull end it?** It keeps it, and
nothing on the ground ends it. Standing, running, crouched, turning on the ground, moonwalking
and against the wall all move vertically by the ground pull alone ($90:A3CD, $90:A3EB, $90:A579,
$90:A682, $90:A69A, $90:A765, each `JSR $923F`), which reads and writes no direction or speed
($90:923F-$9287). The floor it meets gives "landed" with "no change" for those movement types
($90:E61B → $90:E676: 0, 1, 5, Eh, 10h, 15h), and the start of a crouch or morph clears the
direction only for a bouncing ball ($91:ED14-$91:ED1F). So she moves exactly as with no vertical
speed and keeps it: a walk off a ledge sets speed 0, down, as it would anyway ($91:EFEF); a jump
sets the direction up; and a change of shape shows it, because the transition tests the direction
alone ($90:A638 → $90:92C7): it moves her 0 px (never stopped, so neither landed nor a walk-off:
$90:E65A gives "no change" for movement type Fh) and the pull grows the speed to 0.1C00, where with
none she is pulled onto the floor. In scope nothing hands a standing pose this state: a landing
clears it ($91:F1D3, $90:A640), the ball falling at 0 walks off its first frame (BG2), and a change
of shape falling at 0 grows the speed the same frame; only a knockback's end ($91:F31D: speed 0,
down) or a bomb jump's (gap 29), both out of scope, leave it. The spec says so under
`FollowingTheGround` (its kept-speed arm now names falling at 0, one arm as before) and in §1.2.
Vector BG3 (constructed): standing falling at 0, down pressed, then the crouch's first frame:
y 00B0.FFFF kept, speed +0.1C00. The round-10 model passes it as it stands. No unit change
(204.3).

## Answered: the code gate's questions (adversary 29; observer, lane/r10-spec)

Adversary 29 asked four questions of round 9's model. Each is answered in the spec.

1. **A spring ball rising from a bounce, both sides held: is it "falling"?** It is if it bounced
   out of a fall. "Falling" and "on its hop" are the spring ball's pose, not the way it moves. A
   bounce keeps the pose the ball landed in: the landing's pose choice keeps the current pose for
   bounce 1 and bounce 2 ($91:EA63 → $91:EA83/$91:EA93), and only the landing that ends the
   bounces picks the pose on the ground ($91:EA9B). So a rise from a bounce after a fall is still
   falling and turns round, and a rise after the hop is still the hop and turns rightward.
   `PadChart` states this beside the two both-held arms. No unit change.
2. **A spin step that lands exactly on the top speed: has she "reached" it?** No. The spin's speed
   step ($90:9B1F, from $90:9069) returns "reached" (carry set, $90:9BCF) only where it caps the
   speed: a pixel part over the top's, or an equal pixel part with a subpixel part over the top's
   ($90:9B44). An exact landing falls to $90:9BB7 (carry clear), so without forward held and
   without a skid the spin stops drifting ($90:906C → $90:9093). Round 9's `speed != sum` test is
   right. `DriftInTheAir` defines "reached" as the step being cut to the top speed (`Exact`: the
   sum over it). No unit change.
3. **Does an adjusted second bounce under 1 px floor at 0?** No, and the spec gives no floor
   (amended after adversary 31). The original takes 1 off the bounce speed's pixel word alone
   ($91:F23F, $91:F2BA `DEC A`), so a bounce speed of 0.8000 would store a rise of FFFF.8000.
   With position wrapping mod 2^32, she would then sink about 0.5 px a frame, and any ceiling
   would stop her. That is perceivable, and an adjusted value must run the same rule as the
   vanilla one (MODEL §3), so a floor would be a different behaviour, not a guard. The vanilla
   speed is 1.0000, which gives exactly 0. So the bounce speed's parameter takes a range instead:
   at least 1.0000 (`Jumping`'s params). `Landing` keeps `bounce − 1.0000`, +0.3 units (its
   operator).
4. **May `Slope::surface` be held from its open side?** Yes. It is a question of representation,
   and the spec can state the surface either way without changing behaviour. As depth from the
   open side (a floor's top row as it is, a ceiling's bottom row as 15 less it), 16 (an empty
   column) and 17 (a column far past) mean the same for floors and ceilings. Every `SlopesCarry`
   rule then reads the same from either side: the last air row and the first air row are both
   one less than the depth, counted from the open side. That is why the vectors already write 16
   and 17 alike for both. `SlopesCarry` says so, with no formula. The vectors' encoding stays as
   it is, and a model converts it at the world's edge. No unit change.

## Answered: round 9 in the engine and the harness (integrator, 2026-11-08, lane/wire-r9; observer, lane/wire-r9)

The bridge (`game::samus_world::step`) takes round 9's API. Gap 28's answer is wired: where pose
fit or the shrink placed her, the camera's previous y pixel (`0B14`) is the step's `camera_from`,
and the bridge no longer infers the push from the looks' cells. On a frame without a placement
`camera_from` is her y at the frame's start, and the engine's word is left as it was: writing it
every frame broke the lockstep ratchet in the elevator room (area-brinstar 476 → 449 Ticks
equal), where the elevator moves her outside the step. The let-go mode word of adversary 28's
AS4 note is masked, not derived: running with neither side held, standing the next frame
("unperceived: the running let-go row's speed mode ...", 37 frames over the 32 rows, all of which
matched). The proof against main 8039dc646 holds: the free-run gate's output and its
localisation are identical, and the lockstep ratchet is at 1372 of 1427 (item-morph-ball) and
476 of 476 (area-brinstar). The domain harness matches 12,526 of 12,782 compared frames (12,339
on main): gaps 20, 21, 24 and 27's frames, the moonfall turn (chain-0232-D5A7) and the let-go row
(samus-space-jump-chain and the three wall-jump rows at 13) now match, and 24 floors rose. One
row fell (item 29); the frames that newly differ are items 29-31.

29. **A walk-off at rest moves her the whole pull (`FollowingTheGround`, SL3).**
    samus-bomb-jump-vertical 1036 → 1037: the ball on the ground (1Dh) the frame after the bomb
    jump's movement, speeds and dash 0, `0B36` = 2 with `0B2E`.`0B2C` 0, nothing held, nothing
    under her (record 1037 is the falling ball, 31h). y: model 00A7.0BFF (the walk-off moves her
    the whole pull, 1.0000), game 00A6.0BFF (no move). Both then fall from rest: 1037 → 1038
    matches (vertical speed 0.1C00). Round 8 matched: its walk-off fell from rest without the
    move. Round 9's walk-off arm (gap 21's answer, SL3) moves her the pull; here, with no
    horizontal move, the game doesn't. The row is 285 against its floor of 286, so the floors
    test fails locally on it until this is answered; the floor is kept. Asked: which walk-offs
    move her the pull (a horizontal move first? the ball at rest?).
    **Harness** (observer, lane/land-r9; the spec's answer comes with round 10): the frame is out of
    this domain's scope and y is masked, "deferred: a bomb jump's or a knockback's end (...)": the
    ball on the ground falling at a speed of 0, which on the ground only a bomb jump's or a
    knockback's end leaves ($90:8F1B, $91:F31D; adversary 30).
    **Answered** (observer, lane/wire-r9; amended on lane/r10-spec): every walk-off moves her the
    whole pull, the ball at rest included, as SL3 has it; this frame differs because the ball isn't
    at rest. A bomb jump lifts the ball without changing its ground pose and hands back at its
    peak, where the rise's end has left her falling at a speed of 0. The ball's ground move runs
    the ground pull only with no vertical motion; falling (or rising) it moves by its own vertical
    speed under gravity instead, here 0 px, and finding no floor that is a walk off a ledge, so she
    falls from rest where she was (record 1037: the falling ball, 0, falling). With no vertical
    motion (the state the spec's ground ball has, §1.2) the pull runs and a walk-off moves her the
    whole of it. Falling at 0 on the ground arises only from a bomb jump's or a knockback's end
    ($90:8F1B, $91:F31D), both out of scope, so the start can't be reached by a lead-in; the spec
    now owns the move anyway (adversary 30's recommendation), so the mask can go once the model
    makes it: the vertical speed's row (§1) and §1.2 name falling at 0 on the ground, handed back
    by such a push; `FollowingTheGround`'s arm for a change of shape or the ball on the ground
    with a vertical speed now reads "with a vertical motion (a vertical speed, or falling at 0)"
    and cites BG2, with a note that a move of 0 is a walk off from where she is (adversary 31:
    a separate arm would have the same outcome, so it was folded in); `SpeedBuildsAndBrakes`' ball-at-rest arms read "a vertical motion"
    (falling at 0 keeps both speeds, no halt). The vectors' `before` gains `falling` (true: falling
    at the start's vertical speed of 0 in a pose on the ground), and BG2
    (samus-bomb-jump-vertical:1037) is this frame, recorded: the ball `Still` at 00BB.FFFF,
    00A6.0BFF, all speeds 0, falling, in open air → y kept, the ball `Falling`. Round 9 fails BG2
    (y 1 px low, and its parser refuses `falling`) until the implementer's next round; the
    harness's mask stays until then. Spec units 204.0 → 204.3 with the code gate's answers above
    (the second bounce's `bounce − 1.0000`, +0.3); budget 6,912 → 6,924. Sources in the decode
    notes (gap 29).
30. **The moonwalk's jump: the way her speed points (`TurningRound`, `PadChart`, MT1).**
    chain-0014-96BA 4 → 5, chain-0232-D5A7 17 → 18 and samus-input-poses 118 → 119: moonwalking
    (49h, 4Ah, 49h) at speed 0, following the pad, jump pressed: the moonwalk's jump (C0h, BFh,
    C0h). Both move her 0.8000 the way she walks and leave her speed at 0.8000. `0B4A`: model 2
    (carries on along her facing), game 1 (a skid, against the turn's facing word). Round 9's
    turn keeps her facing (MT1), and `TurnRound` sets her way to her facing at the frame's start,
    so her way and the turn's facing agree; round 8's turn faced back, so they didn't, and it
    matched. No later frame differs (x and her speeds match through the turn). Asked: which way
    her speed points after the moonwalk's jump; or, if that never shows, whether the mode word on
    that frame is evidence, as with AS4.
    **Answered** (observer, lane/wire-r9): it is evidence, and the spec is amended. The moonwalk's
    jump leaves her skidding like every other turn: her speed (0.8000) points against her facing,
    back the way her sprite looks, though the moonwalk had just carried her along it. The turn's
    start sets the skid whatever her facing, and with the facing kept that reverses the way her
    speed points. Unlike AS4 (carrying on against following the pad, at a standstill), this is
    the sign of a speed of 0.8000, and it is perceivable: no later move shows it, since the turn's
    first frame brakes 0.8000 to 0 whether she skids or carries on, but the camera reads the skid
    on the same frame and leads her backwards, as through the moonwalk; a carried-on speed would
    swing it forwards. So it isn't masked. `TurningRound` gains the arm (the moonwalk's jump →
    she skids against her facing [MT2]) and its consequence (the next frame's brake to 0, which
    only the camera's lead outlasts). Whether she skids is output, in §1.2's skid (amended on
    lane/r10-spec after adversary 31: not on the horizontal speed's Read by, since a skid of zero
    can't show in a speed of 0). MT2 (chain-0014-96BA:5): moonwalking right at 0 with right held, jump pressed →
    019C.0000, speed −0.8000. Round 9 fails MT2 on `speed_x` alone (+0.8000) until the
    implementer's next round; the three harness frames stay at their floors until then. Spec
    units 203.0 → 204.0; budget 6,876 → 6,912. Sources in the decode notes (gap 30).
31. **Down a slope on the speed booster's dash, the pull's row is the booster's (`SlopesCarry`,
    `FollowingTheGround`; with gap 27).** samus-mockball and the four samus-shinespark rows, frames
    10 → 11, 11 → 12, 13 → 14, 15 → 16 and 16 → 17: running down the slope with run held and a dash
    of 3.0000 to 3.5000. y: model 1 px higher (samus-mockball 10 → 11: from 048E.FFFF, model
    0492.FFFF, game 0493.FFFF). Gap 27's frames (9 → 10, 12 → 13, and 17 → 18 to 21 → 22) now match.
    Localised: the booster's dash past 2.0 is deferred (the mask takes x and the dash), so the
    model's dash stops at 2.0000; its pull is 5.C000 (2.C000 + 2.0000 + 1) against the game's
    6.D000, and her centre column is 1.1000 behind. On a sloped floor both decide where the pull
    stops: the carry and the surface's row at her centre column. With the dash's cap lifted in a
    local run (`Rules::dash_cap` 10.0000, not committed), samus-mockball's y matches on all 129
    frames and these frames match on all five rows. So neither gap 27's frames matching nor these
    differing is evidence of `SlopesCarry`. Asked: whether the booster's mask should also take y
    while that dash runs her over a sloped floor.
    **Answered** (observer, lane/wire-r9): yes. On a sloped floor where she ends up depends on her
    x, through the surface's height at her centre column, and on the pull, `|speed| + dash + 1`;
    the booster's dash past 2.0 sets both, so y there is the booster's as x is. The booster mask
    gains a mask of its own: where the booster's x mask applies (running with run held, a dash of
    2.0 or more, the booster equipped) and a sloped floor block lies within one block of her feet,
    y is masked, "deferred: the speed booster on a sloped floor (...)". As first tested it took 24
    frames of samus-mockball and 81 of each shinespark row (adversary 30 narrowed it, below).
    Checked by a local run with the cap lifted and x unmasked: y matches on all of them but shinespark 110, 118, 142 and 146, where x is a pixel
    ahead, the model's growth of the dash not being the booster's, so those are the booster's too.
    Shinespark 198 (left, up-left) and 185 (right, up) are off the slope test and still differ.
    Floors: samus-mockball 124 → 129, shinespark-left 289 → 315, -right 267 → 293, -up 186 → 212,
    -up-left 240 → 266. Sources in the decode notes (gap 31).
    **Harness** (observer, lane/land-r9): yes, y is masked where the booster's x mask applies and a
    floor slope lies under the columns her centre covers from this record's x to the next's, in
    her bottom's row or either neighbour (adversary 30's narrowing: 20 frames of samus-mockball, 70
    of each shinespark row, all five rows still at 129, 315, 293, 212 and 266). The dash reaches y
    through the pull only following the ground down; where the slope adjusted the move the pull
    is 1.0 and it reaches y through x alone.

The harness after these answers: 12,636 of 12,782 compared frames (null model 5,583), every row
at or above its floor, five floors raised.

## Answered: round 8 in the engine and the harness (integrator, 2026-11-08, lane/wire-r8; observer, lane/r9-spec)

Round 8 dropped `Outcome::change` and `Outcome::met` (adversary 27, item 5). The bridge
(`game::samus_world::step`) now derives what the engine read from them, from her before and
after the step and the cells it reached, and the proof holds: the free-run gate's eight green
pairs, the lockstep ratchet at 1372 of 1427 (item-morph-ball) and 476 of 476 (area-brinstar), and
the free run's localisation as on the spine. The domain harness matches 12,325 of 12,782 compared
frames over the 32 rows (11,433 at round 7, 5,379 for the null model). The pad of a frame N to N+1
is record N's.

25. **The frame a hit starts knockback, running: the aim change's step (`AimingAndShooting`,
    `SolidEnemies`?).** chain-0106-AD5E 106 → 107: running (11h), base speed 2.C000, dash 2.0000,
    nothing held; the hit starts knockback (record 107 is in it). x: model 00D0.3C00 (4.C000 and
    the running let-go row's 1 px step), game 00CF.3C00 (4.C000, no step). 650 → 651 the same:
    running (0Ah) at 0.6000, nothing held: model 01D7.B000, game 01D8.B000. Round 7 matched both
    (no let-go step). The knockback mask takes her speeds, not x. Asked, with gap 22: whether the
    hit's pose replaces the chosen running pose before the step, so that frame brings none, or
    whether x on that frame isn't this domain's. This is the one row below its floor
    (chain-0106-AD5E 479, floor 480: 134 now matches, these two don't).
    **Answered** (observer, lane/knockback-mask; with gap 22): the hit's pose replaces the chosen
    pose before the step, so x on that frame is knockback's, not this domain's. The original runs
    the frame's motion first, then the hit interruption ($90:DDE9, after the movement handler in
    $90:E725), which sets the start-knockback command with the knockback's pose (53h or 54h by
    her facing, $90:DEFA; morphed, her own pose, $90:DF15). The pose update ($91:EB88) takes that
    pose ahead of the pad's: it skips the check that brings the aim change's step ($91:EADE) and
    the pad's pose-change command (the let-go row's decelerate, $91:EC50), so x and her base
    speed, dash, mode and momentum stay as the motion left them (106 → 107: base 2.C000, dash
    2.0000, accelerating, momentum kept; the model merged the dash). The start then sets her
    vertical speed upward ($90:99D6: 5.0 in air, 2.0 in water, Up) and the knockback's movement
    handler ($DF38, $91:EE1F). The marker: in record N+1 the movement handler is $DF38 and her
    pose 53h or 54h (movement type 0Ah), and her health drops in the same frame (014Bh → 0132h
    at 107, 0119h → 0100h at 257, 00CEh → 00B5h at 294), with the invincibility timer at 5Fh.
    The harness now detects the start by that handler (a morphed hit keeps her pose but takes
    it) and masks x, y, base speed, dash, momentum, mode, vertical speed and vertical on that
    frame, "deferred: knockback's start …"; bounce stays compared, as nothing in the start
    writes it. The frames after it are already skipped as another domain's: the knockback's
    handler ("deferred: knockback") and its poses 53h, 54h and the damage boost's 50h ("isn't in
    the domain") until she is back in a domain pose with the normal handler (chain-0106-AD5E 109,
    263, 300 are compared as usual). chain-0106-AD5E 479 → 492 (floor 480 → 492: these two,
    gap 22's two and 9 more start frames that differed in y), chain-0057-9FBA
    533 → 535 (297's step, 393's fit; floor raised); the same 37 frames masked over the 32 rows
    as before, with no other row changed.
26. **Standing from a run at speed 0 keeps her carrying on (`PadChart`'s running let-go row,
    AS3).** samus-space-jump-chain 12 → 13, and samus-walljump-16, -22 and -28 12 → 13: running
    (09h), speed 0 at the frame's start, nothing held. Both move her 0.3000 and leave her speed
    0.3000; `0B4A`: model 2 (carries on, along her facing), game 0 (follows the pad). Round 7
    matched. Asked: how she carries on after the let-go row's "upright `Still` when it is 0".
    **Answered** (observer, lane/r9-spec): she doesn't. The let-go effect is decided, like the
    pose, by her horizontal speed at the frame's start: 0, so she stands following the pad, and
    keeps the 0.3000 the frame's run gave her (the run follows the pad, so it gains its step). The
    model read the effect cell's "with a horizontal speed of 0" after the move. The cell now says
    "as running", and `PadChart` states it as two arms: speed at the start not 0 and still not 0
    after the move, she carries on (AS1, AS2); 0 at the start, or brought to 0 by the move, she
    follows the pad again (AS3, AS4). Vector AS4 (samus-space-jump-chain:13). No later frame
    shows it (standing halts her either way), so only the harness's compare of her mode does.
    For the integrator (adversary 28): AS4's distinction shows in no perceivable output, so AS4
    is not evidence. If the harness compares the let-go mode word on such frames (a running pose
    let go at speed 0, standing the next frame), mask it there, or accept the model's value as
    derived, rather than let that word drive the model.
27. **The run with a kept vertical speed, down slopes: 1 px low (`FollowingTheGround`,
    `SlopesCarry`; with gaps 20 and 24).** Gap 23's frames now match but for these: samus-mockball
    and the four samus-shinespark rows, frames 10, 13 and 18-22 (and 79-80, 95-96 on the
    shinespark rows): running (09h) with +0.E000 kept, down a slope. y: model 1 px lower
    (samus-shinespark-left 9 → 10: from 048B.FFFF, model 048E.FFFF, game 048D.FFFF). The same
    1 px as items 20 and 24 where the floor falls away.
    **Answered** (observer, lane/r9-spec; with 20 and 24): the kept speed takes no part; the pixel
    is a subpixel carry. A downward move onto a sloped floor finds its stop from her position plus
    the whole move, subpixels added, but moves her by whole pixels from her position: when her y
    subpixel and the move's subpixel part carry into a pixel (y subpixel FFFF, and a pull of
    6.C000 here), the target's row is one lower and the stop leaves her bottom edge 1 px above the
    surface's last air row. 9 → 10: target 0492.BFFF, bottom row 6, the surface's top row 3 at
    her centre column: moved 6 + 3 − 1 − 6 = 2 px, 048D.FFFF. `SlopesCarry` states the stop as
    `max(0, d + t − 1 − b)` with two arms (no carry: on the last air row; a carry: 1 px above),
    `BlocksStopFlush` the target, and `FollowingTheGround` that the kept speed changes only the
    walk on a slope (`SlopesSlowTheWalk`, at a vertical speed of 0 only). Vectors SL1, SL2
    (samus-mockball 91, 92) and C-slope-carry-kept (the kept speed).
28. **What the edge derives, for the observer to confirm or make an output (`PoseFitAfterMoves`,
    `TouchedCells`).**
    - The camera's previous y pixel (`0B14`) becomes hers where pose fit places her: the shrink,
      and a change to a taller body, not facing the screen, whose looks found something. The
      bridge reads "found" as standing on the ground as the frame starts or ends (a landing on a
      solid enemy leaves no cells), or one of the last cells reached, the looks after every move,
      being other than air. item-morph-ball 738 → 739 (18h → 52h, rising, the look down meets the
      floor) needs the second; the near-wall test's looks come before the vertical move's, so
      816 → 817's don't count. Every recorded frame agrees, but the rule infers the push from
      the cells' order and kinds. Asked: pose fit's push or crouch as a step output, as adversary
      27 suggests, if the spec wants it said.
    - A pose request is recorded when the pad asked for one or the chosen pose differs from hers;
      no recorded frame tells it from "her moves met something".
    - On the ground the game leaves her vertical direction word as it was. chain-0232-D5A7
      38 → 44: turning in a moonfall (92h, movement type 17h, `0B36` = 0, `0B2E`.`0B2C`
      FFFE.7800); the turn's handler pulls her as the ground's does and keeps the word. The bridge
      gives the model `Ground` with the kept speed there, and writes back no direction while she
      is still on the ground; a turn in a jump or a fall with `0B36` = 2 is falling, as the
      vectors' starts have it. Asked: the spec's phase for a turn in a moonfall (the decode notes
      call `0B36` = 0 with a magnitude outside the airborne types not plain).
    **Answered** (observer, lane/r9-spec):
    - The camera: pose fit's push and a change of shape's placement are perceivable, in the
      screen's scroll, which the original makes from her move since a reference height that
      those placements reset (its vertical speed the frame after is the distance from it). The
      step now reports **where the camera follows her from** (§1, an output; vectors
      `camera_from`): her y at the frame's start; on a frame pose fit moves her (a push, or the
      crouch taken instead raising her), her y pixel just after that with the start's subpixel
      (`PoseFitAfterMoves`); on a frame a crouch, a morph, standing up or unmorphing starts, her
      y pixel after the lowering (0 px for the last two), likewise (`ShrinkingLowersHer`). The
      look's cells and their kinds are not the test: a growing body that finds nothing, or a
      refused change, doesn't move the reference. Vectors PF1, PF2, PF3, U1, W13, SH1-SH3 (a
      placement) and W11 (none) gain `camera_from` from the records' scroll speed;
      item-morph-ball 738 → 739 agrees (023D.73FF).
    - The pose request: outside the domain (the engine's pose history). The step's chosen pose
      is its output; no spec change.
    - The turn in a moonfall: `Moonfall` now has it. The moonfall is in her vertical motion, so a
      turn in a jump or a fall during it moves as a turn on the ground over a drop: no gravity,
      no move by her vertical speed, which she keeps unchanged, and the ground pull each frame;
      the moonfall goes on from the turn's end. Vector MT1 (chain-0232-D5A7 38 → 39, with the
      lead-in from the moonwalk at 17); `TurningRound` points to it.

The walk-off drift that round 8 widened (adversary 27, item 4) shows nowhere: the gate's pairs
hold, and no compared frame of the 32 rows newly differs in x but items 25's two.

## Answered: the Tick on the model (integrator, 2026-11-07, lane/wire-tick; observer, lane/gaps-tick)

The engine's Tick now steps Samus with the model (`game::samus_world::step`): the State is
rebuilt as the model's Samus each frame from the game's words and written back after the step
(`samus_world::state`). Every free-run gate pair holds; the lockstep ratchet falls on
item-morph-ball (1372 to 1296 Ticks fully equal). Each item below is a first differing frame
of the lockstep (record N decoded, one Tick, compared with record N+1), with the engine's value
first. None is the adapter's: each step's input matches what the game held at record N.

Items 10-16 are answered in the spec (lane/gaps-tick), each with a recorded vector from
item-morph-ball (`W11`-`W20`; gap 10's is RW1's new `readied`). The pad of a frame N to N+1 is
record N's.

10. **The wall test's result without the kick (`WallJump`).** The engine's animation holds the
    spin a frame short of the kick while a wall is near and the spin hasn't turned far enough
    (the game's armed wall jump). `Outcome` reports only the kick, so the adapter steps a copy
    of her with the spin turned far and jump pressed and reads `Met::WallKick`. Asked: the
    test's answer in `Outcome` (a wall near, kicked or not). No recorded frame of the two rows
    needs it.
    **Answered.** `WallJump`'s new output: each frame reports readied (a wall near, found by the
    block test or the solid enemy test, while the spin hasn't turned far: the animation skips
    ahead), kicked (`WhatHerMovesMet`'s wall jump), or neither (no test, no wall, or a wall near
    once the spin has turned far without jump pressed). A wall near after the spin has turned far
    readies nothing, so a copy stepped with the spin turned far can't stand in for it. Vector: RW1
    (samus-walljump-16:47, the readying frame) gains `readied = true`; no item-morph-ball frame
    readies.
11. **The ground pull with nothing under her takes the whole speed, subpixel and all
    (`FollowingTheGround`, `TurningRound`).** item-morph-ball 682 → 683, turning on the ground
    (pose 25h) over air, speed 0.C000: y 02A9.0000 → engine 02AA.0000, game 02AA.C000. The
    game pulls her by the speed plus 1 px (1.C000); the model by the speed's whole pixels plus
    1 px. On a floor the flush stop hides the difference.
    **Answered.** The pull is the frame's speed (horizontal speed plus dash) with its subpixel
    part, plus 1.0000; "whole speed" meant all of it. `FollowingTheGround` and `TurningRound`
    say so. Vector W11 (682 → 683).
12. **The aim step also comes when the pad asks for a running pose from another pose
    (`AimingAndShooting`).** item-morph-ball 687 → 688 (standing 02h → running 12h, left and aim
    down held): x 057D.D000 → engine 057D.D000, game 057C.D000; 740 → 741 (landing A5h →
    running) and 1327 → 1328 (landing E4h → running 11h) the same 1 px. Into a wall it puts her
    against it: 1334 → 1335 (standing 07h, right and aim down held, a wall ahead): engine pose
    09h (running), game 89h (against the wall, aimed down-forward), and so on every other frame
    to the row's end. The spec has the step only while running and changing aim.
    **Answered.** The step comes whenever the pose chosen from the pad and what her moves met is a
    running pose other than hers, from any pose; forward is her facing at the frame's start. A
    landing or a fall that replaces the pad's choice brings none, and an animation's end neither
    brings nor cancels it (§11's and §7's open item on that is closed). A block or solid enemy in
    the way gives `AgainstWall` with the chosen running pose's aim (`WhatHerMovesMet`). At 1334
    the pad (record 1334's) is run and right, no aim: the chosen pose is running unaimed (09h),
    the wall is 0 px ahead, so she is against it unaimed (89h). Vectors W12 (687), W17 (740),
    W18 (1327), W19 (1334).
13. **A stop short of a solid enemy clears her y subpixel (`SolidEnemies`).** item-morph-ball
    796 → 797, falling onto a solid enemy 1 px below (no block near): y 029A.2BFF → engine
    029B.2BFF, game 029B.0000; 1227 → 1228 the same (029B.9400 against 029B.0000). Standing on
    it after, 1229 → 1230 (running 09h, y 029B.0000), the engine's pull finds no floor (pose 29h,
    falling) and the game keeps her running.
    **Answered.** The spec's gap was off by one: it is the enemy's near side less hers, the sides
    being the boxes' edges at centre ± half-size, with no "less 1" (boxes whose pixels are next
    to each other are touching). At 1227 the enemy is 2 px below (her bottom side 02AEh, its top
    02B0h): the landing stops her 2 px down, subpixel kept (029D.9400); the landing pose's body is
    2 px taller, and pose fit's look down (a test like any move's) finds her touching the enemy,
    which sets her y subpixel to 0, and the push lifts her 2 px: 029B.0000. 796 is the same with
    3 px. At 1229 her bottom side is the enemy's top: the pull touches it and she stays on it. So
    no subpixel rule changes; only the gap's measure. The constructed vectors C-enemy-ahead,
    C-enemy-adjacent, C-enemy-down and C-aim-enemy are redone on the corrected gap (two keep their
    `after` with the enemy or the start moved 1 px; C-enemy-ahead now stops 2 px). §11 item 5 is
    corrected. Vectors W13 (1227), W20 (1229). A note for the adapter: the body size a frame
    uses is the pose's own (§1.3); the game's radius word still holds the old pose's 13h at
    records 740, 797 and 1228 (after a landing that grew her), and the next frame's moves use 15h.
14. **Reversing a spin while skidding keeps the skid (`TurningRound`).** item-morph-ball
    767 → 768: spinning right (19h) while skidding left (`0B4A` = 1, speed 2.3000), left pressed:
    she reverses to 1Ah, moving 1.B000 left that frame. Engine `0B4A` 2 (her way left, along her
    new facing), game 1: by the decode notes her way is now right, and 768 → 769 she moves right
    (04AB.DFFF → 04AB.FFFF).
    **Answered.** Every turn and every spin reversed leaves her skidding: her speed then points
    against her new facing, whatever she was doing. Reversed while already skidding, her travel
    turns round: that frame she moves the old way (1.B000 left), and from the next her speed points
    right. `TurningRound` and §1.2's skid say so. Vector W14 (767 → 768, speed_x +1.B000 after).
15. **The wall test's cells are reached too (`TouchedCells`, `WallJump`).** item-morph-ball
    816 → 817: spinning carried on (19h, `0A27` = 3) with right held, the test looking left
    reaches the item's block (45h, 29h), 8 px past her left edge; the game picks the item up
    (record 817 is in the message box). The model's `Outcome::reached` lists only her moves'
    cells, so the engine never touches the item, and the row diverges from there (the free run's
    first room field is the item's PLM).
    **Answered.** The near-wall test reaches cells as a horizontal move of 8 px would, after the
    horizontal move's cells and before the vertical move's, with the way `Look`: a reaction that
    depends on the way she moves (a crumbling floor, a station's side) doesn't answer a look, the
    others (an item, a scrolling cell) do. A solid enemy that decides the test leaves no cells.
    `TouchedCells` and `WallJump` say so; the vector format adds the way "Look". Vector W15
    (814 → 817 with two lead-in frames: her left edge's column 46h, then the test's column 45h,
    then the fall's row).
16. **Taking off from a crouch with run held gains no dash (`Jumping`, `Dash`).** item-morph-ball
    1311 → 1312 (take-off 4Bh, run and right held): engine x 0523.4FFF, dash 0.1000, `0B4A` 2;
    game x 0523.3FFF, dash 0, `0B4A` 0. The model's take-off gains dash and moves by it, and the
    jump then carries on along her facing.
    **Answered.** Only running gains dash; a take-off gains none, and a dash she is to lose is gone
    before its move. `Jumping`'s "still gains dash" was wrong and is replaced; `Dash` says a
    take-off gains none. With no dash the jump that follows follows the pad. Vector W16
    (1311 → 1312).

Item 9 above now has a frame: item-morph-ball 1310 → 1311, the crouch's take-off (27h → 4Bh,
radius 10h → 13h): y 02C0.FFFF → engine 02B6.FFFF, game 02B3.FFFF. The game pushes her up the
3 px the taller body needs, then lifts her 10 px; the engine's pose fit runs after the model's
lift and finds no floor to push from. Round 7's push (`PoseFitAfterMoves`) answers it: the
frame is equal on lane/wire-r7.

## Answered: the domain harness on round 7 (integrator, 2026-11-07, lane/wire-r7; observer, lane/gaps-harness)

With round 7 the Tick steps her through the model with pose fit and the shrink inside the step
(`game::samus_world::step`), and the proof holds: the free-run gate's eight green pairs, and the
lockstep ratchet at the spine's numbers (item-morph-ball 1372 of 1427 Ticks fully equal, the same
Ticks and fields differing as on the spine). The old model is gone, and the domain harness
(`verify::harness::samus_motion`) steps the model over each record's own level and solid enemies,
with nothing copied from record N+1: 11,559 of 12,782 compared frames match over the 32 cached
rows, against 5,660 for the null model. The rows below the null model or the old model's floor
come from these frames. The pad of a frame N to N+1 is record N's.

Items 17-19 are answered in the spec (lane/gaps-harness). All three are this domain's: none is the
speed booster's, and no grapple runs in gap 19's frames. Vectors HV1-HV4 and AS1-AS3.

17. **Running on the speed booster's run with a downward vertical speed (`MovesAsShePosed`,
    `FollowingTheGround`).** samus-shinespark-up 0 → 1: running (09h), base speed 2.C000, dash
    2.7000, `0B3C` set, `0B36` = 2 with `0B2E`.`0B2C` 0.E000, on the floor. Model: base speed
    0.0000, vertical none (she lands, and the landing halts her); game: base speed 2.C000,
    vertical down 0.E000, both unchanged every frame of the run (frames 1-151; the speed booster's
    mask takes x and the dash, not these). The same on samus-shinespark-left, -right and -up-left
    from frame 1, and samus-mockball frames 1-42. The decode notes call a running pose with
    `0B36` = 2 and a magnitude not plain. Asked: what this State is in the spec's terms, and its
    rule.
    **Answered** (`FollowingTheGround`, the speed she keeps on the ground; `Landing`): on the
    ground her vertical speed is kept as it is. She doesn't move by it, the pull doesn't grow it,
    and the floor under her doesn't land her, so nothing halts her. Only her next change of shape
    shows it. The speed booster plays no part: samus-space-jump-chain and the three wall-jump rows
    run, brake, stand and turn with the same +0.E000, and their dash is 0 from frame 1. The decode
    notes now call such a start plain.
18. **After that run, a vertical speed of 1.3400 on the ground.** samus-shinespark-left 157 → 158
    (to 183) and samus-shinespark-right the same frames: model vertical speed 0.0000, game 1.3400,
    held. Asked: the same.
    **Answered** (the same entry): the crouch from the run (151-155) is a change of shape. It
    moves her by her kept +0.E000 under the pull, which takes it to +1.3400, and it ends before it
    meets the floor. Crouched, she keeps that speed through the turn on the ground to the
    take-off at 196 (HV3).
19. **A dash merged on the ground moves her a pixel further (`Dash`, `SpeedBuildsAndBrakes`).**
    samus-grapple-walljump 0 → 1: running (11h), x 002C.BFFF, base speed 2.C000, dash 1.2000,
    `0B3C` set. Both end with base speed 3.E000, no dash, `0B4A` = 2; x: model 0030.9FFF (by
    3.E000), game 0031.9FFF (by 4.E000), and 1 px behind to frame 8. Asked: the frame's move.
    **Answered** (`AimingAndShooting`, `PadChart`'s running let-go row): the merge doesn't cause it.
    With no row fitting (only select is held), running with a horizontal speed at the frame's start
    that isn't 0, the let-go row keeps her running pose. That kept pose is chosen as a running pose,
    so the aim change's step moves her 1 px on top of the braking move every frame she brakes so:
    frames 0-7 here, and samus-space-jump-chain 0-11, where the frame braking takes 0.2000 to 0
    moves her only the step. When the speed at the frame's start is 0 (space-jump-chain 12) she
    stands, and there is no step. A row naming her own pose brings no step (forward held,
    running). Grapple doesn't hand this frame over: the grapple beam is inactive, her movement
    handler is the normal one and the grapple wall-jump timer is 0 in records 0-9 (AS1-AS3).

Two things the edge now keeps that the model doesn't hold, neither the model's: the camera's
previous y pixel (`0B14`) becomes hers as pose fit, the crouch taken instead or the shrink left
her (before the crouch take-off's lift), so the camera doesn't follow those moves
(item-morph-ball 796 → 797 and 1227 → 1228, which moved the camera's subpixel otherwise); and a
moonfall's vertical speed word is her speed upward (`0B36` = 0: the decode's "− magnitude"), which
the round-6 edge had the wrong way (chain-0014-96BA 11 → 12 onward).

## Answered: frames the narrowed speed-booster mask shows (observer, 2026-11-07, lane/gaps-harness; 20 and 24 in lane/r9-spec)

The harness's speed-booster mask took y and the vertical speed on every frame she had any dash,
and x and the dash on every frame the dash was 2.0 or more. It now takes only what the booster
writes: x and the dash while she runs with run held and a dash of 2.0 or more, and the vertical
speed of a frame on which she starts a rise with a dash. The frames this uncovers are this domain's.
Most match (mockball's x, 19 of 19 compared before, is 87 of 87 now). These don't:

20. **The aim change's step every frame down a slope (`AimingAndShooting`, `SlopesCarry`).**
    chain-0220-D017 39 → 40 to 42 → 43: running down a slope with dash 0.C000 to 0.F000. Her aim
    changes every frame (the step every frame). y: model 1 px lower than the game (40: 006E.FFFF
    against 006D.FFFF). At 43 → 44 she leaves the slope: model 0078.FFFF, game 007D.CFFF, falling.
    **Answered** (observer, lane/r9-spec, with gap 27): 39-42 are the slope stop's carry
    (`SlopesCarry`). At 43 her pull's target is above the last air row of the surface at her
    centre column (row 1 against row 2) and the rectangle beside it is not her centre's block, so
    nothing stops it: she moves the whole 4.D000 and walks off. Vector SL3.
21. **A fall from the ground's edge that the model doesn't move (`FollowingTheGround`).**
    chain-0106-AD5E 13 → 14: running with run held (base speed 2.C000, dash 0.C000), into a fall.
    y: model 028B.FFFF (no move), game 0290.8FFF (the whole pull, 4.9000).
    **Answered** (observer, lane/r9-spec): the walk-off keeps the pull's move. Her horizontal move
    leaves her body over the pit (columns A0h), so the pull meets no floor and she moves all of
    it, 4.9000, then falls from rest; the aim's step doesn't come, as the fall replaces the pad's
    running pose. `FollowingTheGround`'s walk-off arm now says she moves the whole pull. Vector
    SL3 (chain-0220-D017 43, the same walk-off past a sloped floor).
22. **The frame a hit starts knockback, falling (`SolidEnemies`?).** chain-0106-AD5E 256 → 257 and
    293 → 294: falling in a straight jump (at 0.3800 and 0.7000), the frame knockback starts. y:
    model 010C.53FF and 010D.17FF, game 010B.FFFF both times, above where she started. The
    knockback mask takes her speeds, not y. Asked: whether the hit's y is this domain's.
    **Answered** (observer, lane/knockback-mask; gap 25 has the whole start): it isn't. The hit's
    pose change into the knockback's pose (53h, y radius 15h, from the jump's 51h at 13h, bank
    $91 pose table) runs pose fit for the taller body ($91:F404 → $91:FDAE) after the fall's
    move, which pushes her up clear of the floor below: 010B.FFFF both times, her bottom on the
    block's top. Masked with the rest of the start frame's fields.

23. **The run with a kept vertical speed, on slopes (`FollowingTheGround`, gap 17's answer, not
    yet in the model).** y, newly compared: samus-mockball 10-14 and 16-23; samus-shinespark-left,
    -right, -up and -up-left 10-14, 16-23, 79-81, 95-96, 99-100, 104-105, 110-120, 141-142,
    146-147 and 151-152 (39 frames each; with the two each row failed before, the shinespark
    start and one later frame, 41). She runs (09h) with +0.E000 kept, over slopes. Model: it moves
    her down by the kept speed as a fall (growing it to +0.FC00) instead of the ground's pull,
    and once that move meets the floor it lands her and halts her (base speed 0, vertical none).
    Down a slope she is left above the floor (mockball 11: model 048E.DFFF, game 0493.FFFF), up
    one she rises less than the floor lifts her (shinespark-left 96: model 04B5.FFFF, game
    04B0.FFFF). Game: the ground's pull, `|speed| + dash + 1` with the slope carrying her (§8),
    puts her on the floor, and the +0.E000 is kept unchanged. **Answered** by gap 17 (the spec
    already says it, HV1-HV3, AS3): for the implementer, no spec change.
24. **The spring ball's roll 1 px past the floor (`FollowingTheGround`, `SlopesCarry`?).**
    samus-mockball 91 → 92 and 92 → 93: the spring ball on the ground (7Bh), base speed 3.4000,
    dash 5.0000, no vertical speed. y: model 04BE.FFFF and 04C6.FFFF, game 04BD.FFFF and
    04C5.FFFF (1 px higher, from 04B9.FFFF and 04BD.FFFF); the next frame, on the level floor,
    both are at 04C9.FFFF. Like item 20, a ground pull 1 px lower than the game where the floor
    falls away. Asked: the pull's stop on that floor.
    **Answered** (observer, lane/r9-spec, with gap 27): the carry; vectors SL1 and SL2.

area-maridia 190 → 191 (x 1 px short) is gap 19's step: answered.

## Answered: the engine's pose at the edge (integrator, 2026-11-07, after `c44e48c`)

Every item is answered: 6 and 7 by the pose map and `SlopesCarry`, 8 by `OtherBlockKinds`'
table (a door that leads to a room is air, one that leads nowhere solid), and 9 by round 7's push
(`PoseFitAfterMoves`; the frame is item-morph-ball 1310 → 1311, in the Tick gaps above).

Items 6 and 7 are answered by the observer (lane/pose-map): §2's new parts, the map both ways
in the decode notes, and `SlopesCarry`'s empty column. Both are wired (lane/wire-r6): the
bridge reads the importer's map (`data::assets::samus::import::map`) with `clean_types::Pose`
as its key, and `surface_of` answers an empty column `JustPast` and one far past `FarPast`,
for a floor and a ceiling alike.

Items 1-3 below are answered by `c44e48c` (the pose chart, `Outcome::reached`, `World`'s kinds).
`game::samus_world` (`78dc21a`) answers `World`. What stops the next step, the State swap:

6. **The engine's pose from the model's (§2).** Animation, drawing, the body's size and pose fit
   read the engine's pose, one of 253 sprites; the model chooses a §2 pose, which §2 says has
   fewer parts than the sprites (the gun, the landing sprites, the spin drawn as the space jump or
   the screw attack). Asked of the observer: the decode both ways at the edge, as data:
   - each sprite's §2 pose (body, action, facing, aim, gun), above all the crouch and morph
     transitions' four actions and the take-off's poses;
   - for a chosen §2 pose, which sprite the game shows: by the sprite she was in, her equipment,
     and whether she landed, as the records say.
   Without it the edge map guesses, and every guess is a free-run gate pair at risk.
7. **A ceiling's surface where a column has no solid pixel.** The ROM's surface offsets reach 16
   (a column with nothing solid in the block). A floor's `surface` row 16 is past the block and
   never stops her, as in the game. A ceiling's lowest solid row is then -1, which `Slope::surface`
   (`u8`) can't hold; the adapter answers 0, so a rise whose top edge is on the block's row 1
   stops where the game lets it on. Asked: the spec's surface for such a column.
8. **A door to a room is air (`OtherBlockKinds`), but the level alone can't tell it from a door to
   nowhere.** The adapter answers every door solid (`DoorToNowhere`), as the engine's collision
   does today; the door's destination is in the room's door list, not the block.
9. **Pose fit's push is an input, but it follows the pose the step chooses (`PoseFitAfterMoves`,
   round 6's `Frame::push`).** Round 6 answers item 1 below inside the step (`WhatHerMovesMet`
   and the chart), so the engine no longer knows the frame's pose before the step, yet the push
   that pose's body needs is given before it. The engine can only pass 0 and run pose fit after
   the step, which puts the push after the crouch take-off's 10 px lift. Asked of the model: the
   push as an answer the step asks for once it has chosen the pose (for example a `World` method
   given the new body's radius), so the order §`PoseFitAfterMoves` states holds in the engine.
   No recorded frame tells the two orders apart yet (spec gap 6: a block within 10 px above her).

## Answered: what wiring the model into the engine needs (integrator, 2026-11-07)

Every item is answered in the spec: 1 by the pose chart choosing the pose after the moves
(`WhatHerMovesMet`), 2 by `TouchedCells`, 3 by `OtherBlockKinds`, 4 by `SlopesSlowTheWalk`'s walk
factor and 5 by `PoseFitAfterMoves`.

These come from wiring, not from a differing frame: the engine can't drive the model's public API
as it stands without a stand-in. None has a frame yet; each names the entry it touches.

1. **The pose a frame changes to depends on that frame's moves (§7, `Landing`,
   `WallsAndCeilings`, `FollowingTheGround`).** `Frame::next_pose` is an input given before the
   step, but in the game the pose a frame takes is chosen after its moves: a landing, a fall from
   the ground's edge, a ceiling, a wall stop and a wall jump's kick each choose it. The harness
   can read it from the next record (a stand-in that answers from the oracle, MODEL §4); the
   engine can't. Asked of the model: one step that, after the moves and before the pose change,
   asks the caller for the pose with what the moves met (landed, left the ground, hit a ceiling,
   stopped at a wall, kicked off a wall), for example through a trait the caller implements.
   The alternative, two passes (the moves on a copy to learn what they met, then the real step),
   duplicates the frame's work and infers the contacts from the State's change.
2. **Blocks that react when she reaches them (§8, §10).** In the recorded rows her moves reach
   scroll triggers (area-brinstar, the elevator ride's step) and an item's block (item-morph-ball,
   the pickup), which let her through and start something. `World::block` answers only air,
   solid, stepped or sloped, and nothing tells the caller which cells a move reached. The adapter
   can record every cell asked, but the model may ask cells the game's moves don't, or in another
   order. Asked: the set and order of cells a move reaches, as the spec defines it, so the world
   can report which reactive cells she reached.
3. **The other block kinds (§10).** Doors, spikes, crumble and speed blocks, shootable, grapple
   and bombable blocks, and the blocks that stand for another block (a run of one block's cells).
   The adapter answers solid for the solid-to-touch kinds and resolves a standing-for cell to the
   cell it names, as the engine's collision does today; the spec should say so or list them.
4. **A walk along a sloped floor changes the move's length (§10).** Today's engine scales a
   horizontal move on a non-square sloped floor, with no vertical speed, by the shape's factor;
   the model doesn't. Every recorded frame that walks a slope will differ in x until that entry
   exists.
5. **Pose fit moves her between the pose choice and the pose's speed changes.** When her body
   grows, the game pushes her clear of the blocks (out of scope, the header) after choosing the
   pose and before the crouch lift and the pose change's writes. The model does the crouch lift
   and the writes inside the step, so the push lands after them in the engine. Whether any
   recorded frame tells the two orders apart is not yet measured.
