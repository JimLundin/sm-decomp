# Power beam and cinematic missiles

Vectors: [weapons-projectiles.toml](../../scenarios/vectors/weapons-projectiles.toml).

## Header

**Scope.** W1's first part is the uncharged power beam: holding shoot, the muzzle where a shot
appears, motion inherited from Samus, acceleration, contact, disappearance and its visible clip.
W1 also covers the ordinary missiles Samus fires leftward in the intro fight, their impact
and exhaust. Other beams, charging, other missile paths, bombs and weapon selection remain later
W1 work. These rules replace the corresponding flight, firing and presentation behaviour of the
old projectile owner.

**Neighbours.** Samus movement supplies her integer centre, her preceding movement distances,
pose change aim and whether the muzzle uses its moving offset. The pose content supplier
supplies the muzzle descriptor below. Camera supplies its integer origin of view and vertical
fractional speed. Equipment and the surrounding caller supply whether uncharged power-beam
firing applies; this spec receives that eligibility as `shoot` after selection. W2 answers shot
contacts and owns each block's
response. C1/B8 report a shot consumed after its motion; enemy overlap, damage and vulnerability
are theirs. AF3 imports muzzle descriptors, damage, clips and pictures in the named format below.
Drawing supplies `FlickerPhase { Even, Odd }` for the current presentation and a
`DrawView`: Normal carries `ScreenOrigin`; Rotating carries a complete map from live ShotSlot
to already transformed `ScreenCentre` pixels, then consumes ordered picture intents; trail effects and sound queues consume requests.

**Notation.** x grows rightward, y downward. `word(v)` wraps to 16 bits, `signed(v)` interprets
a word as signed, `low(v)` and `high(v)` are its low and high bytes. Position is a wrapping
32-bit `FixedPosition` in 1/65536 px; its high word is the integer pixel centre. Speeds are
signed wrapping `BeamSpeed` in 1/256 px per update. Distances and offsets are integer pixels.
Every formula wraps only where stated. Headings are `DownLeft`, `DownRight`, `Down` with
left/right facing, `Left`, `Right`, `UpLeft`, `UpRight`, `Up` with left/right facing. Facing
distinguishes pictures and muzzle offsets for vertical shots, not their motion.

## Memory a caller holds

The complete State is serialisable through serde `Serialize` and `Deserialize`; a caller can
set every quantity to any value its type allows, save it and read it back. Slot order is five
stable `ShotSlot` values. A slot is `Free { trail, retained_fraction }`, `Flying { body, consumed }`, or
`Impact { body }`; the body carries the quantities below. Count is independent of occupancy:
zero-damage content can select an occupied slot, and replacing that slot still increments count.
Its range is the whole `ShotCount` word, because callers can restore it independently. The
signed comparisons below define unusual values too.

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| cooldown | `FrameWord`, wrapping 16-bit updates | `ShotSpacing`, `HoldingShoot`; output |
| count | `ShotCount`, wrapping 16-bit count | `HoldingShoot`, `EndingShot`; output |
| body position | x,y `FixedPosition` | `FlyingBeam`, `ShotContact`, `LeavingView`, `MuzzleContact`, `BeamVisibility`; output |
| heading | `Heading`, the named directions above | `InheritedMotion`, `FlyingBeam`, `ShotContact`; output |
| speed | x,y `BeamSpeed` | `FlyingBeam`, `ShotContact`; output |
| radius | x,y `Radius`, 0..255 px | `ShotContact`; output |
| damage | `Damage`, 0..32767 hit points | `HoldingShoot`; C1 output |
| trail countdown | `FrameWord`, retained even in a free slot | `BeamTrail`; output |
| retained fraction in a free slot | x,y `PixelFraction`, 1/65536 px | `MuzzleContact` |
| consumed | bool, flying shot hit an actor | `EndingShot`; output |
| clip and next frame | `ClipId`, checked `ClipFrame` index including the end | `BeamPictures`; output |
| remaining hold | `FrameWord`, updates | `BeamPictures`; output |
| visible picture and trail appearance | optional `PictureId`, optional `TrailAppearanceId` | `BeamTrail`, `BeamVisibility`; drawing output |

