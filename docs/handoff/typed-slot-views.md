# Typed slot views handoff (2026-11-03)

Branch `collab/typed-all` (PR #155) converts the address-API layer to typed Session
fields. Engine address APIs (`ram_ratchet.py`, ROM record reads excluded) fall from
5805 on main `0115740f5` to 36, and `collab/f36-owners` takes them to 27.

## Pattern

`crates/sm_game/src/actors/slot_view.rs`:
- `slot_view!` declares a family's record fields (`Actor` lanes) and its extended words (`var_NN`).
- `Family::of(host, k)` reads and `FamilyEdit::of(host, k)` stores.
- A dropped edit that borrowed the record runs the Runtime's input refresh once, as the old per-byte projected stores did.
- Slot 0's Botwoon death words are served from `Session::botwoon_death`.
- Non-slot words use their Session fields. Where the old store refreshed inputs, the new one calls `SessionAccess::words_stored()` after it.
- Families' `Lease` traits require `SlotState`, which only `Runtime` implements.
- The `boss_ram` gateway is gone; callers use `boss_interaction`'s word map.

Commits that change write order:
- Lease merges: the Mother Brain tube landing and unpause hook; Draygon, with all its routines now on the lease.
- Record byte order: the Ceres elevator overlay.

Everything else is `[exact]`.

## Replays decide

Replays always run with Legacy presentation. `Presentation::Owned` comes only from a
supplied or restored session start, which only verification cases reach. So OAM, the
display mirrors and the NMI counter are plain RAM in replays.

A new Session owner must move every writer of its bytes onto it. Four owners have
landed this way:
- room scroll cells ($CD20..$CD51)
- pause hooks ($0601..$0606)
- right border ($1874)
- Work Robot palette ($D652..$D657)

The resulting checkpoint formats are SMROOMS 9 and SMACTORS 9 (the ending's star records add four spare-word owners). Owners whose writers stay on Game, which stores RAM and is re-adopted at the next lease, are scalar words like their neighbours: the demo input frame count $0A8A and spare $0A5E (SMPLAYER 5), and the ending's wavy Samus words $0D9C..$0DA1 (SMAUXGS 14). Replays can't check
$1874, because the verifier masks $186E..$1886.

The first scroll attempt diverged `boss-crocomire` at $CD24 (run 41561271426). Six
Crocomire `word_transfers!` stores still used the cell indexes as RAM addresses. Once
those stores were fixed, the four failing rows passed (run 41574730839).

Raw RAM accesses to the scroll cells still run on Game (the demo's first
scene hook now stores `Room::set_scroll`):
- `hdma_program.rs:297-298`
- the boot WRAM clear

## Remaining sites, by blocker

1. **New Session owners, plus all of their writers.**
   - (Drop origin $F434/$F436 is now owned by `enemy_projectiles.drop_origin` (SMESHOT 3). The $3000..$6FFF tile buffer is owned by the presentation workspace; Kraid's `load_bg2` reads it typed.)
2. **Legacy presentation.**
   - (OAM $0370..$0591 is now owned by `LegacyPresentation::objects`, decoded and encoded at every lease boundary; cannon and atmosphere tiles store through it.)
   - $57/$69/$5B6. (The Samus half requests $071D..$0722 are now owned by `LegacyPresentation::samus_requests`, decoded and encoded at every lease boundary.)
   - Phantoon's `set_mosaic`/`enable_bg2`/`frame`.
   - Owning the presentation on the normal replay path is an architecture step.
3. **Still on `Game`.**
   - The game-over menu: 11 sites. A cohort patch exists at `$TEMP/typed-hosts/game_over_cohort.patch` but raises `sm_game_ram_sites` by 3.
   - `beam_windows`/`calculate_xray_window`, which runs inside `foreign`.
   - (The HUD's digit pointer $00..$02 stores `Session::pointer_scratch`; the options screens at $7F:C000.. are read as `QueuedTilemaps` background blocks.)
   - The game-over `prepare` display mirrors: $53/$54/$86 have no display `Field`. Adding them changes `preparation::State`, `DisplayControls`, the Owned presentation's projection and checkpoint, and Game's frame-display ranges.
   - The demo's setup words $09E4/$09E6/$09E8/$0DF8/$0DFC: $09E4 and $0DF8 are input observations that `inputs_from_session` reads from RAM, and save loads and new games write them on Game.
   - The X-ray/suit window table at $9800 (`xray::set_window_word`): `fx_domain::Sources` owns it only while the Session has FX sources or an Owned presentation, and its store panics where the RAM store succeeds.
   - Game-side routines that touch enemy 0 (the game-over baby, demo rooms, Kraid's unpause) read and store a typed `Actor` through `ActorImage`, which `Game` implements over RAM in sm/.
4. **Address-indexed by design.**
   - Out-of-range fallbacks: Phantoon's part and shot fallbacks, `visual_host` handles, `bg2_staging::set_unbounded` for Crocomire's melt cursor.
   - Original reads and stores that run past their tables: Crocomire's `erase_melt_columns` (offsets reach $8xxx/$Cxxx/$10xxx) and melt column heights, covern $0AF6+k, rinka $87FE+id, draygon_sweep $9002+i past $97DC, the statues level block.
   - Level-data indexes that run past the cells.
   - A typed index would panic where RAM didn't, so these need a LEGACY entry or a bounded typed fallback.
   - Done (2026-11-03): these go through `legacy_overflow` (`LEGACY(original-overflow)` in docs/LEGACY.md), typed where in range; cycle_report counts them as `legacy_overflow`.
5. **Machinery.** `session_slots.rs` is gone (`session_slot!`, `SlotAccess`, `SessionSlots`); overflow paths resolve through `legacy_overflow::SlotBytes`. The views' byte helpers moved to `sm/src/projection.rs`: sm_game's views now require `projection::Views` (a supertrait of `CompatibilityRam`), which names words by `Field` and groups by `Layout`/`Decode` (cohorts, liquid, palette fade, camera, flow, room). `sm` implements it for `Game` over its projected bytes, and `observed_inputs` is a required method there. On a current lease the Runtime reads and stores the Session fields directly (`Field::session_word`/`session_word_mut`, `Decode::from_session`, `Layout::session_copy`/`store_session`; proved equal to the projected bytes by sm_game's projection tests); a dirty lease, the room header/door exit/FX blending stores, table elements past their tables and cohort stores (which run only on `Game`, through `session_cohort::store`'s lease) keep the byte path. `Field::address`, `Layout::visit` and `Decode::decode` stay in sm_game: `Game`'s generic `Views` impl, the Runtime's byte fallback, the verify build's held-copy checks and those tests use them. The `word!` macros still reach `compatibility_byte` (they count as RAM macros).
6. **Counter noise.** `enemy_projectiles/content` (a `self` ROM record) and `menus/canvas` (a workspace `Point`) are not RAM.

## Caveat

Palette writes index `colors[i]`/`targets[i]` directly: work_robot, covern, face block
and draygon. An out-of-range index would panic where the old code wrote into the next
RAM block. Real palette indexes are always in range.

## 2026-11-03 batch (PR after #164)

New owners: Player `moonwalk`/`debug_flag`/`spare_09e8`/`turn_request`/`spare_0dfc` ($09E4..$09E8,
$0DF8, $0DFC; SMPLAYER 6, input arms for $09E4/$0DF8) and display Fields `ObjectAddressLow/High`,
`MemorySelect` ($53/$54/$86; SMDISPL 5). The X-ray window, Mother Brain's rainbow, the corpse rot
tables and the cinematic $7F tilemap transfers read and store through their owners; out-of-range
remnants go through `legacy_overflow` (60 sites). What remains of the 14: `legacy_overflow.rs`
internals (11), `projection.rs`/`ram.rs` (the `CompatibilityRam` byte helpers Game serves from its
RAM image). No engine call site outside these three files names a RAM address.

## Cohort copies: a failed attempt (2026-11-03)

Replacing the RAM-backed `session_cohort` copies with a `session_part` helper failed. The helper
opened a fresh lease (`read`/`with`/`edit`) from Game-hosted code in the middle of a frame. Commit
`65b617006` (the power bomb explosion on `session_part`) alone breaks rows unrelated to power bombs:
item-speed-booster, enemy-fire-arc and music-fir-selector10. A sound effect queued in RAM goes
missing at `$0643..$0665` (SFX queue 1); CI bisect run 41643239578 isolated it. That lane was left
out of #170. A rework must not open extra leases mid-frame: either take the values from a lease the
caller already holds, or keep the Session authoritative for the whole tick (docs/PERFORMANCE.md
step 2).

## Named words lane (2026-11-03, `collab/rs-words`)

New owners: `Room::mode7_irq` ($0783) and `Room::ceres_hdma_table` ($07EB), scalar words with
room-state layout lanes (SMROOMS 10). Every sm_game writer stores the field; the HDMA
instruction's $07EB store and sm's scroll DMA reads of $0783 reach them through the projection.
Left on Game: Kraid's pause hooks ($077E is `messages.delay`, $0780 has no owner), the frame
start's object cursor and Samus half requests, the demo stop's $077C..$0997 clear, and the lava
surface's random number swap and the liquid background reads (HDMA pre-actions run in a foreign
call with no lease to take the values from).

## Macro callers lane (2026-11-03, `collab/rs-callers`)

New owner: `Room::background_size` ($098E, the size of BG2), a scalar word with a room-state
layout lane (SMROOMS 11). Lease callers read and store their Session fields (zebetites, escape
critters, Shaktool's previous piece, the room effects reset, the camera chase and door scrolls);
Game-hosted callers go through their existing owners (`TilemapQueue`, `Mode7Queue`, FX controls,
the gameplay and inventory owners, display fields, `sprite_drawing::reset_cursor`). Two ROM
address tables now resolve to typed words and fail on an address outside them: the controller
binding table ($82:F54A) and the ending's capacity table. `ram_words!` and two unused arms of
`word!`/`checked_word!` are gone. The remaining arms all have callers in sm/, so ram.rs and
word_transfers.rs can only shrink once sm/ stops using them or takes its own copy. Left: the
save slots' $0A00/$D7C0/$CD52 block copies and the demo's $D8F0 words (no owners), the palette
and tile buffer fills in `initialize_gameplay` (Game-hosted; `PaletteAccess` on Game opens a
lease per word), the VRAM read queue tail, the audio upload flag (only `audio_word` reaches
it), Samus's half requests on the logo screen, and the files the brief excluded.

## Cohorts lane (2026-11-03, `collab/rs-cohorts`)

No lease is opened that the copies did not open. The menu copies in the cinematics (the logo
hold, the title and intro menu resets, the Ceres start's save selection) store through the menu
owner (`menus::state::with`). The power bomb's ten explosion words ($0592, $0CE2..$0CF3) joined
`SessionWords`, with write-class ranges on Game, so the explosion windows, the HDMA handler's
frozen-time release and the intro's explosion origin go through `sequence_state::with_words`.
Both owners commit through a lease only while the retained Session is current, as
`session_cohort::store` did, and commit only the words a body changed. Left (11 copies): the
cinematic frames' `Cinematic` copies, the fade planes ($2000..$2FFF), the palette buffer
($C000..$C1FF) and Samus's shot words in the intro. They run on Game outside any lease, and no
Game owner holds those words. They need the cinematic frame on a lease, or the Session kept
authoritative for the whole tick (docs/PERFORMANCE.md step 2).

## Owners lane (2026-11-03, `collab/rs-owners`)

The ratchet now counts `use ... as` renames of the RAM macros (save_slots' `game_word!`;
219 becomes 223), and reports the RAM-named Session word views (`sequence_word!`,
`gameplay_word!`) as the advisory `session_word_view_sites`. New owners:
`Room::unpause_queue_tail` ($0780, SMROOMS 12) and `GameplayFlags::unused` ($D8F0..$D8F7,
SMAUXGS 15). New `projection::Field` keys: `MessageDelay`, `UnpauseQueueTail`, `ApuUploading`
(the owned audio requests' `uploading`, else the byte path), `RandomNumber`, `Moonwalk`,
`DebugFlag` and `Spare09E8`. Left: the save slots' $0A00/$D7C0 and $CD52 block copies and the
demo's $CD52 clear (no owners; the save file and the map transfer reach them as raw RAM), the
tile buffer fills in `initialize_gameplay` and `queue_fx_clear` (the workspace's role points
don't cover a plain fill), the $07B4..$0997 clear in the gameplay frame, the options fade's
$198D..$19BC clear (no typed keys), Crocomire's melt tile copies and $069C..$071D clear, and
the cohort lane's `cinematic.rs` and `ending_screens.rs` transfers.

## Residuals lane (2026-11-03, `collab/rs-resid2`)

New `projection::Field` keys: `SpawnArgument` ($16), `MovementAndFacing` ($0A1E),
`PauseHook(index)` ($0601..), `Layer2ScrollX/Y` ($091B/$091C, byte path for stores),
`SuitRedGreen`/`CrystalFlashDelay` ($0DF0/$0DF2), `LiquidOptions`, `LiquidSurface`, and the
camera's `Layer1`/`Layer2`/`Bg1Offset`/`Bg2Scroll(axis)`. `ContinuationAccess` gained the
secondary ($0613) and outer ($0611) continuations, the message continuation read and the
frame start's $0612..$0615 clear; Game stores them as RAM, as the callers did.
`display_refresh::Host::store_readback_byte` stores the VRAM readback records, and
`enqueue_readback` takes its mirror as a display field. No new lease is opened. Left: the
save slots' $0A00/$D7C0/$CD52 copies and the demo's $CD52 clear (no owner for the saved
player block or the saved maps), the demo stop's $077C..$0997 clear, Crocomire's melt copies
and $069C..$071D clear (the clear overlaps the Samus half requests), the tile buffer fills,
`copy_level_planes` (a raw copy inside the level buffer), the liquid RAM bridge, and the menu
prelude's `RawRam` re-export, which a dozen menu files rely on.

## Saves lane (2026-11-03, `collab/rs-saves`)

New owner: `Session::saved_maps` (`save_file::SavedMaps`, $CD52..$D551, eight 256-byte
area maps, SMAUXGS 16). New `projection::Field` keys, all stored through the byte path:
`SavedMap(index)`, `ExploredMap(index)` ($07F7) and `MapStations(index)` ($D908 + index).
The save, load, new-game and demo map copies and clears go through them. The map packing
in `transfer_map` ($D91C..) and the boot clear stay raw RAM on Game, re-adopted at the
next lease. `presentation_workspace::Access::screen_fill_native` fills the tile buffer
(Game stores RAM, no lease). `LevelData::copy_level_planes` moves the level split onto
the level owner (Game's RAM). No new lease is opened.

Left:
- The saved player block. The two $09A2..$0A01 <-> $D7C0..$D81F copies need a
  typed record built from about ten owners: inventory, bindings, selected weapon, game
  time, options, player flags and HUD auto-cancel. $09EA and $09EE..$0A01 have no owner
  yet, and $D7C0..$D81F has none either.
- The demo stop's $077C..$0997 clear. It spans about thirty owners and needs a per-owner
  audit.
- Crocomire's $069C..$071C melting heights. They have no owner, and the data-bank reads
  and the `legacy_overflow` path reach them by address. $071D is the first Samus half
  request.
- The melt copies, whose destinations come from a ROM table.
- The menu prelude's `RawRam` re-export. Nine menu files still call `.ram()`.

## Saves lane 2 (2026-11-04, `collab/rs-saves2`)

New owners: `Session::saved_player` (`save_file::SavedPlayer`, $D7C0..$D81F, the save's
48-word copy of the player block, SMAUXGS 17) and, on `Player`, the HUD auto-cancel flag
$09EA (`hud_auto_cancel`, an input observation `inputs_from_session` and the scroll
refresh now take from the Session) and the ten spare words $09EE..$0A01 (`spare_09ee`,
SMPLAYER 8). New `projection::Field` keys `Player(PlayerWord)` and `SavedPlayer(PlayerWord)`
name the block's words by `save_file::PlayerWord`; `Player` reads each from its owner
(inventory, game loop, control bindings, selected weapon, options, player), and the
control's bindings and the frame display's $09EC store through the byte path. The save
and load copies go through them on Game in the original word order. The SRAM transfer of
$D7C0..$DE1C and the boot clear stay raw RAM on Game, re-adopted at the next lease.

Crocomire's heights clear stores the overflow word $071C/$071D through
`legacy_overflow::set_slot_word` (LEGACY row updated) and clears
`crocomire_melt.column_heights` typed. The heights already had an owner.

Left:
- The melt copies ($A4:943D). Their ROM destinations are $7E:4000.. in the melting tiles
  buffer, which the presentation workspace's shared bytes own; the workspace has no role
  for it, so a typed store needs a melt-buffer role (or an owner) first. `Point::physical`
  would be an address API.
- The demo stop's $077C..$0997 clear (`gameplay_frame::stop_demo`, on Game, no lease open).
  It needs a `Field` (or typed reset) for each of about thirty owners, stored in the
  original descending word order; on Game every typed store takes the byte path, so the
  bytes are unchanged once each word has a key.
- The menu prelude's `RawRam` re-export: about 45 `.ram()` calls across nine menu files
  (menu_window 15, map_menu 8, save_slots 7, options_settings 5, ...). Dropping the
  re-export only pays once all of them are typed; per-file imports would count instead.

## Menus lane (2026-11-04, `collab/rs-menus`)

The ratchet now counts each raw work RAM call through `.ram()` (cycle_report's
`raw RAM calls` kind, the engine inventory's `raw_ram_call` sites): 56 calls the
prelude's single `RawRam` import hid. 85/14 became 141/24, then fell to 95/14.

New typed paths, each the same store or read on Game as the raw call it replaced:
- `fx_domain::Sources::buffer_byte` and `store_buffer_bytes`, beside
  `store_buffer_byte`: the map window's tables ($9E00, $9E10) and open edges
  ($9E32, $9E36) by `LiveOwner` and offset.
- `save_file::PlayerWord::named` decodes a menu ROM table's RAM address; the
  equipment toggle, special settings and binding choices use `Field::Player`.
- `QueueAccess::queue_byte` (the sky's bank bytes), `MapFacts::used_station_byte` and
  `store_used_station_byte` (the elevator bits), `Field::FileMapStartDelay`
  ($0F7A, byte path), and the physical display bytes for the MOSAIC and
  brightness words.
- `menu_tilemap::upload` takes its screen register as a display `Field`.
- Out-of-range table addresses and the helmet timer past the slots go through
  `legacy_overflow` (two LEGACY rows).

Reads that took the raw image now take the compatibility bytes (`view_word`,
`map_station_byte`, `used_station_native_word`), as the map facts already did.

Left (10 raw calls):
- `save_slots`: the save transfer of $D7C0..$DE1C (`SaveFile::ram_word`,
  `set_ram_word`) and the packed map copy between the saved maps and $D91C.. (no
  Session owner holds the compressed map). Both need typed owners for the whole
  save block first.
- `projection::image_word`, `set_image_word` and `copy_into_image`: the enemy
  projectile table's raw copy on Game (`enemy_projectiles::table::with_projectiles`)
  and the palette fade's colour store. The table copy needs a typed Game path before
  the helpers can go.
- The demo stop's $077C..$0997 clear and the Crocomire melt copies, as before.

## Foreign calls lane (2026-11-04, `collab/rs-fc4`, on `tick-s9`)

`foreign_calls` 168 to 138, `game_host_impls` 95 to 94; `sm_game_ram_sites` unchanged
(88). No lease is opened that the code did not open, and every retired call refreshes
the observations on entry and return (`owned_step`, `owned_player_call`, `on_lease`).
Moved to the lease:
- the Torizos' palette loads, hurt flash and Golden Torizo health bands (the pairs moved
  from sm/'s `palette_effects::torizo` to `torizo::palettes`);
- the palette fades, through `fading::Host` (the game keeps its RAM image store): Kraid's
  three, the Torizo's colour change and the room entry's;
- Mother Brain's Samus vitals (`actors::mother_brain_vitals`), contact damage, part
  hitboxes and dead Torizo push-out (`actors::part_hitboxes`), and sprite tile entries;
- the ninja pirate's palette and both pirate flinches from a held lease; Ridley's drops
  and music; Yapping Maw's segment frees (`table::free`, new LEGACY overflow row); the
  corpse rot overflow read; the BG3 velocity pick; the four pure invalid-window reports.

Left, with what each needs:
- Hardware (stage 4, D2): the room entry's register, VRAM and decompression calls, Bomb
  Torizo's haze, the morph ball eye beam, Phantoon's wave, `hdma_program`'s channel
  register, the animated tile upload, the sine/cosine multiplier, Yapping Maw's
  multiplicand, the Mode 7 queue (`Mode7Queue` has no lease impl).
- Dispatcher fallbacks (`species::ai`/`touch`/`shot`/`draw_hook`/`instruction`/
  `pre_instruction`, `run_a7_function`, Torizo's `run_function`, the pirates' state
  dispatch, the HDMA calls and pre-actions): they retire as their last species moves.
- Diagnostics with a verify observer that snapshots RAM (`native_rom_diagnostic`,
  the room background's): they need the observer to read the lease.
- Mother Brain's `on_game` arm: one `foreign` for 17 entries; it goes when the last does.
- Message boxes, save, item restore and pickup visuals: Game-hosted families.
- Yapping Maw's visibility: `Runtime::update_maw_visibility` re-stamps the persistent
  epoch for a fresh lease; on the held lease it needs a per-site audit.
- The grapple tiles: the queue words alias the tile fields; both TileAccess impls exist,
  but the Game side writes the sequence owner, so the alias cases need a replay first.

## Residuals lane 3 (2026-11-04, `collab/rs-resid3`)

The save file is one typed record. `save_file::FileWord` names its 814 words in file
order ($D7C0..$DE1B): the saved player block, the gameplay flags' 122 words, the save
selection's loading state, station and area, the room's loaded items count and the
compressed maps. `SaveHost` reads and stores file words, and the SRAM save, load and
clear walk `FileWord::all()`.

New owners:
- `GameplayFlags::spare` ($D910..$D913).
- `Session::compressed_maps` (`save_file::CompressedMaps`, $D91C..$DE1B).

Both are byte-encoded, with layout tests (format bump: SMAUXGS 18).

New `Field` keys:
- `SaveFile(word)`, which replaces `SavedPlayer`.
- `SavedMapAt(offset)` and `CompressedMapAt(offset)`, the word at a map byte.

The map packing reads each byte as its word's low byte and stores it with
`set_view_low_byte`.

Other changes:
- The enemy projectile table's Game copy uses `Views::load_words`/`store_words`, storing
  every word but the zoom. `table::free`'s store past the slots goes through
  `legacy_overflow` (new LEGACY row). The palette fade stores through
  `Field::PaletteColor`. The `projection::image_*` helpers are gone.
- Crocomire's melting tiles load stores into the new `Role::MeltingTiles` ($7E:4000..)
  through the lease Session's workspace publication, with `legacy_overflow` for a
  destination outside $4000..$6FFF (format bump: SMDISPL 6, the role's binding).

Left: the demo stop's $077C..$0997 clear (`gameplay_frame::stop_demo`, 270 words,
descending, on Game). Every word needs a `Field` key whose `session_word` matches the
projection, so the projection test covers it. Owners include:
- the message delay and unpause queue tail ($077C..$0781);
- the coroutine completion byte $0782;
- the Mode 7 and menu words $0783..;
- the door and room entry words, including the room header and the asset long pointers
  $07BD..$07C8 (world keys, byte path);
- the room music and code words;
- the explored map $07F7..$08F6 (`Field::ExploredMap`);
- the camera blocks and positions $08F7..$0932;
- the door transition words;
- the Ceres status, camera distance and timer $093F..$094B;
- the file select words $0950..;
- the BG1 update words up to $0997.

Several have no `Field` yet. This is about 30 owners and was not started here.

## Foreign calls lane (2026-11-04, `collab/rs-fc5`, on `rs-fc4`)

`foreign_calls` 138 to 121, `game_host_impls` 94 to 91, `legacy_engine_ram_sites` 483 to
460; `sm_game_ram_sites` unchanged (95). No lease is added; each retired call runs in
an owned player call (`owned_step`, `lease_owned_step`, `draw_objects`,
`report_invalid_window`), which refreshes the observations on entry and return.
Moved to the lease:
- Mother Brain's `on_game!` macro is gone: its 17 entries run on the fight's lease
  (`Lease::player_command`/`player_mother_brain` for Samus; the rest through sm_game
  bounds). The wall pushes are ported onto `PlayerPosition`, and the player collision
  ownership check runs the same port on the game. The wall seal, the death explosions
  and the tube landing's earthquake also moved; sm's `mother_brain_visuals` and
  `actors::mother_brain_effects` modules are deleted.
- `ActorLease::lease_owned_step`: Alcoon, Kago, Fune/Namihe and Rinka spawns, the
  Yard's crawler speed, the roach's sine products and the shutter timers.
- Draygon's grab (command, then grab, one step each); Draygon's `Host` is a blanket alias.
- Ridley's Ceres door spritemap; the room mechanisms' invalid-window reports; the pickup
  visuals and the transitions' beam update (`hud::install` needs only `HudFrameHost`).

Left, with what each needs:
- Hardware (stage 4, D2): the neck's and Ridley's sine multiplier, Yapping Maw's
  multiplicand, Bomb Torizo's haze, the morph ball eye beam, Phantoon's wave, the HDMA
  channel registers, lava background video, the Mode 7 queue, animated tile upload.
- Dispatcher fallbacks: the species `ai`/`touch`/`shot`/`draw_hook`/`instruction`/
  `pre_instruction` arms and `run_a7`/`run_aa`/Torizo's `run_movement` fallback still
  reach the generic table on Game; the pirates' `dispatch_state` handlers each open
  `game.lease()` several times, so it needs every handler rewritten onto `PirateLease`.
  The HDMA calls and pre-actions left are the explosion window and X-ray ones, which
  need `sequence_state::Access` and `MenuRuntime` on the lease (the crystal flash spawn too).
- Message boxes and item restore: their verify observers snapshot RAM on the game
  (`observe_message_display`, `restore_cases::observe`). Save is resid3's.
- Ridley's tail floor test: the lease has no `LevelData`.
- Yapping Maw's visibility: the audit is clean for every scheduler slot (the word,
  $7E:802C + slot, is in the extended block, which adopts it, and meets no Botwoon death
  word), so the held lease could store `off_screen` through the view. But the D7 checks
  (`actor_runtime_checks`, `persistent_session_cases`) call `Runtime::update_maw_visibility`
  directly, including the $7F:0002 overflow slot 7FD6h, so the function would live only
  for them. It retires with D7.
- The grapple tiles (unchanged from fc4).

## Infrastructure lane (2026-11-04, `collab/rs-infra`)

The demo stop's clear is done. `demo_clear` lists the owner of every byte of
$077C..$0997 in address order, and `Field::DemoClear(index)` reads word `index` byte by
byte from those owners, so the projection test checks each word whose bytes all have
Session owners. Stores take the byte path, word $0996 first, as the transfer did. No
Session fields or formats changed.

These bytes have no Session owner, and their words read `None`:
- the Mode 7 rotation angle $0785;
- the removed words $078F, $07B7 and $0933..$093E;
- $07D3..$07DE, $07ED..$07F2 and $094C..$094F;
- the BG1/BG2 tilemap update requests $0956..$098D. Their numeric stores still need
  semantic counterparts (`Presentation::requires_semantic_counterpart`).

The HUD palette $077C, the NMI acknowledgment $0782 and the committed fake mode $07EC
read from an owned display only.

Other changes:
- The cinematic palette capture stores through `Field::PaletteColor` and
  `Field::CinematicElement(TilemapStuff, ..)`.
- `RawRam`, `WorkRamAccess` and `RamHost` are gone. Nothing in sm_game called `.ram()`,
  so the bounds were unused.
- sm_game's `ram_word!`/`word!` and `ram_checked_word!`/`checked_word!` are gone. sm/
  has its own macros in `ram_words.rs`.
- `word_transfers!` lost its `rom`, `table` and stride arms, which nothing called.

What is left in ram.rs and projection.rs (4 sites), for D6:

| Item | Sites | Callers |
| --- | ---: | --- |
| `CompatibilityRam::compatibility_byte`/`compatibility_write_byte` (ram.rs) | 2 | sm/ only: 156 references, 109 of them in `game_state.rs`. In sm_game, only test hosts implement them. |
| `SessionProjection`'s `CompatibilityRam` impl (projection.rs, a test host) | 2 | the trait's methods above, and `session_cohort::load` in cohort tests (stage 7) |

`word_transfers!` is gone (lane `wt`, 2026-11-04). Its last eight transfers, all
constant fills in sm/, store through owners: the boot and room-background tile
buffer fills through the presentation workspace (`screen_fill_native` and the new
little-endian `screen_fill`), the escape timer's white target palette through
`Field::PaletteTarget`, the level clears through `LevelData::clear_level_data`, and
the transition's $7E:9C00..$A000 clear through `Sources::store_buffer_word` on its
three HDMA buffers.

The ratchet does not count these, but only sm/ calls them: `RAM_VARS`, `RamVar`, `Shape`
(`ram_map.rs`, `cpu_infra.rs:1258`, `verification/observation/oracle.rs`).

## D6 lane (2026-11-04, `collab/rs-d6`)

Jim's D6 decision (REVIEW, PR #211) puts RAM compatibility shims and the RAM to
Session translation in sm/. The four ram.rs and projection.rs sites above are gone;
`sm_game_ram_sites` 42 to 38, `sm_game_coupled_files` 10 to 8. No Session field or
format changed.

- `compatibility_byte`/`compatibility_write_byte` are sm/'s own trait,
  `ProjectedBytes` in `sm/src/projection.rs`, implemented by `Game` and the Runtime
  with the delegates they had. sm_game's `CompatibilityRam` keeps its typed
  supertraits and `observed_inputs`; its name is now a misnomer. Renaming it touches
  about 30 files, including the `d9` and `fc9` lanes', so it waits for them.
- `session_cohort` needs only the projection's `Views` (`load_words`,
  `store_words`), so `load`, `store`, `with` and `with_projected` bound on `Views`.
  The cohort copies stay.
- The sm_game test hosts (`tests/state_layout.rs`, `tests/escape_typewriter.rs`)
  address their bytes through their own helpers.
- `RAM_VARS`, `RamVar` and `Shape` are in `sm/src/verification/ram_names.rs`.
  sm_game keeps the one name list as `ram_table!` and generates its offsets
  (`words`, `word_arrays`, `bytes`) from it. sm/ builds the named table from the
  same rows, and the register mirrors' name check moved with it.
- `session_encoding::menu`, a RAM decode nothing called, is gone.

What stays in sm_game until `foreign` and `Game` go (production lines, before
each file's tests):

| Code | Lines | Why it stays |
| --- | ---: | --- |
| `session_encoding.rs` | about 2,000 | the lease's export and import (`encode`, `encode_spans`, `decode`, `projected_byte`, `write_projected_byte`, `inputs`, `inputs_from_session`, `InputProjection`); scene commits and persistent sessions also call `projected_byte` |
| `fx_encoding.rs` | about 2,030 | the FX lanes of the same projection |
| `actors/codec.rs` | about 480 | the actor lanes |
| `menu_window_encoding.rs` | about 80 | the map-menu window lanes |
| `ram.rs` | about 810 | the name list and offsets every encoder above addresses |
| `projection.rs`'s `Field::address`, `Decode::decode` | about 70 | the byte path of `Views` hosts: `Game`, and the Runtime outside its lease |

Another 16 files have a `fn address` for their words. sm/ calls `encode` and
`decode` in about 20 verification observers too; those move to sm/ verification
with the encoder.

## Foreign calls lane (2026-11-04, `collab/rs-fc6`, on `rs-fc5`)

`foreign_calls` 121 to 103, `legacy_engine_ram_sites` 460 to 458; `sm_game_ram_sites` and
`game_host_impls` unchanged (82, 91). No lease is added mid-frame: each retired call runs
on the lease that called it, in an owned player call (`owned_step`, `owned_player_call`,
`on_lease`, `draw_objects`), which refreshes the observations on entry and return.
Moved to the lease:
- The pirates' `dispatch_state`: every state handler takes the AI's `PirateLease`
  (behaviour: lease merge); `PirateLease` loses `foreign` and its `Game` type.
- Ridley's tail floor test and the projectiles' `ridley_offscreen`, through
  `room_state::LevelCells` (a blanket over `LevelData`; the Runtime reads its Session
  terrain, `lease_level_word`, LEGACY row "Level data past the cells").
- The message box poll (`dialogue_ram::Access`) and the gunship's item restores; the
  restore cases' observer reads the lease's projected words.
- The native RomPtr diagnostics (`Runtime::report_native_diagnostic`): the Samus table
  pointers, the boss lease's `NativeDiagnostics`, `legacy_native_record`, the room entry
  pointers and the player update content. The low-window observer
  (`room_contact_start_observer::diagnostic_image`) snapshots the lease's projected RAM.
  The room contact start cases (6, 7, 10, 15, 19) decide that this image equals the
  foreign export's; commit `47c2c4ad6` holds the sites they reach and can be dropped alone.
- The room mechanisms' setup diagnostics (`rtl::unreachable` directly).
- The Samus echoes (`draw_objects`), the escape countdown (`escape_timer::Host`; the
  Runtime is a `MapHost`) for the Ceres push and the gamma actions, the power bomb and
  crystal flash spawns (`spawn_hdma_owned`; the channel registers stay a D2 `foreign`
  per object), and the HDMA explosion start calls (`HdmaLease` gains the window sources,
  the sequence words and the sound queue).

Left, with what each needs:
- The X-ray HDMA arms and the explosion afterglow, scaled, prescaled and finish arms:
  `with_object`, `with_current_object` and `run_call`'s fallback retire only together
  with the X-ray background, the sky's horizontal scroll and the Tourian statue.
  `xray_background::Host` needs `display_refresh::Host` and opens `game.runtime()` nine
  times; `explosion_window::Host` needs `MenuRuntime` (the Runtime cannot take it: its
  `Allocator` impl would conflict with the blanket), `WorkRamAccess` and the game's
  `k_out` window row. Narrow both hosts to what they call, as `escape_timer::Host` does.
- The pre-action fallback (X-ray restore right, sky vertical scroll, title gradient
  backdrop), the title gradient restart, the sky rows and the room initializers: all take
  `menus::host::Host`.
- Species dispatcher fallbacks (`species::pre_instruction`/`ai`/`instruction`/`touch`/
  `shot`/`draw_hook`, `run_a7`, `run_aa`, Torizo's `run_movement`): the fallbacks route any
  address into the generic table on Game, so they retire only when that table runs on the
  lease; no species is small enough to remove a site alone.
- The grapple tiles: not proved. The queue's native transfer can store at $00D0 + any
  cursor, over the tile fields; on the game the tile store goes to the sequence owner
  (`publish_sequence_words`), on the lease to the projection. A replay of an aliasing
  cursor has to show the two agree first.
- The room entry leaf effects (transition sounds, Samus drawn, timer draw, spin jump
  sound) can now move: `player_command_in` and `escape_timer::Host` exist on the lease.
- `rom_record` (Runtime `Host::record`, `room_capture`, the room entry content): four
  verify observers snapshot RAM and the backend; the room background diagnostic's too.
- `reset_room_enemy_workspace` ($7E:7000..$97FF fill) needs a typed owner; the vertical
  slope behaviour read is a bus read that wraps from $7F:FFFF into bank $80.

## Foreign calls lane (2026-11-04, `collab/rs-fc7`, on `rs-fc6`)

`foreign_calls` 103 to 90; `sm_game_ram_sites`, `game_host_impls` and
`engine_ram_address_sites` unchanged. No lease is added mid-frame. Each retired call runs
on the lease that called it, in an owned player call, which refreshes the observations
on entry and return. No Session field or format changed.

Retired:
- Unreachable fallbacks. The lease's draw hook fallback and Phantoon's unknown $A7
  function were only ever the table's unreachable arm; they call `Fatal::unreachable`.
- Bank $AA. `species::EMPTY_AA_FUNCTIONS` holds the six empty functions, which are the
  pre-instruction table's only $AA arms besides the Torizos'. The Torizo's movement
  fallback and Shaktool's `run_function` run an empty function in an owned call and
  fail on anything else. `torizo::Ai::run_function` is gone.
- Contact and shot reactions. `lease_touch` and `lease_shot` now hold every arm the game
  ran: the common reactions, Ridley's contact, the dud, and the pirates'
  `*_on(PirateLease)`. `LeaseActors::Lease` and `LeaseScope::Scoped` require
  `PirateLease`. The lease answers `EMPTY_TOUCHES`/`EMPTY_SHOTS` with an owned call and
  fails on the rest.
- Saves (gunship, save station). `save_slots::SlotHost` is a blanket over menu `Host`,
  and the Runtime implements it. The flags-save observer takes the lease's projected
  RAM (`observe_image`).
- The $7E:7000..$97FF fill is `actors::codec::clear_room_workspace`. It clears the
  saved records, extended blocks 0-3, the Botwoon death words and the projectile angles.
- The X-ray window tables. `BeamHost` is `SessionRam` plus `gradient_word`.
- The sky tile rows (`scrolling_sky::RowHost`) and the HDMA program's title gradient
  restart (`hdma_objects::GradientHost`, required by `HdmaLease`).

Corpus that exercises these: the species rows with pirates, Ridley, the A0/A5/A6/A8 common
reactions and Torizo/Shaktool (Golden Torizo, Bomb Torizo, Shaktool's room). Also:
- the gunship and save-station saves, and the flags-save verification case;
- every room load (the workspace fill);
- the X-ray scope rows;
- the Wrecked Ship's land and ocean sky rooms;
- the title (gradient restart).

Left (18 calls outside D2/D7/D9), with what each needs:
- `actor_ai_legacy`, `actor_instruction_legacy` and the gunship's other
  pre-instruction (session_runtime): the AI, instruction and pre-instruction tables'
  generic arms take `H: LeaseActors` (Game) and open `game.lease()`. Each species
  (crawlers, bang, metroid, evir, beetom, geruta and about 30 more) needs a `LeaseScope`
  (`with_lease`) form, and crawlers, yard and geruta re-enter `pre_instruction`. The
  `native_ai` arms are bank $A9's and are already caught by `bank_a9_routine`.
- The HDMA call/pre-action fallbacks (hdma_dispatch 251, 366), `with_current_object`,
  `with_object`, `Leased::foreign` (hdma_program) and the X-ray restore right
  (player_transitions): `xray_background::Host` and `explosion_window::Host` still need
  narrowing, as `BeamHost` and `RowHost` were here.
- The fx initializers (room_state): eleven arms on menu `Host`. Each needs a narrow
  bound, as the sky rows got.
- The message display (room_mechanisms) and `rom_record` (session_runtime `record` and
  `actor_legacy_record`, room_entry/content `room_capture`): their verify observers
  inject RAM and ROM writes (`put_parts`, `ram_word`), not only snapshots. On the lease
  the injections would have to be stored back through the projection.
- The room-entry leaf effects (compatibility 83): 14 arms. The call retires only when
  music, enemy graphics, sprite palettes, the BG3 scroll HDMA and the area-map
  save/load all have lease paths.
- The grapple tiles (hosts): not proved, as fc6 found.
- The slope behaviour read (player_collision): past $7F:FFFF it reads bank $80's low
  half. That half is the WRAM mirror, the PPU/CPU registers (some of them, such as
  $2180, read WRAM) and open bus, so the bus read can't be a projection.

## Foreign calls lane (2026-11-04, `collab/rs-fc8`, on `rs-fc7`)

`foreign_calls` 90 to 80, `game_host_impls` 90 to 89; `sm_game_ram_sites`,
`engine_ram_address_sites` and `legacy_engine_ram_sites` unchanged. No lease is added
mid-frame. Each retired call runs on the lease that called it, in an owned player call
(`on_lease`, `on_object`, `owned_player_call`, a boss's `owned_step`), which refreshes
the observations on entry and return. No Session field or format changed.

Retired:
- The X-ray background (3). `xray_background::Host` is the owned-Session view plus the
  readback queue (`display_refresh::ReadbackQueue`, split out of the refresh host), the
  VRAM write queue, the screen workspace, the HDMA objects, the sequence words, the
  window sources and sound. The Runtime implements it; Game does not. The HDMA call
  fallback, `with_current_object` and the restore right are gone.
- The explosion windows. `explosion_window::Host` drops `MenuRuntime`,
  `RuntimeAccess` and `CompatibilityRam` for what the windows call, plus
  `pose_command`; the window row is still Game's `k_out`.
- The sky scrolls (`scrolling_sky::ScrollHost`), the title gradient backdrop and the
  Tourian statue (`room_visuals::StatueHost`, now the Session view plus the shake).
  With these, `with_object` and the pre-action fallback are gone (2), and the
  interpreter's own `foreign` forwarder, whose one caller left was the channel register
  write, is gone too (1): that D2 write now calls the lease's `foreign` directly.
- The FX initializers (1), through `room_state::fx::SetupHost`. Game is a SetupHost
  through a blanket over menu hosts that opens a fresh lease per rule, so the room setup
  code's land and haze calls are unchanged.
- HDMA spawns (3): the morph ball eye's beam, Phantoon's wave and the Bomb Torizo haze.
  Each spawn's channel registers make the allocator's existing D2 foreign call on the
  lease, as the power bomb spawns do.

Corpus that exercises these: the X-ray scope rows (capture, overlay, restore); power
bombs and the Crystal Flash; the Wrecked Ship's land and ocean sky rooms and the banded
sky; the title (gradient backdrop); the Tourian statue room; every room whose FX starts
lava, acid, water, weather, fireflea lighting, haze or the Ceres effects; the morph ball
eye, Phantoon and Bomb Torizo rows.

Left (outside D2/D7/D9), with what each needs:
- The species fallbacks (`actor_ai_legacy`, `actor_instruction_legacy`, the gunship's
  other pre-instruction): unchanged from fc7.
- The room-entry leaf effects (compatibility 83). Moving every arm does not lower the
  count: `LoadEnemyGraphics` copies to VRAM now (`RoomDma::copy_to_vram_now`, a forced
  blank DMA), so on the lease it needs a D2 foreign of its own. The music arms can use
  `music_requests` over the lease's `AudioHost`; the map arms need `map_station_byte`
  and the used-station bytes on the lease.
- The message display (room_mechanisms runtime 155) and `rom_record` ×3: unchanged; their
  verify observers replace the loaded ROM and write RAM inside the read.
- The grapple tiles: not proved.
- The slope read (player_collision): D2. For a block of $9BFE or more the read leaves
  $7F and reaches bank $80's low half: the WRAM mirror, then from block $BBFE the PPU
  and CPU registers ($2180 WMDATA advances the WRAM address on read) and open bus. It
  is a bus access to hardware, so it stays with D2.
- Also seen: Yapping Maw's visibility (`Runtime::new(game).update_maw_visibility`) stores
  raw RAM and projects only some lanes into the fresh lease, so running it on the
  calling lease could leave lanes that the publication would overwrite. It needs a typed
  store first. Mother Brain's neck offsets use the multiplier registers (D2).

## Foreign calls lane (2026-11-04, `collab/rs-fc9`, on `rs-fc8`)

`foreign_calls` 80 to 75, `game_host_impls` 89 to 86, `legacy_engine_ram_sites` 452 to 450;
`sm_game_ram_sites` and `engine_ram_address_sites` unchanged. No lease is added mid-frame:
each retired call runs on the lease that called it, as an owned call, which refreshes the
observations on entry and return. No Session field or format changed.

Retired:
- ROM records (3): `Runtime::low_window_record` takes the low-window `rom_record` of the
  content `Host::record`, `actor_legacy_record` and `room_capture`. The four verifier
  observers are `rtl::low_window_observers`, shared with `rom_record_parts`. On the lease
  they get a detached image of the projected RAM (`WorkRam::detached`) and the Session's
  enemy projectile spawn; each byte they inject is stored back through `set_byte`, where
  the foreign import took it. The cases that decide this: sprite effects 111-115, room
  references 32-33, room background 29, actor population 11, 17, 18 and 20.
- Species fallbacks (3): `species::table_ai`, `table_pre_instruction` and
  `table_instruction` run on `TableLease`, which the Runtime implements. Every species the
  tables reached on Game takes the lease (behaviour: lease merge where a routine opened
  several leases or called the game between them). Pirates: `PirateHost` replaces
  `Pirates`. Mother Brain's body instructions run on the lease (the footstep earthquake
  is the Session's feedback words). Game loses `PreInstructionHost`, `InstructionHost`
  and `Pirates`; the `address_dispatch`/`callback_dispatch` macros lost their last users.
- The $A2:804C RTL does nothing on the lease. The initializer and population cases check
  that the game owns the payload inside that dispatch, so only when one is armed does
  `dispatch_noop_on_lease` (verification, D7) export the lease to run their hook. The
  count moves one call from the engine into D7.

Corpus that exercises these: every species row (crawlers, geemers, zeelas, sovas, yards,
gerutas, holtzes, squeepts, rios, wavers, multiviolas, metroids, beetoms, rippers,
dragons, fire geysers, oums, the work robot, sidehoppers, the space pirates and ninja
pirates, bang, tatori, cacatac and owtch, polyp, puyo, rinka, reflec, kago, fune, alcoon,
evir), Mother Brain's fight (body steps, the falling tubes), Botwoon's and Gamet's
pre-instructions, Phantoon, the gunship, and the actor population, sprite effect, room
reference and room background verification cases.

Left (outside D2/D7/D9):
- The grapple tiles (hosts 460): not proved. `queue_native_transfer` stores at
  $00D0 + a full u16 cursor, seven bytes per entry, so a tail of $0BFA or more reaches the
  charge word ($0CD0), and higher tails the point animation timer and tiles ($0D3E..). On the lease the store and `tile_word` both go through the projection
  (`set_byte`, `TileField::current`); inside the foreign call both go to RAM, which the
  import re-reads. They agree only if the projection of those addresses is exactly the
  import's decode. A replay would have to show a frame whose tilemap queue tail is at
  least $0BFA when `update_tiles` runs (a corrupted tail during grapple), with the tile
  timer, frame and charge words compared after the frame on both paths. No corpus row
  reaches such a tail.
- The message display (room_mechanisms runtime 155): not attempted. Its observer is only
  half the problem: `message_boxes::display` takes the whole menu `Host`, which the
  Runtime does not implement, so `PresentationDialogue`'s 21 effects need a narrow host
  first (as `SetupHost` was for the FX), then the observer can use the detached-image
  pattern above.
- The room background diagnostic (room_entry content 96, `LoadMode::Initial`) is the same
  shape as the ROM records and can take `low_window_record`'s observer pattern.
- Unchanged from fc8: the room-entry leaf effects (compatibility 83), the level and buffer
  operations and setup codes in compatibility, the Maw visibility store, `flush_tilemaps`,
  the boss lease's decompression, the mode 7 queue and animated tile uploads.

## Foreign calls lane (2026-11-04, `collab/rs-fc10`, on `rs-fc9`)

`foreign_calls` 75 to 73, `game_host_impls` 86 to 85; `sm_game_ram_sites`,
`engine_ram_address_sites` and `legacy_engine_ram_sites` unchanged. No lease is added
mid-frame: each retired call runs on the lease that called it, in an owned call, which
refreshes the observations on entry and return. No Session field or format changed.

Retired:
- The room background's initial-load diagnostic (room_entry content 96):
  `Runtime::room_background_diagnostic` prints, marks the game failed and runs the room
  background observer over a detached image of the projected RAM. `Runtime::observe_detached`
  is now the shared detached-image helper; `observe_low_window` uses it too. The observer's
  Game form (`room_background_observer::diagnostic`) is gone. Case that decides it: room
  background 29.
- Yapping Maw's off-screen word: the main AI stores the slot view's `off_screen` (var_36) on
  its own lease in `owned_step`, which refreshes the camera the fresh lease imported. For a
  slot in the table the word is at $7E:802C + slot, in the extended RAM the fresh lease
  already adopted, and never one of Botwoon's death words, so the final state is the fresh
  lease's. The Maw's `Host` trait and its Game impl are gone. `Runtime::update_maw_visibility`
  is now `cfg(sm_verify_dll)` for the two D7 cases that call it. Row: `enemy-yapping-maw`.
- No count change: the four cfg-gated verify-only hooks in sm/src/game (the sprite effect
  draw, the mechanism native read, the actor population stage and opcode) now live in their
  observers (`draw_on_lease`, `native_read_on_lease`, `stage_on_lease`, `opcode_on_lease`).
  The engine methods only forward. They stay foreign calls until `foreign` goes (D7).

Left, with what each needs:
- The message display (room_mechanisms runtime 155). Not attempted, because it would not
  lower the count. Of `PresentationDialogue`'s 21 effects, the audio ones (`emit_cue`), the
  dialogue words, the workspace, the display controls, the palette, the das0l word
  (`samus_game()`) and `display_refresh::wait` all have lease forms. Three things do not:
  - About ten effects (`PrepareDisplay`, the background save, upload and restore, the
    scroll enable and HDMA, the message and selection uploads, `RestoreDisplay`) are
    `Video::apply` commands. On Game these are immediate DMAs and register writes
    (`display_commands::apply`) that read and write the message workspace in RAM. On the
    lease each would be a D2 device foreign of its own, unless `display_commands::apply`
    were made generic over the lease's `Registers`. That costs one export and import per
    register write.
  - `UpdateHdma` runs `hdma_objects::update_hdma`. Its `DispatchHost` bound is the whole
    menu `Host`, and `hdma_objects` is the d9 lane's file. It also opens `hdma_lease()`,
    which would be a nested lease.
  - `input` samples through `menus::controller_input::sample`, which needs the menu `Host`
    and reads JOY1L (D2).
  So a dialogue host on the Runtime would remove one call and add at least two (Video and
  the HDMA update). It belongs after D2's device port and D9's renderer.
- Unchanged from fc9: the grapple tiles (not proved) and the room-entry leaf effects
  (compatibility 83). The rest are D2, D7 or D9.

## Foreign calls lane (2026-11-04, `collab/rs-fc11`, on `tick-s7c`)

`foreign_calls` 55 to 46; `game_host_impls` 82, `sm_game_ram_sites` 0 and
`engine_ram_address_sites` 19 unchanged; `legacy_engine_ram_sites` 441 to 425. Outside
`sm/src/verification` three calls are left (from 12). No lease is added mid-frame: each
retired call runs on the lease that called it, in an owned player call, which refreshes the
observations on entry and return. No Session field or format changed.

Retired:
- The Mode 7 queue (2): Ceres Ridley's and the Ceres doors' `queue_mode7_transfer` run
  `enqueue_mode7_rom` on the lease, which already implements `Mode7Queue` for the cinematics.
- `flush_tilemaps` (room scrolling): one encoder over the sm/ trait
  `display_transfers::FlushHost`, implemented by Game (the NMI path) and by
  `display_commands::Lease`. The lease publishes, then makes the same register writes through
  the device interval and `start_dma`. The end word, the cleared tail and the consumed enable
  bytes are `set_byte` stores. `CompatibilityQueue` is gone.
- The grapple tiles. The lease's queue stores and tile words use `write_projected_byte`, the
  same per-byte write that the foreign return (`take_written_bytes`) applies to the bytes RAM
  logged. The typed fields that `tile_word` reads are lanes of that projection. So a cursor
  that aliases the charge word or the point animation words stores them as before.
  `grapple_tile_alias_checks` now runs the update on a lease at tail 0 and at the three alias
  tails ($0C00, $0C6E, $0C70), and requires the RAM the Game path leaves.
- The room-entry device writes (3). An MDMAEN start publishes, then calls `start_dma`. Every
  other write is a device write. While a room background case captures, a verify build runs
  the observer over a detached image (`observe_detached`, `rtl::write_device_observed_over`)
  in place of the exported RAM. Case 4's swaps at VMADDH and DMAP1 are the check.
- The room-entry clears (1). The level clear sets each of the 12800 terrain cells to $8000;
  both original store orders reach each word once. The door clears fill the staging
  workspace on the lease, then call `copy_to_vram_on_lease`. The initial FX and BG2 clears
  start from bank $80 or the prepared image on the lease's wire, and read no work RAM. The
  X-ray transition clear runs through the lease's FX sources.
- The room-entry waits (1). The NMI waits are `display_refresh::wait` on the lease (D9).
  `Runtime::wait_for_music` is the same loop, with $0614 and the counter on the lease. Its
  queue step, `Runtime::advance_music_requests`, sends bank uploads and tracks through the
  cue port. The door mechanisms run on this lease (`run_mechanisms_in`).

Display owner fields: not added, no format bump.
- The queue ($00D0..$02CF) and its tail ($0330) already are Session fields
  (`queued_tilemaps`).
- The scrolling requests ($0956..$098D) were already lease stores: `Scrolling::store_request`
  writes them with `set_byte`. The cinematic lease already stores the Mode 7 queue the same
  way.
- The Display's own forms, `scrolling: [ScrollTransfer; 4]` and `mode7_uploads`, need asset
  identities that a numeric store cannot supply. A raw byte field beside them would be a
  second owner.
- Under a persistent owned Display, these stores still mark the Display failed, as the
  existing stores do. Replays run Legacy, where they are the raw stores Game made.

Left outside verification (3):
- The room-entry leaf effects (compatibility 87).
  - Most arms have lease forms: the player commands, `draw_body`, the beam tiles, the three
    music arms over `queue_music_request`/`queue_music_delayed`, and probably the BG3 spawn.
  - `LoadEnemyGraphics` needs `EnemyGraphicsHost` with `copy_to_vram_on_lease`.
  - The three map arms (`save_area_map`, `load_area_map`, `mark_elevator`) are bound on the
    menu `save_slots::Host`, which the Runtime does not implement, and `load_area_map` goes
    through `session_cohort::with`.
  - `DrawTimer` and `LoadSpritePalettes` need their hosts checked. The count falls only when
    all 14 arms move.
- The setup codes (compatibility 461). `execute_setup_code` and `execute_door_code` dispatch
  the room and door ASM routines on Game; each routine needs a lease host.
- The message display (room_mechanisms runtime 155): unchanged from fc10. It needs a narrow
  dialogue host; its `UpdateHdma` must run on the current lease, not open a nested
  `hdma_lease()`.

Corpus that exercises these:
- `boss-ceres-ridley`, `area-ceres` and `tas-ceres-four-doors` (Mode 7).
- Every room load and door: `door-transition`, `door-loading-lag-nmi`, the `area-*` routes,
  and `elevator-traversal` (flush, clears, device writes and waits, including the door
  mechanisms and room music).
- The `samus-grapple-*` rows.
- The room background verification cases (4 and 5) and `grapple_tile_alias_checks`.
- The SFX-loss rows `item-speed-booster`, `enemy-fire-arc` and `music-fir-selector10`.

## Foreign calls lane (2026-11-04, `collab/rs-fc12`, on `rs-fc11`)

`foreign_calls` 46 to 43, `game_host_impls` 82 to 81, `session_entries` 77 to 70;
`sm_game_ram_sites` 0 and `engine_ram_address_sites` 19 unchanged; `legacy_engine_ram_sites`
425 to 409. Outside `sm/src/verification` no `foreign` call is left (from 3): every remaining
one is verification (D7). No lease is added mid-frame: each retired call runs on the lease that
called it, in an owned player call, which refreshes the observations on entry and on return as
the foreign call left them. No Session field or format changed.

Retired:
- The room-entry leaf effects (14 arms). The player commands, the frozen body (gated on the
  Session's `room.entry.elevator_flags`), the escape timer, the beam tiles, the sprite palettes
  (now over `LoadedRom` and the Session) and the BG3 scroll HDMA run on the lease.
  `EnemyGraphicsHost` is `RoomDma` plus a ROM record; the Runtime's record is
  `actor_legacy_record` (the `rtl::rom_record` boundary) and its copy is
  `copy_to_vram_on_lease`. The music leaves are `music_requests` routines over the audio host
  and the room entry's music fields. The map leaves are bound on `save_slots::MapHost` (a
  `SlotHost` with the map facts and a gameplay flags store); the explored map loads into the
  Session's save selection, committed when it changed, in place of the cohort copy.
  `rtl::rom_record` is gone.
- The room setup and door codes. `room_state::ScriptHost` is `fx::SetupHost` + `MainCodeHost`
  + the gameplay context, menu state, video and `CommonRuntime`, plus an event mark and a
  door's scroll updates. The fresh-lease stores are `store_session` (refreshing where
  `with_session` did). The ocean sky spawns with `spawn_hdma_owned`.
- The message display. `message_boxes::DialogueHost` names its 21 effects' needs, with a
  blanket over the menu `Host`. `UpdateHdma` runs `update_hdma` on the current lease through
  `hdma_objects::UpdateHost` (`hdma_program::run_objects_on`); no nested `hdma_lease()`.
  `input` samples through `controller_input::SampleHost` (JOY1L/JOY2L through
  `read_register_word`, the soft reset as the Session's game state). The verify observer
  reads and stores $0613 on the lease.

Corpus that exercises these:
- Every room load and door: `door-transition`, `door-loading-lag-nmi`, the `area-*` routes and
  `elevator-traversal` (leaf effects, music, enemy tilesets, setup and door codes, including
  the elevatube, treadmill and Ceres elevator doors).
- Map stations and area maps; the Ceres and Tourian escape timer rooms (`area-ceres`,
  `tas-ceres-four-doors`).
- The `item-*` message box rows and save-memory (message display, HDMA update, controller
  sample, dialogue sounds).
- The SFX-loss rows `item-speed-booster`, `enemy-fire-arc` and `music-fir-selector10`.
- The room mechanism callback-edge case (message display observer).
