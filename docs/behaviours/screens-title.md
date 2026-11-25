# The title: loading, the skipped opening and the title screen

What the player sees from the end of the power-on sequence until file select begins: black
while the title's pictures load, the opening lighting up and, when A, B or Start is pressed,
fading out at once, black again while the title tune loads, the title screen fading in, and,
on a press there, the title screen slowly dimming to black. Vectors:
[screens-title.toml](../../scenarios/vectors/screens-title.toml).

## Header

**Scope.** The title sequence, one frame end at a time, from the first frame that finishes
after the game settles (the [power-on spec](screens-boot.md) leaves that frame here) until the
frame that hands over to file select: its stage, the screen's brightness and whether it shows,
the title screen's countdown to the demos, and the tunes it asks the audio side for. This
introduces the title's owner. There is no old production owner to replace: the engine refuses
power-on today.

**Reads.**
- From the platform, at each frame end: how many frames of work finished since the last frame
  end, 0, 1 or 2 (borrowed until timing is modelled; MODEL decision 35). In the any% movie two
  finish by one frame end once, as the title's load ends.
- From the controller, for each frame of work: the buttons just pressed as it began. Only A,
  B and Start are acted on.
- From the power-on owner, as the game settles: its screen.

**Reports.** The screen at each frame end, the countdown while the title screen shows, the
tunes asked for in each frame, and which of the three ways out the title took: the opening
playing on, the demos, or file select.

