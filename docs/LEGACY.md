# Legacy register

Every known difference between the engine, the oracle and the real game, and every behaviour we
keep only because a check needs it. Code marks a kept behaviour with a `LEGACY(<kind>)` comment,
so the register and the code can't drift:

```sh
grep -rn 'LEGACY(' --include=*.rs crates
```

Add an entry in the change that introduces the difference. When a check no longer needs an entry,
remove the behaviour, its tags and its entry in the same change.

## The engine: original overflows it doesn't reproduce yet

The engine has no RAM image, so where the original reads past a table it can't read "whatever lies
there". Each case is listed until a check needs it; then it becomes a named behaviour with the
original's bytes as data.

- **HUD digits past their cells** (`game::hud`, `$80:9DBF`/`$80:9DD3`): a count above 99 in a
  two-digit cell, or above 999 in a three-digit one, makes the original index past the digit table
  into the code at `$80:9DE7` and draw those bytes as tiles. The engine leaves the cell unchanged. No
  vanilla count gets there; a Map Rando or custom-content count could.
- **Scroll areas past the room's table** (`game::scrolling`, `$80:A528`, `$80:A731`, `$80:A8FC`): with
  the screen below the camera past the room's last row, the scroll zone routines read the scroll
  buffer past the room's screens, as scrolling down does when the camera moves into the bottom row
  ahead of its ideal position (`area-brinstar` frame 414). The engine keeps that part of the buffer
  (`RoomScoped::stale_scrolls`, decoded from the oracle) and reads it as the original does; a byte no
  area has (`StaleScroll::Unknown`) is neither red nor blue, so the camera goes through it. Past the
  50-byte buffer the original reads the RAM after it, which the engine reads as red. The engine's room
  load leaves the stale part empty (red) until the room transition joins.
- **Samus spritemaps with a zero pointer or past the table** (`data::pose::spritemap`, `$81:89AE`): 183
  of the 2,096 pointers at `$92:808D` are zero, and the original reads such a spritemap's count from
  `$92:0000`, low work RAM; an index past the table reads a pointer from what follows. The engine draws
  nothing for either. No pose and frame an animation reaches selects one.
- **The mini-map's edges** (`game::mini_map`, `data::map`, `$90:A9A2`, `$90:AB75`, `$90:AAA8`): the 3 by
  5 window around Samus reads the explored bitmap, the map station bitmap and the map tilemap a row
  above and below and two columns either side, in layout order (`data::map::LayoutIndex`). On the
  map's top or bottom rows, and for a room placed off the 64 by 32 grid, those reads leave the area's
  tables and the original reads what follows (the next area's maps, the variables around `$07F7`). The
  engine reads no cells and the blank tile there. Reads past the map's outer edge
  (`Crossing::PastMap`) follow the original's. Slice 0's rooms are inside the edges.
- **Level columns and rows past the level buffer** (`game::bg_graphics`, `$80:A9DE`, `$80:AB78`): the
  engine reads the buffer as the original holds it up to its 3200h blocks (the BTS and layer 2 left
  after layer 1, then the `$8000` fill, `RoomScoped::level_tail`). Past those, or past layer 2's copy,
  the original reads what earlier rooms left in `$7F`, and the engine reads block word 0.
- **Tile numbers past the tile tables** (`game::bg_graphics`, `data::tileset::TileTables`, `$82:E7D3`):
  a block whose tile number is past what the CRE's and the tileset's tables fill reads `$7E:A000` past
  them, which an earlier room or the boot left. The engine draws zero words. Vanilla rooms stay within
  their tables.

## Tolerated values kept as named variants

- `data::pose::PoseFacing::Unknown(u8)`: the unused poses $21 and $22 (`$91:B731`, `$91:B739`) hold X
  direction bytes 1 and 2, neither left (4), right (8) nor forward (0). The original's facing tests read
  the left and right bits, so it tolerates them; the byte goes back out unchanged. No row reaches them.
- `game::state::video::BBusRegister::Other(u8)`, `DmaControl::unused` and the mirror encodings of
  `DmaPattern` and `DmaAddressStep`: a DMA channel nothing programmed holds power-on bytes (BBAD $FF),
  and the hardware ignores DMAP bit 5 and repeats patterns 1 to 3 as 5 to 7. The registers are
  compared state today, so every byte decodes and goes back out unchanged. They leave the State as
  uncompared leftovers (MODEL decision 3).
- `game::state::room::StaleScroll::Unknown(u8)`: a stale scroll byte no area has (above).

## The oracle

- **`LEGACY(verifier)`** in `crates/oracle/src/streams.rs`: the `Definition::Legacy` streams, which
  compare with the old lock `scenarios/lock.toml` while `oracle corpus` still reads it, zero the WRAM
  that lock's `wram` digests zeroed (`LOCK_WRAM_MASK`, end-exclusive WRAM offsets): the direct page
  `$0000-$0051` and the stack `$1F5B-$2000`; `$00AD-$00B1`, `$05BC-$05C5`, `$05ED-$05F5`,
  `$060B-$0617`, `$077E-$0783`, `$078F-$0791`, `$07B7-$07B9`, `$0933-$093D`, `$0B24-$0B28`,
  `$0D82-$0D90`, `$0D9C-$0D9E`, `$0DD2-$0DD8`, `$0E02-$0E04`, `$0E20-$0E46`, `$0E4A-$0E4C`,
  `$0E54-$0E5A`, `$1784-$178C`, `$17A8-$17AC`, `$1834-$183C`, `$184A-$185C`, `$186E-$1886`,
  `$189A-$18A8`, `$1966-$196C`, `$EF74-$EF78`, `$F37A-$F380`; and the bug-fix counter `$00C2-$00C4`,
  which only the old patched oracle wrote. The V1 streams and the format-2 lock mask nothing. Remove both when `oracle corpus` reads
  `scenarios/locks/lsnes.toml` instead ([ORACLE §9](ORACLE.md#9-open-items)).
- **`LEGACY(seed)`** in `crates/oracle/src/seed.rs`: lsnes room captures (`Start::Capture`) hold no
  SPC700, DSP or timer state, so a migration seed resumes the SPC700 at the driver's idle loop (`$15BF`,
  SP `$CF`), the CPU-to-APU ports from the driver's copy at SPC `$00-$03`, the timers from their RAM
  shadows, and the DSP from reset with echo writes off. In some rows the resumed driver answers no sound
  request, so uploads, door transitions and message boxes stall. Native seeds retire this entry.
- **lsnes's cut at line 241** without a previewed pad: an NMI whose writes pass that line shows a step
  later (`Cut::NmiStillRunning`). The format-2 lock is recorded from frame pads with preview, so it has
  no such cut ([ORACLE, MEASUREMENTS (g), (i)](ORACLE.md#appendix-a-measurements-measurements)).
- **The five `ppu` fields lsnes doesn't expose** (`nmitimen`, `htime`, `vtime`, `hdmaen`, `memsel`) are
  not compared (ORACLE Q4).
- **A one-frame lag shift after an APU upload** between oracles is an oracle difference, not engine
  behaviour; the lag pattern is pinned to lsnes ([ORACLE appendix B](ORACLE.md#appendix-b-emulator-accuracy-accuracy)).
