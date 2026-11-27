# The Zebes flashback encounter: decode and replay

Companion to [screens-flashback](screens-flashback.md). Keep this file and the reference
scripts outside confined exports. The intro owns both dissolve stages and the page turn.

## Records

The witness is the lean cache row `any-intro-2219`. Its cut 0 is power-on cut 2,219.
The start comes from the latched new press at `any-intro-0568` cut 1,650. The normalized comparison
starts at cut 0 with the pending encounter start. Cut 1 finishes its first work frame. Row cut 650 is one finished frame after
the return dissolve is complete.

| Row cut | Observation |
| ---: | --- |
| 0 | Setup has installed the boss and schedule. Decode this as a pending start and the inherited Y. Neither actor has taken its first turn. |
| 1 | Mother Brain's first visible picture and both initial holds. Gameplay has begun and the start has been consumed. |
| 75 | The first two rings appear before the outbound dissolve finishes. |
| 101, 106 | Cuts inside frames, after the encounter has stepped. At 106 the rings switch to their moving clip while their centres are still at their spawn points. |
| 102, 107 | The same work frames finish. The encounter has already advanced and its timers and positions stay as at the previous cut. |
| 108 | The first rings advance by their half-pixel steps. |
| 109, 110 | The first missile's whole X is 84 at 109. The shot survives. At 110 the boss claims it and the missile slot turns into an impact at whole X 78. |
| 111–118 | The sprite palette alternates white and normal over eight writes. The dissolve also composes colours on 111 and 115, before the flash writes. |
| 131, 132 | The last outbound fade step overruns. Cut 131 is before the encounter. Cut 132 advances it once and completes the transition. |
| 152, 350 | The second and third missile impacts. |
| 167, 168 | At 167 the first ring is half a pixel short of the whole-X threshold. At 168 the ring is gone, both Samus hit timers are 11, direction is right and energy is still 99. |
| 206 | The second pair appears. The remaining first ring keeps its foreground rank. |
| 391 | The fourth missile impacts. The boss enters the shake phase with elapsed count zero. Three moving rings are still present. Eight bursts exist and all are still blank. |
| 392 | The first shake step moves the background from 8 to 4 px. The moving rings are gone. The first small and large bursts show. |
| 396 | The demo locks Samus facing left. Her intro new-state update continues. |
| 399, 410, 426 | The second small burst starts, the first small burst goes blank, then its first picture starts again. |
| 518, 519 | The hold advances from 127 to 128. The cue arrives, the boss restarts its breathing clip and the intro enters its page 2 pending state. |
| 520, 521 | Page 2 setup spans these cuts. At 520 the encounter has yet to step. At 521 it shakes once and page 2's blank step shows with the return count at 127. |
| 647, 648 | The return count goes from 1 to 0. Samus and weapons get their last step. The boss and bursts disappear, and the draw omits Samus and her shots. The last background shake step is visible. |
| 649–650 | The intro completes the last colour step. The encounter stays gone and the background offset holds. |

## Sources and State mapping

Record names and observations in the vectors are normalized values. The art tiles, palettes
and object lists are imported from the player's ROM into named clips and palette roles.

