# Ceres enemy interactions and drawing

C1, enemy-interactions. Covers ordinary power shots at Ceres doors and the shaft's mounted
vent and burst pictures.
Vectors: [enemy-interactions.toml](../../scenarios/vectors/enemy-interactions.toml).

## Ceres-door beam contact

This piece decides which ordinary power shot a single Ceres door spends. It reads W1's current
shot bodies after their movement and appearance update, and B8's current door contact state.
It reports a stable `ShotSlot` to W1's `EndingShot`; it never moves a shot or changes a door.
It replaces the missing Ceres-door shot-contact decision in the enemy framework. The framework
still owns actor selection, invincibility countdown, actor scheduling and other contact kinds.
[B8](boss-ceres-ridley.md) owns door animation, solidity, sounds and shakes.

The caller supplies a door when that actor is due for contact, with its properties as they stand
then, including effects of earlier actors. A retained beginning-of-frame processing membership
is distinct from the solid interaction membership Samus's movement reads. Neither membership
freezes actor properties. Contact takes place after the shots move, before this door changes its
own phase. The caller applies each returned consumption immediately; later actors read the
resulting current shot bodies. No expected future output is an input.

Supported actors are B8's ordinary left/right Ceres doors and shaft door, with a single box.
The domain is total over the quantities below. Other species, multiple contact boxes, missiles,
bombs, other beam families, incoming shots, Samus touch/damage and actor selection rules
are neighboring capabilities, not silently mapped to this contract. A producer that cannot
establish the supported contract reports an unsupported capability at its edge.

## Quantities

Beam contact retains no State. Each call reads the following independently meaningful quantities.
Positions and radii are whole pixels, unsigned 16-bit words; fractions do not affect overlap.

| Quantity | Domain type and meaning | Read by |
| --- | --- | --- |
| actor updates enabled | bool, the frame processes actors | ContactAllowed |
| interactions enabled | bool, contact is not globally suppressed | ContactAllowed |
| Samus time stopped | bool, her stopped-time condition | ContactAllowed |
| enemies stopped in time | bool, the enemies' stopped-time condition | ContactAllowed |
| processing membership | bool, this actor belongs to the actual current processing receipt | ContactAllowed |
| protection remaining | `ProtectionFrames`, unsigned 16-bit count of frames as they stood at the start of this contact opportunity | ContactAllowed |
| door contact | `DoorContact`: Passable, WithoutFigure or Box with centre and radii; visibility alone does not make a door passable | ContactAllowed, BoxOverlap |
| shot count | `ShotCount`, unsigned 16-bit number of owned shots | ContactAllowed |
| shots | five stable `ShotSlot` values, each Free, Impact or Power with centre, radii and consumed bool | FirstPowerShot |
| consumption | optional `ShotSlot`, the shot this door spends | W1 EndingShot; output |

The model combines these named conditions itself. The framework supplies facts, not an opaque
permission. It owns protection countdown, but supplies the protection remaining at the start
of this opportunity: an initial value of one suppresses this contact even if the caller reduces
it to zero later. `WithoutFigure` means no contact figure, not merely hidden drawing. A hidden
tangible shaft wall with a contact figure remains a Box.

## Behaviours

### ContactAllowed
- kind: mechanic.
- summary: a passable door or a paused interaction cannot spend a beam.
- params: none.
- switch: Fixed.
- actor updates disabled, interactions not enabled, either stopped-time condition, absent processing membership, or protection remaining positive → no consumption [C-disabled-actors, C-disabled-interactions, C-Samus-stopped, C-enemies-stopped, C-not-processing, C-protected-one, C-protected-two].
- door Passable or WithoutFigure, or shot count zero → no consumption [C-passable, C-no-figure, C-count-zero].
- actor updates and interactions enabled, neither stopped-time condition, present processing membership, zero protection, door Box and shot count nonzero → resolve its overlap with the power shots [C-inside, C-edge].
- checked: constructed (C-inside, C-passable, C-disabled-actors, C-disabled-interactions, C-Samus-stopped, C-enemies-stopped, C-not-processing, C-protected-one, C-protected-two, C-no-figure, C-count-zero, C-edge).

### BoxOverlap
- kind: quirk.
- summary: a beam must cross the edge of the door's box; touching an edge is insufficient.
- params: none.
- switch: Fixed.
- on each axis → take `d = word(shot_centre - door_centre)`; if its signed 16-bit value is negative replace d by `word(-d)`; overlap holds if `d < shot_radius` or `word(d - shot_radius) < door_radius` [C-inside, C-edge, C-y-edge, C-negative, C-wrap, C-half-world, C-zero-radius].
- both axes overlap → the shot overlaps; either axis fails → it does not [C-y-edge, C-inside].
- checked: constructed (C-inside, C-edge, C-y-edge, C-negative, C-wrap, C-half-world, C-zero-radius).