Position, speed, radius, damage, cooldown and holds are copied quantities because each affects
later motion, contact, firing or drawing. A free slot has no body: removal clears all body
quantities and leaves zero retained fractions. R1's room/elevator reset clears slots, trails,
count and cooldown, but preserves each slot's fractions; the caller supplies these reset
fractions when replacing State. They matter to the initial leftward muzzle probe. A free slot's
discarded hold is not State; it has no reader before a spawn replaces it.

## Content and inputs

`Muzzle` is `Forbidden`, `PreviousAim`, or `Aimed(Heading)`, with an unsigned vertical lift and
two complete maps from `Heading` to `Offset { x, y }`: stationary and moving. `PreviousAim`
accepts only an `Aimed` previous descriptor; it takes that heading and the current descriptor's
lift and offset maps. A pending `Heading` overrides the descriptor's aim, including Forbidden.
The importer assembles each descriptor; no pose number enters W1.

Cells: x offset, y offset


`PowerBeam` content has nonnegative damage, a complete map from Heading to flying ClipId,
and one common impact clip.

Cells: clip

A `Clip` has a nonempty sequence of `Frame { hold, picture, radius, trail_appearance }`, and
ending `Remove` or `RepeatFrom(ClipFrame)`. Holds are 0..32767 updates, radii 0..255 pixels.
`trail_appearance` is optional: Inactive means None, a frame that leaves that channel unchanged.
Actual imported flying frames have a named appearance; actual impact frames are Inactive.
RepeatFrom names an existing frame. This is an
animation clip, with no commands or original control words. The importer validates all indices,
maps and identifiers; the model handles every valid combination, including zero damage/holds.

Cells: hold, picture, radius, trail appearance


Frame inputs also give update mode `CannonAndShots` or `ShotsOnly`, `time_stopped`, `shoot`, `new_press`, `previous_new_press`, the current and
previous muzzle descriptors, `moving_muzzle`, `pending_aim`, her centre, left/right/up/down
movement distances as whole and fractional words, camera position and camera vertical fractional
speed. Pending aim consumption is returned as an output; the caller remembers shoot history.
`shoot` means held shoot after weapon and pose eligibility: standing, running, jumping without spinning,
crouching, falling, moonwalking or stopped by a wall may fire this power beam; spin jump,
wall jump cannot fire. Knockback cannot fire. A pose change's input is eligible when it is a crouch/stand
transition or an aiming transition; morphing suppresses it, as does unmorphing. Ground, jump and fall turns permit the ordinary cannon only while pending aim is present. Grabbed poses use
their muzzle descriptor. The caller supplies whether the uncharged power-beam path applies;
the item selection alone does not decide it. Active grapple suppresses this input.
Those decisions are caller inputs, not something inferred from velocity or raw pose numbers.

`World` receives a shot slot, heading, axis, centre, leading boundary, perpendicular start and
cell count. It supplies an ordered `Blocks` or `Passes` response for each cell in the span,
top to bottom on a horizontal edge, left to right on a vertical edge. Every cell is visited,
even after a Passes, applying each reaction immediately before the next visit. The room gives
width/height in screens as words. World owns mapping this geometry into its content, including
the named `ContactContinuation` contract: aliases caused by limited row/width range and contact
past the level must preserve the original's responses and reaction order. They are not an
implicit solid outside the room. W1 has no byte or partial-cell address API. Production support
for ContactContinuation is the integrator's W2 capability check; declaring this contract does
not establish that the existing World supports it.

## Order that matters

The caller retains controls pressed this frame at the drawing phase after this frame's cannon and
pose changes; the following cannon reads that retained value as `previous_new_press` through
its configured shoot control. It is distinct from held-button history. A control-restoration
path restores that saved history, and a skipped history writer retains it.

