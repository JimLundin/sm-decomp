# Mother Brain in the Zebes flashback

## Header

**Scope.** The encounter starts on the same scene frame as the page 1 press in
[the intro](screens-intro.md). Mother Brain breathes while missiles cross the room and rings
appear. A ring can make Samus recoil. After enough hits, bursts and a vertical shake lead to
the cue for page 2. The boss and bursts disappear as the intro dissolves back to text.
[Encounter vectors](../../scenarios/vectors/screens-flashback.toml).
This introduces the encounter owner. The engine has no encounter to replace.

**Reads.** The intro supplies the start and inherited room background Y, the even or odd beat
of the current intro work frame, and the return dissolve reaching zero on this frame. Room
play supplies Samus's whole X after movement and the live ordinary missile candidates after
weapons step, in the weapons owner's precedence order. Each candidate has a shot identity
and whole X. The caller can also pass other shot kinds, which are ignored. Samus's carried whole X remains available if a ring survives her hide. The platform's
work frame can extend across more than one video cut.

**Reports.** A missile impact request for a shot identity, a rightward ring knockback with
two timer values, an optional palette replacement, the background Y, foreground pictures in
draw order, a cue for page 2 and a request to hide Samus and her shots. The impact request
asks weapons to turn the flying missile into its impact at the same place. Samus's owners
apply the ring result after movement and before drawing. Energy and ammo stay at their
pre-contact values.

**Leaves to others.** The intro owns the page, both dissolves and the scene's flashing lights.
The demo owns the controls, the room owns collision content, the Samus owners own setup,
motion, equipment and appearance, and weapons owns missile flight, impact animation and
any associated sound. Video owners compose and encode the requested pictures, background
offset and palette writes. The encounter produces no sound requests of its own.

**Notation.** Room and picture coordinates use pixels, positive right and down. A ring's
position has 16 fractional bits per axis. A whole coordinate is the floor of that position.
A beat is one encounter step in one scene work frame. The encounter also steps during the
outbound dissolve and during the return. The imported scene content supplies the picture
clips and normal boss palette. Picture numbers below count images of each clip starting at 1.
Vectors use a zero-based animation step and count holds in beats.

## Memory

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| Start pending | `PendingStart`, whether startup is ready for its first beat. Inherited Y is stored as `RoomBackgroundY` | EncounterStarts, EncounterBeats |
| Mother Brain | optional `Brain` with `Endures` and prior hits, `Shaking` and elapsed shake beats, or `Returning` | BrainEndures, BrainFlashes, RingsSweep, BackgroundBobs, ReturnToPage |
| Flash beats remaining | `FlashBeats`, 0 to 8 | BrainFlashes |
| Mother Brain animation | `ClipStep`, with a hold in beats | BrainBreathes, ReturnToPage, output |
| Next pair | optional `RingPair`, first or second, with a hold in beats | RingsAppear, FightPictures |
| Each ring | `RingPath`, two fixed-point pixel coordinates, `ClipStep` and hold in beats, draw rank | RingsAppear, RingsSweep, FightPictures, output |
| Each burst | `BurstPlace` and size, a position in whole pixels, a delay or a `ClipStep` and hold in beats, draw rank | BurstsPulse, FightPictures, output |
| Room background Y | `RoomBackgroundY`, signed 16-bit whole pixels | BackgroundBobs, output |

A caller can set, carry and restore this State. A live hold is positive. The pending pair reserves the foremost rank.
An enduring boss has accumulated fewer hits than the hit threshold. A shaking boss has elapsed fewer beats
than its hold. The four ring paths and eight burst places have at most one instance each.
Their draw ranks are unique among live pictures and those waiting to appear. Before the
encounter the boss and next pair are absent, and the ring and burst collections are empty.
The start carries its background Y across a pre-dispatch cut. Between scene work frames
State stays put. A frame that
resumes after an intervening video cut keeps whatever encounter step already ran.

## The encounter's clock

### EncounterBeats

- kind: timing
- summary: the encounter advances once per scene work frame, including a frame that spans
  multiple video cuts.
