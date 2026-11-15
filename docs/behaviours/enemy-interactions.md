# Ceres-door beam contact: behaviour spec

C1, enemy-interactions, first piece: an ordinary uncharged power beam meeting a Ceres door.
Vectors: [enemy-interactions.toml](../../scenarios/vectors/enemy-interactions.toml).

## Header

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

There is no retained State. Each call reads the following independently meaningful quantities.
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
