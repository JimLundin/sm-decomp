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

- **Scroll areas past the room's table** (the camera domain, `docs/behaviours/camera.md`
  `BelowTheRoom` and its open question 1, which reads the screens below the room as one run
  continuing the room's, as the room's loading left them, and past the run `Closed`; today
  `game::camera_world` over the camera model, `$80:A528`, `$80:A731`, `$80:A8FC`): with the screen
  below the camera past the room's last row, the scroll zone routines read the scroll buffer past
  the room's screens, as scrolling down does when the camera moves into the bottom row ahead of its
  ideal position (`area-brinstar` frame 414). Every read indexes the one buffer by row × width +
  column, so the bytes past the room are one run. A room with a scroll pointer copies all 50 bytes
  from the ROM whatever its size (`$82:EB28`), so its run is the ROM bytes after its screens; a room
  with preset scrolls fills width × height only and leaves the rest as the last room left it. The
  engine keeps that part of the buffer (`RoomScoped::stale_scrolls`, decoded from the oracle) and
  reads it as the original does; a byte no area has (`StaleScroll::Unknown`) is neither red nor
  blue, so the camera goes through it. Past the 50-byte buffer the original reads the RAM after it,
  which the engine reads as red: the four shinespark rows read indices 50 and 51 there 409 times,
  RAM `00 00`, red, so they agree. The engine's room load leaves the stale part empty (red) until
  the room transition joins.
- **Samus spritemaps with a zero pointer or past the table** (`data::pose::spritemap`, `$81:89AE`): 183
  of the 2,096 pointers at `$92:808D` are zero, and the original reads such a spritemap's count from
  `$92:0000`, low work RAM; an index past the table reads a pointer from what follows. The engine draws
  nothing for either. No pose and frame an animation reaches selects one.
- **The HUD spec's defined domain** (the clean-room HUD, `docs/behaviours/hud.md` §1.5; the
  original's reads in `docs/behaviours/hud.decode.md`, "Counts past their cells" and "The map's
  outer edge"): two stated simplifications, both unreachable in vanilla.
  - Missiles are defined up to 999 and super missiles and power bombs up to 99; past that the
    original draws its digit table's neighbours and the code at `$80:9DE7` as tiles. Vanilla
    capacities stop at 230, 50 and 50. The engine (`game::hud_world`) stops the Session with
    `HudFault::CountPastCells` there.
  - A room whose top-left cell lies off the map (`HudFault::RoomOffMap`), and an area with no map
    (the debug area, whose pointers name none: the engine shows a blank map that charts nothing),
    where the original reads what lies past its tables.
  - A mini-map cell outside the 64 by 32 map (rows -1, 32 and 33, which the window reaches from her
    cell in row 31, in row 1 at columns 0-1, or in rows 30-31 at columns 62-63, and her own cell
    below the map) is unexplored, uncharted and blank `01F`, and exploring it does nothing. The
    original reads the right page's row 0 for the left page's row 32 and the left page's row 31
    for the right page's row -1, and the RAM around the explored and charted bitmaps and past the
    area's tilemap for the rest (the left page's row -1, the right page's row 32; `$90:A91B`,
    `$90:AB75`). Vanilla maps draw and chart only columns 2-57 and rows 1-22; a randomised map
    could reach these cells, and the column rules themselves stay in the spec for it.
- **Level columns and rows past the level buffer** (`game::bg_graphics`, `$80:A9DE`, `$80:AB78`): the
  engine reads the buffer as the original holds it up to its 3200h blocks (the BTS and layer 2 left
  after layer 1, then the `$8000` fill, `RoomScoped::level_tail`). Past those, or past layer 2's copy,
  the original reads what earlier rooms left in `$7F`, and the engine reads block word 0.
- **Tile numbers past the tile tables** (`game::bg_graphics`, `data::tileset::TileTables`, `$82:E7D3`):
  a block whose tile number is past what the CRE's and the tileset's tables fill reads `$7E:A000` past
  them, which an earlier room or the boot left. The engine draws zero words. Vanilla rooms stay within
  their tables.

- **Jump held through's signed window** (the samus-motion domain, `docs/behaviours/samus-motion.md`
  `Jumping`, jump held through; `$90:E92E`, `$90:EAB3`): the original tests the count of frames
  jump has stayed held with a signed compare against 9, so counts 8009h-FFFFh (32,777 to 65,535
  frames, about 9 to 18 minutes of holding jump) make a press as 1 to 8 do, and the count wraps
  from 65,535 to 0. The spec states it as original only: no record or vector holds jump that long,
  so nothing checks the engine there.

- **Emptied item places past the engine's bits** (`game::progress_world`'s `store_emptied`,
  `$84:8899`): the engine keeps the 512 bits of `$7E:D870`. A room argument past them makes the
  original set a bit in the work RAM that follows; the engine drops the place, so its item would
  come back. The release's 100 item PLMs name places 0 to 9Ah; a custom room's could name more.
- **The debug area's item pickups** (`game::progress_world::take`): the progress model has no
  area for the debug rooms (area 7), so their item pickups report area 0's, Crateria. Taking an
  item reads no area (only a boss's, a map's and a station's reports do), so nothing differs; a
  report that reads one would have to name the debug area first.

## The engine: outputs a domain doesn't export yet

- **A placement of 0 px after another mover** (the camera domain, `docs/behaviours/camera.md` §1;
  `game::samus_world`'s `settle` and the harness's `placement` in `verify::harness::samus_motion`;
  the decode notes' Remembering row, `$91:ECDA`..`$91:ED0E`): samus-motion reports where the camera
  follows her from on every frame, her start y where nothing placed her, so both callers take a
  placement only where it differs from her start y. The original writes the camera's previous y
  (`0B14`) on every pose change that places her, 0 px included: the standing and unmorphing
  transitions write her own y. On a frame where another mover (the elevator) moved her before her
  step, so that where the camera last saw her differs from her start y, such a placement makes the
  original follow her from her start y; the engine leaves the camera's previous y as it was and
  follows her from where it last saw her. No recorded row reaches it: over the 44 rows of the CI
  cache (20,558 frames of one frame of gameplay in one room), her y moves after the camera's turn
  on 7 frames (chain-0232-D5A7 4, item-charge-beam, item-morph-ball and item-reserve-tank 1 each),
  and her pose changes on none of the frames that follow them. The entry goes when samus-motion
  exports the placement as an option and the callers pass it through.

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
