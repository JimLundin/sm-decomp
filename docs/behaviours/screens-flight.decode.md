# The flight to Ceres: decode and harness

Companion to the [flight's behaviour spec](screens-flight.md). Keep this note outside confined
exports.

## How it was made

Behaviour first, from CI's lean record cache (`$TEMP/collab/ci-cache/rows`): `any-fly-to-ceres`
cuts 0-802, read per cut by a scratch reader of the lean tier (`$TEMP/collab/ceres-739/lean.py`,
local only): game state, the cinematic function, BG mode, brightness, the mode 7 angle, zoom and
cinematic BG1 position (with fractions), the matrix and BG1 scroll mirrors, the colour math
subscreen backdrop, the screen fade's delay and counter, the cinematic sprite objects (slot,
instruction pointer, position with fractions, velocity), the palette FX objects, CGRAM, the
cinematic BG object's timer and the BG1 tilemap in VRAM, the music queue timers and the count of
finished frames. The bank logs (8B, 8C, 8D, 80, A0) were then read to pin the arithmetic.

The reference model, `$TEMP/collab/flight-756/scratch/model.py` (local only), steps the spec's
rules once per finished frame (a cut whose finished-frame count `05B6` rose), with the borrowed
input "a track waits" read as any music queue timer (`0629..0637`) non-zero in the cut's own
record, and compares at every cut from 32 (the setup frame's end) to 802 these fields: the
stage (`1F51`), the angle, the zoom, the position with its fractions, the transformation, the
scroll, the brightness, the flash, the fade's count, the sprites by kind with both places'
fractions, the rushing stars' speed (while placed) and the label's step and hold. It does not
compare CGRAM, OAM, VRAM or the PPU layer, blend and mode mirrors (the harness's masks below).
The scene's content in the model is the game's, read from the bank logs (§ Sources), as the
importer will give it.
- `vectors.py`: 27 of 27 vectors agree.
- `any-fly-to-ceres` cuts 32-802: 771 of 771 agree on those fields: 743 finished frames and 28
  lag cuts (41-68) where nothing of the flight changes. A do-nothing model (the seed's record
  held) agrees on 0 of 771 (`null.py`).
- The letters (`typed.py`): the model types at 468 and every 16 cuts to 628 (11 letters); the BG1
  tilemap in VRAM changes in one word at each of 484 to 628, and at 468 in 960 words (the layer
  emptied, the first letter with it).

The music-only reference model in the observer's local evidence directory checks the ordered
requests on all 27 vectors and every interval of the cached row. It matches 802 of 802 cuts
(743 finished and 59 lag). An empty request list on every cut matches 801 of 802 and first
differs at cut 32. Reversing the requests or swapping their delays also first differs at 32.

## Sources

| Entry | Sources |
| --- | --- |
| FlightIsMadeReady | `$8B:BCA0` (cinematic function, fly to Ceres, initial): first `$8B:819B` (set up the PPU for the Ceres cutscene): `0051` = 80h (blanked, level 0), `0074..0076` = 20h/40h/80h (flash 0, 0, 0), `1991`..`1997` zeroed (so the fractions are 0); then palettes, decompression and VRAM uploads under NMI off (the long frame, cuts 1-31); matrix identity, origin (38h, 18h); `1993` = −48h, `1997` = −68h (the integer words, the fractions already zeroed); angle `198D` = E0h; zoom `198F` = 200h; palette FX cleared and enabled; cinematic BG objects cleared; spawn cinematic sprite object `$CF0F` (Ceres stars) parameter 0 (`$8B:BE7E`: velocity `1B7D` = −400h, X 70h, Y 57h); next function `$BDE4`; at `$8B:BDD2`..`$8B:BDE3`, music data `$FF2D` queued via `$80:8FC1` with its default 8-frame delay, then track 5 via `$80:8FF7` with a 14-frame delay |
| PictureAppears | `$8B:BDE4`: `$80:8EF4` (carry set if any of the eight music queue timers is non-zero) → return; else `0051` = 0Fh, next function `$BDF9` |
| GunshipRushesIn | `$8B:BDF9`: zoom ≥ 20h → zoom − 10h, return; else the turn (below) |
| StarsRushPast | `$8B:BEB5` (pre-instruction of `$CF0F`): only while the cinematic function is `$BDF9`; velocity += 80h; delta = velocity as signed 8.8 (`$12` sign-extended high byte, `$14` low byte × 100h) added to the object's X and Y and to cinematic BG1 X and Y (16.16) |
| ViewTurnsAway | `$8B:BE09`..`$8B:BE73`: subscreen layers off, colour math on BG1/sprites/backdrop, backdrop `0074..0076` = 3Fh, 5Fh, 9Fh (31, 31, 31); clear cinematic sprite objects; mode 7 transfer `$BE74` (the back of the gunship tilemap); `1993` = −20h, `1997` = −80h (integers only); angle 20h; spawn `$CF39` (`$BF22`: 50h, 9Fh), `$CE85` (`$BF4C`: 74h, A0h), `$CE8B` (`$BF76`: 80h, 60h), `$CE91` parameter 1 (`$BFA0`: E0h, 57h), `$CF0F` parameter 1 (`$BE7E`: −20h, 57h, pre-instruction `$BFC6`); next function `$BFDA`; palette FX `$E1A8` (engine flicker), `$E1AC` (Ceres navigation lights, sprite Ceres): R4's |
| SpritesDrift (the turn's drifts) | pre-instructions `$BF35` (+0.4000h, X %= 200h), `$BF5F` (+0.1000h, %= 200h), `$BF89` (+0.0800h, %= 200h), `$BFC6` (−0.2000h, no wrap; used by the vortex and the far stars) |
| GunshipFliesAway | `$8B:BFDA`: backdrop channels −1 each to floors 20h/40h/80h (level 0); BG1 X −= 0.2000h; zoom < C00h → +10h; < 2000h → +20h; else `$C03E` |
| FlashFades | `$8B:BFDA`'s first three blocks |
| SpritesDrift | the pre-instructions above, run by the cinematic sprite object handler every frame after the cinematic function, including the frame of their spawn |
| StationComesIntoView | `$8B:C03E`: main screen BG1/sprites, BG mode 1, BG1 tilemap 5C00h, tiles 6000h; zoom 100h, angle 0, BG1 X/Y (with fractions) 0; enable cinematic BG objects and their tilemap updates; spawn BG object `$CF69` (`$8C:D629`: the label "SPACE COLONY", eleven letters with hold 10h each, a Japanese line skipped for English text (`$8B:C096`), a hold 80h with no letter (`$8C:D683`), then `$8B:C0A2`); next function RTS (`$A38F`) |
| LabelIsTyped | the cinematic BG object handler (`$8B:95F0`..`$9742`): the object's timer `19E3` counts down; the next instruction draws its tiles and sets the timer; `$8B:C0A2` (finish fly to Ceres): next function `$C0C5`, screen fade delay `0723` = 1 and counter `0725` = 1 |
| PictureDarkens | `$8B:C0C5` → `$80:8924`: counter − 1 if it stays ≥ 0; else counter = delay and the brightness: level 0 → kept; level 1 → 80h (forced blank, level 0); else level − 1 |
| FlightEnds | `$8B:C0C9`..`$8B:C0FF`: `0051` = 80h → enable NMI, delay and counter 0, BG2/3 scrolls 0, clear 198Dh..1C1Fh (angle, zoom, BG1 position, every cinematic sprite and BG object), next function `$C100`. `$8B:C100` runs on the next frame: game state 1Fh, loading game state 1Fh, area Ceres, load station 0, save slot to SRAM (P2) |
| PlaneTransforms | `$8B:8532`: cos = `$A0:B443`[(angle + 40h) & FFh], sin = `$A0:B443`[angle & FFh]; `$8B:858F` signed 16×16 multiply / 10000h, the middle 16 bits taken (so the floor of product / 256, two's complement); A = D = cos × zoom, B = sin × zoom, C = −B; BG1 scroll `00B1`/`00B3` = BG1 X/Y integers. The table at `$A0:B443` equals `trunc(256 × sin(2πk/256))` for all 256 entries (checked by `sine.py` against the bank log). Origin `0080`/`0082` stays (38h, 18h) |

## RAM mapping

| Spec | RAM | Notes |
| --- | --- | --- |
| the stage | `1F51` (cinematic function): `$BCA0` Starting, `$BDE4` WaitingForMusic, `$BDF9` RushingIn, `$BFDA` FlyingAway, `$A38F` Label, `$C0C5` FadingOut, `$C100` Done | game state `0998` = 1Eh throughout; 1Fh from the next frame |
| the plane's angle | `198D` | |
| the plane's zoom | `198F` | |
| the plane's position | x `1993`.`1991`, y `1997`.`1995` (integer.fraction) | |
| the stars' speed | `1B7D` + the stars' slot (velocity) | slot 15 (`1B9B`) in this row |
| the sprites | cinematic sprite objects: instruction pointer `1B1D`+slot (non-zero = placed), X `1A7D`, X fraction `1ADD`, Y `1A9D`, Y fraction `1AFD` | the row's slots: 15 the rushing stars (32-115); 11 far stars, 12 vortex, 13 small asteroids, 14 Ceres, 15 large asteroids (116-801); kinds, not slots, are compared |
| the flash | `0074`, `0075`, `0076` low 5 bits (COLDATA mirrors) | 0 before the turn (the intro left 20h/40h/80h) |
| the label's step and hold | cinematic BG object 0: instruction pointer `19D3`, timer `19E3` | hold 0 outside the label |
| the brightness | `0051` | bit 7 blanked, low nibble the level |
| the fade's count | `0725`; rhythm `0723` | |
| transformation (output) | `0078`, `007A`, `007C`, `007E` (M7A-D mirrors) | |
| scroll (output) | `00B1`, `00B3` | |
| music requests (output) | queue entries `0619..0628`, corresponding initial delays `0629..0638`, next index `0639` | read the slots appended between adjacent cuts by walking the previous next index to the current one in order, advancing two bytes modulo 16. At cut 32 the slots contain `$FF2D` (load, 8 frames), then `$0005` (play, 14 frames). The next index is unchanged at every other cut. |
| a track waits (input) | music queue timers `0629..0637` | false from cut 84 |

## Harness (for the integrator)

The row is `any-fly-to-ceres` (seed at movie frame 7,512, 803 cuts). What it needs:

1. **Admission from the flight's start.** The seed is the intro's last record (game state 1Eh,
   cinematic function `$BCA0`). Build the flight's State `Starting` with everything zero and the
   brightness blanked at 0; decode nothing else from the seed.
2. **The owner in the loop:** the flight, stepped once per finished frame. Lag cuts (1-31, 41-68,
   59 in all) are T1's, read from the record (`BORROWED`'s lag line).
3. **The borrowed input:** "a track waits", through `BORROWED`'s existing producer ("audio:
   whether a track waits in the queue of tracks as the frame of work begins", the previous
   record's queue stepped once), not a second reader. The reference model reads the cut's own
   record's timers; the two agree on this row (the queue's last timer leaves 1 for 0 at cut 84).
   The reply lets the picture appear. The flight's music requests are a separate output.
4. **Music output at every cut after the seed.** Give the ordered requests in the flight's
   `FrameReport` to U1 through `production::played::flight`. In the played comparison
   use the index, entries and initial delays in the RAM mapping. Compare the fresh report
   from this step, with an empty list on lag cuts. Keep the order and delays in the observation.
   The front-end's sorted queue-entry comparison omits both. An empty list is also expected
   during the wait and on the frame the picture appears. The audio queue starts each delay
   when its request reaches the head (`$80:8F0C`).
5. **Compare at each cut from 32 to 802:** the RAM mapping's State fields and outputs. Cuts 1-31
   are the setup frame's working values: compare only the brightness (blanked, 0). The streams
   no owner here produces, each masked with its writer:
   - CGRAM: the setup's content load at cut 32 (197 colours; AF5 content, through T1's load),
     and from 116 colours 95, 237 and 238 (R4's palette cycles, `$E1A8` and `$E1AC`) until R4
     joins the loop.
   - The PPU mirrors the stages write: `006F` and `0072` at the turn (`$BE09`: subscreen off,
     colour math on BG1/sprites/backdrop); `0069`, `0055`, `005F`, `0058` and `005D` at the
     station's view (`$C03E`: layers, mode 1, the mode 7 map wrap, BG1 tilemap and tile bases).
     The platform's (E4); not compared.
   - VRAM's mode 7 tilemap and tiles, including the picture swap at the turn (`$BE22`, the back
     of the gunship, queued as a mode 7 transfer): AF5 content drawn by E4; not compared, so no
     check here says the swap falls on the turn's frame.
   - The BG1 tilemap: its emptying at 468 (E2) and the label's letter words (E2 from the `typed`
     output).
   - OAM and the sprites' instruction pointers: AF5's sprite programs, drawn by E1.
