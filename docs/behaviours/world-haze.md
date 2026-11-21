# Ceres haze

A single-colour haze brightens across the room, then holds the visible gradient.
[Vectors](../../scenarios/vectors/world-haze.toml).

## Header

**Scope.** The single-colour backdrop haze during ordinary running time in a
Ceres room. This covers brightening, settling at full brightness and staying
there while the room remains. It replaces the borrowed backdrop band, fade
completion, and haze display decisions for these opportunities.

**Reads.** The room supplies an ordinary haze opportunity. The haze starts
with a selected colour and its current fade and bands. The room owns haze
placement, creation and departure.

**Leaves to others.** Display preparation combines this backdrop with the
other layers. The haze contributes black as the base backdrop and excludes
the third background from the colour combination. Display
preparation owns how the remaining backgrounds are combined. Scheduling,
physical display transfers, unrelated layer animation, pause and departure
fades remain outside this scope.

## Memory and boundary

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| colour | one of red, green and blue, `HazeColour` | HazeBrightens, display |
| fade | `HazeFade`, Brightening with next peak in 0..16 inclusive, or Steady | HazeBrightens, HazeSettles, HazeHolds |
| bands | `HazeBands`, sixteen ordered band levels in 0..15 inclusive | HazeSettles, HazeHolds, display |

The band index starts at zero at the dim end of the gradient. A brightening
fade's peak is the level to prepare at the bright end at the next opportunity.
Peak 16 marks that the preceding opportunity prepared full brightness. The
current bands and fade are independently restorable. Retain all three
quantities across views and restore them together into fresh state. During a
brightening opportunity the existing levels can be replaced even if their
colour levels happen to match the resulting band values.

An established absence of a room opportunity leaves this state unchanged and
contributes no new display intent. An active room departure is outside the
admitted context. Its caller stops before changing haze state or committing
display effects.

## Rules

### HazeBrightens

- kind: mechanic
- summary: the haze's bright edge grows one level per ordinary opportunity.
- params: none
- switch: fixed
- Brightening with peak below 16 → set each band at index `i` to
  `max(0, peak + i - 15)`, then increase peak by one. Prepare the gradient in
  the selected colour over a black base and exclude the third background
  layer [R-bright-edge, C-early-ramp, C-one-step, C-same-green-levels].
- checked: oracle (R-bright-edge), with constructed lower peaks from the source arithmetic.

### HazeSettles

- kind: mechanic
- summary: the haze finishes its fade one opportunity after preparing full brightness.
- params: none
- switch: fixed
- Brightening with peak 16 → enter Steady with the existing bands and colour.
  Retain the gradient over the black base and exclude the third background
  layer [R-reaches-steady, C-restored-settle].
- checked: oracle (R-reaches-steady, also with held input).

### HazeHolds

- kind: mechanic
- summary: the gradient remains steady during ordinary time in the room.
- params: none
- switch: fixed
- Steady while the room remains → preserve the fade, colour and bands.
  Retain the gradient over the black base and exclude the third background
  layer [R-steady, C-restored-hold].
- checked: oracle (R-steady, completed-cut comparison).

## Order that matters

An ordinary opportunity produces one backdrop decision. The current result
is available to display preparation before the frame's prepared display is
published. The next opportunity continues from the completed fade and
gradient. Settling preserves the gradient that brightening already made
available. A restored result gives the same next state and display intent.
Preparation of bands and retention of existing bands are separately
observable effects. Retention keeps the current backing available to display.
