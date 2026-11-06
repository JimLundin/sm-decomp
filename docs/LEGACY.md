# Legacy behavior register

Behavior we keep only because a check depends on it, not because the game
needs it. Each entry is marked in the code with a `LEGACY(<kind>)` comment, so
the register and the code cannot drift:

```sh
grep -rn 'LEGACY(' --include=*.rs sm crates
```

Do not add new dependencies of these kinds. When a check is changed so it no
longer needs an entry, remove the held behavior, its tags and its row here in
the same change.

## Retained standalone display schema

`LEGACY(display-state-schema)` retains `TransferSource::ActorTilemap` (wire tag 3)
and `DisplayState::actor_tiles.tilemap_bytes` for standalone display codec and
fixture compatibility. The owned Session transfer path rejects this source: real
actor tile staging uses `ActorState` provenance and reads the current canonical
actor/projectile owner. No production constructor of tag 3 was found in the
reviewed source. Remove the tag, retained payload and standalone fixture together
when their schema consumers migrate; they are not an independent live RAM owner.

## Faithful-oracle known divergences

`SM_ORACLE=faithful` runs the original game without snesrev's patches. Some
original behaviour copies uninitialised low-WRAM scratch (the direct page
$00-$50, the stack) into VRAM or RAM tables. The engine does not model that
scratch, and the effect has no gameplay consequence, so these patches stay
installed (verdict "a" in `tools/snesrev_patches.py`) and these faithful rows
are expected to fail at the listed frame. Owning the direct-page scratch in the
Session would retire the list.

| Patch | Faithful rows and first failure | What hardware shows |
| --- | --- | --- |
| `$82:831E` BG3/sprite tile DMA size | `samus-scripted-intro` @8256, `demo-attract-setup` @2619, route `chain-0011` @2054, any% chain room 11 (`chain-any-0011-DF45`) @2055, `save-death-reload` @2410 | WRAM $0000-$11FF in sprite VRAM $7700-$7FFF, overwritten before display |
| `$84:8243`, `$84:EFD3` unpause item-GFX load | `pause-menu-tour` @1378, `boss-kraid-pause-400` @588, `-950` @1138, `-1030` @1218, `-1960` @2148, route `chain-0211` @368 | garbage transfers into empty item-GFX slots |
| `$80:9AF3` broken mini-map init | `samus-scripted-intro` @8316, any% chain room 11 (`chain-any-0011-DF45`) @2115, `save-death-reload` @2470 | one frame of HUD mini-map tiles from scratch, while blanked |

## Rows waiting to be re-cut

`boss-bomb-torizo` retains a checkpoint that its seed and inputs cannot rebuild
into the defeat episode. The rebuilt replay agrees with the oracle but misses
the required boss fact. Keep the row pending until genuine controller inputs
reach that episode and its continuation checks pass.

## Oracle digest lock: scenarios kept on lockstep

The engine-only gate (`ci.yml` Verify) checks every seeded and boot corpus row
and every route room against the faithful oracle's digest lock
(`scenarios/lock.toml`). These scenarios are not locked; they stay on the patched
lockstep corpus and route jobs until the cause is fixed. Each carries an
`unlocked` note in its `scenarios/*.toml` file; `tools/scenario.py lock --check`
fails if a noted scenario locks or an unnoted one is missing; the notes only shrink. Ticks are the first
divergence of an engine free run against the faithful oracle's trace (the
localization `check-lock` prints), recorded 2026-11-01 on `land/batch13`.

