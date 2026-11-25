# The intro's last pages: behaviour spec

What the player sees at the end of the intro: page 4's text dissolves into a picture of the
baby being examined, the baby rising into view as the picture settles; Samus's eyes
blink beside the text; the picture dissolves back to text and page 5 is typed out letter by
letter, each letter glowing as it appears, with a click every other letter and a caret running
ahead; the caret blinks and the game waits for a button; the press clears the text and page 6 is
typed ("CERES STATION WAS UNDER ATTACK!!"), Samus's eyes now staring; after a pause the
picture fades to black and the flight to Ceres begins. Written behaviour first, from the oracle's
records over time. Vectors:
[`scenarios/vectors/screens-intro.toml`](../../scenarios/vectors/screens-intro.toml).

## Header

**Scope: the intro's model (the plan's F2), its first piece: the intro's last row.** The
intro's own State: where the scene has got to, the dissolve's colours and its count, the
picture's height, the baby, the page being typed, the caret, the glowing letters, Samus's eyes,
the screen's brightness and the fade's count. The intro's earlier pages and its two scenes
played by her extend this model later.

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
3. **The brightness fade** (§6): the same rule as the flight's fade out; a later piece can make
   them one entry.
4. **Moving pictures:** the baby's rise here (§4); the flight's drifts and rush are its own.

**What it reads.**
- **The pad's new presses** (the movie's input): whether any button was pressed anew this frame.
- **The language** (the options' setting): this piece plays the English pages; a Japanese
  setting is Unsupported at the intro's start.
- **The scene's content** (in our format): the scripts (the pages' letters, places and holds,
  the eyes', the baby's and the caret's two), the full colours of the scene's 256 colours, which
  colours belong to the text and which to the picture in each dissolve, the colours the caret
  takes once the text is back, and the caret's home.

**What it hands the others, each frame:** the screen's 256 colours, the picture's height, the
pictures the scripts show and where (the letters with their colour sets, the eyes, the baby, the
caret), the brightness, to the renderer; each click asked for and the music's stop at page 6,
to audio (U1); on the frame it ends, that it is done, and that the equipment lent for the
played scenes is returned: the next frame is the flight's (screens-flight `FlightIsMadeReady`).

**What it leaves.** The pictures themselves are content drawn by the platform; the line split
of the colour blend during a dissolve is the platform's line program (E3); the lag frames are
the platform's (T1); playing the click and stopping the music are audio's (U1).

**What it replaces:** nothing. The engine has no intro.

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
| the intro's stage | `IntroStage`: `ToScene`, `Scene`, `ToPageFive` (the baby has signalled page 5), `ToText`, `Text`, `WaitingForPress`, `FadingOut` or `Done` | what the screen shows | every entry |
| the colours | 256 colours, each three channel levels in 256ths, 16-bit | the picture's colours | `PicturesDissolve`, `PageFiveBegins`; output (the colours shown) |
| the colours' steps | 256 colours, each three steps in 256ths: a colour's full level × 8, so 32 steps make it full | how fast each colour fades | `PicturesDissolve` |
| the dissolve's count | frames, 0 to 65,535 | when the dissolve steps and ends | `PicturesDissolve`, `BabyRises`, `BabyLeaves` |
| the picture's height | px, signed 16-bit | where the scene's picture stands | `BabyRises`; output (the scroll) |
| the baby | its y, px, 16-bit (x 112), and its script's place, or none | where the baby is | `BabyRises`, `BabyLeaves`, `ScriptSteps` |
| a script's place | for each script (the page, the eyes, the baby, the caret): which script, its step, the frames its picture still holds (0 to 65,535), and the count left of a repeat it is in, or none | which picture shows and when it changes | `ScriptSteps` |
| the caret | its place, px, and its script's place | where the next letter goes and whether the caret shows | `LetterIsTyped`, `CaretBlinks` |
| the glowing letters | up to 8, each its place (column, row), the colour set it takes next (0 to 3) and frames left | how bright each new letter is | `LetterGlows` |
| the click turn | `bool`: whether the next letter's turn clicks | which letters click | `LetterIsTyped` |
| the screen's brightness | `Brightness`: a level, 0 to 15 (0 black, 15 full), and `Blanked` (nothing shown at any level) or `Shown` | how bright the picture is | `PictureDarkens`, `IntroEnds`; output |
| the fade's count | frames, 0 to 32,768 | when the picture next darkens | `PictureDarkens` |
| the fade's rhythm | frames, `d` (`PictureDarkens`): a rule, not State | how many frames each level holds | `PageEnds`, `PictureDarkens` |

### 1.2 What it remembers: state a caller can hold

Every quantity of §1.1 but the rhythm: nothing else. A caller can build any of them in any value
this spec allows, read them back, carry them and restore them; a vector's `before` builds them
and its `after` reads them back.

### 1.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| a press | `bool`: any button pressed anew this frame (the movie's pad) | `PressTurnsThePage` |
| the language | the options' setting: English (a Japanese setting is Unsupported) | the caller, at the intro's start |
| the scripts | content: each a list of steps (§2): the pages 5 and 6, the eyes' two (blinking, page 6's), the baby's, the caret's two (steady, blinking) | `ScriptSteps`, `LetterIsTyped`, `CaretBlinks` |
| the full colours | content: the scene's 256 colours as the dissolves head for them | `PageFiveBegins` |
| the dissolve's sets | content: for each dissolve, the colours that dim and those that brighten; for page 5, the text's colours that start black | `PicturesDissolve`, `PageFiveBegins` |
| the caret's colours | content: the colours, by index, the caret takes once the text is back | `DissolveEnds` |
| the caret's home | content: its place as a page begins, px | `PageFiveBegins`, `PressTurnsThePage` |