- params: none
- switch: fixed
- The scene reaches the encounter phase of a work frame for the first time → advance
  the encounter once, including its first beat if the start is pending. A video cut before
  the work frame finishes can already observe this State [R-start, R-long-after, R-ring-ready].
- A cut interrupts the work before the encounter, or the encounter already advanced
  in the resuming frame → retain the encounter State and emit no new encounter events.
  The start may already have been accepted on an earlier phase of this work frame
  [R-long-finishes, R-long-before, R-setup-long-before].
- checked: oracle (R-start-pending, R-start, R-long-after, R-ring-ready, R-long-finishes,
  R-long-before, R-setup-long-before).

## The boss and the missiles

### EncounterStarts

- kind: mechanic
- summary: Mother Brain and the ring schedule start as page 1 gives way to the room.
- params: Mother Brain's picture origin (56, 111) px. Initial ring pair wait of 74 beats.
- switch: fixed
- New encounter start during scene setup → remember the pending start and inherited
  background Y. Until the encounter advances, neither the boss nor the ring schedule has
  taken its first step [R-start-pending, C-start-resume].
- First encounter beat with the start pending → consume the pending start, show the first
  breathing image for its full hold, schedule the first pair in 74 more beats and check
  for contact with zero prior hits and zero flash. Keep the inherited background Y
  [R-start, C-start-impact, C-start-resume].
- checked: oracle (R-start-pending, R-start).

### BrainBreathes

- kind: output
- summary: Mother Brain cycles three pictures while she is present.
- params: the repeating breath pictures are 1, 2, 3, 2, held 16 beats each.
- switch: fixed
- Mother Brain stays present for another beat, and the cue did not restart the clip →
  use up one beat of the hold. On expiry show the next picture for its full hold, or
  wrap to the first picture at the end of the clip [R-breathe-next, R-breathe-wrap,
  R-return-first].
- checked: oracle (R-breathe-next, R-breathe-wrap, R-return-first).

### BrainEndures

- kind: mechanic
- summary: the first missile in weapon precedence hits when its centre crosses the contact line.
- params: `contact.limit`, a whole room X in pixels from 0 to 255, reference 84.
  `contact.hits`, the number of impacts needed to start the bursts, from 1 to 255, reference 4.
- switch: parameter
- Mother Brain endures, with no ordinary missile or with her first candidate at or to the
  right of the contact line → keep her hit count. Check only the first ordinary missile
  this beat [R-contact-edge, C-screened-shot, C-other-shot].
- Mother Brain endures, her first candidate is strictly left of the contact line, and at
  least one further hit is needed → request that shot's impact, count one hit and set the
  flash to 8 beats [R-first-contact, R-second-contact, R-third-contact, A-contact-line].
- Mother Brain endures and this impact reaches the hit threshold → request the impact,
  set the flash to 8 beats, enter `Shaking` at zero elapsed beats and begin the delayed
  bursts. The background starts to move on the next beat [R-last-contact, A-one-impact].
- Mother Brain is shaking, returning or absent → leave all flying missiles to weapons
  [C-missiles-after-defeat].
- checked: oracle (R-contact-edge, R-first-contact, R-second-contact, R-third-contact,
  R-last-contact).

### BrainFlashes

- kind: output
- summary: the boss and the rings share the palette that flashes after a missile hit.
- params: 8 flash beats per hit. The unlit replacement is the imported full normal palette.
- switch: fixed
- An enduring or shaking boss has an even, positive flash count at the start of the beat →
  replace all 16 colours of image palette 7 with white and spend one beat of flash
  [R-flash-white].
- An enduring or shaking boss has an odd flash count at the start of the beat →
  write the normal palette to all 16 colours of image palette 7 and spend one beat of
  flash [R-flash-normal, R-flash-ends, C-flash-and-impact].
- Palette writes run on positive flash counts during the enduring and shaking phases
  [R-first-contact, R-return-first, C-short-return].
- checked: oracle (R-flash-white, R-flash-normal, R-flash-ends, R-first-contact,
  R-return-first).

## The rings

### RingsAppear

- kind: mechanic
- summary: two pairs of rings form at set places, then drift across the room.
- params: the second pair appears 128 beats after the first. The starts and paths are
  scene content. Each initial picture holds for 10 beats.
