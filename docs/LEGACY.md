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

- **Scroll areas past the room's table** (the camera domain, `docs/behaviours/world-camera.md`
  `BelowTheRoom` and its open question 1, which reads the screens below the room as one run
  continuing the room's, as the room's loading left them, and past the run `Closed`; today
  `game::world_camera_bridge` over the camera model, `$80:A528`, `$80:A731`, `$80:A8FC`): with the screen
  below the camera past the room's last row, the scroll zone routines read the scroll buffer past
  the room's screens, as scrolling down does when the camera moves into the bottom row ahead of its
  ideal position (`area-brinstar` frame 414). Every read indexes the one buffer by row × width +
  column, so the bytes past the room are one run. A room with a scroll pointer copies all 50 bytes
  from the ROM whatever its size (`$82:EB28`), so its run is the ROM bytes after its screens; a room
  with preset scrolls fills width × height only and leaves the rest as the last room left it. The
  engine keeps that part of the buffer (`RoomScoped::scrolls_below`: the importer's table, or
  decoded from the oracle) and reads it as the original does; a byte no area has (`ScrollArea::Unknown`) is neither red nor
  blue, so the camera goes through it. Past the 50-byte buffer the original reads the RAM after it,
  which the engine reads as red: the four shinespark rows read indices 50 and 51 there 409 times,
  RAM `00 00`, red, so they agree. The engine's load of a room with preset scrolls leaves the part
  empty (red) until the room transition joins.
- **Samus spritemaps with a zero pointer or past the table** (`data::pose::spritemap`, `$81:89AE`): 183
  of the 2,096 pointers at `$92:808D` are zero, and the original reads such a spritemap's count from
  `$92:0000`, low work RAM; an index past the table reads a pointer from what follows. The engine draws
  nothing for either. No pose and frame an animation reaches selects one.
