# The flight to Zebes

F4 owns the scene after the station explosion blacks out. It holds the picture and its timing
through the cut that asks to load the Landing Site. Button presses never change the scene.

The scene reads imported `ZebesScene` content, the player's subtitle selection, and permission
to advance one finished frame from the scheduler (T1). The scheduler carries the destination context to the loader. E2 installs the scene's pages, E4 transforms the ship plane, and E1 draws the planet,
stars and label. The model chooses which drawing to show and when, and supplies its position,
brightness and mosaic. R4 plays colour effects on request. The Loader (R2) and Samus's health
owner receive the two events on the final cut. The current engine has no production owner for
this flight.

## Values the scene carries

One F4 step advances a finished frame. On a borrowed lag cut F4 holds its last published
picture and emits no new events. T1 handles the preparatory drawing transfers during the
setup frame.

| Quantity | Domain type and units | Read by |
| --- | --- | --- |
| Scene stage | `ZebesStage` with `Pending`, `Revealing`, `Caption`, `Rightward`, `Banking`, `Leftward`, `Still`, `Sliding`, `Leaving`, `Complete` | Every entry |
| Drawing selection | `FlightView` with `MosaicPage`, `ShipAndSprites`, `SpritesOnly` | Output to E2, E4 and E1. Held by `LandingSiteLoadBegins` through the final blank. |
| Screen light | `Brightness`, `Blanked` or `Shown(level)`, level 1 through 15 | `GunshipAppears`, `PlanetAppears`, output |
| Fade wait | One frame or zero frames left | `GunshipAppears` |
| Mosaic block size | Square width in screen pixels, 1 through 9 | `GunshipAppears`, `PlanetAppears`, output |
| Mosaic clock | `QuarterBeat`, 0 through 3 finished frames since the previous beat, modulo 4 | `GunshipAppears`, `PlanetAppears` |
| Ship plane pose | `PlanePose`, angle in 256ths of a turn increasing clockwise on screen, zoom in 256ths (256 is actual size), and two signed pixel scroll positions with fractions in 65,536ths | `ShipApproachesPlanet`, output to E4 |
| Caption age | Frames elapsed since the planet appeared, from zero up to one less than the sum of the imported holds | `PlanetIsNamed` |
| Placed pictures | The planet, star fields and currently shown caption pictures by content ID. Each has a signed pixel position in 65,536ths and an upward speed in 256ths of a pixel per frame | `PlanetAndStarsSlide`, output to E1 |
| Hold | Whole frames left, 1 through 65,535 while in `Still` | `PlanetHolds` |

A caller can inspect and restore a valid snapshot. Caption age is stored only in `Caption`,
and hold only in `Still`. The fade wait and mosaic clock are read only in `Revealing`. At full
light the fade wait is zero. Snapshots retain light, mosaic width, drawing selection and plane
pose as current picture values, including while the ship plane is hidden. The final cut
retains `SpritesOnly` and the mosaic width as it blanks the display and clears the pose and
pictures. Each finished frame publishes the resulting picture values. Page installations,
palette-effect requests and the two hand-off outputs occur only on their specified cuts.
The shared E4 plane rule takes the pose, uses the imported origin, and gives the matrix and
integer scroll, rounding toward the lower whole pixel. E1 uses the whole-pixel anchors, with
the planet first, then the star fields in their imported compositing order, then English and
Japanese caption pictures. Changing an anchor leaves the compositing order intact. E1 handles clipping and the tile art's priorities.

## What the scene imports

`ZebesScene` is checked by the importer. `DrawingPages` has a flat page and an affine ship
picture with a pixel origin and transparent overflow. Each page has typed references for the map and its
graphics for E2 and E4. `DrawingPages` also supplies a `SpriteGraphicsRef` for the
sheet the planet, stars and caption pictures use. E2 installs it during setup with the pages
so E1 can draw those pictures. The scene supplies its `engine_accent` as a `ColourEffectRef`.
At setup, R4 replaces the preceding scene's active effects with that accent. It carries
forward the colours already on screen as the starting palette and applies the accent's
samples to it.