The ordinary cannon interval and the elevator/locked ShotsOnly interval clear caller-held
pending aim after W1, even when no shot was attempted or available. Movement's later
`TransitionFiringAim` publication supplies the next interval's pending heading. An interval without W1
 neither updates W1 nor performs this clear. An available attempt consumes pending aim
at `CannonOrigin` before the interval clear.

In CannonAndShots, cooldown changes before firing. Firing happens before existing and newly spawned shots update.
Slots update from last to first, so W2 sees their contacts in that order. A newly fired shot has
both its muzzle contact and its first flight/clip update in the same frame. C1/B8 consumption
after that update is visible at this frame's cut and removes it on the next weapons update.
A trail reports the position and appearance before motion; contact precedes the clip change;
the view check follows flight/contact and precedes the clip change. Time stopped changes cooldown
as below but suspends firing and all shot motion/clip updates in CannonAndShots. ShotsOnly
updates existing shots regardless of time_stopped and leaves cooldown unchanged: when she faces
the player or an elevator carries her, her cannon is inactive but shots already in the room move.

The composed vectors call an ordinary complete frame update, with the later actor event
included at its declared point. `W-whole-spawn` through `W-whole-consumed-removal` are five
consecutive frames of one invented shot. `W-whole-contact-burst` checks all contact visits and
the impact picture within its spawn frame; `W-whole-frozen-shots-only` checks the full special
mode update. Their helper observation operations do not prescribe public model functions.


## Shooting from her cannon

### ShotSpacing
- kind: Timing
- summary: Waiting between power-beam shots.
- params: frozen hold 32 updates; pressed spacing 15; held spacing 25.
- switch: Fixed
- CannonAndShots and time stopped → cooldown becomes 32 [W-cool-frozen].
- ShotsOnly → retain cooldown, do not fire; update existing shots even when time stopped [W-shots-only, W-whole-frozen-shots-only].
- CannonAndShots and time running and signed cooldown positive → cooldown decreases by one [W-cool-positive].
- CannonAndShots and time running and signed cooldown nonpositive → cooldown becomes zero [W-cool-negative].
- successful spawn with new press this or previous frame → cooldown becomes 15 [W-spawn-press, W-spawn-previous].
- successful spawn without either new press → cooldown becomes 25 [W-spawn-held].
- checked: oracle (W-cool-frozen, W-cool-positive, W-cool-negative, W-shots-only, W-whole-frozen-shots-only, W-spawn-press, W-spawn-previous, W-spawn-held)

### HoldingShoot
- kind: Mechanic
- summary: Held shoot repeats when the cannon is available.
- params: five slots.
- switch: Fixed
- no shoot or signed `word(count - 5)` nonnegative or low cooldown nonzero after spacing : no spawn [W-no-shoot, W-full, W-cooling].
- held shoot with availability → increment count wrapping, reserve cooldown 1, select the first zero-damage slot (Free has damage zero), or last slot when none [W-spawn-press, W-slot-zero, W-whole-spawn, W-whole-heading-right, W-whole-heading-left, W-slot-last].
- selected muzzle forbidden or PreviousAim without an Aimed previous descriptor, with no pending aim → restore count by one wrapping, leave cooldown 1 and slots unchanged [W-forbidden, W-previous-forbidden].
- selected muzzle valid → replace selected body with its heading, origin, content damage, first-frame radius, clip selected by that heading at frame zero with hold 1; preserve previous picture/trail appearance until the clip change; set trail countdown 4, report fire sound and incoming-shot protection 10 [W-spawn-press, W-slot-zero, W-whole-spawn, W-whole-heading-right, W-whole-heading-left].
- checked: oracle (W-no-shoot, W-full, W-cooling, W-spawn-press, W-slot-zero, W-whole-spawn, W-whole-heading-right, W-whole-heading-left, W-slot-last, W-forbidden, W-previous-forbidden)