- **Samus's animation timing spec's defined domain** (the clean-room spec
  `docs/behaviours/samus-appearance.md` §1.5; the original's reads in
  `docs/behaviours/samus-appearance.decode.md`, "Findings behind the spec's choices"): stated
  simplifications. The first happens in vanilla; the others are unreachable there.
  - **Five sounds waiting** in library 3's queue as she starts boosting, the blue suit failing
    (the bank log's note at `$90:852C`: "sometimes (mostly in heated rooms)"). The original indexes
    the speed boost timer table `$91:B61F` and the delay table `$91:B5DE` at 0Ah, past both: the
    laps come from `$91:B629` (pose 0's definition, 0000h), so 0, and the lap's first hold is a
    byte of low RAM, read through a pointer made of `$91:B5E8`'s bytes, which can hold step 0 for a
    long time (the note's own words: a timer of 0 "lasts 100h frames", "the apparent
    sliding-on-one-leg pose"). The next lap end's word `DEC` turns 0400h into 03FFh: she boosts for
    that one lap and then sits at stage 3 for 255 laps, the blue suit failing. The spec leaves 5
    waiting outside its domain; the engine (once the domain lands) has no RAM image to read the hold
    from.
  - A clip that ends with no pose change, or a turn's end that gives way and the pad's jump not
    taken (`ClipEnds`): the original's next frame moves on past the end byte into its operand and
    the bytes after the list. Every vanilla end hands over a pose, and the pad's jump is taken.
  - An `again`, a `branch` or a rest leading to another control, a delay list's last byte a delay
    (the clip moving on past its list), and an ending clip with a mark, a running, standing or kept
    pose (spec §1.5, `ended` from the frame's start): none in the game's lists.
- **Samus's motion in the demos** (the samus-movement domain, `docs/behaviours/samus-movement.md`
  `AnimationEnds` and §10; `docs/behaviours/samus-movement.decode.md`, "Round 18's amendments"): the
  spec covers play. While a demo plays (`0A60` = `$90:E91D`), the original's end unless a jump
  (`$90:8370`) hands over whatever the pad, where the spec's gives way to the pad's take-off or
  spin. The engine's pose comes from samus-movement's hand-over (`game::samus_movement_bridge::step`), so a
  demo pressing jump on a turn's or a landing's last frame would take off where the original
  stands. No recorded frame has one.
- **The HUD spec's defined domain** (the clean-room HUD, `docs/behaviours/interface-hud.md` §1.5; the
  original's reads in `docs/behaviours/interface-hud.decode.md`, "Counts past their cells" and "The map's
  outer edge"): stated simplifications, each unreachable in vanilla.
  - Missiles are defined up to 999 and super missiles and power bombs up to 99; past that the
    original draws its digit table's neighbours and the code at `$80:9DE7` as tiles. Vanilla
    capacities stop at 230, 50 and 50. The engine (`game::interface_hud_bridge`) stops the Session with
    `HudFault::CountPastCells` there.
  - An area with no map (the debug area, whose pointers name none: the engine shows a blank map
    that charts nothing), where the original reads what lies past its tables.
  - An auto-cancel item other than the selected one stops the Session
    (`HudFault::OneShotNotSelected`), and the harness counts a record holding one a defect. No
    routine of the original writes one (`docs/behaviours/interface-hud.decode.md`, "State").
  - A mini-map cell outside the 64 by 32 map (rows -1, 32 and 33, which the window reaches from her
    cell in row 31, in row 1 at columns 0-1, or in rows 30-31 at columns 62-63, and her own cell
    below the map) is unexplored, uncharted and the map's blank, and exploring it does nothing. The
    original reads the right page's row 0 for the left page's row 32 and the left page's row 31
    for the right page's row -1, and the RAM around the explored and charted bitmaps and past the
    area's tilemap for the rest (the left page's row -1, the right page's row 32; `$90:A91B`,
    `$90:AB75`). Vanilla maps draw and chart only columns 2-57 and rows 1-22; a randomised map
    could reach these cells, and the column rules themselves stay in the spec for it.
  - A frame where the mini-map is turned on and Mother Brain's first form falls takes the turn-on
    first (spec §5), so the mini-map stays off and doesn't update. In the original both are enemy
    AI run by the main enemy routine (`$A0:8FD4`: Crocomire's `$A4:90C4`, Mother Brain's
    `$A9:8809`), so the enemy processed later wins; with Crocomire's later, the mini-map would be
    on and update that frame (`$90:E722` runs after). No vanilla room holds both bosses.
- **Level columns and rows past the level buffer** (`game::bg_graphics`, `$80:A9DE`, `$80:AB78`): the
  engine reads the buffer as the original holds it up to its 3200h blocks (the BTS and layer 2 left
  after layer 1, then the `$8000` fill, `RoomScoped::level_tail`). Past those, or past layer 2's copy,
  the original reads what earlier rooms left in `$7F`, and the engine reads block word 0.
- **Tile numbers past the tile tables** (`game::bg_graphics`, `data::assets::tilesets::Tilesets::block`,
  `$82:E7D3`): a block whose tile number is past what the CRE's and the tileset's tables fill reads
  `$7E:A000` past them, which an earlier room or the boot left. The engine draws blank cells. Vanilla rooms stay within
  their tables.

- **Emptied item places past the engine's bits** (`game::run_progress_bridge`'s `store_emptied`,
  `$84:8899`): the engine keeps the 512 bits of `$7E:D870`. A room argument past them makes the
  original set a bit in the work RAM that follows; the engine drops the place, so its item would
  come back. The release's 100 item PLMs name places 0 to 9Ah; a custom room's could name more.
- **The debug area's item pickups** (`game::run_progress_bridge::take`): the progress model has no
  area for the debug rooms (area 7), so their item pickups report area 0's, Crateria. Taking an
  item reads no area (only a boss's, a map's and a station's reports do), so nothing differs; a
  report that reads one would have to name the debug area first.
- **A scroll cell with no scroll reaction** (the blocks domain, `docs/behaviours/world-blocks.md`
  `ScrollCells`; `$84:B3A6`): when she touches a scroll cell whose block no scroll PLM sits on,
  the original searches the PLMs for one, finds none and loops for ever. The spec leaves such a
  cell to the room's loading, which is to refuse the room. No room of the game has one (the
  importer's scan of the 261 rooms the doors reach, 322 states, 265 scroll cells, and of the two
  rooms no door reaches, the unused room `$8F:B3E1` and the debug room `$8F:E82C`); a custom room
  could. The engine has no room loading to refuse it yet, so its Session stops with
  `SessionFault::Blocks` when a touch meets such a cell (`game::world_blocks_bridge`), as it stopped at
  the hang before.
- **A stand-in chain that loops** (the samus-movement domain, `docs/behaviours/samus-movement.md`
  `OtherBlockKinds`; `data::level`, `game::samus_movement_bridge`, `game::collision::inside`): when her
  moves, her looks or her inside points meet an extension block whose chain of extensions comes
  back on itself, the original follows the chain for ever and hangs. The importer refuses a room
  state whose level has such a chain once its start has written its extensions' and scroll PLMs'
  blocks (`LevelError::StandInLoop`), as the room loads. No room of the game has one: the walk of
  the 261 rooms the doors reach (322 states) and of the unused and debug rooms loads every level
  (`every_level_the_game_holds_loads`). One room needs its start's blocks for that: in `$8F:A011`
  two rightwards extensions and a level extension loop until the scroll PLM placed after them
  writes its cell. A custom room could still close a loop as it runs: a reaction's show can make a
  cell a stand-in that reaches as its BTS says, and the importer doesn't check where the room's
  reactions show, nor the blocks the items and the PLMs no model runs write. So each engine follow
  stays bounded, and a Session that meets such a chain stops, with
  `SessionFault::Samus(SamusFault::StandInLoop)` at the first cell her moves meet, or
  `SessionFault::Blocks(BlocksFault::StandInLoop)` at her inside point's. None of the 44 cached
  rows meets one.
- **Reacting cells whose tables end before their mark** (the blocks domain,
  `docs/behaviours/world-blocks.md` §1's marks; the decode notes' importer's scan): each asker of a
  reacting cell (her touch, her inside points, a shot, a blast, the grapple) reads the cell's
  reaction from its own table by the cell's class and BTS, and some tables end before others: the
  special block's bombed table at BTS 10h (and 8 per area), the special touch, inside and
  shootable tables at 50h (16, or 8 for the shootable block, per area), the bombable tables at 10h.
  Past an end the original reads the next area's table, pointer words or code, so one cell can be
  a crumble block to her touch and a speed block's reveal to a bomb. The importer refuses a room
  with such a cell, but for a special block's BTS 10h..19h and 1Eh..4Fh: past the bombed table
  those read `$B62F`, a reaction that only holds its place, and the importer types them as the
  original reads them (met as crumble blocks by a blast). The stations' access cells and the save
  stations, which their placements write when the room starts (49 cells in 33 rooms), are such
  cells, so a power bomb set off in their rooms holds a place for each it reaches. No room of the
  game has a refused cell (the 261 rooms the doors reach and the two no door reaches, `$8F:B3E1`
  and `$8F:E82C`).
- **A door block whose place is past its room's door list** (the samus-movement domain,
  `docs/behaviours/samus-movement.md` `OtherBlockKinds`; `data::room::DoorList`,
  `game::samus_movement_bridge`; `$94:938B`, `$94:93CE`): a door block names its door by its BTS, the place
  in the room's door list, and the list holds no count of its own. The importer reads one door for
  each place up to the last a door block of any of the room's states names, so every door block of
  the level data has its door. A block written as the room runs (a PLM's or a reaction's) could
  name a place past that, where the original reads the bytes after the list as a door header's
  pointer and goes where they lead. The engine answers such a door solid.
- **Reactions the original would misread** (the blocks domain, `docs/behaviours/world-blocks.md` §2,
  `BombBlocks`; `$84:85E0`, `$84:CF0C`): a `Show` of 0 frames waits 65,536 frames in the original,
  one of 8000h or more is read as an instruction, and a bomb's blast skips the first three bytes of
  a bomb block's broken list whatever they hold. The importer refuses a reaction with a `Show`
  outside 1 to 32,767 frames, or a bomb block whose broken list doesn't begin with a sound; the
  game has none.
- **Patterns past the room's edge** (the blocks domain, `docs/behaviours/world-blocks.md` §2;
  `$84:861E`): a show draws by block index, so a pattern cell past the room's right or left edge is
  drawn on the next or the previous row, and one past the top or the bottom is written outside the
  room's cells. The spec states patterns in cells, and the room's loading refuses a room whose
  reacting cells' patterns reach past its edge; no room of the game has one. Until the engine's
  room loading refuses it, the engine draws no such cell (`game::world_blocks_bridge`).
