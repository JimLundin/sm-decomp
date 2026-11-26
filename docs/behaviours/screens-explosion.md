# The station explosion

F4 owns the cinematic after the escape elevator blacks out and saves. It takes
`BeginStationExplosion` from the scheduler and sends `BeginZebesFlight` on the cut
that blanks. The scheduler carries the Landing Site destination through both scenes and supplies
the opening blank interval. The elevator owner's stop and three silence requests
are already in flight at intake.
The elevator, save, escape countdown and gameplay state belong to their existing
owners. Button presses leave the scene unchanged. There is no production owner to replace.

T1 supplies completed scene frames and the setup completion. U1 receives song requests
and supplies `MusicBusy`. E2 installs the art, E4 transforms the affine plane, E1
renders sprite pictures, and R4 replaces the palette and starts the navigation lights.
E3 owns the line-window presentation and draws the station flash on request. State
and outputs use imported `ExplosionScene` content.

## Values

| Quantity | Domain type and units | Read by |
| --- | --- | --- |
| Stage | `ExplosionStage`: Pending, WaitingForMusic, Revealing, Station, Departing, Holding, Dimming, Complete | All rules |
| Light | `Brightness`, Blanked or Shown(level) with level 1 through 15 | StationBrightens, SceneBlacksOut, output |
| Fade wait | 0 or 1 completed frame | StationBrightens, SceneBlacksOut |
| Pose | `PlanePose`, angle in 256ths of a clockwise turn, zoom in 256ths of actual size, signed scroll in 65,536ths of a pixel | StationBrightens, StationBreaks, ShipDeparts, E4 |
| Plane drawing | `Station` or `Gunship` and the imported pivot in whole pixels | StationBreaks, E2, E4 |
| Picture age | Count of scene frames since pictures were placed, including the placement frame | SpaceDrifts, StationBursts |
| Burst state | Each live burst's group and occurrence, signed sample age in frames, position in 65,536ths of a pixel, and current sample in presentation order | StationBursts, E1 |
| Hold | Whole frames left, 1 through 65,535 while Holding | SpaceHolds |
| Music reply | Boolean sampled after audio service on a completed WaitingForMusic frame | StationAwaitsMusic |

A valid snapshot can be restored without retransmitting one-time outputs. Setup is
scheduled while the display is blank. During scheduler stalls hold the last F4 picture
and emit no new F4 event [C-lag]. Audio and transfers can progress separately. A
restorable departure snapshot has already passed the last burst group's start. After
placement advance the sky and live bursts on every finished scene frame through
the handoff cut.
Frame outputs report the resulting picture. Page installs, effect starts, audio requests
and the flow handoff are edges.

## Imported content

`ExplosionScene` describes the station's plane picture and graphics, the gunship front, the empty
lower half, common affine graphics and the rear gunship drawing needed by the next scene. It also
provides the sprite drawings and palettes, the initial space palette, the navigation
light effect and a `StationFlashStyle` reference for the presentation owner. The
pivot is (52, 48) px. The common affine graphics and the back of the ship map
remain available to Zebes. Use transparent overflow around the plane.

The three sky pictures have ordered drawing references. Large asteroids start at (80, 159)
and move right 1/4 px per frame. Small asteroids start at (128, 96) and move right 1/32 px per
frame. The purple background stays at (112, 87) and alternates its two drawings each frame
starting with the first. The importer supplies positions, palette and pictures in this
drawing order, with burst pictures drawn after the large field and before the small field.
The burst tracks contain positive frame lengths, optionally blank samples, a finite
repetition count, and their palette references. The importer checks references and that
composed timing and placement fit the admitted snapshot types.

## Opening

### StationPictureArrives

- kind: mechanic.
- summary: the black screen changes to a prepared picture of the station in space.
- params: initial angle 0, zoom 256, scroll (−44, −112) px.
- switch: Fixed.
- `Pending`, setup still in progress → keep the last black picture and publish no events [C-setup-waits].
- `Pending`, setup completes → install stage and sprite art, choose the station plane,
  use the initial pose, start the sky, and set fade wait to 1. The first complete published setup
  picture includes two sky updates. Replace the space palette
  and its active effects with the imported palette and navigation light. Request music stop,
  Ceres music data, then the flight to Zebes song, with delays 8, 8 and 14 respectively.
  Keep the view blank and enter `WaitingForMusic` [C-setup].
- checked: constructed; replay setup witness.

### StationAwaitsMusic

- kind: mechanic.
- summary: the station stays dark until the audio service is ready.
- params: none.
- switch: Fixed.
- `WaitingForMusic`, music busy → hold the plane and light [R-music-waits].
- `WaitingForMusic`, music ready → enter `Revealing` without changing pose or light [R-music-ready].
- checked: oracle.

### StationBrightens

- kind: mechanic.
- summary: the station drifts into view and brightens.
- params: drift (−1/4, +1/16) px, zoom increase 1, one level every two frames.
- switch: Fixed.
- `Revealing` → apply drift and add 1 to zoom before advancing the light [R-reveal-wait].
- `Revealing`, wait 1 → set wait to 0 and keep the same light [R-reveal-wait].
- `Revealing`, wait 0 → raise the light one level, counting blank as zero,
  and set wait to 1 [R-first-light].
- The light reaches 15 → set wait to 0 and enter `Station` [R-full-light].
- checked: oracle.

### SpaceDrifts