### CannonOrigin
- kind: Mechanic
- summary: The shot starts at the cannon of the pose she is showing.
- params: imported muzzle descriptors.
- switch: Fixed
- pending aim present on an available firing attempt → use it and report it consumed [W-pending].
- no pending aim and muzzle PreviousAim with Aimed previous muzzle → use previous heading with current offsets and lift [W-previous].
- chosen aim and offset map (moving when moving_muzzle, stationary otherwise) → integer origin is `word(samus_x + x_offset)`, `word(samus_y + y_offset - lift)` [W-spawn-press, W-moving, W-whole-spawn, W-whole-heading-right, W-whole-heading-left].
- checked: oracle (W-pending, W-previous, W-moving, W-whole-spawn, W-whole-heading-right, W-whole-heading-left, W-spawn-press)

### InheritedMotion
- kind: Quirk
- summary: The shot inherits a peculiar part of her preceding motion.
- params: cardinal base 1024, diagonal base 683, in 1/256 px per update.
- switch: Fixed
- heading has a rightward axis → x speed is `word(base + low(right_whole) * 256 + high(left_fraction))` [W-drift-right, W-drift-left, W-drift-diagonal].
- leftward → `word(-base + low(left_whole) * 256 + high(camera_vertical_fraction))` [W-drift-right, W-drift-left, W-drift-diagonal].
- no horizontal axis → x speed zero [W-drift-down, W-drift-up-zero, W-drift-up].
- heading has a downward axis → y speed is `word(base + low(down_whole) * 256 + high(up_fraction))` [W-drift-down, W-drift-up-zero, W-drift-up, W-drift-diagonal].
- upward with low(up_whole) zero → y speed `word(-base)` [W-drift-down, W-drift-up-zero, W-drift-up, W-drift-diagonal].
- upward otherwise → y speed `word(-base + (((low(up_whole) * 256 + high(right_fraction)) >> 2) | 49152))` [W-drift-down, W-drift-up-zero, W-drift-up, W-drift-diagonal].
- no vertical axis → y speed zero [W-drift-right, W-drift-left].
- checked: oracle (W-drift-right, W-drift-left, W-drift-diagonal, W-drift-down, W-drift-up-zero, W-drift-up)

## A shot moving through the room

### MuzzleContact
- kind: Quirk
- summary: The muzzle can already touch a block before the shot flies.
- params: leftward probe -1 in 1/256 px.
- switch: Fixed
- spawn → contact uses zero speeds, except pure Left uses x speed -1; it retains selected slot fractions during this check, then zeroes both fractions and assigns InheritedMotion only if still Flying [W-muzzle-left, W-muzzle-left-fraction, W-muzzle-up, W-muzzle-impact, W-whole-contact-burst].
- checked: oracle (W-muzzle-left, W-muzzle-left-fraction, W-muzzle-up, W-muzzle-impact, W-whole-contact-burst)

### FlyingBeam
- kind: Mechanic
- summary: The beam gets faster as it flies.
- params: acceleration 16 in 1/256 px per update on each aimed axis.
- switch: Parameter
- unconsumed Flying → each aimed positive axis adds acceleration, each aimed negative axis subtracts it, wrapping speeds; each moved position is `wrap32(position + signed(speed) * 256)` [W-flight-right, W-flight-left, W-flight-wrap, W-flight-adjusted, W-whole-flight-2, W-whole-flight-3].
- checked: oracle (W-flight-right, W-flight-left, W-flight-wrap, W-flight-adjusted, W-whole-flight-2, W-whole-flight-3)