### FirstPowerShot
- kind: mechanic.
- summary: each door spends the first overlapping power shot, with no special reaction from the door.
- params: none.
- switch: Fixed.
- contact allowed → consider stable ShotSlot values in ascending order, ignoring Free and Impact; return the first overlapping Power, including a Power already consumed [C-first, C-skip-free-impact, C-already-consumed].
- no overlapping Power → no consumption [C-miss].
- a shot returned → the door is unchanged; no damage, animation, sound, shake or other shot response is emitted [C-inside, C-first].
- caller applies consumption → retain the shot body this frame; a later door may report that same slot again; W1 removes it on its next update [C-two-doors].
- checked: constructed (C-inside, C-first, C-skip-free-impact, C-already-consumed, C-miss, C-two-doors).

## Shaft actor and effect pictures

C1 owns the shaft overlay's and mounted vents' shake clocks and the stationary effects the
overlay spawns. B8 supplies each actor's actual participation, location and shown figure.
It also supplies the vent's projected offset and overlay's burst choice and centre. The room quake supplies the request to shake the current
actors and the displacement for the effects' picture separately. AF3 supplies each effect's
static clip and the actors' art. Exact rendering places sprite pieces, including clipping
individual pieces, and publishes the prepared picture.

| Quantity | Domain type and meaning | Read by |
| --- | --- | --- |
| actor clock | `ActorClock`, per stable shaft actor, counts participating actor turns modulo 65536 and drawn turns of shake remaining (unsigned 16-bit) | ActorShake |
| ran this turn | bool from the actual B8 participation result, including turns with no shown figure | ShaftActorDraw, ActorShake |
| shown figure | optional figure identity supplied by B8 for this turn | ShaftActorDraw, PictureWindow |
| centre, horizontal radius and projected offset | `RoomPoint`, whole-pixel unsigned centre, overlay extent in whole pixels, and the vent's signed displacement from B8 | ShaftActorDraw, PictureWindow |
| draw visit | bool, whether to spend drawing shake before resolving the picture origin | ActorShake, PictureWindow |
| refresh | bool from the post-draw room quake, for this turn's participating actors | ActorShake |
| effect slots | `EffectSlots`, 18 stable places numbered 0 through 17, each empty or occupied | EffectSlots, EffectClip, ShaftPictureOrder |
| burst | kind (Smoke or SmallExplosion), with whole-pixel room centre from B8 | EffectSlots |
| effect clip | AF3's nonempty sequence of figures, each with a positive whole-turn duration, ending after its last figure | EffectClip |
| effect phase | waiting for first figure, or a figure position in the clip and its positive duration remaining | EffectClip, ShaftPictureOrder |
| pre-scroll view | `RoomPoint`, whole-pixel top left of the view before scrolling | ShaftActorDraw, EffectClip |
| draw view and effect shake | whole-pixel top left at draw and the room quake's signed whole-pixel displacement at draw | PictureWindow |
| figure request | visible figure identity and screen origin, in drawing order | exact sprite writer |

An actor clock and the occupied slots are retained State. A continuation can restore the
two actor counts for each stable actor and the exact kind, centre and phase of each occupied
slot. An empty slot has no retained effect. New rooms replace these clocks and slots at
placement. Unmodelled actor families and effect kinds require their own capabilities.

### ShaftActorDraw
- kind: mechanic.
- summary: an active visible overlay can leave the picture while it keeps rumbling, while a drawing visit for a mounted puff follows its turn.
- params: none.
- switch: Fixed.
- actor did not participate, or B8 supplies no shown figure → no draw visit [C-vent-clock-skip, C-vent-hidden-refresh].
- participating vent with a shown figure → visit its drawing. The projected origin can still miss the picture window [C-vent-clipped-spends, R-vent-first-left].
- participating overlay with a shown figure → visit its drawing only when the unprojected centre is within `−radius through 256 + radius` horizontally and `−8 through 248` vertically, relative to the pre-scroll view, endpoints included. Use wrapped signed 16-bit differences. It can continue participating, with no drawing visit, outside that window [C-overlay-left-edge, C-overlay-beyond, C-overlay-right-edge, C-overlay-right-beyond, C-overlay-top-edge, C-overlay-top-beyond, C-overlay-lower-edge, C-overlay-lower-beyond, R-overlay-leaves].
- checked: oracle and constructed.

