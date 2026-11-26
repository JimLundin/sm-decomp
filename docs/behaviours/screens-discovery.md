# The baby's discovery

In the second played scene of the intro, Samus approaches an egg. It rocks, cracks and scatters
shell fragments. The baby rises through the shell, sheds droplets and floats in place. Then it
follows Samus, passes in front of her and behind her, and cries twice more. The spent shell
cues page three. Samus and the baby are still visible in the return dissolve.

## Scope

This F2 companion owns the placed egg and baby, their pictures and movement, the shell
fragments and droplets, the hatch and cry requests, the relative sprite order, and the
page-three cue. The engine has no owner for these actors in the second demo.

[The intro](screens-intro.md) sends `StartDiscovery` with the room setup and
`FinishDiscovery` on the frame the return dissolve's remaining count reaches zero. Discovery
returns `ReadyForPageThree` once. The intro prepares page three on its next finished frame.
The demo supplies Samus's whole-pixel centre after her turn in the room. It owns her controls,
movement, appearance and projectiles. E1 draws the screen pictures. The intro owns the
dissolve, palette values, text and caret. Audio U1 handles playing the ordered sound offers.
T1 says when the encounter advances a finished frame.

A position is a signed pixel coordinate with fractions in 65,536ths of a pixel. X increases
rightward and Y increases downward. Whole-pixel comparisons round toward the lower pixel.
Baby velocity uses signed eighths of a pixel per finished frame. Fractions appear in draw
positions only when they carry into another whole pixel.

## State and outputs

A caller can inspect or restore any valid snapshot. Positions in a restored encounter use the
same room coordinates as Samus. The dormant state has no encounter drawings. A newly placed
egg and baby have no picture until their first finished frame.

| Quantity | Type and units | Read by |
| --- | --- | --- |
| Egg | `Egg`: absent, waiting, hatching with an age in finished frames, or spent | `EggHatches`, `ShellScatters`, `PageThreeApproaches`; output |
| Baby's place | `Position` in room pixels | `BabyEmerges`, `BabyFollows`, `SlimeFalls`; output |
| Baby's motion | `BabyMotion`: absent, inside, rising with vertical speed and whether the baby has shed its droplets, hovering with frames left, or following with both speeds and a cry age | `BabyEmerges`, `BabyFollows`, `SlimeFalls` |
| Baby's pulse | `PulseAge`: finished frames since the start of the current drawing cycle, or no picture yet | `BabyPulses`; output |
| Pieces in flight | Up to six shell fragments and four droplets. Each has a `Position`, a content identity, and the number of finished frames of flight already applied | `ShellScatters`, `SlimeFalls`; output |
| Splashes | Each has the stopped droplet's `Position` and the number of finished frames since impact | `SlimeFalls`; output |
| Relative sprite order | `DiscoveryOrder`: `SceneFirst`, `SamusFirst` or `Hidden` | Production schedules the room's turn from this at frame start and publishes the draw order at frame end |
| Sound and cue | Ordered U1 offers, each with a sound identity and maximum waiting sounds. Optional `ReadyForPageThree` | Output to audio and the intro |

## Discovery content

The importer provides its own typed `DiscoveryScene`. A placed sprite supplies a drawing
reference and a palette reference. The sprite sheet and colours are installed by the intro
with the room. The drawings refer to those resources. The rest of the model reads these
content fields.

- `Home` gives the egg and baby's shared starting anchor. The shipped anchor is (112, 155).
- `ShellPictures` gives a closed picture, a finite hatch animation of pictures with positive
  finished-frame holds, and a spent picture. It marks the displayed hatch frame on which
  the shell scatters and the baby comes free. Markers name ages relative to the first hatch
  frame, which is age zero. Use the spent picture at the end of the hatch animation.
- `BabyPictures` is a nonempty repeating sequence of pictures with positive holds, from
  round through stretched, wide and stretched in the shipped scene.
