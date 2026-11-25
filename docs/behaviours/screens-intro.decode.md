# The intro: decode and harness

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

## The opening and page 1: how it was made

Behaviour first, from the lean record cache (`$SM_CACHE`, row `any-intro-0568`, cuts 0-1,650,
movie 568-2,218), read per cut by a scratch reader of the lean tier
(`$TEMP/collab/intro-early/lean.py`, local only): the cinematic function, brightness, the screen
fade's delay and counter, the cinematic function timer, the music queue timers, the cinematic
BG objects (the page, the eyes), the caret sprite object, the text glow objects, the click flag,
cinematic BG1 Y, the sound library 3 queue and the finished-frame count. The bank logs (8B, 8C,
80) were then read to pin the arithmetic.

The reference models (local only, `$TEMP/collab/intro-early/`): `model.py` holds the rules of
§7 and §8 with the scripts, glows and caret of §2 and §5; `row.py` admits the State `Begins` at
cut 0 (the fade's count 0, as the front end leaves it; the intro's first frame sets it to 3) and steps it once per
finished frame (a cut whose finished-frame count `05B6` rose by 1; it never rises by more), with
"a track waits" read as any music queue timer (`0629..0637`) non-zero in the cut's own record,
as the flight's model reads it, and page 1's script parsed from the bank log's text
(`content.py`: 173 letters, the first step a blank picture, the end signal and the end), the
eyes' and the caret's scripts written from `$8C:D5DF`, `$8B:CBFB` and `$CC03`. At every finished
frame from cut 1 to 1,650 (1,504 frames; cuts 1-53 and 71-104 are inside long frames) it
compares: the stage; the brightness (level and blank); the fade's count (in the slow fades'
stages); the stage's wait (in `OpeningPause`, `FirstTune`, `SecondTune`); from `PageOneLoads` on
the dissolve's count, the picture's height, the page's and the eyes' step and hold (from their
list pointers), the caret's place, script, step and hold, the glows (as a set) and the click
turn.
- `row.py`: 1,504 of 1,504 frames agree; the do-nothing model (the admitted `Begins` held)
  agrees on 0 of 1,504. The clicks asked agree on every frame with the sound library 3 queue's
  index (`0648`) changing: 75 in the row.
- `vmodel.py` (the whole spec's rules, §2-§8) on the vectors: 69 of 69; a press while page 1 waits raises Unsupported.
- Planted faults: the slow fade's rhythm 3 for 2 (row 274 of 1,504, first at cut 105, the
  fade's count; vectors 61 of 69); the stage's waits one frame short (row 106 of 1,504, first at
  cut 194, the stage and the wait); page 1's blank first step typed as a letter (row 642 of
  1,504, first at cut 789, the glows and the click turn).