### ShotContact
- kind: Mechanic
- summary: A whole contacted edge must stop the shot for it to burst.
- params: 16 px cells; maximum checked span 16 cells.
- switch: Fixed
- an axis moves → leading boundary is `word(integer_centre - radius)` for negative speed, otherwise `word(integer_centre + radius - 1)`; perpendicular start is `word(perpendicular_centre - perpendicular_radius)`, span minus one is `word(perpendicular_centre + perpendicular_radius - 1 - (start & 65520)) >> 4` [W-edge-negative, W-edge-positive, W-contact-partial].
- span minus one at least 16 or signed `word(high(leading_boundary) - axis_room_screens)` nonnegative → make no world requests and do not block [W-contact-wide, W-contact-outside].
- several live shots in a frame → complete their motion, contact and clip updates in descending stable ShotSlot order [W-whole-reverse-slots].
- valid edge → visit every response of World's ordered span, even after a Passes, applying all reactions in order [W-world-visits-all, W-whole-contact-burst].
- every requested cell Blocks → burst [W-contact-all, W-contact-partial].
- at least one Passes → keep Flying [W-contact-all, W-contact-partial].
- diagonal shot → horizontal motion/contact comes first, and vertical motion/contact happens only while still Flying [W-diagonal-stop, W-diagonal-pass].
- burst → shift each integer centre by radius along each aimed axis (wrapping), keep fractions and speeds, become Impact with damage 8, impact clip at frame zero and hold 1, keep current radius and picture until BeamPictures, report impact sound [W-burst, W-diagonal-stop].
- checked: oracle (W-edge-negative, W-edge-positive, W-contact-partial, W-contact-wide, W-contact-outside, W-world-visits-all, W-whole-contact-burst, W-contact-all, W-diagonal-stop, W-diagonal-pass, W-burst, W-whole-reverse-slots)

### BeamTrail
- kind: Timing
- summary: A moving beam leaves a trail every fourth update.
- params: trail interval 4 updates.
- switch: Parameter
- unconsumed Flying → decrement trail countdown wrapping; at zero restore interval and report trail using position/appearance before motion [W-trail, W-trail-wrap, W-trail-adjusted, W-whole-trail-and-consume].
- checked: oracle (W-trail, W-trail-wrap, W-trail-adjusted, W-whole-trail-and-consume)

### LeavingView
- kind: Mechanic
- summary: A flying shot or its fresh impact disappears sufficiently far from the view.
- params: lower distance -64 px inclusive, upper distance 320 px exclusive.
- switch: Fixed
- after Flying motion/contact, either axis has `signed(word(centre - camera))` outside [-64,320) → remove shot [W-view-low, W-view-high, W-view-inside, W-view-wrap].
- checked: oracle (W-view-low, W-view-high, W-view-inside, W-view-wrap)

## A shot ending and changing its picture

### EndingShot
- kind: Mechanic
- summary: Actor contact spends a shot; a completed impact disappears.
- params: none.
- switch: Fixed
- C1/B8 reports consumed for a Flying slot after weapons update → retain its body and mark consumed [W-consume-late, W-whole-trail-and-consume].
- consumed Flying at next weapons update, view removal or clip Remove → free slot, clear body and decrement count wrapping then clamp negative signed result to zero; keep trail countdown [W-consumed, W-remove-empty-counter, W-whole-consumed-removal, W-clip-remove].
- Free or Impact receives actor consumption : unchanged [W-consume-free, W-consume-impact].
- checked: oracle (W-consume-late, W-whole-trail-and-consume, W-consumed, W-remove-empty-counter, W-whole-consumed-removal, W-clip-remove, W-consume-free, W-consume-impact)

### BeamPictures
- kind: Timing
- summary: Flight repeats a clip; a burst plays its impact clip to the end.
- params: imported clips.
- switch: Fixed
- live Flying or Impact after motion → decrement remaining hold wrapping; nonzero retains picture/radius/next frame [W-clip-wait, W-clip-zero, W-whole-flight-2].
- hold becomes zero with a next frame available → show its picture and radius, copy its trail appearance when present, set its hold and advance next frame [W-clip-next, W-spawn-picture, W-whole-spawn, W-whole-heading-right, W-whole-heading-left, W-whole-flight-3, W-whole-contact-burst].
- a displayed frame has Inactive trail appearance → preserve the earlier retained trail appearance [W-whole-impact-inactive, W-whole-reverse-slots].
- hold becomes zero at ending RepeatFrom → show that named frame as above [W-clip-repeat].
- hold becomes zero at ending Remove → EndingShot removes the slot [W-clip-remove].
- checked: oracle (W-clip-wait, W-clip-zero, W-whole-flight-2, W-clip-next, W-spawn-picture, W-whole-spawn, W-whole-heading-right, W-whole-heading-left, W-whole-flight-3, W-whole-contact-burst, W-clip-repeat, W-clip-remove, W-whole-impact-inactive)