- `Fragments` has six placed pictures with anchors and individual horizontal drifts.
  `Droplets` has four pictures with individual horizontal drifts. All drifts are signed
  pixels per finished frame. A fragment or droplet carries a finite series of signed vertical
  displacements, one in pixels for each frame of flight. The displacements are applied
  directly to position. The track for each fragment ends at the latest frame on which it can
  still be drawn. A droplet uses its last descent speed until impact if it reaches the end
  of its series.
- `SplashPictures` gives a nonempty sequence of impact pictures with positive holds.
  `HatchSound` and `CrySound` name the sound resources with a queue allowance of six each.

The importer checks the references, that both hatch markers fall within the finite
animation, that the summed holds fit in a frame count of at most 65,535, that the movement tracks are
nonempty, and that the final droplet displacement is positive. Positions and displacements
are checked to keep the supported scene inside the signed room coordinate range. The
shipped demo's movement yields one droplet release. A rising snapshot retains whether
the release has already happened.

## Encounter rules

### EggHatches

- kind: mechanic.
- summary: the egg waits for Samus, then rocks and opens.
- params: Samus's approach threshold is x 169 px.
- switch: Fixed.
- Absent, on `StartDiscovery` → place egg and baby at `Home`, set the egg waiting and the baby inside,
  clear the encounter's motion, and set `SceneFirst`. Pictures begin on the first finished
  frame after placement, which may be the completion of the same game frame [R-placement, C-start].
- Waiting, Samus's x at least 169 → show the closed picture [R-first-draw, C-boundary].
- Waiting, Samus's x below 169 → enter hatching at age zero and show its first picture [R-trigger, C-trigger].
- Hatching, a further finished frame before the animation's end → advance one age and show the
  picture whose hold includes it [C-next-picture].
- checked: oracle; constructed.

### ShellScatters

- kind: mechanic.
- summary: the shell breaks, and fragments arc away.
- params: floor y 168 px.
- switch: Fixed.
- On the scatter marker → place the fragment cohort and offer `HatchSound` once. Apply the first
  flight step on this frame [R-break, C-burst].
- Fragment in flight with a remaining displacement → add its drift to X and add the next vertical
  displacement to its Y. Keep the picture when whole-pixel Y is less than 168, otherwise
  remove it before draw [R-fragment-motion, R-fragment-floor, C-fragment-floor].
- Fragment with its track exhausted → remove it before draw [R-fragment-exit, C-fragment-exit].
- checked: oracle; constructed.

### BabyEmerges

- kind: mechanic.
- summary: the baby rises above Samus then holds still.
- params: the rise aims 32 px above Samus's centre. Acceleration is 1/8 px per frame per
  frame, the speed limit is 17/8 px per frame in either direction, and the hover lasts 128 frames.
- switch: Fixed.
- Baby inside, on the release marker → start rising with vertical speed zero and droplets still to shed.
  The first movement is on the following finished frame [R-release, R-rise, C-burst].
- Rising with whole-pixel y no greater than the target (`baby_y <= samus_y - 32`)
  → increase vertical speed toward +17 eighths. Otherwise decrease it toward −17 eighths.
  Move by the new speed [R-rise, R-rise-limit, R-turn, C-rise-equal].
- After a rising step with speed nonnegative → hold the new place for 128 frames
  [R-hover-start, C-hover-start].
- Hovering → decrease the hold by one. On the frame it reaches zero, become following with
  both speeds zero and cry age zero, at the same place [R-hover, R-follow-ready, C-hover-end].
- checked: oracle; constructed.

### SlimeFalls

- kind: mechanic.
- summary: the rising baby sheds droplets that burst against the floor.
- params: rise trigger y 145 px and floor y 168 px.
- switch: Fixed.
- Rising, droplets still to shed and whole-pixel y 145 before the rise step → put the droplets at
  the baby's whole-pixel anchor, mark them shed, offer `CrySound`, and apply their first flight step on the same
  frame [R-slime, C-slime].
- Rising with droplets already shed, including a restored rise that still rounds to y 145
  → keep the existing droplets on their normal path and issue no new cry [C-slime-repeat].
- Droplet in flight → add its drift and vertical displacement. While `y < 168` it
  keeps flying and counts the applied step. At `y >= 168` it enters splash at the
  resulting position, drops its flight age and shows the first splash picture at age zero
  [R-slime-flight, R-impact, C-impact].