- **The grapple on a bomb, a super missile or a power bomb block** (the blocks domain,
  `docs/behaviours/world-blocks.md` `BombBlocks`, `SuperMissileBlocks`, `PowerBombBlocks`; `$84:CEDA`,
  `$84:CF2E`, `$84:CF67`): these blocks decide by the weapon of the projectile last handled
  (`$0DDE`), which the grapple doesn't set, so the original may break one under the grapple as a
  bomb's, a super missile's or a power bomb's hit when that projectile's slot still holds one. The
  engine never starts them under the grapple, as for a beam. Not seen on record.

## The engine: memory a model keeps that the engine can't hold yet

- **Samus's animation timing in the engine** (the samus-appearance domain,
  `docs/behaviours/samus-appearance.md`; `game::samus_appearance_bridge`): the engine runs the model, and
  gives it what the domains it lacks would: no slowdown, no start's slowdown and never submerged
  until the liquids join (R3, S2); the sounds the model asks for (the wall jump's spin, the boost)
  go nowhere and none wait, which every boosting lap on record had, until the sound queue joins
  (U1); and the boost stage it keeps is read by none of its readers (the shinespark, the echoes,
  her glow, the blocks, the enemies) until they join. Her animation keeps a step only in the
  domain's poses: in a pose outside it (the shinespark's, the grapple's, the x-ray scope's,
  knockback, the drained fall), which the engine doesn't reach, her animation stands still, she
  is drawn with no frame, and the RAM translation refuses a record in one as undecodable.