### ActorShake
- kind: mechanic.
- summary: the overlay and its vents move left and right while the room refreshes their shake.
- params: none.
- switch: Fixed.
- place an actor → both counts start at zero [C-vent-clock-placement].
- each participating turn → add one to the actor count, wrapping after 65535. A turn without participation holds the count [C-vent-clock-wrap, C-vent-clock-skip, R-vent-first-left].
- a draw visit with shake remaining positive → use the updated count. Move the projected origin 1 px right when count modulo 4 is 0 or 1, or 1 px left when it is 2 or 3. Spend one shake turn even when this moved origin misses the picture window [C-vent-clock-wrap, C-vent-clipped-spends, R-vent-first-left, R-overlay-first-right].
- no draw visit or zero shake remaining → no shake is spent. A drawn actor with zero shake remaining has no horizontal shift [C-vent-clock-skip, C-vent-hidden-refresh, C-vent-unshaken].
- after drawing, a refresh on a participating turn → replace shake remaining with two, including when there was no draw visit. A turn without refresh keeps the remainder from drawing [C-vent-hidden-refresh, C-vent-clock-skip, R-vent-first-left].
- checked: oracle and constructed.

### EffectSlots
- kind: mechanic.
- summary: each burst takes the highest empty place in the room's pool.
- params: none.
- switch: Fixed.
- place the room's effect pool → all 18 slots start empty [C-effects-empty].
- burst arrives with an empty slot → set the highest empty slot to the requested kind and centre, waiting for its first figure. Accept arrivals in their given order, before any existing effect advances [C-effects-reuse, C-effects-full, R-effect-first].
- all slots occupied at arrival → drop the new burst, even if an older slot empties later on that turn [C-effects-full].
- a held turn with no new work → keep all slots unchanged [C-effects-hold, R-effect-hold].
- checked: oracle and constructed.

### EffectClip
- kind: mechanic.
- summary: a stationary burst plays once and disappears when it finishes or leaves the view.
- params: none. Clip figures and durations are content.
- switch: Fixed.
- an advancing turn visits each occupied slot, including one filled this turn. Compute the centre relative to the lifetime view in whole pixels with 16-bit wrapping. Either axis outside 0 through 255 inclusive → empty the slot. This applies even to a burst that just arrived and could otherwise have a visible picture [C-effect-life-window, C-effects-reuse, R-effect-cut-off].
- inside that window while waiting for the first figure → select the first figure and its full duration [C-effect-first, R-effect-first].
- inside while showing a figure with more than one turn remaining → spend one turn, keeping the figure. With one remaining → select the next figure and its full duration, or empty the slot if the last figure has finished [C-effect-clip, R-effect-change, R-effect-finishes].
- the effect runner disabled → retain the slot and its phase. A waiting effect emits no figure, and a previously selected figure remains available to drawing [C-effects-hold, C-effect-waiting].
- checked: oracle and constructed.

### PictureWindow
- kind: mechanic.
- summary: a puff or burst near the edge can still show some sprite pieces.
- params: none.
- switch: Fixed.
- a shown vent → subtract the draw view from the mount and add B8's projected offset and its own horizontal shift. A showing effect → subtract the draw view from its centre and add the quake's effect displacement [C-vent-unshaken, R-vent-first-left, C-effect-picture-window, R-effect-first].
- compute these screen origins with 16-bit wrapping and read both axes as signed. Both axes in the range −128 through 383 inclusive → supply the figure and origin to exact drawing. Either axis outside → emit no figure. The draw window leaves lifetime and the spent vent shake as they stood [C-vent-clipped-spends, C-effect-picture-window, C-effect-picture-y-window, R-effect-cut-off].
- a visited overlay → subtract the draw view from its centre and add its horizontal shift. Supply its figure and origin to exact drawing using the overlay's ordinary sprite placement [C-overlay-left-edge, R-overlay-first-right].
- checked: oracle and constructed.

### ShaftPictureOrder
- kind: mechanic.
- summary: the flashes and smoke are drawn ahead of the overlay, and the vents keep their room order.
- params: none.
- switch: Fixed.
- on the shaft route, draw visible stationary effects first among the shaft pictures, in descending effect slot order. They precede both the overlay figure and Samus's normal picture [C-effects-order, R-effect-order].
- draw the overlay before the mounted vents. Draw mounted vents in ascending placement slot order, omitting each with no figure request. The shaft's mounted vents follow Samus's normal picture [C-effects-order, R-vent-first-left].
- no figure requests → this owner adds nothing to the picture [C-effect-waiting, C-effects-empty].
- checked: oracle and constructed.

### Order that matters

A running shaft turn uses the pre-scroll view for participation and effect lifetime. The
actors' turns can produce a burst. Accept it before advancing the effect pool. At drawing,
use the scrolled view, the newly advanced actor counts and effect figures, and the quake
offset for that draw. The room quake can refresh actors after drawing. A held turn
that only publishes a pending picture takes no actor, effect, drawing or refresh turns.
