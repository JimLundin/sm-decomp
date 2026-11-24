# Power-on: decode and harness

Companion to the [power-on spec](screens-boot.md). Keep these notes outside confined exports.

## Sources

Watched first in the `any-power-on` records (lsnes, the any% movie from power-on), then pinned
in the bank logs.

### QuietUntilTheLogo

Source: [Bank 80](https://patrickjohnston.org/bank/80) `$80:841C..8455` (the boot: a wait of
the rest of the frame and three more, then `$0000..1FFF` cleared);
[Bank 8B](https://patrickjohnston.org/bank/8B) `$8B:9146..92DE` (forced blank at level 15,
the display set for sprites only, the title sprite tiles decompressed to `$7F:5000` and sent to
VRAM `$6000`, the title palettes staged at `$7E:C000`, forced blank at level 0 in the mirror,
then NMI enabled). NMI is off through all of this: cuts 0 to 17 are vblanks without NMI.
The display register reads 8Fh at cuts 0 to 4 (the emulator's power-on value, before the game
writes it at cut 5's interval) and 8Fh from the game at 5 to 17.

### LogoBrightens, LogoHeld, LogoDims

Source: Bank 8B `$8B:92E2..935F` (fade delay and counter 1, LOOP_FADE_IN, the logo timer 120
at `$0DE2`, LOOP_NINTENDO_LOGO, LOOP_FADE_OUT, then `$2100` and `$51` = 8Fh directly);
`$8B:9100..911A` (advance fast fade in: `(brightness + counter) & 1Fh`, at 0Fh or above sets
0Fh and returns done); `$8B:90B8..90D4` (advance fast fade out: brightness minus counter, at 0
or below sets 80h). The fade step is the screen fade counter `$0725`; the boot sets it to 1
for both fades. Each loop pass ends in a wait for NMI, so each pass is one finished frame. The
timer is set after the frame that reaches 0Fh, and decremented before each hold frame's wait,
so the hold shows 120 finished frames at 0Fh after the one that reached it.

### SoundLoads

Source: Bank 80 `$80:8459` (upload SPC engine from `$CF:8000`), `$80:8028..80F1` (the upload
protocol: kick, blocks, the zero-size block's entry address on ports 2-3 and 0 on port 1,
then the kick and a bounded wait for the echo). Bank 8B `$8B:935F` sets forced blank at 0Fh
before the upload, so the register shows 8Fh from cut 168.

### Settling, FramesCounted

Source: Bank 80 `$80:8482..856E` (the common boot section: forced blank, bank `$7E` cleared,
NMI and auto-joypad off, CPU and PPU registers initialised, sound queues and OAM cleared, NMI
enabled, three frames waited, the random number seeded with 61h, the music queue cleared, the
NTSC/PAL and SRAM mapping check `$80:85F6`, the demo sets `$80:8261` loading the three SRAM
slots, sounds enabled, then the main game loop `$82:893D`, whose first finished frame is game
state 1, the title). The NMI handler `$80:9583` counts every NMI at `$05B8` and the requested
ones at `$05B6`; the clear of bank `$7E` resets both.

## RAM mapping

| Quantity | Source | Notes |
| --- | --- | --- |
| screen | the record's PPU registers, `inidisp` (`$2100`) at the cut | bit 7 forced blank (`Blank`), low nibble the level; at finished cuts it equals the mirror `$7E:0051` |
| frames of the hold left | `$7E:0DE2` | the logo timer; stale (0) at the cut that reaches 0Fh, so compared only at cuts where the stage was `Held` before and after |
| finished frames | `$7E:05B6` | 16 bits; 0 from the clear at cut 4's interval to the first NMI at 18 |
| stage | none | the main thread's place in the boot; compared through its effects only |
| whether the frame's work finished (borrowed) | the record's cut | `NmiReturn` with kind `Requested`: finished; `VblankWithoutNmi` and kind `Overrun`: not finished |
| the boot's driver upload ended in this frame (borrowed) | the record's `apu_log` for the interval | looked for only while the stage is `SoundLoading`: the interval holding a CPU write of 0 to port 1 right after the writes to ports 2-3, which is the CPU sending the upload's end-of-data block. It is in interval 253 alone among cuts 0 to 270. The runner asserts exactly one per power-on run |

Two vectors give a field that differs from the record at the cut they are modelled on, so they
are `constructed`. C-full (cut 32) carries the whole hold left, because the game sets its timer
only after that frame's wait, and the record's `$0DE2` there is still 0. C-upload-ended
(cut 253) carries the count back at 0, because the clear of bank `$7E` is under way through the
busy cuts 253 to 255. The record's `$05B6` is 150 at those cuts and 0 from 256.

## Harness contract

Row `any-power-on`, `Start::PowerOn`, no seed record: the run admits the spec's power-on
State without reading cut 0, and plays cuts 1 to 270; cut 271 is the title's first finished
frame (game state 1) and stops the run as Unsupported until the title is owned.

Compared, per cut:
- Cuts where the stage is `BeforeLogo` before and after the frame (1 to 17): only whether the
  screen is blank. The level under forced blank comes first from the platform's power-on value
  of `inidisp` (8Fh at cuts 0 to 4 in lsnes), before the game's own write of 8Fh in cut 5's
  interval. Another core or the hardware may hold another level, and it cannot be seen.
- Every other cut: `screen` (blank or lit, and the level) from `inidisp`.
- `finished frames` (`$05B6`):
  - at finished cuts and at every cut while `SoundLoading`;
  - while `Settling`, at every cut from the third after the upload ended onward. The upload's
    cut `L` and the two after it form an exclusion window: the clear of bank `$7E` is still
    running there. The window is counted from `L`, not from fixed cuts. In the record it
    covers 253 to 255, and the count is compared at 256 to 270, where it is 0.
- `frames of the hold left` (`$0DE2`): at cuts where the stage is `Held` before and after the
  frame.

Planted fault for the case (the integrator's): move the upload's end one cut earlier, to 252.
The run must then fail on `finished frames` at cut 255, the first compared cut after the moved
window, where the model has 0 and the record 150. The reference model shows the same failure
for ends moved to 180, 200 and 240: each fails three cuts after its moved cut (183, 203, 243).
An end moved later by one (254) is not caught, because the count is 150 on both sides at 254
and 0 on both sides from 257. A missing end fails at 256.

Exclusion windows: the settle's part-done work. These are not borrowed inputs; no rule reads
them, and they are simply not compared:
- the clear of bank `$7E` across the busy cuts from the upload's end (253 to 255 here). The
  NMI interrupts it: `$05B8` is 1 at cut 256 because the NMI landed after the clear had
  passed it;
- the settle's later steps, all within busy cuts: the random seed (set by cut 263), the SRAM
  mapping test (263 to 267), and the save slots checked and written (268 to 270). Their timing
  is T1's (MODEL decision 35), and their owners are game-random and P2.

Not compared, and why:
- The logo's picture: OAM (the logo spritemap `$8C:80BB` at (80h, 70h) from cut 18 through
  256 in hardware, and in the buffer at `$7E:0370` from cut 18 until the clear), CGRAM (the
  title palettes from cut 18 through 270) and VRAM (the title sprite tiles at `$6000` from
  cut 18). These are AF5 content and E1's sprite list, and their spec is the next piece. They
  can be seen only at the lit cuts 18 to 166, which are compared without the picture. The root
  rules that those cuts count as played on the comparisons this spec makes, and the claim and
  STATUS list the picture as not compared until its spec lands.
- The other display registers, set at cut 5: `obsel` (the sprite tile base), `bgmode`, and
  `tm` (main screen: sprites only). These are left to the picture's owners.
- `$05B8` (every NMI counted) and `$05BA`/`$05BB` (the lag counts): no gameplay rule in this
  stretch reads them.
- SRAM (the mapping test and the slot writes at 263 to 270) and the random seed: inside the
  exclusion window above; P2's and game-random's.
- The APU port contents: the upload is the audio side's, and only its end is borrowed.
- The pad: the movie alternates Start and A from frame 0, and the game reads it from cut 18
  (`$8B`, `$8F`). No boot rule acts on it, and the soft-reset combination is never held.
- The emulator's power-on fill: work RAM at cut 0 is 55h throughout, and SRAM is one value
  throughout until cut 263. The engine reads none of it. SRAM at power-on is the movie's
  starting save memory, which P2 reads at 263 to 270.

Borrowed (named in the runner's `BORROWED`), both under MODEL decisions 35 and 37:
- whether each frame's work finished (the existing lag input, extended to vblanks without NMI);
- the boot's driver upload ended in this frame (CPU and audio timing until T1).

## Reference

`$TEMP/collab/boot-752/scratch/model.py` runs the spec on every vector, and 31 of 31 agree.
Per cut over `any-power-on` 1 to 270, with the borrowed inputs from the record and the
comparisons above, all 270 cuts agree:
- blank 17 of 17;
- screen 253 of 253;
- finished frames 250 of 250;
- hold 120 of 120.

The upload's end is found once, at 253. A do-nothing model (State frozen at power-on), judged
on the same fields, agrees at 35 of 270 cuts:
- blank 17 of 17;
- screen 103 of 253;
- finished frames 15 of 250;
- hold 0 of 120.

Planted faults:
- upload end at 180, 200, 240 or 252: fails at 183, 203, 243 or 255 on `finished frames`;
- upload end at 257: fails at 256;
- hold 119 or 121: fails at 33 on `hold`.

## Budget

Fixed base `1e2674a1ce4fa2944357bfe49d7dbac03b9ca635`. There is no production power-on owner
there to replace (the application refuses power-on), so there is no attributable old-code
size baseline.

k = 37.5 weighted tokens per spec unit.

- Area screen: QuietUntilTheLogo, LogoBrightens, LogoHeld, LogoDims
- Area loading: SoundLoads, Settling, FramesCounted

The spec-units counter reports seven entries, 17 arms and three operators; it reads the five
no-change bullets as explanation. Budget: 18.0 units, 675 weighted tokens (screen 450,
loading 225).