- kind: mechanic.
- summary: asteroid fields drift in front of the purple background.
- params: asteroid horizontal anchors wrap at 512 px.
- switch: Fixed.
- Sky placement or any subsequent finished scene frame → advance both asteroids by
  their respective imported deltas, wrapping x. Keep each y fixed. Use the sky age
  to select the purple background's alternating picture [R-sky, C-sky-wrap].
- checked: oracle; constructed.

## Station and departure

### StationBursts

- kind: mechanic.
- summary: three bands of bursts cross the station before the final explosion.
- params: the shipped tracks and placements below. Offsets are screen pixels.
- switch: Fixed.
- Picture age reaches 129 → create the first group, using the ordered
  (horizontal offset, vertical offset, first picture delay) triples
  (16, −16, 1), (−16, 16, 16), (16, 16, 32), (−16, −16, 48), (0, 0, 64) [R-first-group].
- Picture age reaches 210 → create the middle group's first burst. Add another every 12 frames while the
  group remains active. Use the ordered offsets
  (14, −8), (8, 12), (−16, 12), (−8, −14), (0, 0), (16, 14).
  The age 273 cut ends the middle schedule [R-middle-start, R-middle-next].
- Picture age reaches 273 → create the last group using triples
  (8, −4, 1), (12, 8, 4), (−8, −10, 8), (−12, 12, 16) [R-last-group].
- On each placement, the initial screen anchor is the pivot minus the floor of
  the current scroll, plus the offset. Existing bursts drift (+1/4, −1/16) px
  on every finished frame, including when their animation is blank [R-drift].
- Every launch cut applies the burst's first drift and animation update. A first
  picture delay of one makes that picture visible immediately. Longer delays keep
  the burst hidden while it drifts and reduce the delay on each update
  [R-first-group, R-first-picture, R-drift].
- Within each group the bursts appear in the stated launch order, with earlier
  entries in front when they overlap. The middle group draws in front of the last group.
  The final burst draws in front of the survivors at breakup. Remove a burst when
  its track finishes [R-middle-next, R-last-group, R-flash, R-burst-end].
- The first group shows six pictures of three frames apiece. The middle group
  plays that sequence six times. Each pass ends with 16 blank frames.
  The last group has seven passes. Each pass shows four pictures for five frames
  apiece, then blanks for eight frames. The final burst shows six pictures once
  for five frames apiece [R-first-picture, R-burst-end, C-loop].
- checked: oracle; constructed.

### StationBreaks

- kind: mechanic.
- summary: after the station has receded, a flash covers it and the ship appears.
- params: drift (−1/4, +1/16) px, station zoom limit 640, departing zoom 768.
- switch: Fixed.
- `Station` → apply the drift. If zoom at the start of the frame is below 640,
  add 1 to zoom and keep the station drawing [R-station-drifts, R-zoom-limit].
- `Station`, start zoom at least 640 → after drift, emit `StartStationFlash(style, center)`
  using the imported flash style and screen center at pivot minus floored scroll.
  Draw the final burst at this center plus its first drift. Ask E2 to install the
  gunship front in the upper map region, then the empty drawing in the lower region.
  Set zoom to 768 and enter `Departing`. Keep the palette effect and active bursts
  [R-flash, C-typed-flash].
- E3 receives the flash on the breakup cut and enables additive colour inside its window.
  On the second completed frame afterward it also enables the second background for the
  effect. E3 plays the imported window animation and its colour samples at the fixed
  center. When the animation ends, the window goes away and the layer selection persists
  until Zebes setup replaces it [R-flash].
- checked: oracle; constructed.

### ShipDeparts

- kind: mechanic.
- summary: the gunship turns and recedes until it disappears.
- params: drift +2 px along x, angle −1 modulo 256, zoom decrement 16.
- switch: Fixed.
- `Departing` → apply the drift and turn. If zoom at frame start is at least 16,
  subtract 16 and stay in `Departing` [R-turn-wrap, R-zero-zoom].
- `Departing`, start zoom below 16 → apply the drift and turn, keep zoom,
  enter `Holding` and start its hold [R-hold-start, C-short-hold].
- checked: oracle; constructed.

### SpaceHolds

- kind: mechanic and parameter.
- summary: after the ship disappears, space holds before the final fade.
- params: `hold_frames` 1 through 65,535, reference 192.
- switch: **Parameter**, `hold_frames`.
- `Holding`, more than one frame left → subtract one [R-hold-ticks, C-short-tick].
- `Holding`, one frame left → enter `Dimming` with fade wait 1, at full light [R-fade-start, C-short-end].
- checked: oracle; constructed.

### SceneBlacksOut

- kind: mechanic.
- summary: space dims to black and gives way to the flight to Zebes.
- params: one level of brightness every two finished frames.
- switch: Fixed.
- `Dimming`, wait 1 → keep the light and set wait to 0 [R-fade-wait].
- `Dimming`, wait 0, brightness above 1 → lower the light by one and set wait to 1 [R-fade-step].
- `Dimming`, wait 0, brightness 1 → blank, enter `Complete`, clear the fade wait
  and send `BeginZebesFlight` to T1 on this cut. Retain the final plane and draws under
  forced blank. Zebes begins Pending with the inherited palette, music and affine assets
  [R-handoff].
- `Complete` → hold the picture and emit no events [C-complete].
- checked: oracle; constructed.