- **Jump held through outside her motion's poses** (the samus-movement domain,
  `docs/behaviours/samus-movement.md` `Jumping`; `game::samus_movement_bridge::state`; `$90:EB35` →
  `$90:EAB3`): the original counts how long jump has stayed held (`0AF4`) once each gameplay
  frame at the draw, whatever her pose. The engine counts it in the model's step, which runs only
  in the domain's poses, so in another pose (the knockback's, a shinespark's, the grapple's, the
  gunship's) the count stays where it was and a take-off from a landing after it can differ. On
  record, the count moves in such a pose on 307 gameplay frames over 13 rows of the 44-row cache
  (boss-kraid-defeated 31, boss-phantoon 8, chain-0059-A293 38, chain-0232-D5A7 16,
  event-zebes-awake 51, samus-ceres-push 41, samus-grapple-release 2, samus-grapple-walljump 1,
  samus-screw-attack 71, and the four shinespark rows 13, 13, 9 and 13). The lockstep rows
  (area-brinstar, item-morph-ball) have none. The entry goes when the count is the draw's, run
  every gameplay frame.

- **The blocks' askers the engine doesn't have yet** (the blocks domain, `docs/behaviours/world-blocks.md`
  §1.3; `game::world_blocks_bridge`): her moves meet each reacting cell by its body and the touches reach
  the blocks after them, so a touch's answer (`AnswersToHerMotion`, `FullPlacesLetHerThrough`)
  changes nothing she does; the blocks read no boost stage her animation keeps, so to them she never
  dashes, shinesparks or screw attacks; her beam's hits reach only shootable blocks, and the bombs,
  power bombs, missiles and the grapple hit none; the run is never paused; and the sounds the
  scripts ask for go nowhere. Slice 0's rooms meet none of it: their reacting cells are scroll
  cells, which are air to her, the shot blocks her beam breaks, and the item. Each goes when its
  asker joins: her motion asking the blocks (samus-movement), her weapons (W1), the grapple (S6),
  the room's loading and the doors (R1, R2), the sound queue (U1).
- **Her passage through a door** (the samus-movement domain, `docs/behaviours/samus-movement.md`
  `OtherBlockKinds`; `game::samus_movement_bridge`; `$94:938B`, `$94:93CE`): a door that leads to a room
  lets her move on, as in the original, but the engine has no doors yet, so it starts no passage
  (game state 9 and the door pointer `$078D`, set as she reaches the door): she moves on into the
  door's cells and the room goes on running. The harness compares the frame that reaches the door,
  her move, and skips those after it, which aren't gameplay. The entry goes with the doors (R1, R2).