- switch: fixed
- A pair is due in more than one beat → spend one beat of its hold [R-before-pair].
- First pair due → show the two first rings at their origins and set the second
  pair's hold to 128 [R-first-pair].
- Second pair due → show its two rings and clear the schedule [R-second-pair,
  C-rings-after-hide].
- A ring is forming → show images 1, 2, 3 for 10 beats each. At the next picture change
  show image 2 and make it ready to move. Movement starts one beat later [R-ring-ready,
  R-ring-moves].
- checked: oracle (R-before-pair, R-first-pair, R-second-pair, R-ring-ready, R-ring-moves).

Ring starts and travel:

Cells: path, spawn X and Y in pixels, X and Y travel in pixels per beat, can hit Samus.

| Path | X | Y | Travel X | Travel Y | Can hit Samus |
| --- | ---: | ---: | ---: | ---: | --- |
| First toward | 112 | 72 | +0.5 | +0.5 | Yes |
| First across | 192 | 56 | -0.5 | +0.5 | No |
| Second toward | 128 | 48 | +0.5 | +0.5 | No |
| Second across | 232 | 80 | -0.5 | +0.5 | No |

### RingsSweep

- kind: mechanic
- summary: the moving rings slide diagonally, with one ring able to hit Samus.
- params: the ring that can hit reaches 8 px right from its centre and tests against
  5 px left of Samus's whole X. Both hit timers are 11 beats. The other rings stay
  within whole room Y 16 inclusive to 208 exclusive. The moving clip repeats images
  2, 1, 2, 3, in 10-beat holds.
- switch: fixed
- A moving ring → add its travel to both precise coordinates. Read the whole coordinates
  of the result for the contact or exit check. Keep the clip moving for a surviving ring
  [R-ring-moves, R-ring-before-hit, C-pass-no-contact].
- The first toward ring reaches or passes the horizontal contact line, regardless of Y
  or Samus's earlier hit timers → remove it and offer 11 beats each of invincibility and
  rightward knockback. A surviving ring uses Samus's last whole X after she is hidden
  [R-ring-contact, C-ring-contact-during-defeat, C-rings-after-hide].
- A moving ring that misses, and Mother Brain was already `Shaking` as the beat began →
  remove the ring. A different ring also disappears if its whole Y leaves its permitted
  range [R-rings-clear, C-ring-exits].
- checked: oracle (R-ring-moves, R-ring-before-hit, R-ring-contact, R-rings-clear).

## The bursts and the return

### BurstsPulse

- kind: output
- summary: flashes of different sizes start at different times around the boss.
- params: eight burst places and delays are scene content. Each small picture holds
  3 beats and each large picture holds 6 beats. Each clip has six pictures, then
  16 blank beats, and repeats.
- switch: fixed
- A defeat begins → reserve all the burst places with their initial delays and no
  pictures yet [R-last-contact, A-one-impact].
- A burst waiting for its first picture reaches its delay → show its first picture
  for a full hold. Between changes spend one beat of the current hold. When the hold
  expires, show the next picture, begin the blank interval, or begin the clip again
  after the blank interval [R-first-bursts, R-later-burst, R-burst-blank, R-burst-repeat].
- The return dissolve reaches zero → remove all bursts before drawing
  [R-hide, C-hide-with-ring].
- checked: oracle (R-last-contact, R-first-bursts, R-later-burst, R-burst-blank,
  R-burst-repeat, R-hide).

Burst starts:

Cells: place, size, X and Y offset from Mother Brain in pixels, beats after the decisive impact.

| Place | Size | X | Y | Delay |
| --- | --- | ---: | ---: | ---: |
| Near | Small | +16 | 0 | 1 |
| Low | Small | -16 | +4 | 8 |
| High | Small | -16 | -8 | 16 |
| Centre | Large | 0 | 0 | 1 |
| Upper right | Large | +16 | -16 | 16 |
| Lower left | Large | -16 | +8 | 32 |
| Upper left | Large | -8 | -16 | 48 |
| Lower right | Large | +8 | +8 | 64 |

### BackgroundBobs

- kind: output
- summary: the room background moves up or down on alternate beats while the boss explodes
  and while the picture dissolves back to text.
