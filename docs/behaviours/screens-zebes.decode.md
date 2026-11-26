# The flight to Zebes, decode and replay evidence

Companion to the [behaviour spec](screens-zebes.md). Observer and integrator only.

## Sources

The record is `chain-any-0011-DF45` in the pinned lean cache. Movie frame equals cut plus
13,072. Its last cut is the flight hand-off before the milestone. The setup request arrives at cut 1,004. The setup
begins between cuts 1,004 and 1,005, and finishes at 1,014.

| Behaviour | Original sources and interpretation |
| --- | --- |
| GunshipAppears | `$8B:C627..C64D` publishes the incoming flight setup at the end of the station fade for the escape route. `$8230` and `$C699..C79B` initialise the page, upload the assets and reset the screen and cinematic State. `$C771..C787` clears and re-enables palette FX before starting the imported gunship accent. The preceding scene starts its navigation light at `$C2A1`. `$C79C..C7C9` uses the general reveal `$80:894D`, with delay and counter set to 1. `$A35B..A38B` increments `1A51` after each scene step. While the fade and mosaic run, the pre-increment value modulo 4 controls whether the block size drops. |
| PlanetAppears | `$8B:C7CA..C83A` reduces the mosaic to single pixels, selects the affine gunship view with transparent overflow, and places the planet, stars and a hidden caption. `$C83B` and `$C942..C991` give their anchors and palettes, and `$CEA3`, `$CEF7..CF0E` with their corresponding instruction lists give the sprite pictures. The unused star at `$CEA9` is outside the scene. |
| PlanetIsNamed | `$8B:CCBB..CCDA`, `$C992..C9F8` and the sprite runner `$93EF..943F` show the English picture and place optional Japanese text on the same frame its hold starts. Japanese displays for 192 frames. The palette packets are `$8D:C90E` and `$C964`, eight three-frame samples per effect, affecting three colours shared by the text. The caption is a sprite throughout, with no writes to the flat page. Its fade-out request is at cut 1,334 even though its first colour sample repeats the previous colours. The first visible dimming is at 1,337. |
| ShipApproachesPlanet | `$8B:C9C7..C9E5` sets the moving pose. `$C9F9..CA35`, `$CA36..CA84` and `$CA85..CACF` set the three motion rules. The immediate operand at `$CA64` is 16, correcting the bank annotation. The turn finishes on the first cut where the new whole x is below −128. Both zoom thresholds are tested before increment, and displacement on their crossing cuts still uses the old phase. The ship is hidden on the hold's start cut. |
| PlanetHolds | `$8B:CAC9..CADE` sets and counts the hold. `$C84E`, `$C8AA` and `$C8F9` arm the slide on the cut that ends the hold. Movement begins one frame later. |
| PlanetAndStarsSlide | `$8B:C85D..C896`, `$C8B9..C8F8`, `$C908..C941` increment upward speed before displacement and remove each draw above the top boundary. `$CE53` deletes the sprite that cut. The lower-right field advances the scene when removed. |
| LandingSiteLoadBegins | `$8B:C8E1..C8F8`, `$CADF..CB04` and `$A35B..A38B` publish the ready state on the field's departure, then perform the switch on the next frame. `$82:83D6..83DF` sets the loading context at cut 240 during the station blackout. It is retained through this flight. `$82:801D..805F` reads that context in the receiving load and selects the Crateria landing entrance. |