### BeamVisibility
- kind: Mechanic
- summary: Beam slots alternate their visible phase; an impact stays visible on both.
- reads: current body integer centres and picture, stable ShotSlot, FlickerPhase, DrawView { Normal(ScreenOrigin), Rotating(ShotSlot to ScreenCentre map) }.
- writes: separate ordered Flying and Impact picture intents with ShotSlot, PictureId and wrapping screen x,y pixels; E1 encodes sprites.
- params: vertical screen range 0..256 pixels; impact horizontal range -48..304 pixels.
- switch: Fixed
- Free or no picture : no picture intent [W-draw-absent].
- Flying → even ShotSlot appears on Odd phase, odd ShotSlot on Even phase; consumed has the same eligibility until EndingShot removes it [W-draw-even, W-draw-odd, W-draw-consumed].
- any eligible body → Normal screen centre is `word(integer_centre - view_origin)` per axis; Rotating uses the supplied screen centre directly for Flying and Impact, with no subtraction; vertical screen word at least 256 yields no intent [W-draw-view, W-draw-rotating, W-draw-rotating-flying].
- Impact → both phases eligible; in Normal view signed screen x must be at least -48 and below 304; Rotating view uses the supplied transformed screen centre without this horizontal cutoff [W-draw-impact-even, W-draw-impact-odd, W-draw-impact-edge, W-draw-rotating].
- several eligible bodies → emit descending ShotSlot order within each Flying and Impact producer; caller places those producers among the frame's other pictures [W-draw-even, W-draw-odd, W-draw-producers].
- checked: oracle (W-draw-absent, W-draw-even, W-draw-odd, W-draw-consumed, W-draw-view, W-draw-impact-even, W-draw-impact-odd, W-draw-impact-edge, W-draw-rotating, W-draw-rotating-flying, W-draw-producers)

## Ordinary missiles in the intro fight

The missile uses the power beam's five slots, State serialization, update modes and cannon origin. A live missile has its own flight and impact clips. The intro fight fires
standing to the left. Imported pose content gives the cannon origin 11 px left and 5 px
above her integer centre. This path covers that heading and the time-running fight. Missile clips use the same frame
holds, pictures and radii as BeamPictures.

The demo's control writer supplies the current held and freshly pressed controls to the pose and
cannon pass. The previous fresh controls come from the retained caller history after the
ordinary control restoration and drawing. Apply the assigned shoot button to both sets of
fresh controls. Use the current demo fresh controls even when the setup begins with different
movie controls. Samus supplies the cannon's eligibility, current and previous muzzle descriptors,
pending aim, muzzle offset choice, integer centre and the preceding movement receipt. Inventory
supplies selected weapon, remaining ammo and whether the selection cancels after a shot. W1
reports a one-missile debit, incoming-shot protection in updates and a request to clear selection
and its automatic-cancel setting if applicable. Inventory and contact owners apply those reports.

World owns the point contact response at the missile's moved centre. It supplies the width of the room
in screens as written by the caller, separately from the room grid width in cells. During the
intro fight this screen width is zero. A width test that skips contact still permits movement.
In the admitted contact case, World owns the aliasing and reactions to shots.

The intro opens a work beat, and the caller selects CannonAndShots or ShotsOnly on entry. The
demo's final facing-left action changes the choice for the following beat. Its action beat
finishes the current cannon interval. The encounter reads flying ordinary missile candidates
after the weapons update and reports the identity of the missile it hits. Apply that report
before drawing. The intro may hide Samus and shots on the same beat after both updates. A later
hidden beat skips the entire weapons step and presentation. A presentation skipped after work
also skips exhaust aging. There is no implicit weapon reset at either the lock or the hide.
The intro owns the display boundary, and the demo owns the choice of update mode.