6. **The join:** the flight's cut 802 is movie 8,314; the arrival's seed (`any-ceres-arrival`
   cut 0) is movie 8,315. The flight ends `Done`, blanked at 0, which is the arrival's
   `Preparing` brightness; the frame between them (`$8B:C100`, the save) is the arrival row's
   first. The runner needs the same capture-crossing link the arrival needs into room 1.

**Recorded cases** (row `any-fly-to-ceres`, from cut to cut; the vectors keep their State):

| Case | Cuts | Vector |
| --- | --- | --- |
| S1 | 0 → 32 (the setup frame, lag 1-31) | R-setup |
| D1 | 116 → 117 (two sprites' places, with invented drifts) | R-drift |
| S2 | 39 → 40 | R-wait |
| S3 | 83 → 84 | R-appear |
| R1, R2, R3 | 84 → 85, 100 → 101, 115 → 116 | R-rush-first, R-rush-mid, R-turn |
| A1, A2, A3 | 116 → 117, 330 → 331, 467 → 468 | R-away-first, R-away-fast, R-station |
| L1, L2, L3 | 470 → 471, 483 → 484, 771 → 772 | R-label-hold, R-label-letter, R-label-end |
| O1, O2, O3 | 772 → 773, 773 → 774, 801 → 802 | R-fade-count, R-fade-step, R-fade-last |

The label's recorded vectors give invented letters and places with the recorded holds (16, then
128 with no letter); the harness case for the game's label is the row itself, cuts 468-772, with
the label imported from the scene's content.

