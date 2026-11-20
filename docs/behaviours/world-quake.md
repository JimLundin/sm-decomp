# Quiet rooms and quakes

A room stays steady when its quake duration is zero, even as the room keeps
counting its quake opportunities. Vectors:
[world-quake.toml](../../scenarios/vectors/world-quake.toml).

## Header

**Scope.** The ordinary update in a room with no quake time remaining. This is
the quiet part of the room's quake behaviour. It introduces the quake owner for
this scope. There is no existing production quake owner to replace.

**Reads.** The scheduler supplies the current room's ordinary quake opportunity.
The quake owner supplies the room's current duration and carried phase.

**Leaves to others.** The room's events decide when to start a quake. Active
quakes, their duration and expiry rules, and active actor shaking are outside
this scope. Drawing and Camera own composition and camera placement.
Actors own their own motion and ongoing effects. The room quake and the player's
saved play time advance independently.

## Memory and boundary

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| phase | `QuakePhase`, 0..65535 elapsed quake opportunities around a cycle | QuietQuakePhase, output |
| remaining | `QuakeDuration`, 0..65535 frames of quake time remaining | QuietQuakePhase, Drawing's quake displacement query, output |

Both quantities are independent. Together they are complete State for the
bounded quake owner. Set and read both, carry both and restore both into fresh
State. The phase is a wrapping count of room updates, including the ones during
which the room stays steady. The phase earns no position change by itself.
Remaining zero means the quake contributes zero displacement to Drawing's
positions regardless of phase.

The supported operation is one ordinary quake opportunity with current
remaining zero. A requested update with nonzero remaining or an unestablished
ordinary opportunity is Unsupported before changing either quantity or
publishing quake effects. The quiet rule is total over all phases at zero
remaining. A scheduler-established absence of an opportunity leaves both
quantities alone.

## Rules

### QuietQuakePhase

- kind: mechanic
- summary: the quake phase advances while the room stays steady.
- params: none
- switch: fixed
- Ordinary opportunity with remaining zero → set phase to
  `(phase + 1) mod 65536`, keep remaining zero, and contribute no quake
  displacement or actor shake request [R-quiet-frame, C-other-phase,
  C-mid-cycle, C-wrap, C-after-wrap, C-restored-next].
- checked: oracle (R-quiet-frame), with independent constructed starts and wrap.

## Order that matters

The ordinary opportunity advances the phase once before the completed room
State for that frame is observed. Drawing's quiet displacement uses the
current remaining duration independently of the phase update. A continuation
uses the full previous result. A fresh restoration of that result gives the
same next phase, remaining duration and quiet result. A steady picture alone
cannot distinguish a missed or duplicated phase advance.