**Leaves to others.** The count of finished frames and the shared random number's draws, which
the game's frame owner carries across the power-on, the title and the menus (one count and one
draw per finished frame of work; the seed as the game settles is borrowed until game-random
states the power-on's seeding). The pictures: the opening's and the title screen's backgrounds
and sprites (the logo, the copyright line, the opening's text), the rotating and zooming picture
of the creature in its tube, the colour gradient down the title screen, the tube's light and the
flickering displays (all imported content and their owners' rules), and which layers show. The
tunes themselves, their queue, their loading and every reply of the audio side. The save memory
and the files on it. The opening's scenes when nothing is pressed, the demos, and file select,
each a later piece.

**Notation.** A frame end is the moment a video frame ends. A frame of work is one pass of the
game's work; it either finishes by a frame end or runs on through it. A brightness level runs
from 0 (dark) to 15 (full). The screen is either blank (nothing shows, whatever its level) or
lit at its level, as in the power-on spec; a blank screen counts as level 0 when it brightens.
A skip button is A, B or Start.

## Memory

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| stage | `TitleStage`: `Begun` (the game settled, the title's first frame not yet finished), `Loading`, `SkipDimming`, `Preparing`, `Brightening`, `TitleScreen`, `Leaving`, and the ways out `OpeningPlays`, `DemoBegins`, `FileSelectBegins` | every entry |
| screen | `Screen`: `Blank(Level)` or `Lit(Level)`; `Level` 0 to 15 (the power-on spec's type). While `SkipDimming` and `Leaving`, `Lit(1)` to `Lit(15)`; while `Brightening`, `Blank(0)` or `Lit(1)` to `Lit(14)` | `TitleLoads`, `SkipDims`, `TitleBrightens`, `TitleLeaves`, output |
| countdown | `DemoCountdown`, finished frames: 1 to the demo wait while `TitleScreen`; 0 to the demo wait less one while `Leaving` | `TitleWaits`, output |
| frames to the next dim | `DimDelay`, finished frames, 1 to the leaving pace, only while `Leaving` | `TitleLeaves` |
| tunes asked for | `TuneRequest` list, oldest first, for the frame end: `TitleMusic` (the title's set of tunes), `OpeningTheme`, `TitleTheme` | output |

The first four quantities are the owner's complete State: a caller can set any combination the
table allows, read it back, carry it forward and restore it. The tunes asked for are output only,
empty at a frame end where none was asked. As the game settles, the State is `Begun` with the
power-on owner's screen (`Blank(15)`).

## Rules

### TitleBegins

- kind: mechanic
- summary: the first frame that finishes after the game settles is the title's; then it loads.
- params: none
- switch: fixed
- `Begun`, a finished frame → `Loading`, the screen unchanged [R-begins].
- `Begun`, a frame end with no frame finished → no change [C-begun-busy].
- checked: oracle (R-begins).

### TitleLoads

- kind: mechanic
- summary: black while the title's pictures load; when the load's frame finishes, the opening
  lights at full and asks for its tunes.
- params: none
- switch: fixed
- `Loading`, a frame end with no frame finished → `Blank(0)` [R-loading, R-loading-still-dark].
- `Loading`, a finished frame → asks for `TitleMusic` then `OpeningTheme`; the opening shows at
  `Lit(15)`; then, if a skip button was just pressed as the frame began, the skip of
  `OpeningSkipped` in the same frame [R-two-frames, C-load-done-skip], otherwise
  `OpeningPlays` [C-load-done-no-press, C-load-done-other-button].
- checked: oracle (R-loading, R-loading-still-dark, R-two-frames); original only for a load that
  ends alone at a frame end and the opening without a press.

### OpeningSkipped

- kind: mechanic
- summary: a skip button pressed while the opening shows starts its fast fade-out at once.
- params: none
- switch: fixed
- The opening showing at `Lit(15)`, a skip button just pressed → `SkipDims` from level 15 in
  the same frame [R-two-frames, C-load-done-skip, C-load-done-b, C-load-done-start].
- Once the skip has begun, presses do nothing more [R-skip-dims, R-skip-dark,
  C-skip-ignores-press].
- checked: oracle (R-two-frames, R-skip-dims, R-skip-dark); original only for B and Start.

### SkipDims

- kind: mechanic
- summary: the skipped opening darkens by the fast step on each finished frame; at dark it asks
  for the title tune.
- params: fast step `k`, levels per finished frame, 1 to 15 (`fast_step`, reference 2)
- switch: parameter
- The power-on spec's `LogoDims` with the fast step for its step, ending in `Preparing` where
  the logo's ends in `SoundLoading`:
- `SkipDimming` at level `l` (or the opening at 15), a finished frame, `l - k` above 0 →
  `Lit(l - k)`, `SkipDimming` [R-two-frames, R-skip-dims, A-fast-step-3-dims].
- The same, `l - k` at most 0 → `Blank(0)`, `Preparing`, asks for `TitleTheme` [R-skip-dark,
  C-skip-dark-exact, A-fast-step-15-dark-at-once].
- `SkipDimming`, a frame end with no frame finished → no change [C-unfinished-skip].
- checked: oracle (R-two-frames, R-skip-dims, R-skip-dark); original only for other steps.

### TitlePrepared

- kind: mechanic
- summary: one dark frame sets the title screen up; it then brightens.
- params: none
- switch: fixed
- `Preparing`, a finished frame, whatever was pressed → `Brightening`, the screen still
  `Blank(0)` [R-prepared].
- checked: oracle (R-prepared).

### TitleBrightens

- kind: mechanic
- summary: the title screen brightens by the fast step on each finished frame, up to full.
- params: the fast step `k` of `SkipDims`
- switch: parameter
- The power-on spec's `LogoBrightens` with the fast step for its step, ending in `TitleScreen`
  where the logo's ends in `Held`; presses do nothing:
- `Brightening` at level `l` (0 when blank), a finished frame, `l + k` below 15 → `Lit(l + k)`
  [R-first-light, R-brightening, A-fast-step-3-brightens].
- The same, `l + k` at least 15 → `Lit(15)`, `TitleScreen`, the countdown at the demo wait
  [R-full, C-full-capped, A-demo-wait-5].
- `Brightening`, a frame end with no frame finished → no change [R-tune-loading].
- checked: oracle (R-first-light, R-brightening, R-full, R-tune-loading); original only for
  other steps.

### TitleWaits

- kind: mechanic
- summary: the title screen counts down to the demos; a skip button pressed first leaves for
  file select.
- params: demo wait `w`, finished frames, 1 to 32,767 (`demo_wait`, reference 900)
- switch: parameter
- `TitleScreen` with countdown `n`, a finished frame → countdown `n - 1`; then, at 0 →
  `DemoBegins`, whatever was pressed [C-demo, C-demo-over-press, A-demo-wait-1];
  otherwise, a skip button just pressed as the frame began → `Leaving` with the leaving pace
  of frames to the next dim [R-press, A-leave-pace-3-press]; otherwise no other change
  [C-title-waits, C-title-other-button].
- `TitleScreen`, a frame end with no frame finished → no change [C-unfinished-title].
- checked: oracle (R-press); original only for the countdown's end and other presses.

### TitleLeaves

- kind: mechanic
- summary: leaving, the title screen dims one level every leaving pace of finished frames; at
  dark file select begins.
- params: leaving pace `p`, finished frames per level, 1 to 128 (`leave_pace`, reference 2)
- switch: parameter
- `Leaving` with `d` frames to the next dim, a finished frame, `d - 1` above 0 → `d - 1` to the
  next dim, the screen unchanged [R-leave-waits, C-leave-ignores-press].
- The same, `d - 1` at 0, at level `l` with `l - 1` above 0 → `Lit(l - 1)`, `p` frames to the
  next dim [R-leave-dims, A-leave-pace-1, A-leave-pace-3-dims].
- The same, `d - 1` at 0, `l - 1` at 0 → `Blank(0)`, `FileSelectBegins` [R-leave-dark].
- `Leaving`, a frame end with no frame finished → no change [C-unfinished-leaving].
- The countdown stays as it was while `Leaving` [R-leave-waits].
- checked: oracle (R-leave-waits, R-leave-dims, R-leave-dark); original only for other paces.

## Order that matters

When two frames finish by one frame end, the whole of the first comes before the second, and
each reads the buttons just pressed as it began [R-two-frames]. The frame that finishes the load
shows the opening at `Lit(15)`, asks for its tunes, and on a skip button dims in that same frame,
so the opening at full is never seen at a frame end when a skip button was pressed. At the frame
that reaches `TitleScreen` the countdown is the whole demo wait; it first counts down on the next
finished frame.

A stage that is a way out (`OpeningPlays`, `DemoBegins`, `FileSelectBegins`) belongs to the
piece that follows: a further frame end there is Unsupported here [C-after-file-select].
