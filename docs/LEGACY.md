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
| `$82:831E` BG3/sprite tile DMA size | `samus-scripted-intro` @8256, `demo-attract-setup` @2619, route `chain-0011` @2054, any% chain room 11 (`chain-any-0011-DF45`) @2055 | WRAM $0000-$11FF in sprite VRAM $7700-$7FFF, overwritten before display |
| `$84:8243`, `$84:EFD3` unpause item-GFX load | `pause-menu-tour` @1378, `boss-kraid-pause-400` @588, `-950` @1138, `-1030` @1218, `-1960` @2148, route `chain-0211` @368 | garbage transfers into empty item-GFX slots |
| `$80:9AF3` broken mini-map init | `samus-scripted-intro` @8316, any% chain room 11 (`chain-any-0011-DF45`) @2115 | one frame of HUD mini-map tiles from scratch, while blanked |

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
| $060B-$0610 | mine | drop count, enemy hitbox entry counters, projectile counter copy | 67 / 15449 | storing the counters the loops leave | actors, enemy drops |
| $0611-$0616 | mine | coroutine states (snesrev's bookkeeping in RAM the original never writes) | 66 / 280 | moving the coroutine state out of game RAM; permanent while it lives there | runtime |
| $077E-$0782 | mine | `MY_COUNTER` (snesrev's counters in RAM the original never writes) | 66 / 384 | as above | runtime |
| $078F-$0790 | mine | door BTS | 24 / 41 | storing the door BTS where the original does | room entry (doors) |
| $07B7-$07B8 | mine | room state condition pointer | 5 / 5 | as above, for the state selection | room entry |
| $0933-$093C | mine | scrolling and tilemap update temporaries | 67 / 16689 | storing the temporaries the scroll update leaves | scrolling |
| $0D82-$0D8F | mine | saved button configuration (and grapple temporaries at $0D82) | 5 / 382 | storing them where the demo and grapple code does | demo, grapple |
| $0D9C-$0D9D | mine | total grapple swing angular velocity | 4 / 174 | as above | grapple |
| $0DD2-$0DD7 | mine | collision temporaries (slope index, distance) | 67 / 16758 | storing the temporaries the block collision leaves | player collision |
| $0E02-$0E03 | mine | Samus bottom boundary position | 66 / 16128 | as above | player collision |
| $0E20-$0E45 | mine | enemy temporaries | 58 / 12992 | widening the actors' compatibility scratch ($0E20-$0E2A) to the whole range | actors |
| $0E4A-$0E4B | mine | new enemy index | 1 / 1 | storing it where enemy spawning does | actors |
| $0E54-$0E55 | mine | current enemy index | 4 / 211 | as above, for the enemy loop | actors |
| $0E56-$0E59 | mine | enemy index backup, enemy data pointer | 51 / 7763 | as above | actors |
| $1784-$178B | mine | enemy AI pointer and backup | 60 / 7731 | storing the species' AI pointer the dispatch would jump through | actors |
| $17A8-$17AB | mine | active and interactive enemy list indices | 60 / 15948 | storing the indices the list loops leave | actors |
| $1834-$183B | mine | Samus-to-solid-enemy collision distances | 8 / 46 | storing them in the solid enemy collision | player collision |
| $184A-$185B | mine | colliding-solid-enemy positions and deltas | 13 / 251 | as above | player collision |
| $186E-$1885 | mine | enemy spritemap and border collision temporaries | 36 / 4260 | storing them in the enemy collision | actors |
| $189A-$18A5 | mine | Samus target positions and radius mirrors | 58 / 13225 | storing them in the Samus/enemy collision | player collision |
| $18A6-$18A7 | mine | collision detection (projectile) index | 65 / 11946 | storing the index the projectile loops leave | projectiles |
| $EF74-$EF77 | mine | next enemy GFX tiles index and VRAM pointer | 42 / 43 | loading the enemy graphics on the frame the original does (one frame per room load) | actors (room load) |
| $F37A-$F37F | mine | debug copies of an enemy's instruction list, index and ID (never read) | 55 / 9907 | storing them where the enemy processing does | actors |

To retire a row: model the range so it stops differing on the chain and
corpus, then delete it from `normalize_snapshot_ram` and from this table in
the same change.

Free run (docs/SCENARIO_HARNESS.md) masks the same ranges in its WRAM digest, and
one more, `ORACLE_RAM` in `sm/src/verification/observation/machine.rs`:

| Range | What it holds | Why | What would retire it | Owner |
| --- | --- | --- | --- | --- |
| $00C2-$00C3 | snesrev's bug-fix counter, the oracle's patch selector; the original never uses it | only an oracle frame writes it; lockstep's sides share RAM, so only free run sees it (from the first Tick of a seeded row) | the oracle keeping its patch selector outside game RAM | verifier |

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
lease, the RAM image on `Game`); it replaced the retired `SlotAccess`.

| Family | Sites | What the original does | What a hardware-faithful typed replacement needs |
| --- | --- | --- | --- |
| Crocomire's melt | `crocomire_reactions::erase_melt_columns`, `data_bank_byte`; `boss_lease::bg2_staging::set_unbounded` | Erase melting pixel column ($A4:96C8) reads X offsets past their 49-entry table into its own code, with the caller's A as the index's high byte; the column heights ($7E:069C + offset) and tile offsets built from them reach $8xxx, $Cxxx and $10xxx; the bus read of a carried index reaches low RAM. The melt cursor and tilemap store past the BG2 staging words ($7E:2000..$2FFF) | a typed model of every RAM block those offsets can reach (in effect a whole-WRAM owner), or proof from the melt's reachable states that the offsets stay in the 49 heights and the tiles buffer |
| Enemy index added to another table | `covern` (Samus X/Y at $0AF6/$0AFA + enemy index), `evir` (parameter 2 high byte at 8 * speed entry into the enemy table), `squeept` flame in slot 0 ($0F38 + field), `phantoon` part fallbacks (no slot at the part offset) and shot kinds past the 10 projectiles ($0C18) | the indexed load uses an enemy index or derived offset on a table it does not index, reading neighbouring RAM (Samus's state after $0AF6, any enemy table byte, the 40h bytes before the table) | the neighbouring owners' bytes by address, or per-site proof that the index stays within the table (covern's reads Samus's position only for the enemy in slot 0) |
| Rinka's spawn availability | `rinka::occupied`, `set_availability` | $7E:87FE + availability index; the spawn table's indexes 2..16h are slot 0's extended words $7E:8800..$8814 (typed), any other value (a stale index) reaches other RAM | proof that a Mother Brain's room rinka's fourth variable is only ever 0 or a spawn table index |
| Draygon's swoop table | `draygon_sweep` (`write_y_position`, the swoop step's read) | the swoop Y positions at $7E:9002 + index, up to 800h bytes, run past the shared boss block's end ($7E:97DC) into the enemy projectile angles and beyond, and Draygon's function timer indexes it wherever it is | a typed swoop table that also owns $7E:97DC..$9802 and the bytes past it, or proof of the timer's range |
| Level data past the cells | `statues::spike_clearing_footstep`, `room_state::random_explosion`, `crawlers::slope_x_velocity`, `grapple_swing::react_to_block` | $7F:0002 + 2 * block index, with indexes past the 3200h cells (block + room width, a random tile, the collision cursor) reading the behaviour bytes and RAM after them | typed cells extended by the RAM that follows ($7F:6402 behaviour bytes and beyond) |
| The current enemy off the slots | `yapping_maw::appearance` (the initial palette and sprite object words), `ridley::contact_damage` (the definition word, for Mother Brain) | `LDA $0F78+offset,x` with X the current enemy index; an index that is not a slot reads whatever RAM lies there (the verifier's sprite alias cases 117/118 run the maw's at E0DAh, reaching $7F:68E4) | proof that the current enemy index is always a slot, with the alias cases seeding a slot instead |
| Visual handles past their slots | `visual_host::session_aliases` (palette effect and animated tile members, the palette flag word, palette colour stores) | a palette-effect or animated-tile handle, or a colour index, past its slot table reaches the next table's RAM ($7E:1E7D.., $7E:1EF5.., $7E:C000..) | the visual owners' neighbouring tables addressed together, or proof that handles stay in range |
| X-ray beam window rows | `beam_windows` (on-screen and off-screen positive rows, the anchor and reopened rows, the generic row writer) | the ROM wraps the 16-bit row offset before adding the table base, so row zero's anchor and reopened rows reach base + FFFEh (the next WRAM bank), and cleanup runs on signed offsets past either end of the 230-row table | a typed window table plus the RAM either side of it, or proof that the beam's start row never wraps |
| Dachora's palette destination | `dachora::load_palette` | a palette index past the 8-entry destination table ($A7:F55F) reads a destination from the code after it; the 32 byte stores go there, below palette RAM or between its words | a typed palette with byte-granular stores into whatever the ROM-derived destination names, or proof that dachora's palette index stays in bits 9-11 |
| Suit pickup's HDMA selector | `suit::advance_pickup` (the program step and timer stores) | the pickup's end adds 2 to the instruction list pointer ($7E:18CC + X) and stores the timer ($7E:18E4 + X) with X the selected object word ($7E:18B2); a selector whose word lies past the HDMA object table ($7E:195C) reaches the RAM after it | proof that the selector is only the object loop's even index below 12 while the pickup's pre-instruction runs (`hdma_program::run` is its only writer there), then the typed object words alone |
| Window table 1 rows past its end | `xray::set_window_word` (the suit pickup's edge fills and every other window word writer) | the suit pickup's edges store `count` rows down from offset 1FEh and up from 0 with a wrapping 16-bit offset, so a count past 100h reaches $7E:9800 + FFFEh and the RAM after the 200h-word table ($7E:9C00..) | proof that the pickup's edge count stays at most 100h, then the typed `HdmaTable1` store alone |
| Mother Brain's rainbow window rows | `mother_brain_rainbow::write_row` | the beam's rasterizers ($AD:DE7F-$AD:E433) store one word per row from row 20h to E8h two bytes apart from $7E:9D02, so the rows from about 9Eh on run beyond HDMA table 2's end ($7E:9DFF) into the window tables at $7E:9E00; an origin row above 20h runs them below $7E:9D00 | one typed window owner spanning $7E:9D00..$7E:9FFF (table 2 and the window tables after it), or proof of the rows the beam reaches |
| Corpse rot table cursors | `corpses::rot_word` | the corpse rotting (`ProcessCorpseRotting`, bank $A9) reads the rot entries at the slot's rot table pointer + 4 * row, a RAM address from the corpse definition; every definition's table lies in the actors' fourth extended block ($7E:9000..$93FF, $9700..$97BF), and a pointer outside it (a stale slot word) would read whatever RAM lies there | proof that the slot's rot table word is only ever written from a corpse definition, then the typed block alone |
| Cinematic tilemap sources past the level buffers | `cinematic::decompressed_byte` | the credits copy each row from $7F:0000 + a row offset in their ROM instruction list; the vanilla list's offsets stay below 2000h, but any offset reaches $7F:0000 + FFFFh, past the custom background blocks ($7F:FA01) into RAM no Session field owns | proof from the instruction list that row offsets stay in the level data, or an owner for $7F:FA02..$7F:FFFF |
| Boss interaction collections | `boss_interaction_runtime` (`boss_collection`, `set_boss_collection` fallbacks) | a widened index past its projectile or enemy collection reaches the neighbouring fields' RAM; the same fallback serves a dirty lease | per-collection proof that indexes stay in range, and the dirty-lease path on its typed owners |
| FX parallax selectors | `session_runtime::Runtime::parallax_byte`, `store_parallax_byte` | the FX entry's parallax selector is a RAM address from ROM; one outside the FX source buffers names whatever RAM lies there | proof that every room's selectors name an FX source, or owners for the RAM they name |

To retire a row: prove the index in range (then use the typed owner and delete
the fallback) or give the Session an owner for the RAM the overflow reaches;
remove the calls, their tags and the row together.
