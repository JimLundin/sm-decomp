# The title: decode and harness

Companion to the [title spec](screens-title.md). Keep these notes outside confined exports.

## Sources

Watched first in the `any-power-on` records (lsnes, the any% movie from power-on), cuts 271 to
392, then pinned in the bank logs.

### TitleBegins

Source: [Bank 82](https://patrickjohnston.org/bank/82) `$82:893D..8981` (the main game loop:
per pass the HDMA object handler, which also handles the music queue, then `$80:8111` the
random number, then the game state's routine, sound effects, OAM, the previous input, and the
wait for NMI). The first pass after the boot's settle runs game state 0 (`$82:8AE4`), which sets
game state 1 and the cinematic function `$8B:9B68`; its NMI is cut 271 (`$05B6` = 1, `$0998` =
1). The display register is still 8Fh there.

### TitleLoads

Source: [Bank 8B](https://patrickjohnston.org/bank/8B) `$8B:9A22..9A46` (game state 1: the
cinematic function, then cinematic sprite objects, mode 7 objects, palette FX objects, drawing,
the skip check `$8B:9A48` and the skip handler `$8B:9A6C`, then the mode 7 matrix);
`$8B:9B68..9B86` (set-up: load the title graphics `$8B:9B87`, queue the title music data FF03h,
cinematic function = RTS `$8B:9A47`, spawn the '1994' text object `$8B:A0EF`, queue song 0's
track 05h); `$8B:9B87..9CBB` (the PPU set up for the title, forced blank, the title palettes,
four decompressions, three VRAM transfers under NMI off, the mode 7 matrix, NMI enabled, fade
delay and counter 1, HDMA and palette FX objects enabled, the tube light `$8B:E1A0` and
flickering displays `$8B:E1A4`, the baby metroid mode 7 object `$8B:A355`); `$8B:9CCF` (the text
object's pre-instruction sets `$51` = 0Fh in the same frame).

The set-up's frame is lag from 272 to 307 (NMI off at 303 and 306). The display register reads
80h from 272: `$8B:8000` sets forced blank at level 0 early in the frame. In the interval ending
308 the game's NMI count `$05B8` and finished count `$05B6` both step by 2 and the random number
is drawn once: the set-up's frame finished at an NMI inside the interval (the set-up enables
NMI inside the vertical blank), and the next frame began, drew, dimmed once more and finished
at 308's NMI. The interval ending 305 also holds two NMIs, both lag.

### OpeningSkipped

Source: as TitleLoads for the pass (`$8B:9A22..9A46`), plus the skip check `$8B:9A48`: while
the cinematic function is below `$8B:9F28` and the skip index `$1A53` is 0, A, B or Start in
`$8F` (mask 9080h) sets the index to 1 and the fade counter `$0725` to 2; then `$8B:9A6C`. The
cinematic sprite objects run before the check, so the opening is at 0Fh when the skip starts,
and it dims in the same pass.

### SkipDims

Source: Bank 8B `$8B:9A83..9A9B` (advance fast fade out `$8B:90B8`; at zero: index 2, cinematic
function `$8B:9F28`, queue song 1's track 06h); `$8B:90B8..90D4` (brightness minus `$0725`; at 0
or below, `$51` = 80h and done).

### TitlePrepared

Source: Bank 8B `$8B:9A9C..9B19` (index 2: clear the cinematic sprite objects, spawn the title
logo `$8B:A119` and copyright `$8B:A125`, fade counter 2, the mode 7 zoom 100h, BG1 position and
speed 0, index 3, the sprite palettes, two colours, palette FX cleared and the two objects
spawned again, the blue light off, main screen BG1 and sprites (`tm` 11h at 315), the gradient
HDMA objects `$88:EB58`).

### TitleBrightens

Source: Bank 8B `$8B:9B53..9B67` (index 3: advance fast fade in `$8B:9100`; at full: index 0,
the demo timer `$1F53` = 900 (384h), cinematic function `$8B:9F29`); `$8B:9100..911A`
(`(brightness + counter) & 1Fh`, at 0Fh or above sets 0Fh and returns done; from 80h it reads
level 0).

Cuts 316 to 353 are lag: the next pass starts with the HDMA object handler's music queue, which
uploads the title's music data (the APU log has writes at every one of these cuts). The random
draw comes after it in the pass, so the record's number changes at 354, not at 316.

### TitleWaits

Source: Bank 8B `$8B:9F29..9F51` (decrement the demo timer; at zero or below, cinematic function
`$8B:9FAE` (to the demos); otherwise A, B or Start in `$8F` sets `$8B:9F52`; both set the fade
delay `$0723` and counter `$0725` to 2; `$8B:8697` is the debug version display, off in the
release).