- params: `shake_travel`, vertical travel in whole pixels per beat, from 0 to 16, reference 4.
- switch: parameter
- Mother Brain was already shaking or returning as the beat began → add the travel
  on even beats or subtract it on odd beats, wrapping the signed 16-bit background Y
  [R-first-bursts, R-cue, R-return-first, R-hide, A-shake-travel, C-shake-wrap].
- All other beats → retain the background Y [R-start, R-last-contact, R-after-hide].
- checked: oracle (R-start, R-last-contact, R-first-bursts, R-cue, R-return-first, R-hide,
  R-after-hide).

### ReturnToPage

- kind: mechanic
- summary: the bursts run for a hold before they cue page 2. The boss stays visible during
  most of the return dissolve.
- params: `shake_hold`, completed encounter beats after the decisive impact, from 1 to
  65,535, reference 128.
- switch: parameter
- Mother Brain was shaking and another shaking beat leaves fewer than the hold →
  advance her elapsed count and keep shaking [R-before-cue].
- The shaking beat reaches the hold → emit the cue for page 2, enter `Returning` and
  restart the boss's breathing animation at its first picture with a full hold
  [R-cue, A-quick-return].
- Mother Brain is returning and the return dissolve reaches zero → take the last shake
  step, remove Mother Brain and ask the intro to hide Samus and her shots for drawing
  [R-hide, C-hide-with-ring].
- The boss has been removed and a ring or its schedule survives → keep that ring or schedule
  on the scene's clock. Continue its movement, contacts and drawing on later encounter
  beats [C-hide-with-ring, C-rings-after-hide].
- checked: oracle (R-before-cue, R-cue, R-hide).

### FightPictures

- kind: output
- summary: ring and burst pictures can overlap, with Mother Brain behind them.
- params: Mother Brain and the rings use image palette 7, and bursts use image palette 5.
  The image content and its part order come from the scene import.
- switch: fixed
- The encounter has visible images → report their art, palette and whole room positions
  in priority order, foremost first with Mother Brain behind the rings and bursts.
  New pictures take the foremost vacant position in that order. Reserve positions for
  a pair's pending arrival and for bursts waiting or on a blank beat. Within a pair the
  toward ring gets the first available position. When the last pair appears, give
  both rings their positions before freeing the schedule's position. Reserve incoming
  rings' ranks before reclaiming rings that leave that beat. Bursts take their positions
  after the ring departures, in the order in the burst table [R-first-pair, R-second-pair,
  R-last-contact, R-first-bursts, R-burst-blank, R-hide, C-shared-beat,
  C-hide-with-ring, C-rings-after-hide].
- checked: oracle (R-first-pair, R-second-pair, R-last-contact, R-first-bursts,
  R-burst-blank, R-hide).

## Order that matters

- Admit the start before Samus's first gameplay step. If setup spans a video cut, carry
  the pending start and Y and resume that work frame. Dispatch the first encounter beat
  after gameplay, which supplies the current position and flying missile candidates. The weapons owner provides
  at most the five candidates that can fly as beams or missiles, with highest precedence
  first. A stopped missile keeps its identity and position in weapons State as an impact.
- Rings already in motion use Mother Brain's phase at the start of the beat. A killing
  missile can leave them present on its impact beat. New rings show their first picture on
  their birth beat. The bursts have no image on the impact beat.
- Publish a pending flash before refreshing the flash count for a new hit. A flash can
  replace colours just published by the outbound dissolve. The intro's later return
  preparation restores the scene palette before dissolving out.
- The intro has already taken its stage step when the cue arrives. It retains the cue on
  this work frame and prepares page 2 on the next. At the end of the return, Samus and
  weapons still step on the hide beat. Use the hide for drawing after the encounter.
- Video uses the encounter's Y to place the background. It uses the current
  camera for sprite positions. Ring and burst picture origins remain in room coordinates.
- A work frame can cross a video cut before or after the encounter step. Save the scene's
  progress through the frame so that resume continues from the phase that ran last.
  The platform's finished-frame marker can schedule one step per work frame for a
  finished-cut scoped check. Intermediate comparisons also need the phase at the cut.
