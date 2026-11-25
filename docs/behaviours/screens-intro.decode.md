# The intro's last pages: decode and harness

Companion to the [intro's behaviour spec](screens-intro.md). Keep this note outside confined
exports.

## How it was made

Behaviour first, from CI's lean record cache (`$TEMP/collab/ci-cache/rows`): `any-intro-6012`
cuts 0-1,499, read per cut by a scratch reader of the lean tier (`$TEMP/collab/ceres-739/lean.py`,
local only): the cinematic function, BG mode, brightness, the cinematic sprite objects (slot,
instruction pointer, place, timers), the cinematic BG objects (the text page, Samus's eyes),
the text glow objects, the two cross-fade timers, cinematic BG1 Y, the HDMA object list, the
fading palette buffers, the palette, CGRAM, VRAM, the pad's new presses and the music queue.
The bank logs (8B, 8C, 88, 80) were then read to pin the arithmetic.

The reference models (local only, `$TEMP/collab/intro6012-756/scratch/`): `vmodel.py` holds the
spec's rules over a State; `row.py` admits that State from the record at cut 1 (after page 4's
turn has finished) and steps it once per finished frame (a cut whose finished-frame count
`05B6` rose), with the press read from the previous record's new presses (`008F`) and the
scene's content as the importer will give it: the pages and the eyes parsed from the bank
log's text (`content.py`), the baby's and the caret's scripts written from `$8B:CBCD`, `$CBFB`
and `$CC03`, the full colours (`$8C:E3E9`) and the caret's colours (`$8C:E5A9`) read from the
player's ROM, never from the record. The baby and the caret run through the same script
stepper as the pages and the eyes. At every cut from 2 to 1,499 but the lag cuts 129 and 403
(records inside a long frame) it compares: the stage; the dissolve's count (in `ToScene` and
`ToText`); the 256 colours' channel levels and steps and the palette shown; the picture's
height; each script's script, step, hold and repeat count (the page, the eyes, the baby, the
caret), mapped from the record's list pointers; the caret's place and whether it shows; the
glows; the click turn; the clicks asked each frame (the sound library 3 queue's next index,
`0648`, against the model's clicks; 79 in the row); the brightness; the fade's count (in
`FadingOut`).
- `row.py`: 1,496 of 1,496 cuts agree; a do-nothing model (the admitted State held) agrees on 0
  of 1,496.
- `vmodel.py` on the vectors: 41 of 41.
- Planted faults: the dissolve stepping on the wrong quarter (row 1,432 of 1,496, first at cut
  4, colours; vectors 36 of 41); the glow's hold 4 for 5 (row 544 of 1,496, first at cut 5;
  vectors 37 of 41); a repeat one cycle short (row 664 of 1,496, first at cut 362, the stage and
  the baby; vectors 41 of 41, so only the row catches it).
- Not compared: the text layer's tile words, the pictures (content), the music's stop request
  and the text cleared (events with no RAM trace the row model reads).
- The fade's last step, blanked at level 0, falls in the frame recorded as the flight's seed
  (`any-fly-to-ceres` cut 0, brightness 80h, function `$BCA0`, missiles and their capacity 0
  where cut 1,499 holds 896 and 900): the row's own cut 1,499 is still level 1 (case O3).

## Sources