- Not compared: the text layer's tile words and the opening's words (content drawn by E2), the
  music's requests (U1's boundary, no RAM trace the row model reads), the equipment lent (her
  missiles, their capacity and the button bindings: her owners'), Samus's own initialisation
  (`$91:E00D`, the samus domains', unseen until the fight).

## The flashback and page 2

The hand-on is `any-intro-0568` cut 1,650 (movie 2,218), with page 1 waiting and the new press
latched. The row `any-intro-2219` starts at the next movie cut, inside that press's frame.
The setup is visible in the seed, cut 0 (movie 2,219), but the work frame finishes at cut 1
(movie and power-on cut 2,220). Take the prior intro State and the latched pad into that frame.
The scoped production case compares the intro's battle start request, dissolve preparation and
caret at finished cut 2,220. Integrated power-on needs the Zebes room at that cut.

| Case | Row cut | Witness |
| --- | ---: | --- |
| F1 | 1 | Page 1's press completes. The dissolve sits at 127, the caret is offscreen and the room has been selected. |
| F2 | 2 | The dissolve starts counting down without a colour step. |
| F3 | 5 | The first colour step is taken at count 124. |
| F4 | 132 | The final colour step and transition to the fight are complete. Cut 131 caught the frame before it finished. |
| F5 | 519 | The fight signals page 2. The recorded fourth hit was at cut 391, 128 finished frames before. |
| F6 | 521 | Page 2 shows its first blank step and the steady caret at its home (8, 24). Cut 520 caught that frame before it finished. |
| F7 | 522 | Page 2's first letter, glow and click. |
| F8 | 648 | The return count reaches 0. Samus, her shots and Mother Brain are removed from the picture. |
| F9 | 649 | The return's last colour step completes. Page 2 continues typing. |
| F10 | 1297 | Page 2 finishes and the new press for the next scene is latched. |
| F11 | 518 | The fight still holds and the eyes advance. |

The next input boundary is cut 1,298 (movie 3,517). The baby's discovery setup first finishes
at cut 1,299 (movie 3,518).

**Sources and mapping**

| Entry | Sources and mapping |
| --- | --- |
| BattleBegins | `$8B:AEB8` reads the prior cut's new pad press (`008F`) with the English timer clear. `$B018` selects the split blend, places the caret at (8, 248) via `$ADE1`, and decomposes the full palette at `$8C:E3E9`. It clears the fight's incoming colour ranges and recomposes. On entry the outbound fade count `1A49` is already 127. The press sets the future return count `1A4B` to 127 as well, so the model can use one active count and initialise it at each dissolve. `1A57` is negative while this fight draws Samus and her shots over the cinematic sprites. |
| BattleDissolves | `$8B:B250` fades into the fight and `$B3F4` fades back, with the same fixed-point arithmetic as `PicturesDissolve`. The outgoing text ranges are BG 0 and BG 1 colours 0–3, BG 3 and sprite 6 colours 9–14. The incoming fight ranges are BG 1 colours 4–6, BG 7, sprites 4 and 5, and sprite 7. The return dims those fight ranges and brightens BG 0, BG 3 and sprite 6 colours 9–14. The level/step representation is the shared mapping below. `$B250` clears the text and starts palette FX `$8D:E1BC` at its end. `$B82E` deletes the boss sprite and clears `1A57` when the return count has reached 0, one frame before the return's final colour step. `$B3F4` selects the plain text layers at its end. |
| PageTwoBegins | `$8B:B80F` sets up the cue `$CB19` after 128 shake frames. The cue's `$B336` suspends text tilemap updates and picks the pending page state. `$B35F` places `$CF45` (page 2 at `$8C:C797`, VRAM `4C00h`). `$B3A1` restores the same full palette, clears the return's incoming colours, enables text tilemap updates and uses `$ADEE` to place the caret at home. The first page step is blank with hold 1; `$AE79` clears the click turn. |
| PageTwoEnds | `$8C:C797` ends with `$8B:AE91` and `$9698`: place the blinking caret, set the wait for discovery and delete the page. The next press is read by `$AF6C`. |
| The stage | `1F51`: `$B250` IntoFight, `$A391` Fight before the return signal, `$B35F` PageTwoPending, `$B3F4` OutOfFight, `$A391` Text after the return's last step and `$AF6C` PageTwoWaits. Page 2 uses the same BG slot and timers as page 1. |

**Owners and joins**

- The intro selects the room and asks for its setup. `$AEB8` loads room collision content
  (`$8C:BEC3`), sets Samus's pose and current/previous position, sets the missile selection and
  loads the intro demo object `$91:8784`. Apply the start intent before room play on the same
  work frame. The room owner decodes the collision content. The Samus owners initialise her
  movement and appearance, equipment and weapons apply the selected item and carried missiles,
  and the demo owner supplies the held and newly pressed pad words after the start. Production
  must keep the room, Samus and demo State across the join.
- `$8B:B786` owns Mother Brain's missile contact and consumption. Four contacts start the
  explosion and shake sequence. `$B80F` and `$CB19` supply the return cue after the explosion
  hold. `$B877` writes the shake offset in `1997` in the fight and during its return. `$B896`,
  `$B8D8` and `$B93B` are the rings' initialisation, movement and contact. The encounter,
  projectile and Samus owners must agree on projectile consumption and ring hits. Room play
  steps before the encounter checks contact. The encounter publishes the return cue in the
  same work frame as the last hold, after the intro has stepped the stage. Page 2 starts on
  the following finished frame.
- The first comparison at cut 2,220 covers the intro's setup boundary. Room play runs in that
  work frame too. The demo, room collision, Samus, weapons and encounter are required for
  integrated fight replay. On the return, use the old visibility for Samus's last step at cut
  648 and the new visibility for drawing. A scoped
  intro replay can borrow the encounter's return cue. Its receipt should name that borrow
  and may not claim fight behaviour. Borrow completed frames through the existing platform
  producer. The intervening cuts with no new finished frame are 0, 101, 106, 131 and 520.
- Compare the intro stage, brightness, active dissolve count, fade channel levels and steps,
  page and eyes steps and holds, caret position/step/hold, glows, click turn, fight sprite order
  intent and intro requests. The final palette/CGRAM has encounter writers (Mother Brain's
  flash and the palette FX) as well as the dissolve writer. E2 owns VRAM, E1 owns OAM,
  E3 owns the split line program, and E4 owns layer/scroll mirrors. The demo, gameplay and
  encounter writers own their own State and outputs. Lag cuts do not advance the intro.
- The next owner needs the `PageTwoWaits` State at cut 1,297 carried with the latched press.
  A page 2 press starts part (c), including a different demo and encounter.

**Reference check.** The local copies are `$TEMP/collab/intro-b-obs-notes/reference.py`
and `battle_row.py`. The frame model carries the page 1 State, admits the previous cut's
new press, steps only on `05B6` advances and borrows the fight cue at cut 519. The content
reader imports the full colours and page script from the bank logs locally. It compares the
stage, brightness, active dissolve count, fight visibility, page and eye step and hold, caret
place, script, step and hold, glows and click turn. The flashback scripts use no repeat, so the
witness omits script repeat counts. It also compares the 256 channel levels and
steps, and checks the clicks against the sound library 3 queue's index during page 2. At
the frames that publish the palette it compares the palette mirror `7EC000`, with sprite
palette 7 on cuts 111 and 115 masked for Mother Brain's hurt flash. It leaves the fight's
shake offset and final CGRAM to their owners.

- State and fade buffers agree at 1,293 of 1,293 finished cuts. A held initial State agrees
  at 0 of 1,293. The 67 page 2 clicks agree across all 777 finished page 2 cuts. With that
  mask, the palette mirror agrees at all 66 publication cuts.
- The vector model agrees on 86 of 86 vectors. Seventeen cover the flashback and page 2, ten
  recorded and seven constructed. Constructed vectors check colour publication at both dissolve
  endpoints.
- A one-frame-short outbound dissolve fails at cut 1 in the dissolve count and at cut 4 in
  the colour levels. These are the first disagreements for their comparisons.

## Sources

| Entry | Sources |
| --- | --- |
| ScriptSteps | cinematic BG objects `$8B:9659` (pre-instruction, timer − 1, instructions with bit 15 set run at once, else timer and indirect instruction); cinematic sprite objects the same in `$8B:93..94` (`$94BC` go to, `$94C3` decrement timer and go to if non-zero, `$94D6` timer =); BG object `$9698` delete, `$971E` go to |
| PicturesDissolve | `$8B:B2D2` (to the scene, timer `1A49`) and `$8B:B458` (to the text, timer `1A4B`): when the timer % 4 = 0, `$8C83` fade out (level −= delta) and `$8CB2` fade in ((level + delta) & 1FFFh) by palette range, then `$8CEA` compose (bits 8-12 of each channel to `7E2000` and `7EC000`); the timer − 1; below 0 ends. Deltas from `$8C09` (channel × 100h / 20h) |
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
| PressTurnsThePage | the eyes' switch: `$8B:B4BC` (pre-instruction of `$CF63`) polls the function for `$B207` or `$A390` and places `$8C:D613` with timer 1 on the press's frame. `$8B:B1DA`: the Japanese timer `1BA3` (0 for English); `008F` ≠ 0 → `1A4B` = 7Fh, then (English) `$B207`: tilemap updates on, `$A856` clear the English text tilemap, spawn `$CF5D` (page 6), function `$A390`, `$ADEE`, music stop queued through `$80:8FC1` with an 8-frame delay (U1) |
| PictureDarkens | `$8B:B72F` → `$80:8924` (as screens-flight's) |
| IntroBegins | `$8B:A395` (cinematic function, intro, initial): `$80DA` PPU set up for the intro (brightness 80h), `1A57` = 0, Samus initialised (`$91:E00D`), missiles and capacity 900 (`09C6`/`09C8`), bindings reset (`$BC08`), screen fade delay and counter (`0723`/`0725`) = 3 (`$A54D`..`$A553`), `198D..1C1F` cleared, the intro palettes, decompression and VRAM uploads (the long frame, cuts 1-53), `1B9F` = FFFFh, next function `$A5A7`, music stop and music data `$FF3F` queued (U1) |
| SlowFade | `$8B:911B` (slow fade in: `0723` − 1, 8-bit; above 0 return; else brightness + 1 (& 1Fh), at Fh or more brightness Fh, delay and counter 0, done; else delay = counter) and `$8B:90D5` (slow fade out: the same count; level 0 done; level − 1 = 0 → 80h, delay and counter 0, done; else level − 1, delay = counter). Counter 2 from `$A5A7`, `$A64C`, `$A82B` |
| OpeningUnfolds | `$8B:A5A7` (`$80:8EF4` any music queue timer non-zero → return; else `$A5BD`, delay = counter = 2); `$A5BD` (slow fade in; done → `$A5F8`, `1A49` = 3Ch; the Japanese text queued only when `09E2` ≠ 0); `$A5F8` (`1A49` − 1, 16-bit; ≤ 0 → `$A613`, `1A49` = C8h, track 5 queued); `$A613` (the same; → `$A639`, music stop, data `$FF42`, track 5 with a 14-frame delay); `$A639` (queue wait → `$A64C`, `1A49` = F0h); `$A64C` (→ `$A663`, delay = counter = 2); `$A663` (slow fade out; done → `$A66F`) |
| PageOneIsMadeReady | `$8B:A66F`: `$A86A` Japanese tiles blanked, layers BG2/BG3/sprites, BG objects cleared and enabled, tilemap updates on, the margins and Japanese region of `7E3000`, spawn `$CE6D` (the caret), `1A49` = 7Fh, spawn `$CF63` (the eyes, `$8C:D5DF`) for VRAM 4C00h, `1997` = BG2/BG3 Y scroll = 8, `$A856` clear the English text, `$ADEE` caret reset (8, 18h) on `$CBFB`, music stop, data `$FF36`, track 5 with a 14-frame delay; next `$A82B` |
| PageOneBegins | `$8B:A82B` (queue wait → `$A84A`, delay = counter = 2, spawn `$CF3F`, page 1 `$8C:C383`, for 4C00h); `$A84A` (slow fade in; done → `$A391`). Page 1's first instruction `$AE43` clears `1BA1` (and spawns the Japanese page only when `09E2` ≠ 0); its first entry is `$D683`, "nothing", hold 1: no glow, no click turn, no caret move |
| PageEnds (page 1) | `$8B:AE5B` (page 1's last instruction): `$ADD4` caret blinks, the Japanese arrow only when `09E2` ≠ 0, function `$AEB8`; then `$9698` deletes the page. `$AEB8` reads the press (after the Japanese timer `1BA3`, 0 for English) and sets up the old Mother Brain fight: the next part |
| IntroEnds | `$8B:B73B`..`$B772`: NMI on, fade delay and counter 0, BG2/3 scrolls 0, clear 198Dh..1C1Fh, function `$BCA0` (the flight), the button bindings restored, missiles 0 (outside this domain) |

## RAM mapping

| Spec | RAM | Notes |
| --- | --- | --- |
| the stage | `1F51`: `$A395` Begins, `$A5A7` OpeningLoads, `$A5BD` OpeningBrightens, `$A5F8` OpeningPause, `$A613` FirstTune, `$A639` SecondTuneLoads, `$A64C` SecondTune, `$A663` OpeningDarkens, `$A66F` PageOneMadeReady, `$A82B` PageOneLoads, `$A84A` PageOneBrightens, `$A391` Text (page 1), `$AEB8` PageOneWaits; `$B2D2` ToScene, `$A391` Scene or Text, `$B392` ToPageFive, `$B458` ToText, `$B1DA` WaitingForPress, `$A390` Text (page 6), `$B72F` FadingOut, `$BCA0` Done (the flight's start) | game state 1Eh throughout |
| the colours | levels `7E2400`/`2600`/`2800`, steps `7E2A00`/`2C00`/`2E00`, 256 words each; shown `7EC000` (and `7E2000`) | |
| the stage's wait | `1A49` in `OpeningPause`, `FirstTune` and `SecondTune` | the same word as the dissolve's count |
| a track waits (input) | music queue timers `0629..0637`, any non-zero | U1's reply, borrowed |
| the dissolve's count | `1A49` in `ToScene` and from `PageOneMadeReady` on, `1A4B` in `ToText` | `$B3E6` sets `1A49` = 7Fh and the press `1A4B` = 7Fh, neither read in the row; not State in the spec |
| the picture's height | `1997` | |
| the baby | sprite object slot 14: Y `1AB9`, list `1B39`, timer `1B79`, loop count `1B99` | X 70h |
| the page, the eyes | BG object slots 0 and 1: list `19D1`/`19D3`, timers `19E1`/`19E3` | |
| the caret | sprite object slot 15: X `1A9B`, Y `1ABB`, list `1B3B`, timer `1B7B` | |
| the glows | `19F7..1A05` (pointer), `1A07` timer, `1A17` column, `1A27` row, `1A37` palette index (400h per set) | |
| the click turn | `1BA1` | |
| the brightness, the fade | `0051`, `0725`, `0723` | the slow fades step on `0723` and reload it from `0725` (2) |
| the page in page 1 | BG object slot 4 (`19D1`, `19E1`); its pointer is the entry after the current one, `$8C:C383` when placed | the eyes are slot 6 (`19D3`), spawned first |
| a press (input) | `008F` | the previous record's, read by the next frame |

## Harness: the opening and page 1 (for the integrator)

The row is `any-intro-0568` (seed at movie frame 568, 1,651 cuts, cuts 0-1,650 are movie
568-2,218). The played run is `any-intro-0568` from 0 to 1,650: everything this part specs.

1. **Admission.** Two ways in, the same State: from the front end's hand-on (the menus' round
   ends `any-power-on` with `HandedOn(IntroBegins)` at its cut 568, the frame this row's seed
   records), or from this row's seed (cut 0, cinematic function `$A395`): the stage `Begins`,
   the brightness `Blanked` at 0, the fade's count 0 (`0723`, as the front end leaves it; `IntroBegins` sets 3), everything else
   at rest. Decode nothing else from the seed. The language: English (`09E2` = 0); a Japanese
   setting is Unsupported at the intro's start.
2. **The owner in the loop:** the intro, stepped once per finished frame (`05B6` rises). Lag
   (T1) borrowed: cuts 1-53 (the setup's long frame) and 71-104 (a long frame while the first
   tune loads) are inside frames; mask the intro's fields there.
3. **The borrowed input:** "a track waits", through `BORROWED`'s existing producer for the
   flight ("audio: whether a track waits in the queue of tracks as the frame of work begins"),
   not a second reader. It decides `OpeningLoads` (false first at cut 105), `SecondTuneLoads`
   (cut 451) and `PageOneLoads` (cut 789). The music requests (`$A395`, `$A5F8`, `$A613`,
   `$A66F`) are this domain's output to U1, not compared until U1 joins the loop. Each goes in
   the queue with its timer: 8 frames through `$80:8FC1` (the stops, the loads and the first
   tune's play at `$A5F8`), max(Y, 8) = 14 through `$80:8FF7` (the second tune's and the pages'
   plays); the spec's "play 8" and "play 14".
4. **Compare at every finished frame from cut 54 to 1,650:** the fields `row.py` compares (the
   RAM mapping above). The clicks against the sound library 3 queue (75 in the row), as the
   last row's harness does.
5. **Unproduced streams,** each with its writer: VRAM (the intro's tiles, the opening's words,
   page 1's letters and the eyes: E2 from content); CGRAM (the intro's palettes loaded at the
   setup: content through T1's load); OAM (the caret: E1); the PPU layer and scroll mirrors
   (`0069`, `00B7`, `00BB`: E3/E4's); her RAM set by `$91:E00D` and the equipment lent
   (`09C6`/`09C8`, the bindings): their owners', none in the loop in this part.
6. **The joins.** At the start, cut 568 of `any-power-on` hands on `IntroBegins`: the chain
   builds the intro's `Begins` State there, with the fade's count 0. At the
   end, the run stops at cut 1,650 (`PageOneWaits`); the press that follows is part (b)'s, so
   the next row (`any-intro-2219`, its cut 0 the fight's first cross-fade frame) needs (b) and
   (c), and `any-intro-4323` needs (d). The landed run `any-intro-6012` (movie 6,013-7,511)
   joins once (d) reaches movie 6,011: its State at 6,012 is the admission its harness builds
   from cut 1 (page 4's turn, `ToScene`), and a chain from 568 replaces that admission with the
   State (d) leaves.

## Harness: the last pages (for the integrator)

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

**Recorded cases, the opening and page 1** (row `any-intro-0568`, from cut to cut; the
vectors keep the recorded State with invented content and the recorded holds):

| Case | Cuts | Vector |
| --- | --- | --- |
| S0 | 0 → 54 (the setup's long frame inside) | R-begins |
| S1, S2 | 55 → 56, 70 → 105 (the queue's long frame inside) | R-opening-loads, R-opening-loaded |
| S3, S4 | 105 → 106, 106 → 107, 134 → 135 | R-brighten-count, R-brighten-step, R-brighten-last |
| S5, S6 | 135 → 136, 194 → 195 | R-pause, R-first-tune |
| S7 | 394 → 395, 450 → 451 | R-second-tune, R-second-loaded |
| S8 | 690 → 691, 691 → 692, 692 → 693 | R-darken-begins, R-darken-count, R-darken-step |
| S9 | 720 → 721 | R-darken-last |
| W1 | 721 → 722 | R-page1-ready |
| W2, W3 | 787 → 788, 788 → 789 | R-page1-loads, R-page1-begins |
| W4 | 818 → 819 | R-page1-text |
| W8 | 1649 → 1650 | R-page1-end |

**Recorded cases, the last pages** (row `any-intro-6012`, from cut to cut; the vectors keep their State, each
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
the engine had no intro before this model, so no area has `old`. The flashback replaces no
room-play code. The existing intro model at `b7dc44c8e` scores 4,050.25 using `tools size
crates/clean/screens_intro/src`.

- Area scripts: ScriptSteps
- Area dissolve: PicturesDissolve, DissolveEnds
- Area baby: BabyRises, BabyLeaves
- Area pages: PageFiveBegins, LetterIsTyped, LetterGlows, CaretBlinks, PageEnds, PressTurnsThePage
- Area fade: PictureDarkens, IntroEnds
- Area opening: IntroBegins, SlowFade, OpeningUnfolds
- Area page one: PageOneIsMadeReady, PageOneBegins
- Area flashback: BattleBegins, BattleDissolves, PageTwoBegins, PageTwoEnds

The spec-units counter reports 22 entries, 67 arms, no cells and 17 operators.
Budget: 37.5 × 72.7 units = 2,723 (scripts 287, dissolve 287, baby 87, pages 687, fade 250, opening 600, page one 150, flashback 375)