- Splash at its stopped place → show its successive pictures for their imported holds.
  Remove it before draw on the frame its final hold expires [R-splash, R-splash-end, C-splash-end].
- checked: oracle; constructed.

### BabyPulses

- kind: mechanic.
- summary: the baby's shape cycles through its pulse, even behind the egg.
- params: the imported baby picture sequence and its holds.
- switch: Fixed.
- Baby placed and taking its first finished frame → show the first picture at pulse age zero
  [R-first-draw, C-first-pulse].
- Baby still present on a later finished frame → advance pulse age by one modulo the cycle
  length and show the picture whose hold includes that age [R-pulse, C-pulse-advance, C-pulse-loop].
- checked: oracle; constructed.

### BabyFollows

- kind: mechanic.
- summary: the baby chases Samus and cries twice.
- params: target `(samus_x, samus_y - 8)`; acceleration 1/8 px per frame per frame per axis.
  Speed limits ±20/8 px per frame horizontally and ±17/8 px per frame vertically.
- switch: Fixed.
- Following, on each axis where target is at or beyond the baby's whole-pixel place
  numerically → increase the speed one eighth toward its positive limit and apply the
  new speed [R-follow, R-follow-cap, C-follow-equal].
- Following, on each axis where the target is less → decrease the speed one eighth
  toward its negative limit and apply the new speed [R-order-left, C-follow-cap].
- Following with negative new horizontal speed → `SceneFirst` [R-order-left, C-follow-cap].
- Following with zero or positive new horizontal speed → `SamusFirst`
  [R-follow, R-order-zero, C-follow-equal].
- Following, cry age below 128 → add one to the age. Offer `CrySound` when that age reaches
  64 or 128 [R-cry-one, R-cry-two, C-cry].
- Following at age 128 → keep the age at 128 and issue no cry [R-quiet, C-quiet].
- checked: oracle; constructed.

### PageThreeApproaches

- kind: mechanic.
- summary: the hatch calls for the next page, then the room's figures disappear into the dissolve.
- params: the imported hatch duration and the intro's finish signal.
- switch: Fixed.
- Hatch animation's last hold ends → enter spent, show the spent picture and send
  `ReadyForPageThree` on that cut. The baby and room retain their normal turns [R-page-cue, C-page-cue].
- Spent before `FinishDiscovery` → keep showing the spent picture without sending the cue
  again. The baby continues its pulse and movement [R-return-start, R-before-hide, C-spent].
- Spent on `FinishDiscovery` → remove the encounter's drawings and publish `Hidden`. This is
  after the room's last turn. Following motion, its cries and the pulse do not advance on
  that cut [R-hide, C-hide].
- Absent after removal → remain hidden with no encounter effects [R-after-hide, C-absent].
- Ordinary unfinished cut → preserve the snapshot and produce no encounter events [R-return-start, C-lag].
- checked: oracle; constructed.

## Order that matters

At setup the room can be visible to its stepping owner while the placement waits for its
first drawing. On finished frames, Samus takes the room turn based on the visibility entering
that turn. Discovery reads her resulting centre. When the hatch cues page three, its one cue
comes after that same room turn. On the frame the return dissolve reaches zero, the demo receives
its final turn with its own input and pose rules. The encounter disappears before the draw
and the next frame has no room turn.

In back-to-front terms, the airborne and splashing droplets are behind the shell fragments,
which are behind the baby, which is behind the shell. Within a cohort, the first listed
picture has foreground precedence. The intro's caret, when drawn, has precedence over
discovery within the cinematic group. `SceneFirst` puts this group ahead of Samus and her
projectiles in sprite priority. `SamusFirst` reverses the groups. `Hidden` omits the
encounter, Samus and her projectiles, and the intro can still draw its caret. Each placed
room picture subtracts the room's whole-pixel scroll from its rounded anchor. E1 applies
clipping and the picture's own tile priorities.

[Recorded and constructed vectors](../../scenarios/vectors/screens-discovery.toml)
