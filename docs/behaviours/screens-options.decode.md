# The options menu: decode and harness

Companion to the [options spec](screens-options.md). Keep these notes outside confined exports.

## Sources

Watched first in the `any-power-on` records, cuts 465 to 567, then pinned in
[Bank 82](https://patrickjohnston.org/bank/82). Game state 2 (`$82:EB9F`) runs the routine of
the options menu index `$0DE2`, then the menu objects and, from index 2, BG1.

### OptionsLoad

- Index 0 `$82:EBDB`: handle fading out (already dark), NMI enabled, fade delay `$0723` and
  counter `$0725` 0, index 1 (the pass that ends at 465).
- Index 1 `$82:EC11`: layers, window and colour math off, scrolls 0, the menu palettes, five
  tilemaps decompressed, the options tilemap, the cursor `$099E` = 0, the missile and the
  border objects, index 2, the language highlight. The pass runs as lag 466 to 471 and ends at
  472 (the index already reads 2 at the lag cut 471).

### OptionsBrighten

- Index 2 `$82:ECE4`: handle fading in (`$80:894D`, delay 0: one level per finished frame) until
  0Fh, then delay and counter 0 and index 3. 1 at 473 to 15 at 487.

### OptionsCursor

Index 3 `$82:ED42`, first: Up (0800h) moves the cursor `(c - 1) mod 5`, otherwise Down (0400h)
`(c + 1) mod 5`, each with sound 37h of library 1 (`CursorMoved`).

### OptionsChoice

Index 3 `$82:ED42`, after the cursor's move (`OptionsCursor`):
- B (8000h) sets index Bh (`$82:EE6A`: fade out, then game state 4, file select).
- Otherwise A (0080h) or Start (1000h): sound 38h (`Selected`) and the item's routine
  `$82:EDA7`:
  - start game `$82:EDB1`: when the loading game state `$7E:D914` is main gameplay (5), index 4
    at once; otherwise fade delay and counter 0 and index Ch;
  - English and Japanese text `$82:EDDA`: the cursor 0 and `$09E2` switched;
  - controller and special setting `$82:EE55`: mosaic on for BG1 and BG2 with block size 0
    (`$57` = 03h), delay and counter 0, index 5.

### SpecialSettings

Index 8 `$82:F024`:
- Up and Down `mod 3` with sound 37h.
- Then B: sound 38h, the cursor 0, `$82:EE55`.
- Otherwise A, Left, Right or Start (mask 1380h): sound 38h and `$82:F088`. Items 0 and 1 switch
  the flag at `$82:F0AE` (`$09EA` weapon-reset, `$09E4` moonwalk) by `$82:F08E`; item 2 is
  `$82:F0B2` (the cursor 0, `$82:EE55`).

In the movie, Down and Left latched at 518 move the cursor to moonwalk and switch it on (seen
at 519). B latched at 519 dissolves back at 520.

### Dissolve

- Index 5 `$82:EF18`: handle fading out with delay 0, block size (`$57` high nibble) up one to
  at most 15. At forced blank: NMI enabled, delay and counter 0, always index 6 (the dissolve
  in), and the destination's tilemap by the cursor (0: options; bit 2: special settings, which
  set the highlights; otherwise controller settings with the bindings loaded and drawn). There
  is no separate target in the game: the cursor is it, so the spec derives the destination from
  the cursor (start game, special settings or controller settings while dissolving).
- Index 6 `$82:EFDB`: handle fading in, block size down one to at least 0; at 0Fh mosaic off,
  delay and counter 0, then index 3 (cursor 0), 8 (special, the cursor 0) or 7 (controller, the
  cursor 0).

The cursor still reads 4 at 503 to 517, the item the dissolve left from; it is 0 from 518.

### OptionsLeave