| Cause | Scenarios and first free-run divergence | Lockstep over the span |
| --- | --- | --- |
| The faithful-oracle divergences above | `samus-scripted-intro` @8256 vram, `pause-menu-tour` @1378, `boss-kraid-pause-400` @588, `-950` @1138, `-1030` @1218 (wram, vram, ppu), `enemy-fire-arc` @119 (wram, actors), route `chain-0011-DF45` @2054, `chain-0211-C98E` @368, `chain-any-0011-DF45` @2055 | fails at the listed frame |
| Not yet identified: WRAM drift that lockstep masks, in all seeded item and pickup rows but `item-wave` | `item-bombs` @437, `-grapple` @408, `-gravity` @402, `-hi-jump` @458, `-ice` @411, `-morph-ball` @972, `-plasma` @526, `-screw-attack` @514, `-space-jump` @401, `-spazer` @440, `-speed-booster` @421, `-spring-ball` @780, `-varia` @403, `-x-ray` @541; `pickup-energy` @424, `-missile` @405, `-power-bomb` @500, `-reserve` @480, `-super` @578; `samus-grapple-connect-swing` @408, `space-jump-bubble-word-counter` @401, `xray-activate-release` @541 | passes: masked scratch or state lockstep re-adopts |
| Not yet identified: drift lockstep masks | `boss-kraid` and `boss-kraid-pause-1960` @1580 wram; route `chain-0330-92FD` @210 wram, `chain-any-0079-ACF0` @345 wram, `chain-any-0196-92FD` @431 wram, `chain-0086-AA41` @95 ppu (one Tick), `chain-any-0074-AFA3` @316 ppu; the re-seeded `boss-crocomire` @1801, `ice-return-business` @411 and `music-fir-selector10` @646 wram | passes |

## `LEGACY(verifier)`: RAM the frame comparison does not check

`normalize_snapshot_ram` (`sm/src/cpu_infra.rs`) copies these ranges from one
side to the other before each frame's RAM comparison, so a difference there
never fails a frame. "Theirs" copies the original's bytes over the engine's
(the engine does not model them); "mine" copies the engine's over the
original's (the engine keeps its own values there). Each range is a blind
spot: a wrong engine value in it goes unseen.

"Differs" counts, from the 67 chain rooms 1, 6, ..., 331 in the default mode
(batch 12), the rooms and frames where the two sides differ before the copy.
$0641, $0D1E and $1790 never differed there and are no longer normalized.

