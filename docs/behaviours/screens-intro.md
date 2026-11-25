# The intro: behaviour spec

What the player sees at the start of the intro: the screen stays dark while the first tune
loads, then slowly brightens on the opening's words; they hold while the first tune plays and the
second follows, then the picture slowly darkens. Page 1 is made ready in the dark, and once its
tune has loaded it slowly brightens while its text is typed beside Samus's blinking eyes; at its
end the caret blinks. The next press starts the flashback to the battle on Zebes. The fight
dissolves back to the text while page 2 types. At the end of the intro, page 4's text dissolves
into a picture of the baby being examined, the baby rising into view as the picture settles; Samus's eyes
blink beside the text; the picture dissolves back to text and page 5 is typed out letter by
letter, each letter glowing as it appears, with a click every other letter and a caret running
ahead; the caret blinks and the game waits for a button; the press clears the text and page 6 is
typed ("CERES STATION WAS UNDER ATTACK!!"), Samus's eyes now staring; after a pause the
picture fades to black and the flight to Ceres begins. Written behaviour first, from the oracle's
records over time. Vectors:
[`scenarios/vectors/screens-intro.toml`](../../scenarios/vectors/screens-intro.toml).

## Header

**Scope: the intro's model (the plan's F2), in parts.** Specced: the intro's opening and page 1,
from the intro's start to the wait for the press that starts the battle on Zebes (§7,
§8), the flashback and the return to page 2 (§9), and the intro's last row, from page 4's
turn to its end (§2-§6). The intro's own State:
where the scene has got to, the stage's wait, the dissolve's colours and its count, the picture's
height, the baby, the page being typed, the caret, the glowing letters, Samus's eyes, the
screen's brightness, the fade's count and whether the fight is still visible. The baby's
discovery, page 3, the baby delivered and page 4 are follow-on work (§12).

**The shared shape.** The intro is built from four mechanisms, each in a player's terms:
1. **Picture scripts** (§2): a script is a list of steps, each showing a picture (a letter, a
   look of the eyes, a frame of the baby, the caret) at its place and holding it for its frames;
   a step can go back to an earlier one, repeat a stretch a set count, or hand the
   scene a signal. The pages, the eyes, the baby and the caret are scripts; a page adds the
   caret's place, the glow and the click (§5). The flight's typed label (screens-flight
   `LabelIsTyped`) is a page without caret, glow or click, so it can later run as a picture
   script with no change of behaviour.
2. **The dissolve** (§3): one set of colours dims to black while another brightens from black,
   in 32 even steps, one every 4 frames.
3. **The brightness fades:** the fade out at the end (§6), the same rule as the flight's fade
   out, which a later piece can make one entry; the slow fades in and out of the opening and
   page 1 (§7, §8), one level every 2 frames.
4. **Moving pictures:** the baby's rise here (§4); the flight's drifts and rush are its own.