Additional carried quantities are available for save and restore with the shared State.

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| shot kind | beam or ordinary missile, distinguished in both flight and impact | MissileTravel, MissileBurst, CinematicMissilePictures, encounter output |
| launch pending | bool for a Flying missile awaiting its first movement | MissileTravel |
| trail appearance | optional imported TrailAppearanceId for the missile's flight clip | MissileExhaust, effects output |
| exhaust slot | one of 18 stable positions, either free or carrying a puff | MissileExhaust, CinematicMissilePictures |
| exhaust position | x,y whole pixels, wrapping on each axis | CinematicMissilePictures, output |
| exhaust clip position | optional displayed PictureId, next ClipFrame and remaining FrameWord | CinematicMissilePictures, output |

### Missile content

Each exhaust appearance describes an offset from the missile's whole centre and a
nonempty clip of stationary pictures with positive holds in presentation visits. Import the
leftward missile appearance and its single puff. The room's flight clip repeats one image
with a hold of 15 updates. Its stationary impact uses six images held for three updates
each and then ends. The exhaust shows four images held for four presentation visits each.
The exact picture identities and radii come from content.

Cells: x offset, y offset, hold, picture

### MissileRelease
- kind: Mechanic
- summary: A fresh shoot press spends one selected ordinary missile when the cannon is ready.
- params: five shot slots, cooldown reservation 1 update, shot spacing 10 updates, incoming-shot protection 20 updates.
- switch: Fixed
- CannonAndShots with time running, ordinary missile eligible and selected, fresh shoot now or previously, signed `word(count - 5)` negative, low cooled cooldown zero → reserve cooldown 1 and attempt to shoot using the first zero-damage slot in ascending order [W-missile-press, W-missile-prior, W-missile-last-ammo].
- attempt with zero ammo, no zero-damage slot or unavailable muzzle → keep count and ammo, reserve the cooldown without making a shot [W-missile-empty, W-missile-no-slot, W-missile-no-origin].
- ready attempt with ammo, an available slot and a valid muzzle → create the leftward missile at CannonOrigin with zero fractions, pending launch, imported damage and first radius. Prime its flight clip with hold 1 and its trail with countdown 4. Increase count by one wrapping, set cooldown 10, report ammo debit and protection. The cinematic emits no firing sound [W-missile-press, W-missile-prior, W-missile-last-ammo].
- successful release with final ammo or automatic cancellation → report clear selection and automatic cancellation [W-missile-last-ammo, W-missile-cancel].
- held shoot without either fresh bit, an ineligible or unselected cannon, full count or nonzero low cooldown after spacing → make no attempt [W-missile-held, W-missile-cooling, W-missile-count-cap, W-missile-ineligible, W-missile-unselected].
- ShotsOnly → hold cooldown and advance any existing shot [W-missile-locked].
- cinematic work suspended → hold weapon State [W-missile-hidden].
- checked: oracle (W-missile-press, W-missile-prior, W-missile-last-ammo, W-missile-empty, W-missile-no-slot, W-missile-no-origin, W-missile-cancel, W-missile-held, W-missile-cooling, W-missile-count-cap, W-missile-ineligible, W-missile-unselected, W-missile-locked, W-missile-hidden)

### MissileTravel
- kind: Mechanic
- summary: A leftward missile starts slowly, then accelerates and checks the block at its centre.
- params: launch base 256 in 1/256 px per update, flight acceleration 80 in 1/256 px per update squared.
- switch: Fixed
- first flying update → clear pending launch, set x speed to `word(low(left_whole) * 256 + high(camera_vertical_fraction) - 256)` and y speed to zero. Advance x by the signed speed with the same fixed-point addition as FlyingBeam. This is the first motion in the launch beat [W-missile-press, W-missile-drift].
- later unconsumed flying update → subtract 80 from x speed with word wrapping and move by the new signed speed [W-missile-fly, W-missile-exhaust].
- moved missile with signed `word(high(integer_x) - width_in_screens)` negative → ask World for one horizontal reaction at its integer centre and burst there if it Blocks [W-missile-fly, W-missile-block].
- A moved missile outside the width test skips the World request [W-missile-press, W-missile-no-grid-probe].
- several live shots → update in descending ShotSlot order and return surviving flying ordinary missile identities and whole X in that order after motion and clip changes [W-missile-candidate-order].
- checked: oracle (W-missile-press, W-missile-drift, W-missile-fly, W-missile-exhaust, W-missile-block, W-missile-no-grid-probe, W-missile-candidate-order)