| Entry | Sources |
| --- | --- |
| ScriptSteps | cinematic BG objects `$8B:9659` (pre-instruction, timer − 1, instructions with bit 15 set run at once, else timer and indirect instruction); cinematic sprite objects the same in `$8B:93..94` (`$94BC` go to, `$94C3` decrement timer and go to if non-zero, `$94D6` timer =); BG object `$9698` delete, `$971E` go to |
| PicturesCrossFade | `$8B:B2D2` (to the scene, timer `1A49`) and `$8B:B458` (to the text, timer `1A4B`): when the timer % 4 = 0, `$8C83` fade out (level −= delta) and `$8CB2` fade in ((level + delta) & 1FFFh) by palette range, then `$8CEA` compose (bits 8-12 of each channel to `7E2000` and `7EC000`); the timer − 1; below 0 ends. Deltas from `$8C09` (channel × 100h / 20h) |
| the sets | to the scene: out BG 0, BG 1 colours 0-3 (`X` 0, `Y` 14h), BG 3, sprite 6 colours 9-Eh; in BG 2, sprite 6 colours 0-8. To the text: in BG 0, BG 3, sprite 6 colours 9-Eh; out BG 2, sprite 6 colours 0-8 |
| DissolveEnds | `$B30F`..`$B332` (layers BG1/BG3/sprites, the English text region `7E3100..35FF` = 2Fh, Japanese tiles blanked); `$B495`..`$B4B8` (layers BG2/BG3/sprites, colour math off, sprite palette 6 colours 0-6 = `$8C:E5A9`) |
| BabyRises | `$8B:ADA6` (pre-instruction of `$CE67`): if `1A4B` = 0 delete; else if `1A49` % 4 = 0 and `1997` < 8: `1997` + 1, Y − 1. `1A49` is the to-scene dissolve's count; in the dissolve to the text it holds 127 (`$B3E6`), never a multiple of 4, so the spec's one count and `ToScene` guard give the same frames |
| BabyLeaves | `$8B:ADA6`, the delete branch: `1A4B`, the to-text dissolve's count, at 0 |
| the baby's script | `$8B:CBCD`: timer = Ah; four frames 10 each; decrement and repeat; `$B34E` (start page 5); then the four frames forever |
| PageFiveBegins | `$8B:B392` (spawn `$CF57`, page 5, at VRAM 4C00h; function `$B458`) and `$8B:B3A1` (layers, decompose `$8C:E3E9`, clear BG 0, BG 3, sprite 6 colours 9-Eh, compose, `1A49` = 7Fh, tilemap updates on, `$ADEE` caret reset to (8, 18h) with `$CBFB`) |
| LetterIsTyped | `$8B:884D` (indirect function: toggle `1BA1`; spawn a text glow object `$97F7`; caret (sprite object Fh) to the next instruction's (column × 8, (row − 1) × 8), or (8, (row + 2) × 8 − 8) when the next is an instruction; sound Dh, library 3, if `1BA1` and not `$D67D` (space)); `$B19B`/`$B228` clear `1BA1` |
| LetterGlows | `$8B:9828`/`$9849`: timer − 1; at 0 the letter's tilemap word's palette bits = the glow's palette index; index ≠ C00h → + 400h, timer 5; else the glow cleared; up to 8 (`19F7..1A05`) |
| CaretBlinks | caret lists `$8B:CBFB` (steady: shown 5, go to) and `$CC03` (blinking: shown 5, empty 5, go to), content; `$ADD4` places `$CC03` with timer 1; `$ADEE` places `$CBFB` at (8, 18h) |
| PageEnds | `$8B:B1B3` (caret blinks, function `$B1DA`); page 6's `$ADD4`, hold 80h, `$B240` (function `$B72F`, fade delay and counter 1) |
| PressTurnsThePage | the eyes' switch: `$8B:B4BC` (pre-instruction of `$CF63`) polls the function for `$B207` or `$A390` and places `$8C:D613` with timer 1 on the press's frame. `$8B:B1DA`: the Japanese timer `1BA3` (0 for English); `008F` ≠ 0 → `1A4B` = 7Fh, then (English) `$B207`: tilemap updates on, `$A856` clear the English text tilemap, spawn `$CF5D` (page 6), function `$A390`, `$ADEE`, music stop queued (U1) |
| PictureDarkens | `$8B:B72F` → `$80:8924` (as screens-flight's) |
| IntroEnds | `$8B:B73B`..`$B772`: NMI on, fade delay and counter 0, BG2/3 scrolls 0, clear 198Dh..1C1Fh, function `$BCA0` (the flight), the button bindings restored, missiles 0 (outside this domain) |

## RAM mapping

| Spec | RAM | Notes |
| --- | --- | --- |
| the stage | `1F51`: `$B2D2` ToScene, `$A391` Scene or Text, `$B392` ToPageFive, `$B458` ToText, `$B1DA` WaitingForPress, `$A390` Text (page 6), `$B72F` FadingOut, `$BCA0` Done (the flight's start) | game state 1Eh throughout |
| the colours | levels `7E2400`/`2600`/`2800`, steps `7E2A00`/`2C00`/`2E00`, 256 words each; shown `7EC000` (and `7E2000`) | |
| the dissolve's count | `1A49` in `ToScene`, `1A4B` in `ToText` | `$B3E6` sets `1A49` = 7Fh and the press `1A4B` = 7Fh, neither read in the row; not State in the spec |
| the picture's height | `1997` | |
| the baby | sprite object slot 14: Y `1AB9`, list `1B39`, timer `1B79`, loop count `1B99` | X 70h |
| the page, the eyes | BG object slots 0 and 1: list `19D1`/`19D3`, timers `19E1`/`19E3` | |
| the caret | sprite object slot 15: X `1A9B`, Y `1ABB`, list `1B3B`, timer `1B7B` | |
| the glows | `19F7..1A05` (pointer), `1A07` timer, `1A17` column, `1A27` row, `1A37` palette index (400h per set) | |
| the click turn | `1BA1` | |
| the brightness, the fade | `0051`, `0725`, `0723` | |
| a press (input) | `008F` | the previous record's, read by the next frame |

## Harness (for the integrator)

The row is `any-intro-6012` (seed at movie frame 6,012, 1,500 cuts). What it needs:

1. **Admission at cut 1.** The seed record (cut 0) is taken inside the frame in which page 4's
   press readies the dissolve to the scene (`$B123` → `$B151`); cut 1 is that frame's end.
   Admit the State from cut 1: the stage `ToScene`, the dissolve's count (`1A49`), the colours'
   levels and steps, the picture's height, the baby (its script's step from its list pointer,
   its repeat count none while the game's counter reads 10), the eyes, the caret (off-screen at
   (8, 248), blinking), two glows still running from page 4, the click turn and the brightness.
   Compare from cut 2. The language: English only; a Japanese setting is Unsupported here.
2. **The owner in the loop**, stepped once per finished frame; lag (T1) borrowed; the pad's
   new presses from the movie.
3. **Compare** from cut 2, but at the lag cuts 129 and 403 (records inside a long frame: mask
   all of the intro's fields there, as the flight masks its setup's cuts).
4. **Unproduced streams**, each with its writer: VRAM (the scene's tiles at 129, page 5's at 403,
   the eyes' and the letters' tile words: E2 from the pictures' content); OAM (the baby and the
   caret: E1); the HDMA object (`$88:EBF0`, the colour blend's bands: E3); the PPU layer and
   blend mirrors; the music's stop request (U1's boundary). The clicks are this domain's output,
   compared per cut against the sound library 3 queue (decision 37).
5. **The join:** the row's cut 1,499 is movie 7,511; the flight's seed (`any-fly-to-ceres` cut 0)
   is movie 7,512, in the same capture. The intro's last frame, the blank and `IntroEnds`, is
   the one the flight's seed records (case O3). A played chain runs that frame from the intro's
   State, then the flight's first from its `Starting`; the one value that crosses is the
   equipment returned: missiles and their capacity 0 (`09C6`/`09C8`) and the bindings restored
   (`$BC75`), applied by the owner of her equipment and of the buttons.

**Recorded cases** (row `any-intro-6012`, from cut to cut; the vectors keep their State, each
colour's levels under an invented index, with invented content and the recorded holds):

| Case | Cuts | Vector |
| --- | --- | --- |
| E1, E2, E3 | 5 → 6, 16 → 17, 40 → 41 | R-eyes-hold, R-eyes-next, R-eyes-back |
| B1 | 401 → 402 | R-baby-signal |
| X1, X2 | 4 → 5, 5 → 6 (colours 1 and 33 as 3 and 40) | R-to-scene-step, R-to-scene-hold |
| X3, X4 | 407 → 408, 405 → 406 | R-to-text-step, R-to-text-hold |
| X5 | 128 → 130 (the lag cut 129 inside) | R-to-scene-end |
| X6 | 531 → 532 (the caret's colours as invented content) | R-to-text-end |
| B2, B3 | 7 → 8, 530 → 531 | R-baby-rises, R-baby-leaves |
| P1 | 402 → 404 (the lag cut 403 inside) | R-page5-begins |
| P2, G1 | 404 → 405; 405 → 406, 409 → 410, 1184 → 1185 | R-letter; R-glow-hold, R-glow-step, R-glow-end |
| P3 | 1472 → 1473 | R-caret-blink |
| P5 | 1184 → 1185 | R-page5-end |
| P7 | 1341 → 1342 | R-page6-caret |
| P6 | 1185 → 1186 (the press: the pad's new press at cut 1185) | R-page6-begins |
| P8 | 1469 → 1470 | R-page6-end |
| O1, O2 | 1470 → 1471, 1471 → 1472 | R-fade-count, R-fade-step |
| O3 | 1499 → `any-fly-to-ceres` cut 0 (the blank and the equipment returned fall in the frame the flight's seed records) | R-fade-last |

## Findings behind the spec's choices

- The dissolve's steps are a 32nd of each colour's full channel (`× 8` in 256ths), so 32 steps
  land exactly on 0 or the full level; the brighten wraps at 8,192 (`& 1FFFh`), the dim at
  65,536.
- The game keeps two counts (`1A49`, `1A4B`), one per dissolve, each set to 127 before its
  dissolve; the spec keeps one, set when page 5 begins and admitted for the dissolve to the
  scene. The to-text count `1A4B` is 127 when page 5 begins either way (page 4's press set it).
- The baby's script repeats its four pictures with the game's timer = 10 and decrement-and-go
  (`$94D6`, `$94C3`), then a separate instruction starts page 5 (`$B34E`): in the spec, one
  repeat step (ten times in all) and a signal step.
- The caret is placed for the next letter by each letter, not by its own script.

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here.

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit. There is no old code:
the engine has no intro, so no area has `old`.

- Area scripts: ScriptSteps
- Area dissolve: PicturesDissolve, DissolveEnds
- Area baby: BabyRises, BabyLeaves
- Area pages: PageFiveBegins, LetterIsTyped, LetterGlows, CaretBlinks, PageEnds, PressTurnsThePage
- Area fade: PictureDarkens, IntroEnds

The spec-units counter reports thirteen entries, 36 arms, no cells and 14 operators, and no
findings.
Budget: 37.5 × 40.7 units = 1,523 (scripts 287, dissolve 287, baby 87, pages 612, fade 250)
