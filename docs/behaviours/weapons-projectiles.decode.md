# Weapons and projectiles: observer decode notes

Implementer-facing rules: [weapons-projectiles.md](weapons-projectiles.md).
Vectors: [weapons-projectiles.toml](../../scenarios/vectors/weapons-projectiles.toml).

## Sources by entry

Sources are the [bank 90](https://patrickjohnston.org/bank/90),
[bank 91](https://patrickjohnston.org/bank/91),
[bank 93](https://patrickjohnston.org/bank/93),
[bank 94](https://patrickjohnston.org/bank/94) and
[bank A0](https://patrickjohnston.org/bank/A0) logs. Content values are imported privately from
the player's ROM; no content bytes are in the vectors.

| Entry | Sources and observations |
| --- | --- |
| ShotSpacing | $90:AC1C, $90:AC39, $90:B8EC-B92A, $90:C254/$90:C283; $90:DCDD-DD04 for facing-forward, locked and frozen-time modes |
| HoldingShoot | $90:AC39, $90:B80D-B886, $90:B887-B96D, $90:BA56-BAFB; $90:DD05-DDD8 for caller eligibility |
| CannonOrigin | $90:BA56-BAFB, $90:C204-C253; imported pose definition's shot aim and vertical lift at $91:B62C/$91:B62D |
| InheritedMotion | $90:B197-B2F5, $90:C2D1-C300; the overlapping-word loads require the unrelated low byte, not the commented displacement alone |
| MuzzleContact | $90:B933-B968, $90:BD64-BDB1; $93:8000-8070 sets initial radius before contact |
| FlyingBeam | $90:AEF3-AF67, $90:C353/$90:C367; $94:A23B-A351 performs signed-speed fixed-position addition |
| ShotContact | $94:A1F7-A23A, $94:A23B-A351, $94:A1B5/$94:A1D6; $90:AE06-AECD shifts impact centres; $93:80CF-814D sets impact clip, sound, hold and damage |
| BeamTrail | $90:AF00-AF0F, $90:B657-B6A8; $93:81D1 reads the preceding displayed frame's trail appearance |
| LeavingView | $90:B16A-B196 tests view bounds and calls $90:ADB7-AE05 to clear the body and update count; $90:AF2F-AF35 calls the test after flying motion/contact, including its freshly changed impact, not stationary impact updates |
| EndingShot | $90:AEF3-AEFF and $90:ADB7-AE05; $A0:A143-A225 marks power shots consumed on enemy contact; $93:822F-8238 clears at clip end |
| BeamPictures | $93:8000-8070, $93:80CF-814D, $93:81E9-823F; clip import converts frame holds, radius, spritemap and trail appearance, repeats and termination |

The observed first active shot is slot zero in `chain-any-0003-DFD7`: live at frames 157-160,
consumed at the cut at 160, absent at 161. All earlier shots are absent. Its position and speeds
change each active frame; there is no block impact in this row. These are five active transitions
(spawn, three later flights including actor contact, removal), among 344 adjacent transitions.

## RAM and content mapping

All slot arrays below use two bytes per slot, first five slots for this part of W1.

| Clean quantity / input | WRAM or source mapping |
| --- | --- |
| position integer x/y | $0B64 / $0B78 |
| position fractional x/y | $0B8C / $0BA0 |
| radius x/y | $0BB4 / $0BC8; imported frame stores the original unsigned bytes as named radii |
| speed x/y | $0BDC / $0BF0, signed 1/256 px |
| heading / consumed | $0C04 low nibble / high low-byte nibble; codes 0..9 decode through the original direction table to the spec's Heading |
| phase | $0C18: uncharged power beam when beam bits are zero and the no-Samus-interaction bit is set; beam impact category $0700 |
| damage | $0C2C; importer rejects a negative signed content damage as the original crashes at $93:8044 |
| clip / next frame | $0C40; map the imported clip's original frame addresses and end to named ClipId and ClipFrame; no pointer enters the model |
| remaining hold | $0C54 |
| trail countdown | $0C90; retained by individual clear, reset by room/elevator clear |
| visible picture | $0CB8; importer maps spritemap pointers to PictureId |
| displayed trail appearance | flying clip frame's trailing word; source $93:81D1 reads $0C40 minus two. Import as TrailAppearanceId, not a control word |
| cooldown / count | $0CCC / $0CCE |
| shoot / newly pressed shoot | controller $008B / $008F masked by configurable shoot binding $09B2; previous newly pressed is $0E00 |
| her integer centre | $0AF6 / $0AFA, before this frame's Samus movement |
| her pose / previous pose | $0A1C / $0A24; importer assembles Muzzle from $91:B629's eight-byte pose records and $90:C204-C253 offsets |
| pending aim | $0B5E: zero means absent, otherwise its low byte is the transition's heading and is consumed on an available attempt |
| left / right / up / down movement | whole/fraction pairs $0DAA/$0DAC, $0DAE/$0DB0, $0DB2/$0DB4, $0DB6/$0DB8 |
| camera vertical fractional speed | $0DA8 |
| camera integer origin | $0911 / $0915, before the camera follows this frame |
| room width blocks / screens / height screens | $07A5 / $07A9 / $07AB |
| time stopped | $0A78 |
| caller mode | facing-forward poses 0/$9B and locked current-state handler $E713 run shots only; $90:DCDD otherwise cools and selects firing, skipping shots when frozen |

The inherited-motion source reads $0DAD for right, $0DA9 for left, $0DB5 for down and
$0DB1 for up. Thus their low bytes are respectively the high bytes of left fraction,
camera vertical fraction, up fraction and right fraction. Up tests the high byte for zero,
then shifts the whole unsigned combined word twice and ORs $C000. It is not a signed shift.

Normal clear ($90:ADB7) zeroes fractions along with body fields. Room/elevator clear ($90:AD22,
called by $82:E4A9 and $A3:9548) leaves the fractions while zeroing trail, count, cooldown and
the body. A Free slot therefore can hold retained fractions: the next left muzzle probe reads
them before the successful-flight initialisation zeroes them at $90:B1F3. R1 supplies this
reset State through the model's public State shape. The unused timer after clear can become
$FFFF when the original instruction handler continues; it has no later reader before a spawn
overwrites it and is deliberately absent from clean State.

The caller's shoot input includes pose and weapon eligibility. The standard handler covers
standing, running, normal jumping, crouching, falling, moonwalking and ran-into-wall. Turning
and crouch/stand transitions have their eligibility guards in $90:DD74/DD8C; spin jumping,
wall jumping and knockback cancel grapple without firing a power beam ($90:DDB6). The M0
reference specifically suppresses the held-shoot spin-jump attempt at frame 81; the cooldown
must remain zero rather than reserve one for the pose's forbidden muzzle. Muzzle Forbidden
describes an attempted shot's origin decision; it is not a substitute for caller eligibility.

## Trail appearance at the content edge

The flying frame's trailing word is a selector, not packed offsets and not a trail animation
list id. $9B:A3CC selects the uncharged beam's heading-specific table through $9B:A4B3, then
the selector times four selects four signed bytes: left x/y and right x/y. The runtime
subtracts four from each centre-relative coordinate. AF3 imports this as a named
TrailOffsets record, two signed i16 Offset pairs with that subtraction already applied,
and pools its TrailAppearanceId. Every referenced flying selector must resolve a complete
record. The uncharged power beam's left/right trail animation lists ($90:B5BB/B609) are
both empty; effect animation is outside this first W1 part.

Impact updates never emit BeamTrail. Their frame trailing word therefore has no reader and
is imported as None/Inactive without resolving or validating it as a selector. Inactive is
an absent channel update, so BeamPictures preserves the prior meaningful appearance. Slot
reuse cannot observe the ignored impact word: a spawn sets countdown four and clip hold one,
and loads its own flying appearance in that same update, before any trail can emit. A zero
content hold is loaded only after that first appearance, and an adjusted trail interval
changes the reset-on-zero interval, not the initial spawn countdown. The new impact vector
distinguishes preservation from overwriting the prior value with None or an invented id.

## World capability: ContactContinuation

W1 supplies geometry and visits every response. W2 maps that geometry to content and applies
the reactions. Horizontal contacts visit the perpendicular span top to bottom; vertical ones
left to right. A passing cell does not terminate visits: shot-reactive cells farther along the
span still act, and the aggregate blocks only when all visits block. Diagonal vertical motion
and its visits are suppressed when the horizontal aggregate blocks.

The original mapping first multiplies the low byte of the starting row in blocks by the low
byte of room width in blocks, adds the full column, then doubles into a wrapping 16-bit byte
offset. Horizontal starting row is the perpendicular start divided by 16; vertical starting
row is the leading boundary divided by 16. Horizontal subsequent offsets add the full word
room width twice, carrying from the first addition into the second; vertical offsets add two.
This is why a clean modulo-cell-index description is insufficient for artificial wrap cases:
the first horizontal addition can carry, leaving an odd byte offset for the next reaction.
The original reads overlapping level words there. These aliases and continuation responses
belong to W2's ContactContinuation capability, not a byte-address API in W1.

The M0 reference privately enumerates those offsets from the before record's level at $10002,
and asserts that every visited cell is the unconditional Air shot reaction. It observes 17
visits and one block-kind class (Air), with no impact. It refuses any reaction requiring a
different W2 rule, and refuses an odd offset; neither happens on this row. The integrator must
audit existing World for the full continuation, repeated-cell and reaction-order contract.
If it is incomplete, name the missing W2 capability in gaps/LEGACY; do not claim the contract
implemented by delegating it. The constructed W1 vectors use invented ordered World responses,
never a ROM room or an address-shaped content layout.

## Harness and reference evidence

Private scripts live in `$TEMP/collab/night-20261113/w1-observer/`: `reference.py`,
`make_vectors.py`, `record_proof.py`, `mutant.py`, `composed.py` and `composed_vectors.py`; `observe.py` reads lean cache records.
Run the scripts from that directory with Python 3.13.7 so they can import each other. These private files
are outside the implementer export and contain no committed ROM content.

`reference.py` agrees with all 81 constructed vectors. Expected vector values were worked by
hand from the rules; the generator writes those answers rather than calling the model to make
them. The adjusted acceleration is 48 versus reference 16; adjusted trail interval is 7 versus
4. The 70 observation vectors isolate quantities (including spawn body before contact/flight
and clip selection separately), without prescribing public functions. Eleven composed vectors
also exercise the ordinary complete update: a five-frame sequence, frozen ShotsOnly and a
spawn-frame burst. They compare complete State, events and World geometry/visitation traces.
The sequence requires the trail to use the preceding Cloud/Mist picture and pre-motion
position, then keeps the consumed shot to this cut and removes it on the next update.
The burst case visits both cells even after its first Passes, then blocks on its flight edge
and shows the impact picture in that same update. These cases catch misordered helper calls.
Most fixtures assign Power to all headings. The distinguishing pair instead maps Right to
East and Left to West, with different frame holds, radii, pictures and complete offset maps;
the pending-left case therefore checks both heading-dependent content selections. A two-shot
reactive World case makes reverse slot order determine which body bursts. An Inactive impact
case advances picture, radius and hold while retaining the previous Mist appearance.

Per-frame M0 uses each preceding cut's W1 State, controller and Samus/camera inputs. The next
cut's controller is one sample too late. Content is privately imported from the actual ROM:
complete power-heading clips, impact clip, nonnegative damage and muzzle descriptor records.
No post-shot position, speed, hold, picture, count or cooldown is an input.

Actor consumption is generated independently from the predicted post-flight W1 bodies against
the preceding cut's enemy-domain boxes and eligibility: enemy position/radii, spritemap,
intangible property and invincibility timer. For M0, independent normal-box entry watches
confirm these preceding-cut enemy fields are still the fields C1 reads. Production C1 runs
per enemy before that enemy's AI ($A0:8FD4, $903E to $9758 to $9778 to $A143); earlier enemies'
AI may already have run.
No after-record neighbour field is used to generate this event. The private reference applies the
$A0:A143 strict overlap and first-hit-per-enemy rule; it finds one consumed-shot event at 160.
The expected W1 direction/consumed marker is never read to generate that event. An integrator
uses C1's actual event, not a decode of the expected shot after-state.

Compared: cooldown, count, slot occupancy, retained trail, integer and fractional positions,
heading, consumed, speed, radii, damage, named clip/cursor, hold and picture. Appearance is
derived content, not a separate WRAM observation; the vectors check it. Sound and invincibility
are vector outputs; this per-frame proof does not compare the sound queue, trail objects or
incoming-projectile timer (other writers run later). Their engine encoders need separate checks.

Masks: on every transition whose before game state is DoorLoading ($0998 = 11), omit only a
Free slot's trail countdown, for R1's $90:AD22 reset called by $82:E4A9. This is 161 transitions;
the only differing free trail is at 261. Apply the same mask to the null model. All live-body
fields and cooldown/count remain compared. Do not mask a slot's live trail or infer the mask
from a differing after value. Fractions retained by R1 remain State; none are nonzero on a
Free slot in the tested row. No R1 after-state is fed back into the weapons prediction.

Observed results: `chain-any-0003-DFD7`, 344/344 per-frame transitions, versus null 328/344;
five active transitions 5/5 versus null 0/5. The null model retains the preceding W1 State and
receives the identical comparison mask. This is reference per-frame proof, not whole-engine
acceptance or the clean model's future free-run floor.

One-line mutant: replace the flight acceleration with 15. It fails the flight vector and
record position/speed at frames 157, 158, 159 and 160: per-frame 340/344, active 1/5; null stays
328/344 and 0/5. The override is private and restored by process exit.

Additional one-line mutants on the eleven complete-update cases: ascending slot order
agrees on 10/11 and fails W-whole-reverse-slots (body phases, events and contact order).
Always selecting the first clip agrees on 10/11 and fails W-whole-heading-left (radius and
contact geometry). Overwriting Inactive agrees on 8/11 and fails the spawn burst, reverse-slot
and explicit impact-Inactive cases. Private gate_mutants.py injects each and leaves source
unchanged.

## Vocabulary review

Narrow additions to the player allowlist: `power beam` is the starting beam's manual name;
the four diagonal phrases are aim directions; `beam speed` and `beam trail` name visible
motion and its trail; `x offset`, `y offset` and `radius` name perceivable muzzle/contact
distances. No address, routine name or broad scanner pattern is exempted. The adversary reviews
these additions together with the World boundary and the spec.

## Picture eligibility and presentation boundary

BeamVisibility sources: $93:8254..82F0 power drawing; $93:834D..83BB impact drawing;
$93:80DD..80E6 classifies the beam burst as impact. The live clip pointer gates occupancy;
the imported optional picture replaces the power producer's valid-pointer test. $05B6 bit 0
is FlickerPhase at presentation, not the update count or a slot timer. $0C04 consumed marker
is never read by either producer. Each producer descends slot order; tick owns their placement
among other producers. $81:8A4B is the E1 sprite encoder; $81:8AB7 is an immediate return.
Power draws lack an x cutoff. Impact normal-view x uses signed [-48,304), then both producers
require screen y's high byte zero. $093F negative delegates transformed coordinates to Camera
($8B:8AD9), bypassing impact's normal-view x cutoff. Both Flying and Impact use the supplied screen coordinates directly in Rotating view.
This is a view input, not W1 rotation math.
The drawing origin is the camera origin at presentation ($0911/$0915), which may differ from
the pre-follow origin used by LeavingView. No previous-cut camera origin is substituted.

The eleven invented picture-eligibility vectors cover both phases, several slots, consumed,
absent picture, vertical edges, impact horizontal edges and the rotating-view exception.
They check ordered picture intents only; sprite encoding, clipping and global producer order
remain E1/tick checks. M0 presentation uses the preceding-cut $05B6 unchanged: drawing prepares the buffer before
requested NMI commits OAM ($80:959E calls $80:933A) and increments $05B6 ($80:95F4).
The compared cut is the NMI return ($80:9601), so the after counter is one sample too late.
The row's active-frame counter increases by one each transition. Independently selected
power picture intents occur at 157 and 159; 158,160 and removed161 have none. Each predicted
one-entry sprite's four low-OAM bytes occurs exactly once in expected WRAM OAM, and each
ineligible active sprite occurs zero times: 5/5 eligibility cases, versus null no-intent 3/5.
Consumed160 is off-phase; its visibility on an eligible phase is constructed, not observed.
This is partial evidence: presence of low-OAM bytes does not prove the high table, producer
index, global order or all output bytes. The 344/344 State result above is separate evidence.

Drawing one-line mutants: swapped phase matches 5/11; hiding consumed matches 10/11;
inclusive impact right edge matches 10/11; ascending draw slots matches 7/11. The corresponding
phase, consumed, boundary and separate-producer vectors fail. draw_mutants.py leaves source
unchanged. draw_proof.py records the partial M0 low-OAM presence evidence above.


## Integration input boundary and bounded caller coverage

The PRE handler ($90:E692, $0A42) runs W1 through $E695/$E6C9 and $DCDD; the POST handler
($90:E722, $0A44) runs movement and camera follow through $E725. $9C5B selects gravity
constants and $ECB6 the water effect; neither moves Samus. Directional movement receipts are
written by $90:9395/939A, 93D0/93D5, 9433/9438 and 9461/9466, including the enemy-blocked
routes $9826/$9842/$9871/$988D. $EB05..EB1A clears them after W1; POST movement refills
them before the next cut. They are the latest per-direction whole/fraction receipts, not net
centre displacement. Camera fraction $0DA8 is measured by
$90:96FF before follow remembers her centre. The sign of the pixel-word difference selects
which fractional subtraction is used: remembered minus current when the pixel difference is
negative, current minus remembered otherwise (equal pixels included). Each subtraction wraps
to 16 bits. Pose placement replaces the remembered pixel only, retaining its old fraction;
therefore this is not the absolute value of a full fixed-point difference. Preserve the receipt
when ordinary following is skipped; fresh Samus initialisation separately resets it. W1 uses
its high byte in leftward x-speed drift. The exact producer and watched placement/borrow cases
are in [the camera contract](world-camera.decode.md#retained-vertical-distance-producer-and-lifetime).

Pending aim is produced by POST pose application ($91:EB88, F404/F433, F490, F5CF..F5E6
and F8D3..F903), consumed on an available muzzle attempt ($90:BA56), then cleared by $EB20.
Previous-new-press $0E00 is written after cannon by drawing $90:EB35/EB42/EAB3 at $EAD4;
special POST handlers restore saved controls at $E82D/$E864. The caller supplies these writer
results at the W1 boundary. Movement/camera outcomes that omit the receipts are insufficient
for general production wiring; do not reconstruct directional fractions from final positions.

Private boundary_probe.rs, boundary-server.lua, boundary-oracle and run_boundary.py add only
observation callbacks to a private oracle copy. They play the unchanged M0 native seed and
frame pads, and compare all cached WRAM cuts. Both W1 and C1 watched runs match 345/345 cuts.
input-boundary-check.py and input-boundary-check.log record aggregate checks: all 29 watched
W1 words match preceding cuts for five active transitions; the single available spawn check
matches too, and initialisation differs only in W1-owned cooldown/count. At transition 156 to
157, right whole is 3 and right fraction is 57344; the right-drift formula reads the LEFT
fraction's high byte, which is zero. The supplied drift is therefore 768 speed units, not zero.
C1's ten normal-box entries match preceding enemy fields; one hit dispatch marks slot zero
consumed at 160. The bounded reference rerun remains 344/344 versus null 328/344, active 5/5
versus 0/5. These are State/input checks; full OAM and global picture ordering remain separate.

The private proof's movement codes 0,1,2,5,6,16,21 are an M0 subset. $90:DD74 also permits
turning codes 14/23/24 with pending aim; $DD8C guards transition code 15 by pose, suppressing
morph/unmorph and the DB..F0 poses; $DDD8 handles grabbed code 26 except pose DF. The selector
$DD61 routes item 3 to beam outside ball paths; item 5 can use beam when run is not held.
Item icon selection alone does not decide the input. Unsupported selection producers or caller
paths are harness defects until their boundary is proved. Poses 0/9B and PRE $E713 run
ShotsOnly; PRE $E8CD and its aliases run neither cannon nor shots. Do not map a no-update
handler to CannonAndShots merely from the time-stopped flag. Room/elevator reset is the actual
R1 event ($82:E4A9 or $A3:9548), not inferred solely from the DoorLoading state.

C1 normal-box eligibility also requires enabled enemy processing ($0E4C), no interaction
suppression ($0E12), neither time-stop flag ($0A78/$185E), the current active enemy list, enemy
invincibility zero, no intangible property, valid spritemap and enemy ID other than DAFF.
Property2 $0F88 bit 4 selects a multibox path outside this predicate. M0 independently visits
enemy slots 0 then 1; both use normal boxes and all guards admit them. The preceding-cut active
list itself is empty. The audited preceding records are row `chain-any-0003-DFD7`, cuts R
at frames 156 through 160, with ordered membership `[0, 1]`; these produce cuts 157 through
161. Bind temporary live-shot admission to those independently watched record identities,
not merely room or species identity. A current ActiveEnemies receipt alone does not establish
that earlier actors leave later collision inputs unchanged. Other live cases with unproved
per-enemy inputs, guards, membership, multibox or earlier AI cross-effects are defects. $A0:9785 checks Samus against
projectiles/bombs and skips owned beams; it is not the producer of enemy consumption. A
production C1 event belongs in the per-enemy pre-AI path, not inside the clean W1 model.

The processing selector $A0:8EB6 excludes empty/DAFF and deletes property $0200. Property
$0800 or frozen handler $0004 allows offscreen processing; global $183C admits every
remaining occupied slot regardless of viewport. Intangible actors still process AI. These
selector rules do not resolve the remaining C1 gap: $8FD4 runs contact before each actor's AI,
so earlier processing actors can alter later inputs, and $A226 dispatches the hit species'
response. Generic preceding-cut box reconstruction needs proof of both AI and response
cross-effects. The audited M0 records establish unchanged inputs only for their ten entries.
With post-W1 count zero, $A143 returns before searching; this proves no C1 consumption,
while caller validity and independent actor effects retain their own admission requirements.


M0 transition 182 to 183 is an independently proved outer no-update case. Before cut 182,
game state is 9, door function $099C is $E17D and $0E16 is zero. The [bank 82](https://patrickjohnston.org/bank/82)
path $E169/$E17D/$E19D enters $E1B7 and advances at $E282: enemy work, drawing and door
setup run, but W1 and $90:EB02 do not. The original watch records no DCDD, EB02 or POST
boundary at that preceding counter. The stored PRE pointer $E695 does not imply a call.
R1 reset is later $82:E4A9. Hold W1 State on this bounded no-update path; M0 count is zero
and all five shots are free, so C1 cannot consume one. A game-state-9 case with live shots
still needs independently proved C1/actor effects; it is not covered by this hold case.

## Ordinary cannon caller lifetime

`TransitionFiringAim` is movement's producer; see its decode note. W1 consumes the retained
Heading at $90:BA5F-BA67 only after availability. $90:EB02-EB20 clears it after the ordinary
$E695/$E6C9 pass or locked $E713 pass whether attempted or not; a no-update path does neither.
$90:EB35-EB42/EAB3-EAD4 writes the current newly pressed controls for the following PRE;
$E82D/$E864 restores saved demo controls before the later drawing writer. Preserve semantic
controls and apply the shoot binding at read time; held history is not prior-new history.

Ordinary eligibility maps named movement families to the existing `shoot` input: $DD05-DD3C
and $DD74 permit ground/jump/fall turns only with pending aim. $DD8C-DDC7 permits crouch/stand
and aimed transitions, suppressing morph/unmorph with inactive grapple. The standard selection
precedence is $DD3D-DD73; $B80D-B82A selects held-shoot uncharged beam when charge is absent.
Other selected weapons, active grapple, charge and forced pose boundaries are outside the
ordinary power producer contract. No per-row or frame-number eligibility table is authorized.

Fresh M0 direct-cut checks: pending lifecycle344/344 versus absent null343/344, using the actual
upstream pose result rather than a complete movement reference. Prior-new controls agree344/344
with preceding newly pressed controls; SHOOT is present only81/157. The new focused movement
publication vectors pass6/6. These do not prove engine wiring or whole movement free-run gain.
The existing gaps remain until the producer and retained caller consumption are integrated.

## Intro fight ordinary missiles

| Entry | Original sources |
| --- | --- |
| MissileRelease | $90:AC1C–AC8C, $BE62–BF44, $BA56–BAFB, $C204/$C218, $C27B, $DCDD–DD63. The shot reserves one cooldown update before the ammo, slot and muzzle checks. The shared cannon origin consumes pending aim. The original y addition for the left standing pose is +1 px before the descriptor's 6 px lift. $90:BEEC–BEF3 skips the sound queue in a cinematic. |
| MissileTravel | $90:B1DD–B2C6, $AF68–AFB2, $AFDD–AFE4, $B2F6–B365, $C303/$C353/$C367, $B16A–B196. The high byte of $0C7C denotes a missile that has moved. The ordinary release path has zero extra base speed. The initial update replaces the velocity before translating. Subsequent leftward updates combine -16 and -64 before translation. |
| Missile point contact | $94:A46F–A4D8 and $A1B5. One cell at the translated whole centre is checked if signed word((x >> 8) - width_in_screens) is negative. The perpendicular word is the whole centre, without shot radius. The World bridge owns the ordered horizontal reaction and the cell addressing width. |
| MissileBurst | $90:AE06–AE39, $93:80F8–814D, and $8B:B786–B7AA. The latter searches descending ordinary missile positions after movement and converts the first only if its whole X is below 84. $93:80F8–8104 suppresses the impact sound. $93:81E9–8240 applies the clip and its end. |
| MissileExhaust | $90:AF75–AF87, $B657–B6A8 and $B5BB/$B609. See $93:81D1 and $9B:A3CC–A4B2 for the offset. During leftward flight the actual offset is (-4, -4) px from the pre-motion whole centre. The left exhaust stream uses $90:B5A1. The right stream points to the empty list at $B4C9, so only one puff draws. |
| CinematicMissilePictures | $8B:A373–A389, $8E2D–8E51, $90:EB35–EB3E and $93:8254–82FC. EB35 draws Samus then calls the flying producer. The intro calls that producer again immediately after EB35. Each producer draws live missiles then exhaust, with exhaust aged by $90:B6A9–B7F4. $93:834D's impact producer is absent from this scene. |

Use the shared W1 RAM mapping for shot State. A live ordinary missile has $0C18 = $8100
and its impact has $8800. Decode the leftward clip beginning at $93:9F03 and the impact
clip beginning at $93:A039. The impact state first points at the beginning of its clip
with hold 1 and a still-retained flight picture. Use the pointer and the visible picture
together when comparing this cut. Flight's one frame repeats with hold 15 and radii
(4, 4). Impact's six frames have hold 3 and radii (8, 8). The missile uses $93:8641 damage
100 and the burst writes damage 8. Audio owns the actual queue, which receives no shot or
impact request from this cinematic path.

Each of 18 exhaust positions uses two-byte stride. $7E:D658 is the live hold, $D6A0
the next frame, $D6E8 the displayed tile and attributes, and $D730/$D778 the
whole position of the only drawn puff. The right side uses $D67C/$D6C4/$D70C/$D754/$D79C.
Its hold is zero at each compared cut. Its retained coordinates match the drawing side while a
puff is live. Mask the idle picture, pointer and coordinates of free exhaust slots because
they have no reader before reuse. Mask the free-shot hold and pre-instruction using the
existing W1 rule. These masks leave the five shot trail countdowns visible after individual
shot retirement.

### Caller and replay joins

Use the flashback encounter contract reviewed at 65a5c811d. The intro calls Samus and
weapons before the encounter. W1 returns the candidate slot identities and their after-flight
whole X in descending slot order. The encounter alone chooses the hit and returns the identity
to W1. W1 converts it during the encounter interval. Neither owner feeds back the next cut's
shot data. Existing C1 consumption returns a distinct report.

The demo owner runs before the cannon call inside the active update. Its imported battle
timeline supplies held and newly pressed input separately. $90:E6C9–E710 saves the movie
controls, and its pose handler at $E91D calls $91:83C0 before $DCDD. The previous fresh input
comes from $0E00 with the actual shoot binding $09B2. The intro POST handler at $90:E851–E864
restores movie input before $EAD2–EAD4 updates retained history. Thus the held demo shoot and
previous demo shoot fresh are both unsuitable substitutes for the prior fresh control read by
weapons. On this row that prior shoot bit stays clear. All four releases originate at the
current demo fresh input. Menu selection is established by the intro setup. Missile ammo
at $09C6 and selection at $09D2/$0A04 belong to the equipment owner. $90:BEB4 reports
incoming-shot protection to $18AC. It writes 20 on a successful shot and stays there during
this fight, because the original intro update only advances $18A8 and $18AA at $8B:8E1A–8E29.
The demo-owned pose and movement supply the cannon inputs. At the four releases the
previous centre is (155, 115), (155, 115), (148, 115), (148, 115). Movement and the camera
supply the left whole and vertical fractional receipts, zero at the four releases.

The flashback sets the room grid to 16 by 16 cells with $8B:A39F–A3A7. It leaves $07A9,
width in screens, zero in the witness. All 68 missile travel updates skip the World request.
Deriving width in screens from the grid dimensions would manufacture 68 contacts. Both
weapons and Samus movement receive the flashback room and its separately retained geometry.

The stage's starting cut is any-intro-2219 cut 0 (movie 2219), with the demo and
equipment installed and all five shots free. The following work beat enters the ordinary cannon
mode. The first W1 firing difference is cut 91 (movie 2310). The record proof covers through
cut 649 (movie 2868), the first hidden work beat after the hide. The latest live weapon
cut is 409. This proof admits the intro fight only.

| Row cut | W1 effect |
| --- | --- |
| 91, 135, 335, 376 | Ordinary missile launches. Ammo changes 900 → 899 → 898 → 897 → 896. Cooldown is 10 on each release. The first held repeat opportunity passes without a release. |
| 94 | First exhaust spawns from the centre before this frame's motion. It has received two presentation visits by this cut. |
| 101, 106 | Overrun cuts already include gameplay and both weapon draws. The following NMI cuts 102 and 107 commit those pictures without advancing W1 State. |
| 109, 110 | At 109 the missile is at whole X 84 and remains flying. At 110 it bursts at whole X 78 with unchanged fixed-point centre and still holds the flight picture in State. The intro draws the exhaust and omits the burst. |
| 152, 350, 391 | Later encounter-requested impacts. The first two impacts are at 78 + 3/16 px. The last two are at 83 + 1/2 px. |
| 129, 171, 369, 410 | The four impacts retire at clip end. Shot count decrements and the trail countdown survives. |
| 396, 397 | The final demo input action installs locked mode on 396 during the ordinary cannon interval. 397 is the first ShotsOnly interval. The fourth impact ages on both cuts and retires in ShotsOnly. |
| 131, 520 | The work beat has not opened at 131. By 520 the intro beat advances and the encounter is incomplete. All shots and exhaust are absent at those cuts. Preserve the intro owner's work and draw boundaries. |
| 648, 649 | Gameplay and the encounter get their final update before the intro hides Samus and shots at 648. 649 holds W1 State with presentation closed. The remaining ammo is unchanged and there are no live shots or puffs. |

Within a visible beat the sign of $1A57 places encounter pictures either before both Samus and
weapons or after both. It is negative throughout a visible battle in the indexed row. The two
weapon producer visits remain successive in the shared output, with Samus drawn before them.
The first visit runs inside Samus's drawing call. Apply the final gate after the encounter for
both visits. Use the camera's presentation position. In this row it is static. The work
opportunity uses the preceding gate before the encounter may hide her. A wait for the NMI of an
already-produced work frame must preserve the prepared pictures and avoid another projectile,
cannon or exhaust update.

### Reference evidence

The independent Python reference imports authored controls and content from local assets. It
carries slot, clip, cooldown, count, exhaust and ammo through cut 649. It reads the demo's
current control outputs from authored tape and borrows the preceding pose, centre, camera and
equipment eligibility. It derives a hit request from the predicted ordered candidates through
the encounter's contact rule using the independently observed preceding boss phase. The
weapons inputs contain no ammo-debit or impact outputs. Width and the work gate come from the
caller. Record cut kinds and the cinematic beat identify work and display opportunities.
Cut 520 has no live weapon picture or exhaust State, so either presentation position yields
the same W1 comparison.

The 31 invented vectors separate dry or obstructed cannon attempts, late impact versus block
impact, locks, skip boundaries, exhaust capacity and both presentation visits. The model
matches all 650 State cuts (5,850 field-group comparisons), versus 91 cuts (3,625
field-group comparisons) for the held cut-0 baseline. Active State matches 142/142 versus
0/142. There are four ammo debits, 68 flight movements, four impact requests and 16 puff
requests. The record covers 393 cannon updates and 251 ShotsOnly updates.

The independent drawing comparison selects the flying missile and exhaust tile attributes
from the committed OAM list with ordered x, y and size, and compares all 644 OAM publications
across the checked interval. It also tests absence of the live impact's hypothetical pieces at
their exact coordinates in 76 publications. The reference matches 644/644 versus 552/644
for a held empty picture list. OAM for unrelated producers and all audio queues remain
outside this reference. The existing indexed production floor still needs the model and join.

Seeded faults are local options in the reference. Applying later acceleration on the launch
beat first fails at 91 in flight x, speed and OAM. Using held shoot instead of current fresh
shoot spends another missile at 101. Dropping the encounter request first changes the
phase and producer output at 110. Drawing only one weapon visit loses one flight copy at
91 and first changes exhaust State at 94. Drawing the live impact first changes output at 110
with State still matching all 650 cuts. Substituting a screen count derived from the grid first
produces an unearned World query on cut 91 and 68 queries total. The locked-cooldown example
fails if ShotsOnly enters the cannon clock. The authentic fight's cooldown at lock is zero.

The stale-hit vectors name a free slot, a live impact and a flying beam. The latter two
retain their normal advance, cooldown and silence. The full exhaust vector compares all
18 carried puffs by slot, position and clip age, so a replacement cannot pass by leaving
the occupancy count unchanged. Re-priming the impact, bursting the beam and replacing an
occupied exhaust position each fail their respective vector.