| Entry | Sources and mapping |
| --- | --- |
| EncounterBeats | The intro's `$8B:A35B` stage and `$A370` advance the cinematic frame count `1A51`. `$8E0D` steps Samus and weapons, `$93EF/$9409` advances the sprites, then `$A379..A389` handles text, palette FX, drawing and scroll. Borrow completion from `05B6`. For the intermediate cuts here, the cinematic count and the boss and ring holds together identify which side of the encounter the cut is on. A changed cinematic count alone does not establish that the encounter finished. |
| EncounterStarts | `$8B:AF10..AF1D` installs Mother Brain and the schedule during setup. `$B773` gives the boss's place and palette. `$CE0D` and `$CB05` supply their holds. `1997` is inherited from the intro, 8 px on this row. |
| BrainBreathes | `$8B:CB05/$CB1F` and `$9409/$94BC`, with `$8C:8C00/8C2F/8C5E` art. The body is live if `1B1D` is nonzero. The next-picture pointer in `1B1D` yields the current step within either repeating list. The current picture in `1A5D` confirms it and `1B5D` is its hold. |
| BrainEndures | `$8B:B786..B80E` inspects the five ordinary projectile positions in descending slot precedence. `0C18+2i` masked to its low 12 bits must be `0100h` and `0B64+2i` must be below 84 using the signed test in the room. The search ends on its first ordinary missile. `$90:AE06/$93:80CF` keeps its position, changes type to missile explosion, primes the impact clip and clamps cooldown to 20. It suppresses missile impact sound during a cinematic. The projectile retains its current picture on the hit cut and reads the first impact picture on its next step. Boss hits are `1B7D` while `1B3D` is `B786h`. |
| BrainFlashes | `$8B:B786/$B80F` call `$B846` before the other boss changes. `1ADD` is the flash count in the boss. It writes the 16 words at `7E:C1E0..C1FF` from white or `$8C:E5C9..E5E8`. This is after `$8B:B250` composes the dissolve and before palette FX at `$A382`. CGRAM words 240–255 receive the result. The first hit beat starts the timer. |
| RingsAppear | `$8B:CE0D/$BA21/$BA36` and `$B896` place pairs. `CE11h` and `CE17h` in the schedule's next-picture pointer mark the two waits, held in the same timer column as an image. `$CDEB..CE0C` and `$B8C5` show the ring clips. Art at `$8C:8C8D/8CA3/8CB9`. |
| RingsSweep | `$8B:B8D8/$B93B` move and remove the live rings. The fractional positions are in columns `1ADD/1AFD` and the integer X/Y in `1A7D/1A9D`. Path comes from a ring's `1B7D`. The threshold uses Samus's whole X at `0AF6`, sets `18A8/18AA` and `0A54`, and consumes the ring. The other rings use Y bounds. `$B8C5` sets the movement phase after the last stationary picture. Movement starts next frame. |
| BurstsPulse | `$8B:B7BD..B80D/$B98D/$B9D4` place the bursts, `$CDAB/$CDCB` provide the cycles and `$BA0F` retires them when the return count is zero. Decode the clip step and hold as for the boss, with a pending first image until the pointer leaves the start. Art at `$8C:97F7..985C` and `$8C:985D..98ED`. |
| BackgroundBobs | `$8B:B80F/$B82E` use `$B877` to move `1997` using the parity of `1A51` after `$A370` increments it. `$8532` publishes Y into the BG1 scroll mirror `00B3`. The camera offsets `0911/0915` used by sprites stay separate. |
| ReturnToPage | `$8B:B80F/$CB19` cue `$B336` and switch to `$B82E`. The English page 2 pending stage is `B35Fh`. The boss's animation resets. In the shaking phase `1B7D` is the elapsed count. At the return count `1A4B = 0`, `$B82E/$BA0F` and `$9438` remove the boss and bursts and clear the draw gate at `1A57`. The schedule and rings still run via `$93EF`. Their removal tests do not use that gate. `$8E0D/$8E2D` read the gate for Samus's step and draw on either side of the encounter. |
| FightPictures | `$8B:938A..9416` takes the foremost free positions and steps the foreground first. It fixes the boss behind the other actors. In this row the caret at slot 15 is excluded, the boss is at slot 0, and rank `14 - slot` puts the foremost available encounter position at rank 0. `$9746/$8E2D` writes encounter sprites in that order after Samus and her shots, subtracting the camera origin. `$81:879F/$8853` encodes the sprites. |

## Scoped replay

Admit the start from the page 1 hand-on and retain it across cut 0. At that cut,
the initialized boss and ring schedule have no picture and both still wait for
their first step. Decode a pending start and inherited Y in place of the native
pending pictures. Use the existing platform completion producer for finished cuts.
Dispatch the encounter once per work frame.
An intermediate comparison also needs the phase at the cut. In the two overruns where the
encounter has run, carry its State and events and finish that work frame without dispatching
it again. The other two overrun cuts can compare the held encounter State.

Compare the normalized State in the source table and each output at its boundary.

- Compare hit requests against a new missile impact in the same weapon slot. Borrow
  Samus's after-motion whole X. The reference builds the incoming missiles without reading
  the expected weapon impact. For an already flying shot it advances the previous
  centre and speed by the next horizontal step, using the acceleration sources
  `$90:AF68/B2F6/C303/C353` and movement at `$94:A46F`. All four missiles fly left in
  open space up to contact. A shot just fired takes the first slot with zero damage.
  Its birth is the upstream ammo debit at `$90:BEBF`, independent of whether the
  encounter consumes it. The standing-left muzzle starts 11 px left of Samus
  (`$90:BA56/C204`) and the missile moves 1 px left on that first beat. Later beats
  subtract 80/256 px per beat from its speed before motion. The reference checks
  these derived centres independently against the retained flight or impact centres.
  A production run needs the real weapons output before collision and full impact
  State afterward.
- Compare the ring result against both fresh 11 timers and rightward knockback. Energy
  and ammo stay at their previous values. Movement and appearance after the hit are
  Samus's comparisons.
- Compare the flash request against the palette mirror and those same 16 CGRAM colours
  on publication cuts. On other cuts the intro dissolve and independent palette FX write
  scene colours. The reference imports the normal colours separately.
- Compare the cue against the English pending stage and the hide against the draw gate.
  Compare each as a one-beat event as well as carrying its effect. E4 consumes the
  encounter's signed Y for the room background.