The three hand-offs of start game, and the return to file select. Index Ch `$82:EE92`: handle
fading out with delay 0; at forced blank NMI enabled, delay and counter 0, index 4 (567). Index
4 `$82:EEB4` runs in the frame that ends 568: a loading game state of 0 starts game state 1Eh
(the intro) with the cinematic function `$8B:A395` (`IntroBegins`); 22h, a file saved after the
station's escape, starts game state 22h with the explosion's function `$8B:C11B`
(`EscapeBegins`); 5, main gameplay, was sent to index 4 at once by `$82:EDB1` without the fade
(`PlayContinues`). P2 gives only these three. The frame that ends 568 only sets the game state,
the intro's function, the cursor 0 and the fade's delay and counter 0, so nothing on screen
changes there; it is the intro's first frame end, and the intro reads the language `$09E2` and
keeps the bindings (it saves and restores them). Index Bh `$82:EE6A`, the same fade, ends in
game state 4.

## RAM mapping

| Quantity | Source | Notes |
| --- | --- | --- |
| screen | the record's `inidisp` | |
| cursor | `$7E:099E` | the item's index in the menu's order |
| block size | `$7E:0057` high nibble, mosaic on when its low bits are 03h | 03h at 488 and 520, F3h at 503 and 535, 00h when off |
| settings | `$7E:09E2` Japanese text, `$7E:09EA` weapon-reset (labelled ICON CANCEL on screen), `$7E:09E4` moonwalk | all 0 from the new file; moonwalk 1 from 519 |
| stage, load step, target | `$7E:0DE2`, `$7E:0998` | compared through their effects |
| how the file begins (P2's input) | `$7E:D914` (loading game state) | 0 intro (a new file), 22h escape, 5 play |
| sounds asked for | the library 1 sound queue `$7E:0656..0665`, its end index `$7E:0646` | the entries added since the last finished cut: `CursorMoved` 37h, `Selected` 38h (37h, 38h at 488 and 519; 38h at 520 and 552) |
| finished frames, random number (the frame owner's) | `$7E:05B6`, `$7E:05E5` | as in the title notes; no frame of work here holds across a frame end |
| whether the game had its work done (borrowed) | the record's cut kind | |

## Harness contract

Row `any-power-on`, continuing file select's run: `OptionsBegin` at 464 hands over `Loading`
with `Blank(0)` and the new file's settings (all off) from P2. The run plays 465 to 567; the
frame that ends at 568 is the intro's (row `any-intro-0568`), which the options hand the
settings and the new file, and which stops the run as Unsupported until the intro is owned.

Compared, per cut:
- `screen`, every cut;
- `finished frames` and `random number` (the frame owner's), as in the title notes;
- at finished cuts: the block size (off or its size), the three settings, and the cursor
  wherever the State holds one (from 472 on);
- the sounds asked for, at finished cuts: the library 1 queue's entries added since the last
  finished cut, against the sounds asked for since it. Expected and played are sorted, so the
  order within one interval is not compared.

Planted faults: leave the moonwalk off on Left, which must fail on the settings at 519; and Up
moving the cursor down, which must fail at 488 on the cursor (the cursor reads 1, not 4).

Not compared, and why (the root's ruling: the picture is not compared): VRAM, CGRAM and OAM (the
menu text, the highlights, the missile, the borders); the controller bindings (unchanged here,
the chosen file's).

Borrowed (named in the runner's `BORROWED`), under MODEL decision 35: whether the game had its
work done by each frame end (the cut kind).

## Reference

`$TEMP/collab/title-752/scratch/opmodel.py` runs the spec on every vector, and 37 of 37 agree.
`oppercut.py` runs it per cut over `any-power-on` 465 to 567 from the hand-over, and all 103 cuts
agree:
- screen 103 of 103;
- finished frames 103 of 103;
- random number 97 of 97 (the finished cuts);
- block size 97 of 97;
- settings 97 of 97;
- cursor 95 of 95;
- sounds asked for 103 of 103.

A do-nothing model (the hand-over State frozen) agrees at 0 of 103 cuts: screen 11 of 103,
finished frames 0 of 103, random number 0 of 97, block size 37 of 97, settings 48 of 97, cursor 0 of 97, sounds 99 of 103.