E4 uses the existing [shared plane arithmetic](screens-flight.decode.md#sources), bank 8B
`$8532..85CF`, sampled after the scene update. The origin is set at `$C74A..C758`. Asset source
identities are bank 97 `$8ADB` for the flat tilemap, bank 96 `$EC76` for the Zebes tile
graphics installed at `$8B:C711..C72C` for the flat page and sprite sheet, and
`$7F:4300..45FF` for the back of the gunship's map, prepared by the preceding scene's
decompression of bank 96 `$FE69`. Bank 95 `$A82F` provides the shared affine tiles and bank 8C
`$E5E9..E7E8` the initial space palette in the station scene. `$8B:C6D1..C732` establishes the
Zebes scene destinations and tile dependencies.
Decode the resulting sprite pictures through bank 8C. Bank 8B `$9746..9798` passes them to E1
in placement order. The scene uses zero gameplay scroll offsets, which gives the picture
anchors directly to the draw service. Decode palette effect `$8D:E1A8` for the
engine accent. The original setup retains inherited palettes and shared affine tile
graphics. A standalone importer needs those dependencies from the player's ROM or the
preceding owner's typed content. There are no sound requests issued by the flight.

## Record mapping

| Evidence | Record fields and interpretation |
| --- | --- |
| Stage | `1F51` trajectory `C699` pending, `C79C` and `C7CA` both revealing, `A38F` caption, `C9F9` rightward, `CA36` banking, `CA85` leftward, `CAD0` still, `CADE` sliding, `CADF` leaving. Complete is determined by game state `0998` becoming 6, with `1F51` still `CADF`. |
| Finished frames and mosaic | `05B6` finished-frame count, `1A51 mod 4` mosaic clock, 1-based square width from the high nibble of `0057`. The low nibble selects BG1 for the effect from the setup on. `0051` bit 7 controls forced blank and its low nibble specifies brightness. Fade wait is `0725` in the reveal. |
| View and plane | `0055` and `0069` distinguish the regular tiled page with sprites, affine plane with sprites, and sprites only. `198D`, `198F` angle and zoom, `1993.1991`, `1997.1995` signed whole and unsigned fractional scroll. `0078..007E` and `00B1/00B3` are the E4 matrix and scroll mirrors. Origin is in `0080` and `0082`. |
| Sprite pictures and anchors | Slot index doubled and added to `1A5D` gives its current picture, zero when hidden or removed. Same offset applies to `1A7D.1ADD` and `1A9D.1AFD` for the position and `1B7D` for velocity. Slots 15–10 hold planet, upper left, upper right, lower left, lower right, English caption. Japanese would use 9. The active caption's combined age is the sum of the durations through the current hold, minus its remaining timer at `1B5D` plus its slot offset. Use `1B1D` plus offset to decode its current hold even while hidden. |
| Setup palette | `1E7D..1E8B` identifies the active palette effects. The only active effect on cut 1,014 is the gunship accent, `E1A8`. Compare inherited CGRAM independently. |
| End hold and hand-off | `1A49` for the hold. `0998` is the game state. `09C2` and `09C4` give current and maximum energy. `7E:D914` is the retained landing context. Area `079F`, load station `078B`, and room header `079B` still designate the Ceres context at 2,123. |

## Replay

The scoped case should start from shaft cut 1,004, before the flight first runs. Keep the
original row as the oracle. Give F4 the imported scene and supply carried palette, health
and destination to their respective owners. Use the indexed replay `cache-cut` start at
1,004 on the existing row. Compare the ready state before and after the station hand-off.

- Cuts 1,005–1,013 are nine borrowed lag cuts within setup. Compare blanking, game flow and
  carried health and destination. Mask transient partial setup writes by T1 and E2 on those
  cuts, including the page, matrix and internal scene storage. The first full scene
  comparison starts at 1,014, when setup has finished.
- On every finished cut compare the decoded scene stage, brightness, mosaic width, view, plane
  pose, matrix and integer scroll, placed picture presence and position with fractions by
  kind, speed while placed, caption age while active, fade wait and mosaic clock in the
  reveal, final hold, flow, energy and retained loading context. Matrix and scroll remain
  observable mirrors even on the sprite-only cuts.
- At 1,014 compare the page and graphics references in the E2 installation request and
  the engine-accent reference and inherited-colour policy in the R4 replacement request.
  The setup record replaces the active navigation-light effect with the gunship accent.
  CGRAM matches the incoming cut at all 256 entries. The three upload regions of VRAM
  change and identify the affine ship map, flat map and Zebes tile graphics transfers.
  At 1,046 compare the change to
  the affine picture and first five sprite draws. At 1,110 and 1,334 compare the colour
  effect requests. The 1,110–2,123 English caption CGRAM comparison applies only to the
  three colour entries written by the label's R4 effects. Palette byte and actor evidence
  are independent.
- At 2,122 the marked star is gone and the loader is scheduled inside the scene. The
  screen is still lit on that cut. At 2,123, movie frame 15,195, compare forced blank,
  a zeroed plane and no actor draws, full energy, and the loading flow with the retained
  Landing Site destination. T1 publishes both owner outputs at 2,123 and dispatches R2
  on the next scheduled frame, movie frame 15,196. R2 updates the area and entrance during that frame. The `any-zebes-arrival` cut 0 record has the same completed-frame count,
  flow, brightness, health and cleared plane. Its capture is already inside the first
  long load frame, with area and entrance selected and page and mode mirrors changed
  by the loader. The two raw snapshots are therefore different.
- Actual page transfers and tile bytes are E2 outputs; the sprites' encoded OAM is E1's;
  the affine map and its drawing are E4's. The observer reference model compares their
  scene inputs and the affine matrix/scroll. Full VRAM, OAM, raw player entity state,
  engine accent CGRAM, remaining palette bytes, sprite picture lifetime counters and
  raw PPU configuration mirrors beyond the named comparisons are outside this proof.
  They have drawing, R4, R2 or T1 as their respective writers. No controller or music
  timing reply is required.

Recorded vector edges are in `chain-any-0011-DF45`, at the cut that ends the vector. Picture
vectors select the named pictures relevant to the arm and omit the unrelated active pictures.

| Cut | Vector |
| --- | --- |
| 1,015 / 1,016 / 1,018 / 1,044 / 1,045 | R-fade-waits / R-first-light / R-mosaic-step / R-full-light / R-lit-waits |
| 1,046 | R-planet |
| 1,431 / 1,715 / 1,716 / 1,810 / 1,811 / 1,812 | R-right / R-bank-starts / R-bank / R-bank-held / R-left-starts / R-left |
| 1,985 / 1,986 / 2,049 | R-hold-starts / R-hold-ticks / R-slide-starts |
| 2,050 / 2,093 / 2,122 | R-slide-first / R-slide-removes / R-last-star |
| 2,123 | R-handoff |

The scratch model in `$TEMP/collab/screens-zebes-evidence` agrees with 34 vectors, 19
recorded and 15 constructed. Both marker vectors fail if the model uses the shipped field
in place of the imported one. The setup witness matches the palette replacement at cut 1,014
and detects the three VRAM upload regions. Running the adjusted hold case with the reference rule fails
the hold assertion. Across cuts 1,005 through 2,123 the carried model agrees on 1,119 of
1,119 cuts, including 1,110 finished frames. Holding the starting State agrees on 9 of 1,119
cuts, all within setup's masked lag. The title colours agree on 1,014 of 1,014 cuts checked.
The Japanese caption is supported by source and constructed cases. This record uses English.
The indexed integrated case compares the setup transfers, visible OAM, the complete published
palette and the scheduler's hand-off. Pixel composition and off-screen OAM retention remain
outside that comparison.

## First Landing Site load cut

The any% seed for the shaft is movie frame 13,072. The flight's last cut is
row cut 2,123, movie frame 15,195, and emulator video frame 15,196. The next
cut is movie frame 15,196 and is also cut 0 of `any-zebes-arrival`.

T1 needs to dispatch the retained Landing Site request on this next opportunity,
even though no work frame finishes. The receiving cut is a vblank with NMI
disabled. It interrupts the loader during the standard sprite DMA (`$82:8327`),
after the standard BG3 transfer and before the first standard tilemap transfer.
The loader needs a saved in-progress phase so a later advance continues the same
load. The finished count, forced blank, full energy and cleared affine plane
already came from the flight.

R2 needs to select Crateria and its Landing Site cutscene station from the carried
destination, and restore the explored map from run progress. At this cut the room
header still names the Ceres shaft. Loading the station's room, door and player
placement follows this prefix. The existing Ceres new-game importer reads the same
station table but only exposes Ceres's first entry. The Landing Site station needs
a typed entry when the loader reaches that table.

A replay of this prefix should check uninterrupted flow, finished count, Health,
destination, the changed area and station, and the retained room. It should check
the gameplay PPU mirrors, including Mode 1, disabled mosaic and the layer and
tilemap selections, and the completed standard BG3 transfer. CGRAM still matches
the last flight cut and there has been no NMI commit. The first 3,245 sprite VRAM
bytes match the imported graphics and the remainder still matches the last flight
cut. A scoped comparison can borrow transfer progress from T1 or mask the in-flight
region. Deriving this partial transfer from the movie alone needs the live T1 clock.
The later continuation of the sprite transfer wraps into low bus memory
(`$82:8314..8322`) and needs its carried source bytes at that point. Tilemap
uploads, palette initialization, room initialization and gameplay follow this cut.
The current indexed integration compares the shared finished count, flow, retained
destination, Health, blanking and plane against the receiving record. The production load prefix remains
Unsupported.

## Budget

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit. At `b7dc44c8e` there is
no replaced legacy flight owner, so the areas have no old code to size. The existing E4
arithmetic and assets are shared dependencies.

- Area reveal: GunshipAppears, PlanetAppears
- Area caption: PlanetIsNamed
- Area flight: ShipApproachesPlanet
- Area departure: PlanetHolds, PlanetAndStarsSlide, LandingSiteLoadBegins

The spec has seven entries, 25 arms, no table cells and no formula operator tokens. The
counter reports no findings.

Budget: 37.5 × 25.0 units = 936 (reveal 262, caption 187, flight 225, departure 262)