- Compare the shipped encounter pictures against the OAM pieces for boss, rings and
  bursts. The reference imports the sprite parts separately and compares X/Y, tile,
  attributes, size and order at finished cuts. The row's encounter tile and attribute
  pairs identify its pieces. Ignore hidden entries at Y 240. Mask Samus and weapon pieces
  and the text caret. Their writers are Samus, weapons and the intro.
  E2 through E4 own the other display fields. OAM upload is outside the intermediate check.

List the platform completion and phase signals, start event, return dissolve signal,
camera and the room and weapons inputs as borrows. The cue comes from the encounter's
carried hold. The existing `Intro(BattleBegins)` indexed production stop remains until
the engine carries the join.

## Dependencies for integrated play

- **Intro and platform.** Carry the page 1 press into startup and preserve work phase
  through the long setup. Carry the cinematic beat from the intro's initial reset at `$8B:A3CB..A3DA`.
  Apply start
  before gameplay. Receive the cue after the current stage step and prepare page 2
  on the next. Supply the post-stage return count reaching zero. Use old visibility
  for gameplay and new visibility for drawing on the hide beat. The companion
  intro boundary publishes one hide event. Wire the encounter's retirement into
  that publication and keep its work-frame order. Keep scheduling any surviving
  rings when adjusted rules drive a different return. An adjusted hold
  checked inside the encounter does not exercise a different cue time through the intro.
- **Room.** Use 16 by 16 cells, 16 px each, as set at `$8B:A39F..A3A7`. Startup
  at `$AF1E..AF40` copies 224 block words from `$8C:BEC3` into `7F0002` and
  clears 512 bytes of the BTS buffer at `7F6402`. The two bottom collision rows
  are inherited. The witness has nonzero inherited cells even though Samus's
  whole Y lies between 67 and 122. Preserve those rows if the replay compares
  the whole grid. Samus and projectile block queries need the same flashback
  room throughout the dissolve and fight. The intro's art is separate content.
- **Demo.** Initialize `$91:8784` and start at `$91:8694`. The authored list continues
  at `$86B8` and uses the end-facing-left and delete actions at `$8739/$8427`. Import
  durations, held and newly pressed controls, and event actions as demo content. Keep
  the cursor and hold across dissolves and long frames. Route its controls to Samus and
  weapons while retaining the movie controller for the intro. `$90:E833` restores movie
  controls after room play. The end-facing-left action locks the current-state update
  at row cut 396. The intro new-state update keeps running through the hide beat.
- **Samus and equipment.** Carry equipment and ammo. Place Samus facing left at (155, 115)
  with current and previous positions coherent. Select missiles and start her standing
  pose (`$8B:AED2..AF06/AF41..AF65`). Connect demo movement, the pose and animation
  pipeline, room collision, knockback, jumps and landing. Apply the ring result after
  the demo's countdown (`$8B:8E0D`) so the next movement step starts with the full 11.
  Route her palette and picture to the drawing services.
- **Weapons and audio.** The [weapons scope](weapons-projectiles.md) stops at the
  uncharged beam. Extend the normal weapon path for ammo, selection, cooldown, leftward
  muzzle and missile acceleration, block interaction, the retained impact and weapon
  precedence. Ammo debits first show at row cuts 91, 135, 335 and 376. Weapon effects
  and audio still own their requests and replies. Missile impact sound is disabled
  in this cinematic.
- **Video and content.** Import the named encounter clips, places and normal boss
  palette. E1 places Samus and her shots before encounter sprites in the common list.
  The flash writes after the dissolve and before palette FX. E4 consumes the background
  offset. R4 owns the room lights that begin when the outbound dissolve ends.

## Reference evidence

Local observer scripts in `$TEMP/collab/fight-obs-notes/` include `reference.py` for the
vectors, `row.py` for the lean cache comparison and `draw_check.py` for imported
sprite output. Use Python 3.11 or newer.

- All 51 vectors agree. They include 33 recorded cases and 18 constructed, four of
  them adjusted.
- Encounter State matches 646 finished cuts, all four intervening cuts and the
  pending start at cut 0. The held initial State matches none.
- Missile impact, ring hit, palette selection, cue and hide each match at all 646
  encounter steps. They witness four impacts, one ring hit, 32 palette requests, one
  cue and one hide. Encounter OAM matches at all 646 finished cuts. CGRAM matches all
  32 flash writes. The BG1 Y mirror matches at all 646 finished cuts.
- Changing the contact line from 84 to 85 first disagrees at cut 109 in the shot
  impact and boss hit state. That missile still flies at exactly X 84. A one-beat
  shorter hold first disagrees in the cue and boss State at cut 518. A second
  encounter step when an overrun finishes first disagrees at cut 102 in the boss,
  ring schedule and ring State. Dropping the start fails the pending start and Y at
  cut 0 and the boss and schedule at cut 1.