| Range | Copy | What it holds | Differs (rooms / frames) | What would retire it | Owner |
| --- | --- | --- | --- | --- | --- |
| $0000-$0050 | theirs | direct-page scratch | 67 / 19718 | the Session owns each scratch word its last writer leaves, as `PointerScratch` ($00-$03) and `SpawnScratch` ($12-$17) already do | Session, with each routine's module |
| $1F5B-$1FFF | theirs | the stack: return addresses and pushed registers | 67 / 14221 | none short of emulating the 65816 stack; permanent | verifier |
| $00AD-$00B0 | theirs | pointer to return-address parameters (the door VRAM update reads through it); region animated palette FX ($89:AC6A) | 48 / 99 | storing both where the original does | room entry (doors), room FX |
| $05BC-$05C4 | theirs | door transition VRAM update flag, destination, source and size | 47 / 98 | the deferred door VRAM update (`$80:D000` flip group) | room entry (doors) |
| $05ED-$05F4 | theirs | the multiplication product ($05F1-$05F4); the original never touches $05ED-$05F0 | 15 / 280 | `MultiplyScratch` covering the product, as it covers $05E9/$05EB | player collision |
| $0B24-$0B27 | theirs | Samus tiles temporaries (top, bottom) | 67 / 9144 | storing the tiles definitions' halves where `$92:8000` does | Samus drawing |
| $1966-$196B | theirs | current FX entry pointer and offset, palette FX bitset | 42 / 44 | loading the FX entry on the frame the original does (one frame per room load) | room FX |
| $060B-$0610 | mine | drop count, enemy hitbox entry counters, projectile counter copy; $060B's low byte is the Session's (Botwoon's spit angle), which the mini-map, the extended spritemap draw and scans, the drops and the palette copies now store (both-73) | 67 / 15449 | the HUD initialisation's broken mini-map setup ($90:A8EF) storing $060B from the stale direct-page $22, which the Session does not hold; then the bytes past $060B | actors, enemy drops, HUD |
| $0611-$0616 | mine | coroutine states (snesrev's bookkeeping in RAM the original never writes) | 66 / 280 | moving the coroutine state out of game RAM; permanent while it lives there | runtime |
| $077E-$0782 | mine | `MY_COUNTER` (snesrev's counters in RAM the original never writes) | 66 / 384 | as above | runtime |
| $078F-$0790 | mine | door BTS | 24 / 41 | storing the door BTS where the original does | room entry (doors) |
| $07B7-$07B8 | mine | room state condition pointer | 5 / 5 | as above, for the state selection | room entry |
| $0933-$093C | mine | scrolling and tilemap update temporaries | 67 / 16689 | storing the temporaries the scroll update leaves | scrolling |
| $0D82-$0D8F | mine | saved button configuration, which the grapple ($94:A861-$AE36), the ending's wavy Samus ($88:EC57) and the rotating Ceres elevator ($90:8C27, $BC46) use as temporaries; the grapple, the connecting grapple and the elevator now store them (`game_loop::Temporary`, both-73) | 5 / 382 | identifying the writer of the low bytes that differ on tick 0 of `elevator-up` and `space-jump-bubble-word-counter` | demo, grapple |
| $0DD2-$0DD7 | mine | collision temporaries (slope index, distance) | 67 / 16758 | storing the temporaries the block collision leaves | player collision |
| $0E02-$0E03 | mine | Samus bottom boundary position | 66 / 16128 | as above | player collision |
| $0E20-$0E45 | mine | enemy temporaries; the draw now stores $0E20 and $0E22 (both-73) | 58 / 12992 | storing $0E22-$0E25, $0E28-$0E2B, $0E2E and $0E32-$0E3B where spawn ($A0:9275), the distance and angle ($A0:ACA8), the enemy projectile collision ($A0:9894), the enemy set load and the solid collision leave them; the PR corpus never saw $0E20, $0E26, $0E2C, $0E30 or $0E3C-$0E45 differ | actors |

| $184A-$185B | mine | colliding-solid-enemy positions and deltas | 13 / 251 | storing them in the solid enemy collision ($A0:AAC8, $AB22) | player collision |
| $186E-$1873, $1876-$1885 | mine | enemy spritemap and border collision temporaries (the right border, $1874, is compared: the projectile and bomb scans store it) | 36 / 4260 | storing them in the enemy collision | actors |
| $189A-$18A5 | mine | Samus target positions and radius mirrors | 58 / 13225 | storing them in the Samus/enemy collision | player collision |
| $F37A-$F37F | mine | debug copies of an enemy's instruction list, index and ID (never read) | 55 / 9907 | storing them where the enemy processing does | actors |

To retire a row: model the range so it stops differing on the chain and
corpus, then delete it from `normalize_snapshot_ram` and from this table in
the same change.

Free run (docs/SCENARIO_HARNESS.md) masks the same ranges in its WRAM digest, and
one more, `ORACLE_RAM` in `crates/sm_host/src/verification/observation/machine.rs`:

| Range | What it holds | Why | What would retire it | Owner |
| --- | --- | --- | --- | --- |
| $00C2-$00C3 | snesrev's bug-fix counter, the oracle's patch selector; the original never uses it | only an oracle frame writes it; lockstep's sides share RAM, so only free run sees it (from the first Tick of a seeded row) | the oracle keeping its patch selector outside game RAM | verifier |
| `LOCK_SESSION_MASK` (`crates/sm_host/src/lock_masks.rs`): the ranges the lock's Session streams decode with zeroed, as they stood when those streams were recorded | retiring a row would otherwise invalidate them | the lock dropping `wram`, with one re-record | verification |
| `LOCK_WRAM_MASK` (`crates/sm_host/src/lock_masks.rs`): 31 ranges, those above plus $0D9C-$0D9D, $0E4A-$0E4B, $0E56-$0E59, $17A8-$17AB, $1834-$183B, $1874-$1875, $18A6-$18A7 and $EF74-$EF77, which `NORMALIZED_RAM` no longer masks | the oracle digest lock's `wram` stream (recorded, checked and free run) still zeroes those bytes | the committed lock's `wram` digests were recorded with that mask; narrowing it would need a full faithful-oracle re-record. Lockstep and the Session compare use `NORMALIZED_RAM` | frozen until the lock drops `wram` (docs/handoff/session-compare.md) | verifier |

## `LEGACY(video-observer)`: typed video preserves register observer boundaries

`rtl::samus_vram_mode` and `rtl::samus_vram_pointer` retain the existing
`observe_register_write` hook after each typed mode/address byte update. The
background verifier can change the loaded ROM at these exact boundaries.
This is verification compatibility, not an emulated PPU write or a second video
producer. Remove these hooks when that verifier consumes equivalent typed video
effects and preserves its source-mutation, failure-prefix and snapshot checks.

## `LEGACY(original-overflow)`: the original's reads and stores past their tables

Some original routines index a table with a value that can leave it, and the
CPU reads or stores whatever RAM lies there. A typed index into the table's
Session owner would panic or give a different result, so these paths keep the
original RAM address. They go through `crates/sm_game/src/legacy_overflow.rs`,
whose functions resolve the address exactly as the address API they wrap (the
projected bytes, which already own the neighbouring RAM). Each call site carries
a `LEGACY(original-overflow)` comment; `tools/cycle_report.py` counts the calls
as kind `legacy_overflow` and `tools/ram_ratchet.py` reports them as
`engine_legacy_overflow_sites`. Where an index is in range, the site uses its
typed owner and only the out-of-range branch goes through the module. Its
`SlotBytes` trait is the slot paths' byte resolution (the actors' extended
RAM from `Actors::extended` and every other byte through the projection on a
lease); it replaced the retired `SlotAccess`.

| Family | Sites | What the original does | What a hardware-faithful typed replacement needs |
| --- | --- | --- | --- |
| Crocomire's melt | `crocomire_reactions::erase_melt_columns`, `data_bank_byte`, `set_scroll_line`, `load_melting_tiles`; `boss_lease::bg2_staging::set_unbounded` | Erase melting pixel column ($A4:96C8) reads X offsets past their 49-entry table into its own code, with the caller's A as the index's high byte; the column heights ($7E:069C + offset) and tile offsets built from them reach $8xxx, $Cxxx and $10xxx; the bus read of a carried index reaches low RAM. Load melting tiles ($A4:943D) clears the heights from X = 80h down (the bank log notes it should be 7Eh), so its first store is the word at $7E:071C, past the heights, whose high byte is Samus's top half request ($071D). The melt cursor and tilemap store past the BG2 staging words ($7E:2000..$2FFF). Load melting tiles copies each ROM tile run to the destination its table entry names; one outside the melting tiles ($7E:4000..$6FFF) would store wherever it points. The BG2 Y scroll lines ($7E:CAF0 + 2 * line) start at line Crocomire's Y - 72 and the melt steps on from it, so a Y below 72 (which wraps) or a line past 255 stores past the 256 lines | a typed model of every RAM block those offsets can reach (in effect a whole-WRAM owner), or proof from the melt's reachable states that the offsets stay in the 49 heights and the tiles buffer |
| Enemy index added to another table | `covern` (Samus X/Y at $0AF6/$0AFA + enemy index), `evir` (parameter 2 high byte at 8 * speed entry into the enemy table), `squeept` flame in slot 0 ($0F38 + field), `phantoon` part fallbacks (no slot at the part offset) and shot kinds past the 10 projectiles ($0C18) | the indexed load uses an enemy index or derived offset on a table it does not index, reading neighbouring RAM (Samus's state after $0AF6, any enemy table byte, the 40h bytes before the table) | the neighbouring owners' bytes by address, or per-site proof that the index stays within the table (covern's reads Samus's position only for the enemy in slot 0) |
| Rinka's spawn availability | `rinka::occupied`, `set_availability` | $7E:87FE + availability index; the spawn table's indexes 2..16h are slot 0's extended words $7E:8800..$8814 (typed), any other value (a stale index) reaches other RAM | proof that a Mother Brain's room rinka's fourth variable is only ever 0 or a spawn table index |
| Draygon's swoop table | `draygon_sweep` (`write_y_position`, the swoop step's read) | the swoop Y positions at $7E:9002 + index, up to 800h bytes, run past the shared boss block's end ($7E:97DC) into the enemy projectile angles and beyond, and Draygon's function timer indexes it wherever it is | a typed swoop table that also owns $7E:97DC..$9802 and the bytes past it, or proof of the timer's range |
| Level data past the cells | `statues::spike_clearing_footstep`, `room_state::random_explosion`, `crawlers::slope_x_velocity`, `grapple_swing::react_to_block`, `room_state::lease_level_word` (Ridley's tail and projectile floor test) | $7F:0002 + 2 * block index, with indexes past the 3200h cells (block + room width, a random tile, the collision cursor, a row times the room width) reading the behaviour bytes and RAM after them | typed cells extended by the RAM that follows ($7F:6402 behaviour bytes and beyond) |
| The current enemy off the slots | `yapping_maw::appearance` (the initial palette and sprite object words), `ridley::contact_damage` (the definition word, for Mother Brain) | `LDA $0F78+offset,x` with X the current enemy index; an index that is not a slot reads whatever RAM lies there (the verifier's sprite alias cases 117/118 run the maw's at E0DAh, reaching $7F:68E4) | proof that the current enemy index is always a slot, with the alias cases seeding a slot instead |
| Visual handles past their slots | `visual_host::session_aliases` (palette effect and animated tile members, the palette flag word, palette colour stores) | a palette-effect or animated-tile handle, or a colour index, past its slot table reaches the next table's RAM ($7E:1E7D.., $7E:1EF5.., $7E:C000..) | the visual owners' neighbouring tables addressed together, or proof that handles stay in range |
| X-ray beam window rows | `beam_windows` (on-screen and off-screen positive rows, the anchor and reopened rows, the generic row writer) | the ROM wraps the 16-bit row offset before adding the table base, so row zero's anchor and reopened rows reach base + FFFEh (the next WRAM bank), and cleanup runs on signed offsets past either end of the 230-row table | a typed window table plus the RAM either side of it, or proof that the beam's start row never wraps |
| Dachora's palette destination | `dachora::load_palette` | a palette index past the 8-entry destination table ($A7:F55F) reads a destination from the code after it; the 32 byte stores go there, below palette RAM or between its words | a typed palette with byte-granular stores into whatever the ROM-derived destination names, or proof that dachora's palette index stays in bits 9-11 |
| Suit pickup's HDMA selector | `suit::advance_pickup` (the program step and timer stores) | the pickup's end adds 2 to the instruction list pointer ($7E:18CC + X) and stores the timer ($7E:18E4 + X) with X the selected object word ($7E:18B2); a selector whose word lies past the HDMA object table ($7E:195C) reaches the RAM after it | proof that the selector is only the object loop's even index below 12 while the pickup's pre-instruction runs (`hdma_program::run` is its only writer there), then the typed object words alone |
| Window table 1 rows past its end | `xray::set_window_word` (the suit pickup's edge fills and every other window word writer) | the suit pickup's edges store `count` rows down from offset 1FEh and up from 0 with a wrapping 16-bit offset, so a count past 100h reaches $7E:9800 + FFFEh and the RAM after the 200h-word table ($7E:9C00..) | proof that the pickup's edge count stays at most 100h, then the typed `HdmaTable1` store alone |
| Mother Brain's rainbow window rows | `mother_brain_rainbow::write_row` | the beam's rasterizers ($AD:DE7F-$AD:E433) store one word per row from row 20h to E8h two bytes apart from $7E:9D02, so the rows from about 9Eh on run beyond HDMA table 2's end ($7E:9DFF) into the window tables at $7E:9E00; an origin row above 20h runs them below $7E:9D00 | one typed window owner spanning $7E:9D00..$7E:9FFF (table 2 and the window tables after it), or proof of the rows the beam reaches |
| Enemy projectile frees past the table | `enemy_projectiles::table::free` (Yapping Maw's body segments) | the maw clears the projectile ID word ($7E:1997 + index) of each body segment it spawned; an odd index or one past the 18 projectiles clears whatever word lies there | proof that the maw's segment words only ever hold a spawned projectile's index, then the typed slots alone |
| Corpse rot table cursors | `corpses::rot_word` | the corpse rotting (`ProcessCorpseRotting`, bank $A9) reads the rot entries at the slot's rot table pointer + 4 * row, a RAM address from the corpse definition; every definition's table lies in the actors' fourth extended block ($7E:9000..$93FF, $9700..$97BF), and a pointer outside it (a stale slot word) would read whatever RAM lies there | proof that the slot's rot table word is only ever written from a corpse definition, then the typed block alone |
| Cinematic tilemap sources past the level buffers | `cinematic::decompressed_byte` | the credits copy each row from $7F:0000 + a row offset in their ROM instruction list; the vanilla list's offsets stay below 2000h, but any offset reaches $7F:0000 + FFFFh, past the custom background blocks ($7F:FA01) into RAM no Session field owns | proof from the instruction list that row offsets stay in the level data, or an owner for $7F:FA02..$7F:FFFF |
| Boss interaction collections | `boss_interaction_runtime` (`boss_collection`, `set_boss_collection` fallbacks) | a widened index past its projectile or enemy collection reaches the neighbouring fields' RAM; the same fallback serves a dirty lease | per-collection proof that indexes stay in range, and the dirty-lease path on its typed owners |
| FX parallax selectors | `session_runtime::Runtime::parallax_byte`, `store_parallax_byte` | the FX entry's parallax selector is a RAM address from ROM; one outside the FX source buffers names whatever RAM lies there | proof that every room's selectors name an FX source, or owners for the RAM they name |
| Menu ROM tables of RAM addresses | `menus::named_player_word`, `set_named_player_word` (the equipment toggle, the special settings, the controller binding choices); `map_menu::animation_ram_word`, `set_animation_ram_word` and the variant read in `animate_pause_sprite` | the equipment screen's equipped-word table ($82:C03C), the special settings' addresses ($82:F0xx), the binding addresses ($82:F54A) and the pause sprite animations' timer, frame and variant tables ($82:C0B2, $82:C0C4, $82:C0D6) are RAM addresses in ROM; the original reads and stores whatever word each names. The first three name player block words ($7E:09A2..$0A01), which go through their `Field::Player` owners, and the animation tables name menu words (`MenuState`); any other address goes through the projection | proof from the ROM tables that every address is a player block or menu word, then the typed word alone |
| File select's helmet timers past the slots | `file_menu` (the slot selection's helmet animation start) | the file select starts the selected slot's helmet animation when `selected - 3` is negative as a 16-bit value, which a slot word of 8003h or more also passes; the store reaches the word at $7E:1991 + 2 * slot, far past the three timers | proof that the selected slot word stays below 3 (or 4 for clear) while the selection runs, then the typed helmet timer alone |
| Enemy projectile frees past the slots | `enemy_projectiles::table::free` (Yapping Maw's death frees its segments' projectiles) | the free stores zero to the definition word at $7E:1997 + the byte slot its caller holds; an odd slot, or one past the 18 slots, reaches the word that byte offset names, inside or past the table | proof that the maw's segment projectile words only hold the even slots its spawns returned, then the typed definition store alone |
| HDMA object stores past the six objects | `hdma_objects::free_hdma_object`, `explosion_window` (`finish_power_bomb`, `finish_crystal_flash`, the right half's table pointer in `calculate_explosion_hdma_table_pointers`) | a boss frees the object slot its spawn returned, and a failed spawn's FFFFh slot names a word far past the channel table ($7E:18B4 + slot); the Power Bomb and Crystal Flash windows store the right half at the next object, which for the last object is the first word of the next field's table ($7E:18C0, $7E:18E4) | proof that the bosses free only spawned slots and that the windows never take the last object, or an HDMA owner for the RAM the stores reach |

To retire a row: prove the index in range (then use the typed owner and delete
the fallback) or give the Session an owner for the RAM the overflow reaches;
remove the calls, their tags and the row together.

## The rebuild's engine: original overflows it doesn't reproduce yet

The new engine (`crates/game`) has no RAM image, so where the original reads past a
table it can't read "whatever lies there". Each such case is listed here until a
check needs it, when it becomes a named behaviour with the original's bytes as data.

- **HUD digits past their cells** (`game::hud`, bank `$80` log `$80:9DBF`/`$80:9DD3`):
  a count above 99 in a two-digit cell, or above 999 in a three-digit one, makes the
  original index past the digit table into the code at `$80:9DE7` and draw those
  bytes as tiles. The engine leaves the cell unchanged. No vanilla count gets there
  (supers and power bombs cap at 99 by their ceilings, missiles at 999 in vanilla
  item placement); a Map Rando or custom-content count could.
- **Scroll areas past the room's table** (`game::scrolling`, bank `$80` log `$80:A528`,
  `$80:A731`, `$80:A8FC`): with the screen below the camera past the room's last row,
  the scroll zone routines read the scroll buffer past the room's screens. Scrolling
  down does so whenever the camera moves into the bottom row ahead of its ideal
  position (area-brinstar frame 414). The engine keeps that part of the buffer
  (`RoomScoped::stale_scrolls`, decoded from the oracle) and reads it as the original
  does; a byte no area has (`StaleScroll::Unknown`, e.g. `$80`) is neither red nor blue,
  so the camera goes through it. Past the 50-byte buffer the original reads the RAM after
  it, which the engine reads as red. A room load writes only its own screens, so the
  rest is what earlier rooms left; the engine's room load leaves it empty (red) until the
  room transition joins.
- **Samus spritemaps with a zero pointer or past the table** (`data::pose::spritemap`,
  bank `$81` log `$81:89AE`): 183 of the 2,096 pointers at `$92:808D` are zero, and the
  original reads such a spritemap's count from `$92:0000`, low work RAM; an index past the
  table reads a pointer from the spritemaps after it. The engine draws nothing for either.
  The half tables plus a pose's animation frames select pointers the game draws; a pose
  and frame no animation reaches (a custom pose, a hand-set frame) could select another.
- **The mini-map's edges** (`game::mini_map`, bank `$90` log `$90:A9A2`, `$90:AB75`,
  `$90:AAA8`): the 3 by 5 window around Samus reads the explored bitmap (`$7E:07F7`), the
  map station bitmap (`$82:9727` on) and the map tilemap (`$B5:8000` on) a row above, a
  row below and two columns either side of her cell. On the map's top row or its bottom
  rows, or in the spill onto the other page, those indices leave the area's 100h bytes or
  800h words: the original reads the RAM or ROM after them (the next area's maps, the
  variables around `$07F7`). The engine reads no cells and the blank tile there. Which
  vanilla rooms reach the edges is not yet checked; slice 0's rooms are inside them.

- **Level columns and rows past the level buffer** (`game::bg_graphics`, bank `$80` log
  `$80:A9DE`, `$80:AB78`): a column or row past the room's last block reads the level
  buffer on. The engine reads it as the original holds it up to the buffer's 3200h blocks
  (the BTS and layer 2 that decompression left after layer 1, then the `$8000` fill,
  `RoomScoped::level_tail`). Past those, or past layer 2's copy, the original reads what
  earlier rooms left in `$7F`, and the engine reads block word 0 there. Slice 0's rows
  reach one row past the room's bottom, inside the buffer.
- **Tile numbers past the tile tables** (`game::bg_graphics`, `data::tileset::TileTables`,
  bank `$82` log `$82:E7D3`): a block whose tile number is past what the CRE's and the
  tileset's tables fill reads `$7E:A000` past them, what an earlier room's tables or the
  boot left. The engine draws zero words for it. A vanilla room's blocks stay within its
  tables; a custom tileset with fewer entries than its level uses would not.

## Tolerated values the content keeps as named variants

- `data::pose::PoseFacing::Unknown(u8)`: the unused poses $21 and $22 (bank $91 log,
  `$91:B731`, `$91:B739`) hold X direction bytes 1 and 2, neither left (4), right (8)
  nor forward (0). The original's facing tests read the left and right bits, so it
  tolerates them; the byte is kept so it goes back out unchanged. No row reaches either pose.

## The oracle harness (`crates/oracle`)

`LEGACY(verifier)` in `crates/oracle/src/streams.rs` copies the frozen
`LOCK_WRAM_MASK` and snesrev's bug-fix counter range `$C2-$C3`. The harness's
`Definition::Legacy` streams can then be compared with today's lock, whose
`wram` digests zero those ranges. The contract's `V1` streams mask nothing.
Remove both with the lock's `faithful` entries, once the harness has
re-recorded the corpus.

`LEGACY(seed)` in `crates/oracle/src/seed.rs` covers the lsnes room captures,
which the corpus still starts from. They hold no SPC700, DSP or timer state,
so a migration seed resumes:

- the SPC700 at the sound driver's idle loop (`$15BF`, SP `$CF`);
- the CPU-to-APU ports from the driver's copy at SPC `$00-$03`;
- the timers from their RAM shadows;
- the DSP from reset, with echo writes off.

In some rows the resumed driver then answers no sound request, so music
uploads, door transitions and message boxes that wait on it stall. Harness-native
seeds (the emulator's own state blob, cut by the harness) retire this entry.

## `LEGACY(old-verify)`: rows the old Verify path no longer covers

`file-copy-clear` (scenarios/corpus.toml) carries `lock = "lsnes"` and has no
entry in the old `scenarios/lock.toml`. Its inputs changed in the rebuild's
re-lock and its authority is the free-run gate's lock,
`scenarios/locks/lsnes.toml`. The old `sm_pipeline` planner leaves such rows out
of its lock and lockstep jobs, and the old lock check does not expect an entry.
Remove the `lock` key and this entry when the old Verify path is retired.
