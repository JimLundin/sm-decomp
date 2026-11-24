# Power-on: the logo and the sound driver's load

What the player sees from switching the console on until the title begins: a black screen, the
Nintendo logo brightening, held and dimming, then black while the sound driver loads and the
game settles. Vectors: [screens-boot.toml](../../scenarios/vectors/screens-boot.toml).

## Header

**Scope.** The power-on sequence, one frame end at a time, from the console's power-on until
the first frame of the title finishes: its stage, the screen's brightness and whether it shows
at all, the count of finished frames, and the restart of that count as the game settles. This
introduces the power-on owner. There is no old production owner to replace: the engine
refuses power-on today.

**Reads.** Whether each frame's work finished, from the platform (borrowed until its timing
is modelled). Whether the boot's driver upload ended within the frame (borrowed: how long the
upload takes is CPU and audio timing, until T1; MODEL decisions 35 and 37); it ends once,
while the sound loads.

**Reports.** The screen at each frame end (blank or lit, and its level), the count of finished
frames, and, on the first finished frame after the game settles, that the title begins.

**Leaves to others.** The logo's picture (its sprites, its colours and its tiles, imported
content) and the sprite list that places it; the other display settings (which layers show,
where the sprite pictures are kept); the sound driver's load itself and every reply of
the audio side; the save memory's check and its files as the game settles; the random number's
first value; the title, which begins on the first frame that finishes after the game settles.
The controller is read through the sequence, but nothing here acts on it.

**Notation.** A frame end is the moment a video frame ends; a frame's work either finished
by then or did not. A brightness level runs from 0
(dark) to 15 (full). The screen is either blank (nothing shows, whatever its level) or lit at
its level. The hold counts frames that finish.

## Memory

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| stage | `PowerOnStage`: `BeforeLogo`, `Brightening`, `Held`, `Dimming`, `SoundLoading`, `Settling` | every entry |
| frames of the hold left | `HoldFrames`, whole finished frames, 0 to the hold, only while `Held` | `LogoHeld` |
| screen | `Screen`: `Blank(Level)` or `Lit(Level)`; `Level` 0 to 15 | `LogoBrightens`, `LogoHeld`, `LogoDims`, output |
| finished frames | `FinishedFrames`, frames, 16 bits, wrapping | `FramesCounted`, output |

These four quantities are the owner's complete State: a caller can set any combination the
table allows, read it back, carry it forward and restore it. At power-on the State is
`BeforeLogo`, `Blank(15)` and 0 finished frames.

## Rules

### QuietUntilTheLogo

- kind: mechanic
- summary: until the logo's first frame finishes, the screen stays blank.
- params: none
- switch: fixed
- `BeforeLogo`, a frame whose work did not finish → keep the stage and `Blank(15)`
  [R-before-logo, C-unfinished-before-logo].
- checked: oracle (R-before-logo).

### LogoBrightens

- kind: mechanic
- summary: the logo brightens by the fade step on each finished frame, from dark to full.
- params: fade step `k`, levels per finished frame, 1 to 15 (`fade_step`, reference 1)
- switch: parameter
- `BeforeLogo`, a finished frame, `k` below 15 → `Lit(k)`, `Brightening` [R-first-light,
  A-step-3-brightening].
- `Brightening` at level `l`, a finished frame, `l + k` below 15 → `Lit(l + k)` [R-brightening].
- `Brightening` at level `l` with `l + k` at least 15, or `BeforeLogo` with `k` of 15, a
  finished frame → `Lit(15)`, `Held` with the whole hold left [C-full, A-step-4-caps, A-hold-5,
  A-step-15-at-once].
- `Brightening`, a frame whose work did not finish → no change [C-unfinished-brightening].
- checked: oracle (R-first-light, R-brightening); original only for reaching full (C-full,
  whose hold is the model's: the game sets its timer after that frame) and other steps.

### LogoHeld

- kind: mechanic
- summary: at full brightness the logo stays for the hold's frames, then starts to dim.
- params: hold `h`, finished frames, 0 to 65,535 (`hold`, reference 120)
- switch: parameter
- `Held` with `n` above 0 left, a finished frame → `n - 1` left, `Lit(15)` [R-hold-first,
  R-hold-last].
- `Held` with 0 left, a finished frame → dims as `LogoDims` from level 15 [R-dimming-starts,
  A-step-2-dims, A-step-15-dims-at-once, A-hold-0].
- `Held`, a frame whose work did not finish → no change [C-unfinished-held].
- checked: oracle (R-hold-first, R-hold-last, R-dimming-starts); original only for other holds.

### LogoDims

- kind: mechanic
- summary: the logo dims by the fade step on each finished frame; at dark the screen blanks.
- params: the fade step `k` of `LogoBrightens`
- switch: parameter
- `Dimming` at level `l` (or `Held` at its end, level 15), a finished frame, `l - k` above 0 →
  `Lit(l - k)`, `Dimming` [R-dimming-starts, R-dimming, A-step-2-dims].
- `Dimming` at level `l` (or `Held` at its end), a finished frame, `l - k` at most 0 →
  `Blank(0)`, `SoundLoading` [R-dark, A-step-4-dims-to-dark, A-step-15-dims-at-once].
- `Dimming`, a frame whose work did not finish → no change [C-unfinished-dimming].
- checked: oracle (R-dimming-starts, R-dimming, R-dark); original only for other steps.

### SoundLoads

- kind: mechanic
- summary: black at full level while the sound driver loads; once its upload ends the game settles.
- params: none
- switch: fixed
- `SoundLoading`, a frame end, the upload not ended in it → `Blank(15)` [R-loading-blank,
  R-loading, C-finished-while-loading].
- `SoundLoading`, a frame end, the upload ended in it → `Blank(15)`, `Settling`, the count of
  finished frames back to 0 [C-upload-ended, C-ended-unfinished, C-ended-finished].
- checked: oracle (R-loading-blank, R-loading; the restart through the count, compared once the
  game has settled); original only for C-upload-ended, whose count of 0 the record shows three
  frame ends later, and a finished end.

### Settling

- kind: mechanic
- summary: black while the game settles; the next finished frame is the title's.
- params: none
- switch: fixed
- `Settling`, a frame whose work did not finish → no change [R-settling, R-settling-later].
- `Settling`, a finished frame → the title's frame, which this owner leaves to the title
  (Unsupported here) [C-settled].
- checked: oracle (R-settling, R-settling-later); original only for C-settled.

### FramesCounted

- kind: mechanic
- summary: every finished frame adds one to the count; a frame that did not finish adds none.
- params: none
- switch: fixed
- A finished frame, in any stage but `Settling` → finished frames `+ 1`, wrapping from
  65,535 to 0 [R-first-light, R-brightening, C-count-wraps].
- A frame whose work did not finish → the count unchanged [R-loading-blank, R-loading,
  C-unfinished-before-logo, C-unfinished-brightening].
- checked: oracle (R-first-light, R-brightening, R-loading); original only for the wrap.

## Order that matters

Within one frame end the count of finished frames moves first, then the stage's rule; the
upload's end in a frame sets the count back to 0 after it [C-ended-finished]. A frame end that brings the logo
dark shows `Blank(0)` at that frame end, and `Blank(15)` from the next.