- **The Ceres doors in the engine** (the boss-ceres-ridley domain,
  `docs/behaviours/boss-ceres-ridley.md`; `game::boss_ceres_ridley_bridge`): the engine runs the
  model for the doors of parameters 0, 1 and 4 in the old enemy runner, and gives it what the
  domains it lacks would. The engine holds no screen shake and no sound queue, so the shakes the
  doors start in the escape (`EscapeShakes`, `$A6:F773`) and the opening sound (`$A6:F6BD`) go
  nowhere, and each frame's first door reads no shake running; the harness compares the shake on
  every frame with a door (5,056 frames over the cache, all but the door passage's frames with one
  running) and the sound is compared nowhere. They go when R3 holds the shake's
  timer and U1 the sound queue. The station's stage (`$093F`) is State the engine decodes and no
  domain writes: Ridley's getaway and the escape's start are later pieces of B8. Her touch and her
  shots on a door are the old runner's (`$A0:9758`, `$A0:A1F7`): the engine's solid-enemy
  collision stops her at a solid door, but it deletes no shot that meets one, where the original
  deletes every shot but plasma; no lockstep row has a shot meet a door. The engine places no
  door at a room's start (it has no room loading; the bridge's `place` waits for R2 and C1), so
  the doors' colours (`TheDoorsColours`) are the seed's; the harness compares the model's
  placing, colours included, on the 11 frames a room places its doors (H1 and H2 among them). Each
  goes with the domain named: R3, U1, B8's later pieces, C1, R2.
- **The door passage in the engine** (the world-doors domain, `docs/behaviours/world-doors.md`;
  no bridge yet): the engine runs no passage. Its only game state is gameplay, so the seed
  decoder refuses a record in a passage (game states 9, Ah and Bh), and a row's frames from her
  touch of a door until play goes on in the next room are not started by the free-run gate or
  lockstep. The model is proven on its own: the harness (`verify::harness::doors`) and its free
  run hold every passage frame of the cache. The engine runs it once the domains it asks join:
  R2's room loading (the next room built, its tiles, contents, target colours and the enemies'
  colours it writes, its CRE bits and the doorway's arrival in the World's records), T1's cost
  per event (until then the harness takes a frame of work done when its cut is `R`, and the
  contents' fetch done at once, which the records bound only by the slide's end) and U1's
  answers (every sound request sent, the music ready); the passage's room calls, sound requests
  and her sounds have no engine reader until then. Kraid's corpse (C1) and the Ceres elevator's
  beacon (B8) write colours during the fades, which the harness masks by name: Kraid's on each
  frame the enemies run, the beacon's on the `Brightening` frame of work at the fade's step 0.
  The loading writes sprite palettes 6 and 7 on frames in `Arriving`, not only the one it ends
  on; the harness takes them from the next record on every such frame, which counts the
  frame the colours change unverified.

## The engine: slots it carries but doesn't run

