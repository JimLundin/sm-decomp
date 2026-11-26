# Samus's turns in the intro fight

The recorded controls decide the handoff and supply pad samples. Movement and
appearance decide the accepted pose and clip. This contract determines what the
room publishes and what its visible actor continues to do after the controls end.

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| Cached body radius | Pair of whole-pixel radii, supplied by the body content | Room replay, output |
| Pose history | Last two full poses recorded by a resolved choice or explicit placement, including facing and action | Actor, appearance, output |
| Resolved choice | Optional result from the actor's chooser | RememberIntroPose |
| Previous centre | Pair of signed 16.16 pixel coordinates | Room replay, output |
| Control authority | Whether the turn entered under recorded control or already locked | Room, weapons, appearance |

Cells: quantities, units and readers.

### SampleIntroBody

- kind: mechanic.
- summary: show the body's size sampled before this turn's input and pose changes.
- params: none. The body's size is imported content.
- switch: Fixed.
- A visible turn enters under recorded control → sample the radius for the pose at
  entry. Keep that sample through the rest of the turn [R-recoil-to-fall, R-fall-reselected].
- A visible turn enters already locked → carry the sampled radius [R-locked-stands, C-locked-body].
- checked: oracle (R cases); constructed (C case).

### RememberIntroPose

- kind: mechanic.
- summary: record resolved pose choices and explicit placements.
- params: none.
- switch: Fixed.
- The pose chooser resolves a pose → remember the result and shift the history by one. This includes a choice of the pose she already
  has [R-fall-reselected].
- Her pose carries on without a choice → keep both remembered poses [R-run-carries].
- The handoff places her standing left → record the placement in the pose history.
  The standing clip starts only when that pose differs from the last remembered
  pose. A matching pose keeps its playhead into the ordinary animation turn
  [R-lock-handoff, C-lock-from-recoil].
- checked: oracle (R cases); constructed (C case).

### FinishIntroControlTurn

- kind: mechanic.
- summary: continue the visible actor and her shots while the battle closes.
- params: none.
- switch: Fixed.
- The standing-left handoff arrives during a controlled turn → apply the placement
  before the last ordinary choice from the pad, cannon controls, movement and animation.
  Use the sample the handoff supplies on this turn [R-lock-handoff].
- A visible turn begins locked → advance existing shots, carry the cannon cooldown
  and skip item controls. Run ordinary movement, animation, reactions and palette.
  The recorded pad supplies no new choice. A pending choice from earlier is available to
  the resolver until it is replaced. The movie's pad during these turns is neutral
  [R-locked-stands].
- checked: oracle (R cases).

## Order that matters

Input and movement contact can supply pose choices. Room placement and the early
handoff can remember a pose before the ordinary choice resolves. A cut before the
actor turn leaves the sampled body and pose history at that point's values. The
last visible room turn runs before the intro hides her and the shots. The previous
centre stays at the room's placement through the fight.