### 1.4 What it reports (Outputs)

| Output | Type | Read by |
| --- | --- | --- |
| the colours shown | 256 15-bit colours: each channel's `level / 256` modulo 32, taken on each step of a dissolve | output (the renderer) |
| the pictures | each script's picture and place; each letter with its colour set; whether the caret shows | output (the tile writing, the sprite drawing) |
| the text cleared | an event | output (the tile writing) |
| a click | an event, per letter that clicks | U1 (the sound's request) |
| the music stops | an event, as page 6 begins | U1 |
| the brightness | `Brightness`, every frame | output |
| the equipment returned | an event, as the intro ends: her missiles and their capacity 0 and the player's own buttons back, undoing what the played scenes lent | the owners of her equipment and of the buttons |
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
  runs: on the same frame for a script placed by the stage's step (§7), on the next for one
  placed by another script's signal [R-page6-begins, R-page5-begins, R-page5-end]
- checked: oracle (recorded cases E1-E3, B1, P5); constructed.

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
- A page's signal "wait for a press" or "caret blinks" → the caret's blinking script placed at
  its start, its place kept [R-page5-end, R-page6-caret]
- The caret's script's current step the caret's picture → the caret shown [R-caret-blink]
- The caret's script's current step showing nothing → the caret hidden [C-caret-steady]
- checked: oracle (recorded cases P3, P5, P7); constructed.

### PageEnds
- kind: mechanic.
- summary: at the end of page 5 the game waits for a press; at the end of page 6 the page holds,
  and the picture starts to fade out.
- params: none (the signals are the page's content).
- switch: Fixed.
- The signal "wait for a press" → the stage `WaitingForPress` [R-page5-end]
- The signal "finish" → the stage `FadingOut`, the fade's count `d` [R-page6-end, C-finish-adjusted-end]
- checked: oracle (recorded cases P5, P8); constructed.

### PressTurnsThePage
- kind: mechanic.
- summary: while the game waits, a press clears the text, page 6 begins, Samus's eyes start to
  stare, and the music is asked to stop.
- params: none.
- switch: Fixed.
- `WaitingForPress`, a press → the stage `Text`; the text cleared; page 6's script placed, the
  click turn off; the caret at its home on its steady script; the eyes on page 6's script from
  its start; the music's stop asked [R-page6-begins]
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

## 7. Order that matters

- **Within a frame:** first the stage's step (§3, §5's turns, §6), then the baby (`BabyLeaves`,
  `BabyRises`, then its script), then the caret's script, then the page's script (its letters
  and signals), then the eyes' script, then the glows.
- So the baby's signal for page 5 takes effect on the next frame's stage step
  (`ToPageFive`); a page placed by the stage's step runs its first step in the same frame; a
  letter's glow shows set 0 in the frame the letter appears; a caret placed by a page's signal
  runs from the next frame.

## 8. Out of scope, and who supplies it

- **The line split (E3):** the colour blend's bands while a dissolve runs.
- **Audio (U1):** playing the clicks; stopping the music.
- **The pictures (content, drawn by the platform):** the letters, the eyes, the baby, the caret.
- **Lag (T1):** the frames that load a picture.
- **The flight (screens-flight):** from the frame after `Done`.
- **Her equipment and the buttons:** applying what `IntroEnds` returns.

## 9. Open questions

1. Page 4 and the earlier pages, their turns, and the two played scenes extend this model in
   later pieces; this row is admitted after page 4's turn has finished.
2. The brightness fade here and in the flight is one rule written twice; a later coherence
   round may make it one shared entry.
3. The Japanese pages are other content and a different end of page 5 (an arrow and a wait);
   this piece is English only.