`PlanetPictures` names the planet and exactly four star fields. A `PlacedPicture` has a typed
sprite picture reference, palette reference and signed screen anchor in whole pixels. It is
drawn using E1. Exactly one star field is marked `last_field`. This marker supplies the slide's
end even if the other pictures are moved in adjusted content. The original marks the
lower-right star field. At placement, all speed and fraction values are zero.

`CaptionContent` has an English picture, an optional Japanese subtitle picture, and their
screen anchors and palette references. They remain at their anchors while shown. It has four positive
frame holds in order for the hidden lead, English lead, full caption and fading caption. The holds in the shipped scene measure
64, 32, 192 and 96 frames. The caption has typed R4 effects for brightening and dimming. A
colour effect consists of the palette entries it changes and a nonempty succession of colour
samples in our format, with a positive duration in finished frames per sample. R4 starts a
requested effect on that frame. The shipped effects have eight samples each, held for three
frames each. The importer checks that the holds sum to at most 65,535 frames and checks all
picture and colour references before play. Language choice changes the subtitle's presence.

## Reveal, caption and flight

### GunshipAppears
- kind: mechanic.
- summary: the scene begins dark with a coarse mosaic, then brightens while its blocks shrink.
- params: initial block width 9 px, one level of brightness every two finished frames.
- switch: Fixed.
- `Pending`, its setup frame finishes → `Revealing`, show `MosaicPage` with the sprite drawing
  enabled, blanked, with blocks 9 px wide and fade wait 1. Set the mosaic clock to 1, ship
  pose to angle 0, zoom 256, scroll (0, 0), no placed pictures. Ask E2 to install the drawing
  pages and their sprite graphics. Ask R4 to replace its active effects using the imported
  `engine_accent` and the current palette [C-setup].
- `Revealing`, wait 1 → clear the wait and keep the light [R-fade-waits].
- `Revealing`, wait 0, the next brightness below 15 → show one level brighter (first light
  is level 1) and set the wait to 1 [R-first-light].
- `Revealing`, wait 0, the next brightness 15 → keep the tiled page, show full light and
  set wait 0 [R-full-light].
- `Revealing` already at full light → keep full light and wait 0 while the mosaic clock
  advances [R-lit-waits].
- During `Revealing`, sample the mosaic clock before incrementing it. When it is 0
  → reduce the block width by 1 to a floor of 1. Advance the clock modulo 4 on each
  finished reveal frame [R-mosaic-step, R-planet].
- checked: oracle; constructed.

### PlanetAppears
- kind: mechanic.
- summary: when the mosaic reaches single pixels, the planet and stars appear with the ship plane.
- params: at the cut to the planet, ship angle 32, zoom 256, scroll (128, −104) px.
- switch: Fixed.
- `Revealing`, the sampled mosaic clock is 0, and after the reveal step the light is
  full and the blocks are 1 px wide → choose
  `ShipAndSprites`, place the planet and star fields from content, set the ship pose to the
  new view and enter `Caption` with age 0 [R-planet, C-planet].
- checked: oracle; constructed.

### PlanetIsNamed
- kind: mechanic.
- summary: the label waits in the dark, brightens, holds and dims before the flight resumes.
- params: the ship resumes at angle 32, zoom 16, scroll (62, −112) px.
- switch: Fixed.
- `Caption`, increment the age by 1 and it reaches the first hold's end → draw the English
  caption and request its brightening effect [C-caption-start].
- `Caption`, increment the age by 1 and it reaches the second hold's end → also draw the
  Japanese subtitle if selected and imported [C-subtitle, C-english].
- `Caption`, increment the age by 1 and it reaches the third hold's end → request the
  dimming effect and remove the subtitle. Keep drawing the English caption [C-caption-dims].
