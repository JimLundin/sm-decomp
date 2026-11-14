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
Run with `$LOCALAPPDATA/sm-decomp-tools/python-3.13.7/python.exe`. The embedded Python requires
adding the script directory to sys.path when importing the other scripts. These private files
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

## Budget

k = 37.5 weighted tokens per spec unit, frozen by MODEL decision 7.

At commit `294746ad5212c9e60df8c08940f907b12e98b557`, `tools size` over
`crates/game/src/projectile.rs` and `crates/game/src/state/projectile.rs` reports:

| File | Score | Logic | Types | Data |
| --- | ---: | ---: | ---: | ---: |
| projectile.rs | 2579.25 | 2584 | 146 | 183 |
| state/projectile.rs | 25.5 | 86 | 522 | 26 |
| total | 2604.75 | 2670 | 668 | 209 |

The old code implements this power-beam flight/contact/clip path, with stubs for other weapons;
it does not fire shots. This first scope's baseline therefore understates its firing behaviour.
Importer and World bridge logic are outside the clean model baseline.

- Area shots (old 2604.75): ShotSpacing, HoldingShoot, CannonOrigin, InheritedMotion, MuzzleContact, FlyingBeam, ShotContact, BeamTrail, LeavingView, EndingShot, BeamPictures, BeamVisibility, Content and inputs

Budget: 37.5 × 55.0 units = 2,062 (shots 2,062).

Official count: 12 entries, 42 arms, 7 cell terms, 32 formula operators; no findings.


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
centre displacement. Camera fraction $0DA8 is the absolute vertical displacement fraction
computed by $90:96FF, before follow remembers her centre; preserve it when that route is skipped.

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