A stop-gap so Ceres's seeds start: the decoder admits an enemy or PLM slot of the ids below,
which no model runs yet, as `EnemyBody::Unmodelled` or `PlmBody::Unmodelled`, holding the words
the seed held (the species or ID as a named kind; an enemy's instruction and spritemap pointers and
six AI words, a PLM's instruction pointer, room argument and variable), which go back out
unchanged where lockstep compares them. Only these ids are admitted (`verify::enemy::UNMODELLED_SPECIES`,
`verify::plm::UNMODELLED_PLMS`); any other id the engine doesn't hold is still refused by name. The
engine never selects such an enemy (`LEGACY(unmodelled)` in `game::enemy::select`), so it runs no
AI or instructions, its frame counter and timers stand still, it isn't drawn and nothing touches
it; such a PLM is no place of the blocks' (`LEGACY(unmodelled)` in `game::world_blocks_bridge`) and
the items' pass skips it, so it never runs or deletes itself. No Ceres row has a Tick fully equal in
lockstep but `chain-any-0003-DFD7` (178 of 181), whose doors the engine runs, and `0004` (101 of 101);
the free-run gate's Ceres rows start, green on `sram` and `timing` only. Each entry goes, with its
id in the list, when the round named replaces it:

- **Enemy `$A0:E23F`, a Ceres door, of parameter 2, 3, 5 or 6** (the shaft's cover in
  `chain-any-0001`, `0010` and `0011`; Ridley's room's door in `0005` and `0006`; the getaway's
  walls in `0006`): no opening, closing or blocking, no drawing and no contact. The doors of
  parameters 0, 1 and 4 are boss-ceres-ridley's (`EnemyBody::CeresDoor`, "The Ceres doors in the
  engine" above); the others are its later pieces, with C1 and AF3
  ([domains.md §4](architecture/domains.md#4-order)).
- **Enemy `$A0:E13F`, Ceres Ridley** (`chain-any-0006-E0B5`): no fight, getaway, drawing or
  contact. Replaced by B8 with C1 and AF3.
- **Enemy `$A0:E1FF`, the steam of Ceres's escape** (`chain-any-0009` to `0011`, up to 11 slots): no
  puffs, drawing or contact. Replaced by B8 with C1 and AF3.
- **PLM `$84:B7EB`, the timer that enables sounds** (`chain-any-0001-DF45`, slot 39, spawned by the new
  game): it never counts down, enables the sounds or deletes itself, so its slot stays taken.
  Replaced by W2's piece for Ceres (the PLM runner's pieces the stretch spawns).

## The engine: planned differences from the original

Differences the plan keeps on purpose, listed ahead of the work that brings them.

- **The sound driver's uploads** (the audio domains U1 and U2, the sound format A3,
  [ARCHITECTURE §11.6](architecture/ARCHITECTURE.md#116-what-isnt-compared-by-name)): at power-on
  the original uploads Nintendo's sound driver and its first song set to the sound chip through
  the APU ports (115,715 port writes in `any-power-on`'s frames 0 to 270), and uploads song sets
  the same way as areas and scenes change. The engine runs no code of the game (MODEL decision
  23), so it makes none of those upload writes: their bytes are an uncompared leftover (decision
  3), their time is T1's cost, and the replies the game reads stay compared on the `apu` stream.
- **The audio out** (U2's sequencer, the mixer E6, ARCHITECTURE D18): the original's sound chip
  mixes its voices with its own interpolation, envelope steps, echo and 32 kHz output; the
  engine's mixer in `runtime` plays the sequencer's events over A3's PCM samples with modern
  ones, so its audio out differs. Sound is compared on the game's sound requests and the
  replies it reads through the ports, and on the sequencer's events per Tick (which sound or note
  starts and stops when), never on the audio out (MODEL decision 23).

## Tolerated values kept as named variants

- `data::room::Door::HeaderInRam(u16)`: the unused room `$8F:B3E1`'s door list names a door header
  below the door bank's ROM, so a door block naming it would read its destination from low work
  RAM (`$94:93AC`). The decoder keeps the pointer; the world importer refuses such a door
  (`WorldError::DoorHeaderInRam`), and the movement harness answers it solid, as a door that leads
  nowhere. No door reaches the room, and no row enters it.

- `data::pose::PoseFacing::Unknown(u8)`: the unused poses $21 and $22 (`$91:B731`, `$91:B739`) hold X
  direction bytes 1 and 2, neither left (4), right (8) nor forward (0). The original's facing tests read
  the left and right bits, so it tolerates them; the byte goes back out unchanged. No row reaches them.
- `game::state::video::BBusRegister::Other(u8)`, `DmaControl::unused` and the mirror encodings of
  `DmaPattern` and `DmaAddressStep`: a DMA channel nothing programmed holds power-on bytes (BBAD $FF),
  and the hardware ignores DMAP bit 5 and repeats patterns 1 to 3 as 5 to 7. The registers are
  compared state today, so every byte decodes and goes back out unchanged. They leave the State as
  uncompared leftovers (MODEL decision 3).
- `data::level::ScrollArea::Unknown(u8)`: a scroll byte no area has, in the run past a room's screens
  (above) or in room $ADAD's bottom row, whose four-byte table runs into the next room's header. The
  camera tests only for red and blue, so it reads one as green; the byte goes back out unchanged.
  A scroll change list keeps one too (`data::plm::scroll_changes`), but the blocks model holds
  screens by kind, so `game::world_blocks_bridge`'s write-back compares and writes kinds: a change to a
  byte no area has writes green, and a change to green over a screen holding one writes nothing,
  where the original writes the list's byte (`$84:8B68`). Vanilla reaches neither: of the 367
  changes in every room state's lists, none holds such a byte or changes a screen holding one.

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