Use LeavingView after missile movement or a block burst. Use BeamPictures for the current
flight or impact clip and EndingShot for disappearance. The missile has no preflight muzzle
contact probe on this path. The shared later-actor consumed marker is distinct from the
encounter's burst request.

### MissileBurst
- kind: Mechanic
- summary: A stopped missile bursts at its own position.
- params: damage 8 hit points, cooldown cap 20 updates.
- switch: Fixed
- flying missile receives a blocking contact or the encounter names its identity → turn that slot into an impact at the same position and keep its speeds, heading, displayed picture, radii and trail countdown. Set its damage to 8 and prime the start of the impact clip with hold 1. If signed `word(cooldown - 21)` is nonnegative, set cooldown to 20. Suppress impact sound in a cinematic [W-missile-block, W-missile-hit].
- burst from a block this update → show the first impact picture during this same update if the shot survives LeavingView [W-missile-block].
- burst from the encounter after weapons update → retain the displayed flight picture through this cut, then show the first impact picture on the next weapons update [W-missile-hit, W-missile-locked].
- A report naming a free, non-missile or already bursting slot leaves it unchanged [W-missile-stale-hit, W-missile-stale-impact, W-missile-stale-beam].
- checked: oracle (W-missile-block, W-missile-hit, W-missile-locked, W-missile-stale-hit, W-missile-stale-impact, W-missile-stale-beam)

### MissileExhaust
- kind: Timing
- summary: A flying missile drops an exhaust puff every fourth update.
- params: interval 4 updates and 18 puffs.
- switch: Fixed
- each unconsumed flying update → decrement the shot's trail countdown wrapping before it moves. At zero restore 4 and request a puff using the whole centre and previously displayed flight appearance [W-missile-exhaust, W-missile-press].
- a puff request with space → choose the highest free exhaust position, place it at the reported centre plus the imported offset and prime its first picture for the next presentation visit [W-missile-exhaust, W-missile-draw-first].
- all exhaust positions occupied → drop the puff request and keep the restored shot countdown [W-missile-exhaust-full].
- checked: oracle (W-missile-exhaust, W-missile-press, W-missile-draw-first, W-missile-exhaust-full)

### CinematicMissilePictures
- kind: Timing
- summary: The intro draws flying missiles and exhaust.
- params: visible screen interval 0..256 px.
- switch: Fixed
- visible intro weapon presentation → visit the flying-missile and exhaust producer twice, at the caller's two drawing opportunities. Each visit emits live flying pictures in descending ShotSlot order on either flicker phase, with the same vertical view check as BeamVisibility. Each visit draws its exhaust after its missiles [W-missile-draw-first, W-missile-draw-twice].
- an exhaust position is visited → advance its hold and clip by one visit and remove it on completion. Draw surviving puffs in descending position order when both screen coordinates are in the visible interval [W-missile-draw-first, W-missile-draw-twice, W-missile-draw-retire, W-missile-draw-clip].
- an impact remains live → each intro visit omits the impact picture. Existing exhaust remains eligible and ages [W-missile-draw-impact].
- When the intro hides Samus and her shots, skip weapon presentation [W-missile-draw-hidden].
- checked: oracle (W-missile-draw-first, W-missile-draw-twice, W-missile-draw-retire, W-missile-draw-clip, W-missile-draw-impact, W-missile-draw-hidden)
