# Samus at the start of the intro

Companion to [Samus at the start of the intro](samus-intro-entry.md).

## Sources and mapping

| Entry | Original sources | Record projection |
| --- | --- | --- |
| SamusEntersTheIntro, new file | File select's new-file path at $81:B2CB. See [file select](screens-file-select.decode.md) and [run progress](run-progress.md). | Energy/capacity $09C2/$09C4, missiles/capacity $09C6/$09C8, other supply words $09CA..09D0, $09D4/$09D6, items and beams $09A2..09A8. |
| SamusEntersTheIntro, setup | $8B:A395 calls $91:E00D before its first long load. $91:E01F clears actor fields, $E111..E11C installs the normal vertical acceleration and $E164..E167 remembers the file's current health. $8B:A3B0..A3B6 lends the missiles. $8B:BC08 keeps the player's bindings and selects the default map. | Selection $09D2 and auto-cancel $0A04. Live setup marker $0A44 and previous energy $0A12. Current/prior/older pose and their facing and movement mirrors $0A1C..0A26. Centre, previous centre and radius $0AF6..0B00 and $0B10..0B16. Vertical acceleration $0B32/$0B34, velocity $0B2C/$0B2E. Protection and knockback $18A8/$18AA. Clip hold/step $0A94/$0A96. Shot cooldown/count $0CCC/$0CCE. |
| SamusEntersTheIntro, controls | $8B:BC08..BC74 sets the directions and keeps the seven customisable buttons before replacing them. | $09AA..09BE and $0D82..0D8E. Up/Down/Left/Right are the respective directions, Shoot X, Jump A, Run B, Cancel Y, Select Select, Aim down L, Aim up R. Menu settings such as moonwalk ($09E4) keep their own values. |
| SamusEntersTheIntro, hidden | $8B:A373 calls $8B:8E0D. Its guard at $8E10 skips the actor turn while $1A57 is zero. $8B:8E2D skips her draw under the same condition. The animation and item controls therefore stay frozen as the text runs. | Hidden/shown $1A57. Cannon readiness is $0AAA, decoded as two minus the source word. See [CannonReadinessReset](samus-appearance.decode.md#sources-by-entry-and-ram-mapping). |
| Fight entry and first turn | $8B:AEC2 reads the latched press. $AECF..AF04 applies the pose and whole centres. $AF41..AF65 selects missiles, loads the demo and shows her. $8B:A373 then takes the room turn. The same turn runs item controls through $90:E6C9 and $DCDD, changing $0AAA through $C520..C52C. | Same actor fields, and the held/new demo words $0A84/$0A86. The first turn establishes radii (5, 21) px, advances the centre's y fraction to FFFFh, leaves previous centre and its fraction at setup, advances the older pose to standing left, lowers clip hold from 10 to 9 and lowers the cannon delay from 2 to 1. |

[Bank 81](https://patrickjohnston.org/bank/81) · [Bank 8B](https://patrickjohnston.org/bank/8B) · [Bank 90](https://patrickjohnston.org/bank/90) · [Bank 91](https://patrickjohnston.org/bank/91)

## Production join

File select makes the new file at movie 433. The run-progress defaults establish energy and
capacity 99, zero other supplies and empty items/beams. The options hand-on at movie 568
carries that file into the intro. The current front-end public file answer carries settings
and start kind but no inventory. The empty-slot path can create run progress from the existing
fresh-file rules when it opens the file. Carry the result through options. Use the demo's
ordinary bindings and retain the player's bindings for the end of the loan. The movie already
uses the default bindings and moonwalk is on from movie 519.

Consume the actor setup and the existing IntroBegins equipment event in the same opening work
frame. The report currently reaches only the completed-frame adapter and its equipment event
is discarded there. The setup needs a point of entry before the first interrupt, at movie 569.
A saved opening phase must distinguish setup already done from the rest of that same work
frame. It prevents reapplying the loan when the frame completes at movie 622. Keep the existing
scoped screen contract for what it compares on finished frames.

The fresh Ceres actor in `production::played::start` already expresses the ordinary rest
constructor and health from Progress. Reuse those initial values with centre zero, without
creating a Ceres room, HUD or camera. Use the ordinary appearance initial State, including its
cannon delay, and the existing empty weapons State. The old pose fields and cached radii are
separate platform observations. Before the actor takes a turn they are the initialized values
above. Keep one writable source for missile supply. The room's actor carry is a projection
from inventory at the join, or the live source while the intro holds it. Subsequent debits and
return of the loan must write the same current supply.

Use the same live owners at `intro_rooms::State::new`. There is no actor update in the opening
after setup, including cut 2,218, when the movie latches the page-one press. The room start
installs its populated geometry and updates her whole centres and pose by cut 2,219. See
`world-intro-rooms/ShowIntroRoom`. Its first pending turn consumes demo input and plays once on
cut 2,220. Use appearance's existing `CannonItemDelay` for that turn's eligible unchanged
selection. The room start changes the selected item outside item controls and does not age
the delay. The room's first-turn witness currently omits cannon readiness.

The ordinary motion rule sets acceleration and the appearance rule takes its hold from the
imported clip. The recorded mirror values of 0.1C00h px per turn squared and ten frames are
checks, not new rule parameters. The pending room setup preserves current and previous centre
fractions independently. The fight reaches it with zero fractions because the opening
created them that way. The later discovery reaches its own start with nonzero fractions and
a partly played clip from the intervening room play.

## Replay boundary and witnesses

Use the power-on row's new-file cut 433 to check supplies and settings. Use
`any-intro-0568:0` (movie 568) to check the hand-on. Cut 1 (movie 569) checks the
loan and live actor inside the first work frame, cut 54 (622) checks its completion,
and cut 1,650 (2,218) checks the hidden carry and latched press. Check every
intervening cut for no movement, clip advancement, cannon readiness advancement,
shot or ammo change. Include a constructed nondefault control map because the
movie's physical bindings before and after the loan are identical.

The same integrated run should then compare `any-intro-2219:0` and cut 1, which
are also the continuation of `any-intro-0568` at cuts 1,651 and 1,652. The later row
is the expected output only. A room setup delayed until work finishes first differs
at movie 2,219 in whole centre, selection and clip hold. A skipped first turn first
differs at 2,220 in the centre fraction, prior poses, radius, clip and cannon delay.
Use checkpoint restoration at 569 and 2,219 to check setup is applied once.
These checks leave the unproduced inherited room memory tail, final picture and
encounter to their existing owners. If only the first room turn is integrated, the
next completed fight cut at 2,221 needs the ongoing demo and encounter turn.

## Reference results

The local reference takes the new-file result as its starting point and plays the actor
setup and hidden carry across the opening. Its inputs are the menu's moonwalk change
and the work frame's start at cut 1. It decodes the observed actor independently at each
cut. The reference uses the imported room and existing room-turn rules with this same
actor. The two later cuts supply the expected output. The comparison adds capacity
and cannon readiness to the existing 15-field room comparison.

| Check | Agreement | Held State |
| --- | --- | --- |
| Power-on new-file supplies and settings, movie 433 to 568 | 136 of 136 | |
| Opening setup and hidden carry, movie 569 to 2,218 | 1,650 of 1,650, including 146 unfinished cuts | 0 of 1,650 |
| Fight setup and first turn, movie 2,219 and 2,220 | 2 of 2, 17 fields each | 0 of 2 |
| Recorded and constructed vectors | 7 of 7 | |

Delaying setup until the opening frame finishes, omitting capacity, and initializing
her standing all fail at cut 569. Advancing the clip or cannon while hidden fails at
the first finished opening cut, 622. Omitting the first eligible item control turn
fails at 2,220 in cannon readiness. These faults exercise the reference comparisons.
The indexed production witness and faults still need wiring.