- `Caption`, increment the age by 1 and it reaches the final hold's end → remove the caption,
  set the ship pose to the flight's start and enter `Rightward` without drifting yet
  [C-caption-ends].
- `Caption`, the incremented age is between those edges → keep the same picture and
  advance the caption age by one frame [C-caption-holds].
- checked: oracle for English; constructed for English and Japanese.

### ShipApproachesPlanet
- kind: mechanic.
- summary: the ship drifts right, banks left and then recedes as it drifts left.
- params: a shift along y of +1/8 px per frame. A shift along x of −1/2 px per frame while
  `Rightward` or `Banking`, +1/8 px per frame while `Leftward`. Each row tests the zoom
  before the increment, and banking tests x after its drift.
- switch: Fixed.
- `Rightward`, zoom below 1,152 → drift and add 4 to zoom [R-right].
- `Rightward`, zoom at least 1,152 → drift, keep zoom, enter `Banking` [R-bank-starts].
- `Banking`, floor of new x at least −128 px → drift and add 16 to zoom. Subtract 1
  from the angle modulo 256 until it reaches 224, then hold the angle [R-bank, C-turn-wrap, R-bank-held].
- `Banking`, floor of new x below −128 px → drift, keep angle and zoom, enter `Leftward`
  [R-left-starts].
- `Leftward`, zoom below 8,192 → drift and add 32 to zoom [R-left].
- `Leftward`, zoom at least 8,192 → drift, keep zoom, hide the ship plane with
  `SpritesOnly` and start the hold [R-hold-starts].
- checked: oracle; constructed.

## Still picture, slide and hand-off

### PlanetHolds
- kind: mechanic and parameter.
- summary: after the ship disappears, the planet and stars stay still before sliding away.
- params: `hold_frames` from 1 through 65,535, reference 64 finished frames.
- switch: **Parameter**, `hold_frames`.
- The ship disappears → enter `Still` with `hold_frames` left [R-hold-starts, C-short-hold].
- `Still`, more than one frame left → subtract one frame [R-hold-ticks].
- `Still`, one frame left → enter `Sliding` with hold 0 [R-slide-starts].
- The pictures first move on the following frame.
- checked: oracle; constructed.

### PlanetAndStarsSlide
- kind: mechanic.
- summary: the planet and stars accelerate upward until they disappear.
- params: upward acceleration of 64/256 px/frame² for the planet and 32/256 px/frame² for
  each star field. Remove a picture when its y is below −128 px.
- switch: Fixed.
- A frame begins `Sliding` with a placed picture → add its acceleration to its speed,
  then subtract its new speed from y with the fraction carried. Keep it if y is at least
  −128 px [R-slide-first, C-slide-floor].
- A frame begins `Sliding`, a picture crosses below −128 px → remove it on this cut.
  If that picture is `last_field`, enter `Leaving` after all pictures have moved for
  this cut [R-slide-removes, R-last-star, C-last-star, C-unmarked-star].
- checked: oracle; constructed.

### LandingSiteLoadBegins
- kind: mechanic.
- summary: after the marked star field leaves, the scene blacks out and hands control to
  the Landing Site load. Samus's energy returns to its maximum.
- params: none.
- switch: Fixed.
- A frame begins `Leaving` → blank the display, clear the plane pose and placed pictures,
  enter `Complete` and publish both `RestoreFullEnergy` to the health owner and
  `BeginLandingSiteLoad` to T1 as part of the same cut [R-handoff, C-handoff].
- A frame begins `Complete` → hold the completed picture and send neither event [C-complete].
- T1 dispatches R2 on the next scheduled frame, which can span borrowed lag cuts. T1
  retains the destination context through the flight and passes it to R2 on dispatch. R2
  uses it to select the Landing Site entrance and prepare the room.
- checked: oracle; constructed.