**What it reads.**
- **The pad's new presses** (the movie's input): whether any button was pressed anew this frame.
- **Whether a track waits** (audio's reply, U1): whether a tune asked for is still waiting to
  load and start, read by the opening's and page 1's waits.
- **The fight is ready to dissolve back to text** (the flashback fight's signal): its explosions have held for their closing interval.
- **The language** (the options' setting): this piece plays the English pages; a Japanese
  setting is Unsupported at the intro's start.
- **The scene's content** (in our format): the scripts (the pages' letters, places and holds,
  including pages 1 and 2, the eyes', the baby's and the caret's two), the full colours of the scene's 256 colours, which
  colours belong to the text and which to the picture in each dissolve, the colours the caret
  takes once the text is back, and the caret's home.

**What it hands the others, each frame:** the screen's 256 colours, the picture's height, the
pictures the scripts show and where (the letters with their colour sets, the eyes, the baby, the
caret), the brightness, to the renderer; each click asked for and the music's stop at page 6,
to audio (U1), and the opening's and page 1's music requests (§7, §8); on the frame it begins,
that the equipment for the played scenes is lent; on the frame it ends, that it is done, and that the equipment lent for the
played scenes is returned: the next frame is the flight's (screens-flight `FlightIsMadeReady`).
During the flashback, the intro asks production to start the encounter, chooses how the fight
and text are shown, and tells the renderer when to hide Samus and her shots.

**What it leaves.** The pictures themselves are content drawn by the platform; the line split
of the colour blend during a dissolve is the platform's line program (E3); the lag frames are
the platform's (T1); playing the click and stopping the music are audio's (U1).

**What it replaces.** The intro's opening, page 1 and last pages keep their existing rules. The
flashback extends the intro at page 1's press boundary.

**Notation.** Places on the text layer are columns and rows of 8 px; screen places are pixels,
x rightward and y downward, 16-bit. A colour is three channels (red, green, blue); a channel's
level is in 256ths of a level, 0 to 8,191 (31 is `31 × 256`), and the colour shown is each
channel's whole levels, `level / 256` modulo 32. Frames are frames of the intro; a frame here
is one the game finishes (a lag frame is the platform's and carries nothing of this domain
over).

---

## 1. What the player can tell

### 1.1 Quantities

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| the intro's stage | `IntroStage`: the opening's `Begins`, `OpeningLoads`, `OpeningBrightens`, `OpeningPause`, `FirstTune`, `SecondTuneLoads`, `SecondTune` or `OpeningDarkens`; page 1's `PageOneMadeReady`, `PageOneLoads`, `PageOneBrightens` or `PageOneWaits`; `IntoFight`, `Fight`, `PageTwoPending`, `OutOfFight` or `PageTwoWaits`; `ToScene`, `Scene`, `ToPageFive` (the baby has signalled page 5), `ToText`, `Text`, `WaitingForPress`, `FadingOut` or `Done` | what the screen shows | every entry |
| the stage's wait | frames, 0 to 32,768; 65,535 only after a hold that ran out from 0, in a stage that does not read it | how long the opening holds each part | `OpeningUnfolds` |
| the colours | 256 colours, each three channel levels in 256ths, 16-bit | the picture's colours | `PicturesDissolve`, `BattleDissolves`; output (the colours shown) |
| the colours' steps | 256 colours, each three steps in 256ths: a colour's full level × 8, so 32 steps make it full | how fast each colour fades | `PicturesDissolve`, `BattleDissolves` |
| the dissolve's count | frames, 0 to 65,535 | when the dissolve steps and ends | `PicturesDissolve`, `BattleDissolves`, `BabyRises`, `BabyLeaves` |
| the fight visible | `bool`: whether Samus and her shots appear in front of the fight sprites | `BattleDissolves`; output (the sprite drawing) |
| the picture's height | px, signed 16-bit | where the scene's picture stands | `BabyRises`; output (the scroll) |
| the baby | its y, px, 16-bit (x 112), and its script's place, or none | where the baby is | `BabyRises`, `BabyLeaves`, `ScriptSteps` |
| a script's place | for each script (the page, the eyes, the baby, the caret): which script, its step, the frames its picture still holds (0 to 65,535), and the count left of a repeat it is in, or none | which picture shows and when it changes | `ScriptSteps` |
| the caret | its place, px, and its script's place | where the next letter goes and whether the caret shows | `LetterIsTyped`, `CaretBlinks` |
| the glowing letters | up to 8, each its place (column, row), the colour set it takes next (0 to 3) and frames left | how bright each new letter is | `LetterGlows` |
| the click turn | `bool`: whether the next letter's turn clicks | which letters click | `LetterIsTyped` |
| the screen's brightness | `Brightness`: a level, 0 to 15 (0 black, 15 full), and `Blanked` (nothing shown at any level) or `Shown` | how bright the picture is | `PictureDarkens`, `IntroEnds`; output |
| the fade's count | frames, 0 to 32,768; 0 to 2 in the slow fades' stages | when the picture next darkens or brightens | `PictureDarkens`, `SlowFade` |
| the fade's rhythm | frames, `d` (`PictureDarkens`): a rule, not State | how many frames each level holds | `PageEnds`, `PictureDarkens` |

### 1.2 What it remembers: state a caller can hold

Every quantity of §1.1 but the rhythm: nothing else. A caller can build any of them in any value
this spec allows, read them back, carry them and restore them; a vector's `before` builds them
and its `after` reads them back.

### 1.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| a press | `bool`: any button pressed anew this frame (the movie's pad) | `PressTurnsThePage`, `PageEnds`, `BattleBegins` |
| the fight ready to return | `bool`: event received from the flashback fight after the ordinary stage step of the same finished frame | `PageTwoBegins` |
| a track waits | `bool`: a tune asked for still waits to load and start (audio's reply, U1) | `OpeningUnfolds`, `PageOneBegins` |
| the language | the options' setting: English (a Japanese setting is Unsupported) | the caller, at the intro's start |
| the scripts | content: each a list of steps (§2): the pages 1, 2, 5 and 6, the eyes' two (blinking, page 6's), the baby's, the caret's two (steady, blinking) | `ScriptSteps`, `LetterIsTyped`, `CaretBlinks` |
| the full colours | content: the scene's 256 colours as the dissolves head for them | `PageFiveBegins`, `BattleBegins`, `PageTwoBegins` |
| the dissolve's sets | content: for each dissolve, the colours that dim and those that brighten; for page 5, the text's colours that start black | `PicturesDissolve`, `BattleBegins`, `BattleDissolves`, `PageTwoBegins`, `PageFiveBegins` |
| the caret's colours | content: the colours, by index, the caret takes once the text is back | `DissolveEnds` |
| the caret's home | content: its place as a page begins, px | `PageFiveBegins`, `PressTurnsThePage`, `PageTwoBegins` |

### 1.4 What it reports (Outputs)

| Output | Type | Read by |
| --- | --- | --- |
| the colours shown | 256 15-bit colours: each channel's `level / 256` modulo 32, taken on each step of a dissolve | output (the renderer) |
| the pictures | each script's picture and place; each letter with its colour set; whether the caret shows | output (the tile writing, the sprite drawing) |
| the text cleared | an event | output (the tile writing) |
| a click | an event, per letter that clicks | U1 (the sound's request) |
| the brightness | `Brightness`, every frame | output |
| the music's requests | events, in order: stop the music, load a tune (the opening's first or second, or the pages'), play the loaded tune after a delay, frames; each request is asked with a delay of 8 frames unless it names a longer one | U1 |
| the equipment lent | an event, as the intro begins: her missiles and their capacity 900 and the default buttons, for the played scenes | the owners of her equipment and of the buttons |
| the equipment returned | an event, as the intro ends: her missiles and their capacity 0 and the player's own buttons back, undoing what the played scenes lent | the owners of her equipment and of the buttons |
| the flashback request | `StartZebesFight` or `HideFightSamusAndShots`, events. Production applies the start before room play and the hide after room play, before drawing | production and the fight owners |
| the flashing lights begin | event as the dissolve into the fight finishes | the fight's light effect |
| the picture layout | event choosing the split blend, the plain fight picture or the plain text picture | E3 and E4 |
| the text updates | an event that pauses or resumes publishing text tiles | E2 |
| the intro done | `bool`: the stage is `Done` | the caller: the next frame is the flight's |

---

## 2. Picture scripts

### ScriptSteps
- kind: mechanic.
- summary: a script shows one picture at a time; when its hold runs out it moves on to its next
  step, going back or repeating where the step says, or handing the scene a signal.
- params: none (the steps are content).
- switch: Fixed.
- A script whose hold `h` is over 1 → the hold `h − 1` [R-eyes-hold]
- The hold 1, the next step a picture (or a step showing nothing) → that picture shown at its
  place, the hold its hold [R-eyes-next, C-script-picture]
- The hold 1, the next step "back to step `k`" → on from step `k`, in the same frame
  [R-eyes-back, C-script-back]
- The hold 1, the next step "repeat from step `k`, `n` times in all", its count left `c` (`n` if
  none yet) over 1 → the count `c − 1`, on from step `k` [C-script-repeat]
- The hold 1, the next step "repeat from step `k`", its count left 1 → the count cleared, on to
  the step after it [R-baby-signal, C-script-repeat-last]
- The hold 1, the next step a signal → the signal handed to the scene, on to the step after
  [R-baby-signal, R-page5-end]
- The hold 1, the next step "end" → the script removed [R-page5-end]
- A script is placed at its start with its hold 1, so its first step comes the next time it
  runs: on the same frame for a script placed by the stage's step (§10), on the next for one
  placed by another script's signal [R-page6-begins, R-page5-begins, R-page5-end]
- checked: oracle (recorded cases E1-E3, B1, P5, W1-W3); constructed.

## 3. The dissolves

### PicturesDissolve
- kind: mechanic.
- summary: in a dissolve, every fourth frame each colour of one set dims by its step and each
  of the other brightens by its step, and the screen shows the new colours; after 128 frames the
  dissolve ends.
- params: the beat, every 4 frames (the sets are the scene's content).
- switch: Fixed.
- `ToScene`, the dissolve's count `n` a multiple of 4 → the "to the scene" dimming set dims, its
  brightening set brightens, the colours shown are all taken; the count `n − 1` [R-to-scene-step]
- `ToText`, the count `n` a multiple of 4 → the "to the text" brightening set brightens, its
  dimming set dims, the colours shown are all taken; the count `n − 1` [R-to-text-step]
- Either dissolve, its count `n` not a multiple of 4 → the count `n − 1`, the colours left
  [R-to-scene-hold, R-to-text-hold]
- A colour that dims → each channel `level − step`, modulo 65,536 [R-to-scene-step, C-dim-wraps]
- A colour that brightens → each channel `(level + step)` modulo 8,192 [R-to-scene-step,
  C-dim-wraps]
- So from a colour at its full level, 32 dims bring it exactly to 0, and from 0, 32 brightens
  exactly to its full level, since the step is a 32nd of it.
- checked: oracle (recorded cases X1-X4, every frame of cuts 2-128 and 404-531); constructed.

### DissolveEnds
- kind: mechanic.
- summary: when a dissolve's count runs past 0 it ends: to the scene, the text is cleared; to
  the text, the caret takes its colours.
- params: none.
- switch: Fixed.
- `ToScene`, the count was 0 → the stage `Scene`, the count 65,535; the text cleared
  [R-to-scene-end]
- `ToText`, the count was 0 → the stage `Text`, the count 65,535; the caret's colours shown,
  their levels left [R-to-text-end]
- checked: oracle (recorded cases X5, X6).

## 4. The baby

### BabyRises
- kind: mechanic.
- summary: as the picture dissolves to the scene, on each frame that leaves the dissolve's count
  a multiple of 4 the picture moves down a pixel and the baby rises a pixel, until the
  picture's height reaches 8.
- params: the height it stops at, 8 px.
- switch: Fixed.
- `ToScene`, the baby placed, the dissolve's count a multiple of 4 after this frame's dissolve,
  the picture's height `y` under 8 → the height `y + 1`, the baby's y − 1 [R-baby-rises,
  C-baby-at-eight]
- checked: oracle (recorded case B2); constructed.

### BabyLeaves
- kind: mechanic.
- summary: once the dissolve back to the text reaches its last count, the baby is gone.
- params: none.
- switch: Fixed.
- `ToText`, the baby placed, the dissolve's count 0 after this frame's dissolve → the baby
  removed [R-baby-leaves]
- checked: oracle (recorded case B3).

## 5. The pages

### PageFiveBegins
- kind: mechanic.
- summary: the baby's script signals page 5: on the next frame the page's script is placed and
  the picture starts back toward the text, its text colours black and every other colour at
  its full.
- params: the dissolve's count 127.
- switch: Fixed.
- `Scene`, the signal "page 5" → the stage `ToPageFive` [R-baby-signal]
- `ToPageFive` → the stage `ToText`; page 5's script placed, the click turn off; every colour at
  its full level and its step its full level × 8, then the text's colours at 0, and every
  colour shown taken; the dissolve's count 127; the caret at its home on its steady script
  [R-page5-begins, C-page-begins]
- checked: oracle (recorded cases B1, P1); constructed.

### LetterIsTyped
- kind: mechanic.
- summary: each letter of a page appears as its step begins, glowing; the caret moves to where
  the next letter goes; every other letter clicks, spaces silent.
- params: the caret's place, `(8 × column, 8 × row − 8)`; at most 8 letters glow at once.
- switch: Fixed.
- A page's step begins with a letter at `(column, row)`, the click turn `t` → the click turn
  `not t`; a click asked if the turn is now on and the letter is not a space; a glow placed at
  `(column, row)` taking set 0, with 1 frame left, if fewer than 8 glow [R-letter,
  C-letter-space, C-letter-glows-full]
- The same, the page's next step a letter at `(c′, r′)` → the caret at `(8 × c′, 8 × r′ − 8)`
  [R-letter]
- The same, the page's next step not a letter → the caret at `(8, 8 × row + 8)` [C-letter-last]
- So a letter typed while 8 glow keeps colour set 0 and never glows (C-letter-glows-full).
- checked: oracle (recorded case P2); constructed.

### LetterGlows
- kind: mechanic.
- summary: a new letter glows through its colour sets: set 0 in the frame it appears, then sets
  1, 2 and 3 five frames apart; it keeps set 3.
- params: the glow's hold, 5 frames.
- switch: Fixed.
- A glow with `n` over 1 frames left → `n − 1` [R-glow-hold]
- A glow with 1 frame left, its next set `s` under 3 → the letter shown in set `s`, the glow's
  next set `s + 1`, 5 frames left [R-glow-step, R-letter]
- A glow with 1 frame left, its next set 3 → the letter shown in set 3, the glow gone
  [R-glow-end]
- checked: oracle (recorded case G1); constructed.

### CaretBlinks
- kind: mechanic.
- summary: the caret runs its steady script while a page types and its blinking script once
  the page waits; it shows whenever its script's step is the caret's picture.
- params: none (the scripts and their holds are content).
- switch: Fixed.
- A page's signal "wait for a press", "wait for a press, then the fight" or "caret blinks" → the
  caret's blinking script placed at its start, its place kept [R-page5-end, R-page6-caret,
  R-page1-end]
- The caret's script's current step the caret's picture → the caret shown [R-caret-blink]
- The caret's script's current step showing nothing → the caret hidden [C-caret-steady]
- checked: oracle (recorded cases P3, P5, P7, W8); constructed.

### PageEnds
- kind: mechanic.
- summary: at the end of page 1 and of page 5 the game waits for a press; at the end of page 6
  the page holds, and the picture starts to fade out.
- params: none (the signals are the page's content).
- switch: Fixed.
- The signal "wait for a press" → the stage `WaitingForPress` [R-page5-end]
- The signal "wait for a press, then the fight" → the stage `PageOneWaits` [R-page1-end]
- `PageOneWaits`, no press → the stage left; the scripts run [C-page1-waits]
- The signal "finish" → the stage `FadingOut`, the fade's count `d` [R-page6-end, C-finish-adjusted-end]
- checked: oracle (recorded cases P5, P8, W8); constructed.

### PressTurnsThePage
- kind: mechanic.
- summary: while the game waits, a press clears the text, page 6 begins, Samus's eyes start to
  stare, and the music is asked to stop.
- params: none.
- switch: Fixed.
- `WaitingForPress`, a press → the stage `Text`; the text cleared; page 6's script placed, the
  click turn off; the caret at its home on its steady script; the eyes on page 6's script from
  its start; one stop on the ordered music requests, with an 8-frame delay [R-page6-begins]
- While the game waits and no button is pressed, nothing of the page changes (C-wait).
- checked: oracle (recorded case P6); constructed.

## 6. The fade out and the end

### PictureDarkens
- kind: mechanic and parameter.
- summary: the picture darkens one level every `d + 1` frames; at level 1 the next step blanks
  it.
- params: the rhythm, `d` = 1 frame; a rule from 0 to 32,768 frames.
- switch: **Parameter**, the rhythm (params).
- `FadingOut`, the count `n` over 0 → the count `n − 1`, the brightness left [R-fade-count]
- `FadingOut`, the count 0, the level `l` over 1 → the count `d`, the level `l − 1`, `Shown`
  [R-fade-step, C-fade-adjusted]
- `FadingOut`, the count 0, the level 1 → the count `d`, the level 0, `Blanked` [R-fade-last]
- `FadingOut`, the count 0, the level 0 → the count `d`, the brightness left, `Blanked` or
  `Shown` [C-fade-at-zero]
- checked: oracle (recorded cases O1-O3); constructed.

### IntroEnds
- kind: mechanic.
- summary: once the picture is black and blanked, the intro is over: everything of the scene is
  cleared, the equipment lent for the played scenes is returned, and the next frame is the
  flight's.
- params: none.
- switch: Fixed.
- `FadingOut`, after the fade's step, the brightness `Blanked` at level 0 → the stage `Done`;
  the dissolve's count 0, the picture's height 0, no baby, no page, no eyes, no caret, no glow,
  the click turn off, the fade's count 0; the equipment returned [R-fade-last]
- `FadingOut`, after the fade's step, any other brightness → the stage stays `FadingOut`
  [R-fade-step, C-fade-at-zero]
- checked: oracle (recorded case O3).

---

## 7. The opening

### IntroBegins
- kind: mechanic.
- summary: the intro's first frame clears the scene, keeps the screen dark, asks for the first
  tune and lends the equipment the played scenes use.
- params: none.
- switch: Fixed.
- `Begins` → the stage `OpeningLoads`; the dissolve's count 0, the picture's height 0, no baby,
  no page, no eyes, no caret, no glow, the click turn off, the stage's wait 0, the brightness
  `Blanked` at level 0, the fade's count 3; the music stopped and the first tune asked to
  load; the equipment lent [R-begins, C-begins-clears]
- checked: oracle (recorded case S0); constructed.

### SlowFade
- kind: mechanic.
- summary: in the opening and as page 1 appears, the picture brightens or darkens one level
  every 2 frames; brightening stops at 15, darkening blanks the screen as it reaches 0.
- params: the rhythm, 2 frames.
- switch: Fixed.
- `OpeningBrightens`, `PageOneBrightens` or `OpeningDarkens`, the fade's count `c` over 1 → the
  count `c − 1`, the brightness left [R-brighten-count, R-darken-count]
- `OpeningBrightens` or `PageOneBrightens`, the count 1 or 0, the level `l` with `l + 1` under
  15 → the level `l + 1`, `Shown`, the count 2 [R-brighten-step, C-brighten-from-rest]
- The same, the count 1 or 0, `l + 1` 15 or more → the level 15, `Shown`, the count 0: the fade
  is done [R-brighten-last, R-page1-text]
- `OpeningDarkens`, the count 1 or 0, the level `l` over 1 → the level `l − 1`, `Shown`, the
  count 2 [R-darken-step, C-darken-from-blanked]
- The same, the count 1 or 0, the level 1 → the level 0, `Blanked`, the count 0: the fade is
  done [R-darken-last]
- The same, the count `c` 1 or 0, the level 0 → the count `c − 1` (0 from 1, 255 from 0), the
  brightness left: the fade is done [C-darken-at-zero]
- checked: oracle (recorded cases S3, S4, S8, S9, W4); constructed.

### OpeningUnfolds
- kind: mechanic.
- summary: the opening waits for the first tune to load, brightens, holds 60 frames, asks the
  first tune to play, holds 200 frames, asks for the second tune, waits for it to load, holds
  240 frames and darkens.
- params: the holds, 60, 200 and 240 frames; the second tune's start, 14 frames after it loads.
- switch: Fixed.
- `OpeningLoads`, no track waits → the stage `OpeningBrightens`, the fade's count 2
  [R-opening-loaded]
- `OpeningBrightens`, the fade done (`SlowFade`) → the stage `OpeningPause`, the wait 60
  [R-brighten-last]
- `OpeningPause`, `FirstTune` or `SecondTune`, the wait `w` over 1 → the wait `w − 1` [R-pause]
- `OpeningPause`, the wait 1 or 0 → the stage `FirstTune`, the wait 200; the loaded tune asked to
  play 8 frames after [R-first-tune, C-wait-zero]
- `FirstTune`, the wait `w` 1 or 0 → the stage `SecondTuneLoads`, the wait `w − 1` (0 from 1,
  65,535 from 0); the music stopped, the
  second tune asked to load and to play 14 frames after [R-second-tune, C-first-tune-zero]
- `SecondTuneLoads`, no track waits → the stage `SecondTune`, the wait 240 [R-second-loaded]
- `SecondTune`, the wait `w` 1 or 0 → the stage `OpeningDarkens`, the wait `w − 1` (0 from 1,
  65,535 from 0), the fade's count 2
  [R-darken-begins]
- `OpeningDarkens`, the fade done → the stage `PageOneMadeReady` [R-darken-last,
  C-darken-at-zero]
- While a track waits, the opening's loading stages change nothing (R-opening-loads,
  C-second-loading).
- So the first tune's wait and the second's end in the frame the reply says none waits, and the
  first brightening comes 2 frames after.
- checked: oracle (recorded cases S1-S7, S9); constructed.

## 8. Page 1

### PageOneIsMadeReady
- kind: mechanic.
- summary: in the dark, the text is cleared, the picture stands at 8, Samus's eyes start to
  blink, the caret waits at its home, and the pages' tune is asked for.
- params: the picture's height 8; the dissolve's count 127, which the next part's dissolve
  starts from.
- switch: Fixed.
- `PageOneMadeReady` → the stage `PageOneLoads`; the text cleared; no page; the dissolve's count 127; the
  picture's height 8; the eyes on their blinking script from its start; the caret at its home on
  its steady script; the music stopped, the pages' tune asked to load and to play 14 frames after
  [R-page1-ready]
- checked: oracle (recorded case W1).

### PageOneBegins
- kind: mechanic.
- summary: once the pages' tune has loaded, page 1 starts to type and the picture slowly
  brightens; once it is full the text goes on typing.
- params: none.
- switch: Fixed.
- `PageOneLoads`, a track waits → the stage left; the scripts run [R-page1-loads]
- `PageOneLoads`, no track waits → the stage `PageOneBrightens`, the fade's count 2; page 1's
  script placed, the click turn off [R-page1-begins, C-page1-click-off]
- `PageOneBrightens`, the fade done (`SlowFade`) → the stage `Text` [R-page1-text]
- So page 1's first step, which shows nothing, comes on the frame it is placed, and its first
  letter 1 frame later (R-page1-begins).
- checked: oracle (recorded cases W2-W4); constructed.

---

## 9. The flashback to Zebes

### BattleBegins
- kind: mechanic.
- summary: the press after page 1 begins the fight and readies the dissolve from the text.
- params: the dissolve starts at 127 frames. The caret moves to (8, 248) px.
- switch: Fixed.
- `PageOneWaits`, a new press → `IntoFight`; request `StartZebesFight`; show Samus and her
  shots over the fight sprites; move the caret offscreen without restarting its script; request
  the split blend. Restore the full scene colours, set the colours that will brighten for the
  fight to black, publish the colours, and set the dissolve's count to 127 [R-battle-start,
  C-battle-palette]
- The request selects the Zebes flashback room and demo. Samus's owners place her at (155, 115)
  px, facing left in her standing pose, with missiles selected. The flashback fight places
  Mother Brain and starts the ring spawner. The movie pad starts the scene. The demo supplies
  the buttons for room play.
- checked: oracle (recorded case F1); constructed.

### BattleDissolves
- kind: mechanic.
- summary: the flashback and text exchange their colours as the picture dissolves each way.
- params: the colour sets of each direction are content. The count and colour arithmetic are
  those of `PicturesDissolve`.
- switch: Fixed.
- `IntoFight` or `OutOfFight` → step the dissolve with that direction's colour sets
  [R-battle-hold, C-battle-step, C-battle-return-step]
- `IntoFight`, the dissolve steps from count 0 → `Fight`; clear the text, choose the plain
  fight picture and begin the fight's flashing lights [R-battle-full, C-battle-full-palette]
- `OutOfFight`, the dissolve steps from count 1 → hide Samus and her shots and emit
  `HideFightSamusAndShots`. Mother Brain disappears on this frame too [R-battle-hide]
- `OutOfFight`, the dissolve steps from count 0 → `Text`; choose the plain text picture.
  Page 2 keeps typing [R-battle-text, C-battle-text-palette]
- At count 0, publish the last colour step before choosing either plain picture.
- checked: oracle (recorded cases F2–F4, F8–F9); constructed.

### PageTwoBegins
- kind: mechanic.
- summary: after the explosions around Mother Brain, page 2 starts typing as the fight fades away.
- params: the dissolve starts at 127 frames. Page 2's script is content.
- switch: Fixed.
- `Fight`, no return cue by the end of room play → stay in `Fight`; keep the encounter
  visible and advance the scripts [R-battle-waits]
- `Fight`, the return cue received after the ordinary stage step → commit `PageTwoPending`
  and pause text tile updates on this finished cut [R-battle-signal]
- `PageTwoPending` → `OutOfFight`; place page 2 at its first step with the click turn off;
  place the caret at home on its steady script; request the split blend and resume text tile
  updates. Restore the full scene colours, set the colours that will brighten for the text to
  black, publish the colours, and set the dissolve's count to 127 [R-page2-start,
  C-page2-palette]
- The first step is a blank held for one frame. The first letter, glow and click come on the
  next frame [R-page2-letter].
- checked: oracle (recorded cases F5–F7, F11); constructed.

### PageTwoEnds
- kind: mechanic.
- summary: page 2 ends with a blinking caret and a wait for the press that starts the baby's
  discovery.
- params: none.
- switch: Fixed.
- The page 2 signal "wait for discovery" → end the page, put the caret on its blinking script
  and enter `PageTwoWaits` [R-page2-end]
- `PageTwoWaits`, no press → the stage left and the scripts run [C-page2-wait]
- The next press starts the baby's discovery (§12).
- checked: oracle (recorded case F10); constructed.

## 10. Order that matters

- **Within a frame:** first the stage's step (§3, §5's turns, §6–§9), then the baby (`BabyLeaves`,
  `BabyRises`, then its script), then the caret's script, then the page's script (its letters
  and signals), then the eyes' script, then the glows.
- So the baby's signal for page 5 takes effect on the next frame's stage step
  (`ToPageFive`); a page placed by the stage's step runs its first step in the same frame; a
  letter's glow shows set 0 in the frame the letter appears; a caret placed by a page's signal
  runs from the next frame.
- Apply the fight start request before Samus's first demo step. On the frame the return count
  reaches 0, Samus and her shots take their last step before they are hidden.
- Receive the fight's return cue after room play and before text scripts. Publish the pause
  and commit `PageTwoPending` on the cue's cut. Page 2 starts on the next finished frame.
  The dissolve back to text starts its countdown on the frame after that. A caller that
  already has the current frame's cue can supply it to a single call with the same ordering.

## 11. Out of scope, and who supplies it

- **The line split (E3):** the colour blend's bands while a dissolve runs.
- **Audio (U1):** playing the clicks; stopping the music, loading and playing the tunes asked
  for; whether a track waits, its reply.
- **The pictures (content, drawn by the platform):** the letters, the eyes, the baby, the caret.
- **Lag (T1):** the frames that load a picture.
- **The flight (screens-flight):** from the frame after `Done`.
- **Her equipment and the buttons:** applying what `IntroBegins` lends and `IntroEnds` returns.
- **Samus and room play.** Movement and appearance own her placement, facing, animation,
  motion, knockback and drawing intent. The equipment and weapons owners select and spend
  missiles, step projectiles and accept consumed-projectile events from the fight. The demo
  provides held and new buttons. World blocks provides the flashback room collision geometry.
- **The flashback fight.** The encounter owns Mother Brain, the rings, their contact with Samus,
  the explosions, their shake offset and the flashing lights. It sends the ready-to-return
  event and hides Mother Brain on the same hide event as Samus and her shots. The intro
  emits the event when the post-step return count reaches 0. Production delivers it after
  room play and before drawing. A
  scoped intro check may supply the return event. Integrated replay needs the encounter to
  produce it.
- **The picture.** E1 renders the sprites in their front-to-back order. E2 draws the room and
  text tiles and consumes text update and clear events. E3 owns the split's line program and
  E4 the layers and scroll mirrors.

## 12. Open questions

1. Part (c) starts at the press after page 2. It covers the baby's discovery played by Samus
   and the dissolve into page 3. Part (d) starts at page 3's press and covers the delivery
   scene and page 4 through its wait.
2. The brightness fade here and in the flight is one rule written twice; a later coherence
   round may make it one shared entry.
3. The Japanese pages are other content and a different end of page 5 (an arrow and a wait);
   this piece is English only.