## Findings behind the spec's choices

- The setup zeroes the plane's fractions, blanks the screen and blacks the flash (`$819B`)
  before it writes the whole pixels (C-setup-lit). The turn writes only the integer words, so its
  fractions are kept (C-turn-fraction). The row's fractions are 0 at both moments, so the
  recorded witnesses can't tell either.
- The scene's placements are content (the setup's angle, zoom and places, the rushing stars'
  speed, the turn's angle, place and sprites with their drifts and wraps), as the label is.
  The game's, for the importer: made ready at angle E0h, zoom 200h, plane (−48h, −68h), the
  stars at (70h, 57h) with speed −400h; the turn at angle 20h, plane (−20h, −80h), the sprites
  (by kind) station (74h, A0h) +0.1000h wrapping, small asteroids (80h, 60h) +0.0800h wrapping,
  large asteroids (50h, 9Fh) +0.4000h wrapping, vortex (E0h, 57h) −0.2000h, distant stars
  (−20h, 57h) −0.2000h.
- The zoom is held to 0 to 32,767: the game compares it signed and multiplies it signed, so a
  larger zoom would turn or step differently; the row's zoom stays 16 to 8,224.
- The transformation is the cinematic handler's (`$A35B` → `$8B:8532`), shared by the intro, both
  flights and the explosion: specified here first, to be lifted to E4.
- The stars' rush runs on the frame the picture appears (the pre-instruction tests the new
  function); the zoom first falls on the next frame (cut 85).
- On the turn's frame the new sprites drift once (the far stars show −33 at 116 from −32).
- The matrix is recomputed every frame in every stage, so it reads zero after the clear at 802.
- The label's letters and holds are content (bank 8C); the spec takes them as an input and
  states only the stepping, as the intro's text pages will. Which label is shown for a language
  (`$8B:C096` skips the Japanese line unless `09E2` is set) is the caller's choice of content.

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here.

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit. There is no old code:
the engine has no flight (the played runner refuses a seed outside ordinary play), so no area
has `old`.

- Area scene: FlightIsMadeReady, PictureAppears, GunshipRushesIn, StarsRushPast, ViewTurnsAway, GunshipFliesAway, FlashFades, SpritesDrift, StationComesIntoView
- Area label: LabelIsTyped
- Area fade: PictureDarkens, FlightEnds
- Area plane: PlaneTransforms

The spec-units counter reports thirteen entries, 24 arms, no cells and 25 operators, and no
findings.
Budget: 37.5 × 32.3 units = 1,212 (scene 700, label 125, fade 250, plane 137)