### TitleLeaves

Source: Bank 8B `$8B:9F52..9FAD` (advance slow fade out `$8B:90D5`; at zero: NMI enabled, BG
mode 1, fade delay and counter 0, scrolls 0, cinematic function `$8B:9B68` again, palette FX
disabled and cleared, the non-gameplay RAM `$198D..1C1F` cleared, then game state 4);
`$8B:90D5..90FF` (decrement the delay; above zero, return; otherwise decrement the brightness, at
0 forced blank and delay and counter 0, else the delay back to the counter). The demo timer is
left at 899.

### The frame owner's count and draw (not this spec's)

Source: Bank 80 `$80:9583` (the NMI counts requested frames at `$05B6`); `$80:8111` (the
random number, game-random's `AdvanceRandom`), called once per main loop pass at `$82:894F`,
in every game state. The settle seeds `$05E5` with 61h (Bank 80 `$80:8482..856E`). No title rule
reads the count or the number, so both belong to the shared frame owner (production's
`played/frame.rs`, which already counts and draws once per begun frame for the played loop), and
it carries them across the power-on, the title and the menus (the root's ruling). The power-on
spec's own `FramesCounted` moves there at that spec's next touch; file select and the options
follow the same rule. The harness below still compares both, as the frame owner's fields.

### Shared with the power-on spec

`SkipDims` is the power-on spec's `LogoDims` arithmetic (`$8B:90B8`, the same routine) and
`TitleBrightens` its `LogoBrightens` (`$8B:9100`), each with the fast step 2 in place of the
logo's 1; only the stage each ends in differs. `TitleLeaves` is a different fade (`$8B:90D5`, one
level per pace). The power-on implementation's one `Fade` serves both; the integrator may share
it.

## RAM mapping

| Quantity | Source | Notes |
| --- | --- | --- |
| screen | the record's PPU registers, `inidisp` (`$2100`) at the cut | as in the power-on notes; equals `$7E:0051` at every cut 271 to 392 |
| countdown | `$7E:1F53` | the demo timer; 900 at 361, 899 from 362; still 899 at 392 (outside the cleared range), where the title no longer holds it |
| frames to the next dim | `$7E:0723` | the screen fade delay; meaningful only while `Leaving` (it is 0 elsewhere, or 1 at the set-up) |
| stage | `$7E:0998` (game state), `$7E:1F51` (cinematic function), `$7E:1A53` (skip index) | `Begun`: state 0 before 271's frame; `Loading`: state 1, function `$8B:9B68`; `SkipDimming`: index 1; `Preparing`: index 2; `Brightening`: index 3; `TitleScreen`: function `$8B:9F29`; `Leaving`: `$8B:9F52`; `FileSelectBegins`: state 4; `OpeningPlays`: function `$8B:9A47` with index 0; `DemoBegins`: `$8B:9FAE`. Compared through its effects only |
| tunes asked for | the music queue `$7E:0619..0628` (entries) | `TitleMusic` FF03h, `OpeningTheme` 05h, `TitleTheme` 06h; FF03h and 05h are in the queue at 308, 06h is added at 314 |
| finished frames (the frame owner's) | `$7E:05B6` | 16 bits |
| random number (the frame owner's) | `$7E:05E5` | at finished cuts only (see the harness) |
| frames of work finished in the interval (borrowed) | per cut: the record's cut kind, and the step of `$05B6` only where the two disagree | 1 at a `Requested` NMI, 0 at `Overrun` and `VblankWithoutNmi`. They disagree at 308 alone: the step is 2, because an NMI inside the interval, which the record's cut list doesn't show, found the set-up's frame done. The runner takes the step where it is 2 at a `Requested` NMI, and fails the run on any other disagreement (a step of 0 at a `Requested` NMI, a step above 0 at a lag cut, a step above 2), so the borrowed step cannot hide a miscount |
| buttons just pressed as each frame began (input) | the movie's pad, latched at the last finished cut | `$7E:008F` at the previous finished cut (0080h, A, at 271 and at 361). The second frame finished by 308 reads 271's latch again: the record holds no latch for the NMI inside the interval, and the skip has begun by then, so no press acts there |

## Harness contract

Row `any-power-on`, continuing the power-on run (no new seed): the power-on owner hands over at
cut 271, the first finished frame after its `Settling`, as `Begun` with its screen; the frame
owner carries the count (0 there) and the seed 61h. The run plays cuts 271 to 392; cut 393 is
file select's (game state 4) and stops the run as Unsupported until file select is owned.

Compared, per cut:
- `screen` from `inidisp`, every cut;
- `finished frames` (`$05B6`, the frame owner's), every cut. It is not an independent check:
  the borrowed frames finished in each interval are read from its step, so the played count is
  their sum. The screen and the random draws check the frames finished; at 308 the screen (11,
  not 13) and the two random draws;
- `random number` (`$05E5`, the frame owner's), at finished cuts only. At a lag cut the frame in
  progress may or may not have drawn yet: it has at 272 to 307 (the draw comes before the
  set-up), and has not at 316 to 353 (the music upload comes before the draw). Where the draw
  falls in a frame is the platform's timing (T1, MODEL decision 35), so lag cuts mask it;
- `countdown` (`$1F53`), at the cuts 361 to 391, where the stage after the frame is
  `TitleScreen` or `Leaving`;
- `tunes asked for`, at finished cuts: the music queue's entries that differ from those at the
  last finished cut, against the tunes asked for since it. Expected and played are sorted, so
  the order of two tunes asked for in one interval is not compared. The queue's head, its timers
  and their removal (at 354, 363 and 371) are the audio side's (U1), not compared.

Planted fault for the case (the integrator's): the fast step at 3. The run must fail on `screen`
at 308 (the model has `Lit(9)`, the record 0Bh). The reference model shows the same cut for a
fast step of 1 and for one frame finished at 308 instead of two; a leaving pace of 3 fails at
364, and a demo wait of 899 fails at 361 on `countdown`.

Not compared, and why (the root's ruling: the picture is not compared, as for the boot's logo):
- The pictures: VRAM (the title BG tiles, tilemap and sprite tiles, loaded 303 to 306 and
  at 355 and 365), CGRAM (the title palettes; palette FX alternate two images per frame from
  308), OAM (the opening's text, the logo and copyright), the mode 7 matrix and the HDMA tables.
  These are AF5 content with E4's mode 7 object, E3's gradient HDMA and R4's palette FX; they
  can be seen at the lit cuts 308 to 313 and 354 to 391.
- The other display registers (`bgmode` 07h from 272, `tm` 10h at 272 and 11h from 315); left to
  the pictures' owners.
- `$05B8` (every NMI counted) and the lag counts: no rule here reads them.
- SRAM: unchanged from 271 to 392; it is P2's settle result, first visible at 271, and P2's to
  compare.
- The APU ports: the music upload is the audio side's; its length shows only as lag.
- The fade delay outside `Leaving`, the skip index, the cinematic function: structure.

Borrowed (named in the runner's `BORROWED`), under MODEL decision 35 (the root's rulings):
- how many frames of work finished in each interval: the existing lag input, read from the cut
  kind, and from the game's finished count where the record shows two finished by one cut (308);
- the random number as the game settles (61h), until game-random states the power-on's seeding.

## Reference

`$TEMP/collab/title-752/scratch/model.py` runs the spec on every vector, and 41 of 41 agree.
`percut.py` runs it per cut over `any-power-on` 271 to 392 from the hand-over (`Begun`, the
screen `Blank(15)`; the frame owner's count 0 and seed 61h), with the borrowed input and the
comparisons above; all 122 cuts agree:
- screen 122 of 122;
- finished frames 121 of 121 independent, and 308 (from the borrowed step) 1 of 1;
- random number 48 of 48 (the finished cuts);
- countdown 31 of 31;
- tunes asked for 122 of 122.

A do-nothing model (the hand-over State frozen), judged on the same fields, agrees at 0 of 122
cuts: screen 1 of 122, finished frames 0 of 121 (and 0 of 1 at 308), random number 0 of 48,
tunes 120 of 122.

Planted faults (`faults.py`): fast step 3 or 1, and one frame at 308, each fail at 308; leaving
pace 3 fails at 364; demo wait 899 fails at 361; leaving pace 1 reaches file select early and
stops Unsupported.

Recorded vectors take `before` from the last finished cut ahead of the one they model, and give
`pressed` as the latch the frame read (Start at 308 and 314, A at 313 and 315).

## Budget

Fixed base `1e2674a1ce4fa2944357bfe49d7dbac03b9ca635`. There is no production title owner
there to replace (the application refuses power-on), so there is no attributable old-code
size baseline.

k = 37.5 weighted tokens per spec unit.

- Area screen: TitleBegins, TitleLoads, OpeningSkipped, SkipDims, TitlePrepared, TitleBrightens, TitleWaits, TitleLeaves

Counted by hand to the counter's rules (the `tools` executable is blocked locally): eight
entries, 18 arms and five operators, 19.7 units; 18.7 if `OpeningSkipped`'s arm, which refers to
`SkipDims`, counts as a cross-reference. The root runs `tools spec-units` for the official figure.
