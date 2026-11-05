# Map Rando: what the foundation must offer

Research lane `research/foundation-maprando`, based on `origin/main` at `e3cf770`.
It changes no code. It answers one question for the rebuild (REVIEW, "Rebuild on a
new foundation (Jim, 2026-11-05)"): what modularity does Map Rando need from the
foundation's content, ID and options model, and what must stay vanilla-exact?

## Sources and licences

| Source | Pinned at | Licence | Use here |
| --- | --- | --- | --- |
| [Map Rando][mr] (`blkerby/MapRandomizer`) | `5cb618b8a64157d4164f02575e88a340537b8c9a`, `rust/VERSION` 124 | MIT | Read source; cited by path |
| [sm-json-data][sjd] | `f4ae6709bea3ee15713bbfd84363ea52361f1b2a` | CC BY 4.0 | Read schema and docs; counts measured |
| [maprando.com][site] About page | `rust/maprando-web/templates/about.html` at the pin above | MIT (same repo) | List of built-in changes |

Nothing from either project is copied into this repository. File paths below are
relative to the pinned Map Rando tree unless stated. Counts were measured with a
throwaway script over the pinned trees, kept outside the repository. Patch names
and setting names are named here so they can be found; their code and data are
not reproduced.

## How Map Rando makes a game

1. **Map generation (offline).** A trained model places every Zebes room on a
   72 × 72 grid with no overlaps. Every door transition meets a compatible one
   (sand to sand, elevator to elevator), and every room can reach every other.
   Post-processing splits the rooms into 6 areas with 2 sub-areas (and
   sub-sub-areas for tile themes), and moves map, save and refill rooms. The
   result is a pool of about 434,000 stored maps (about.html, "Map generation").
2. **Per seed (online).** The generator picks a map, places items by forward fill
   over [sm-json-data] logic, picks objectives, start location and door locks,
   and computes the escape timer (`rust/maprando/src/randomize.rs`).
3. **Patching.** `rust/maprando/src/patch.rs::make_rom` writes all of this into a
   vanilla ROM: ASM patches, door headers, PLM lists, room-state words and new
   tables in free space.
4. **Customization (at download, client side via wasm).** Cosmetic and input
   options are applied to the patched ROM (`rust/maprando/src/customize.rs`).

A seed is stored as typed records as well as a patch
(`rust/maprando-web/src/randomize_helpers.rs::save_seed`): `public/settings.json`
(the `RandomizerSettings` tree), `randomization.json` (the `Randomization` struct:
objectives, save-animals mode, map, toilet intersections, locked doors, the item
for each location, start location, escape time, seed), `map.json` and the spoiler
log. **This record, not the patched ROM, is the natural import seam for us**: the
engine can't execute 65816 patches, but every patch effect is selected by a field
in these two records.

### Identities Map Rando uses

`rust/maprando-game/src/lib.rs` (type aliases near the top):

- **Room**: sm-json-data room id (`RoomId`) for logic, and the vanilla room-header
  address (`RoomPtr`) for patching. The map record indexes rooms by position in
  `room_geometry.json` (253 rooms, Zebes only, twins merged).
- **Door**: a `DoorPtrPair` of `(exit_ptr, entrance_ptr)`, both vanilla door-data
  addresses. `exit_ptr` is the door header in this room's door list (used when
  leaving through this doorway); `entrance_ptr` is the vanilla header that
  arrives at this doorway. Either may be absent (12 one-sided doorways, such as
  sand entries with no exit). sm-json-data door nodes carry the same `exit_ptr`
  as `nodeAddress`.
- **Map**: `Map { room_mask, rooms: [(x, y)], doors: [(DoorPtrPair, DoorPtrPair,
  bidirectional)], area, subarea, subsubarea }`.
- **Item location**: an sm-json-data `(room id, node id)`; the 100 locations map
  to vanilla PLM addresses. Collected state uses the vanilla item bit index.
- **Flags**: sm-json-data game-flag names (`f_DefeatedKraid`, ...), 30 of them.
- **Start location**: `(room id, node id, x, y, camera offset, door to load
  through)`; `rust/data/start_locations.json` lists 328.

Door counts in `room_geometry.json`: 582 doorways: 494 horizontal and 50
vertical ordinary doors, 24 sand doors, 14 elevator doors.

## Catalogue

Each item says what Map Rando changes and how, what the foundation must offer,
and what must stay vanilla-exact. **F** marks a foundation requirement (the
content, ID or options model has to carry it from the start). **L** marks
something that can be added later without changing the foundation, provided the F
items hold.

### 1. Room placement and the map grid

**What changes.** Every room gets a new map position and a map area, sub-area and
sub-sub-area. "Small" maps omit rooms (`room_mask`) and wall off their doors.
Two vanilla room pairs are twins that share one map footprint and move together:
Pants Room with East Pants Room, and West Ocean with Homing Geemer Room
(`twin_rom_address`). Map Rando writes both twins whenever it changes one (door
data, music). The Toilet (the Maridia glass tube) may overlap another room on the
map. Map Rando records the room it crosses and copies item PLMs that fall inside
the tube into the Toilet's PLM list (`patch.rs::apply_toilet_data`). Tile themes
also redraw the Toilet with the crossed room's backdrop (`customize/retiling.rs`).

**Foundation must offer.**

- **F.** A room's identity is a stable `RoomId`, never its index, header address
  or vanilla neighbour. Its *placement* (map x/y, map area, sub-area) is a
  separate per-world record. The room's content doesn't change when it is placed
  somewhere else. Today `sm_data::world::RoomDefinition` holds `index`, `area`,
  `map_x` and `map_y` together with the room's content. These need to be split
  apart.
- **F.** A placement unit can hold more than one room (twins), and a room can be
  left out of a world. Validation then needs every exit in the world to resolve,
  or to be a declared wall.
- **L.** The Toilet overlap: an offline content transform (an extra object list
  and a backdrop for one room), which needs no engine change.

**Vanilla-exact.** The vanilla world is one placement record among many. Built
from vanilla data, it must reproduce today's room headers, map pages and
area-indexed state exactly.

### 2. Door graph rewiring

**What changes.** `patch.rs::write_door_data` / `write_one_door_data`: for each
connection, the 12-byte vanilla door header of the *destination entrance* is
copied over the *source exit*. Only the area-change bit (`0x40` in byte 2) is
recomputed from the two rooms' map areas. When an exit needs extra behaviour, its
door-ASM word points at a generated stub that runs that behaviour and then jumps to
the destination entrance's own door ASM (`prepare_extra_door_asm`). The extras are:

- marking map tiles explored or revealed on the far side (elevator shafts, area-transition
  arrows; `auto_explore_elevators`, `auto_reveal_arrows`);
- clamping Samus' X into the shaft when entering a sand room (`clamp_samus_position`,
  `sand_clamp.asm`);
- the Toilet exit hook, and a boss-room exit cleanup for Kraid, Draygon, Crocomire and
  Phantoon (`boss_exit.asm`).

The connection constraints come from map generation: the direction must match
(left↔right, up↔down), sand only meets sand and elevators only meet elevators,
and the graph must be strongly connected.

These are the consequences Map Rando has to repair:

- **Door-specific FX.** A room-state FX list selects its entry by the door used to
  enter, for example lava rising in Rising Tide, Volcano Room, Speed Booster Hall and
  Climb, or acid level in the Acid Statue Room and Amphitheatre.
  `setup_door_specific_fx` rewrites each key to the new partner's exit header.
- **Save stations.** Save-station records point at an entrance door header for
  the scroll setup they use when loading. `fix_save_stations` repoints them at
  the connected door. When no door leads there any more (a wall), it sacrifices
  an unused door header.
- **Rooms that assume the entry side.** Kraid's AI restricts Samus to the left
  screen. Crocomire, Spore Spawn and Kraid force red scrolls. Big Boy checks the
  layer-1 X position. Shaktool sets red scrolls. Mother Brain's room expects the
  right entrance. Map Rando patches all of these: camera fixes, `fix_kraid_door`,
  the Big Boy door-direction check in `vanilla_bugfixes`, `shaktool.asm`,
  `mb_left_entrance`.
- **Transition races** that never show with vanilla neighbours, because timing
  differs between pairs: `load_plms_early`, `fix_transition_bad_tiles`,
  `fix_horiz_doors`, `horizontal_door_fix`, `door_irq_fix`, `fix_choot`,
  `fix_kraid_vomit`, `fix_kraid_hud`.
- **Gray doors** that close on entry are replaced by blue doors, except in boss,
  miniboss and four pirate rooms (`remove_non_blue_doors`).

**Foundation must offer.**

- **F. Doorway, not exit, is the unit.** Give every physical doorway a stable
  `DoorwayId`. It is the same thing as Map Rando's `DoorPtrPair` and sm-json-data's
  `(room, door node)`. A doorway owns its *arrival* data: spawn position,
  direction, distance to spawn, door-cap position and the scroll setup run on
  entering. It also owns its exit-side data: where the transition starts and which
  axis carries position. A world's topology is a set of
  `Connection { a: DoorwayId, b: DoorwayId }` plus declared one-way arrivals and
  walls. "Leave through A" resolves to "arrive at partner(A)". Today
  `ExitDefinition { destination, arrival }` holds arrival data per vanilla exit,
  which is exactly the copy Map Rando has to patch around.
- **F. Data derived from a connection is computed from the pair.** The area-change
  and CRE-reload flags of a transition (today `ArrivalDefinition.flags` and
  `ExitDefinition.skip_cre_upload`) come from the two rooms' placements and
  tilesets, so they are not stored per vanilla exit.
- **F. Anything keyed by "the door you came in by" is keyed by this room's
  arrival `DoorwayId`.** This covers FX entry selection, the door-matched
  background transfer (today `BackgroundCommand::MatchExitTransfer { exit }`),
  save/load-station scroll setup (today `StartDefinition.exit`), and any room or
  boss logic that reads the entry side. Map Rando's FX rewrite shows the meaning
  is "entered through this doorway", which doesn't change when the map is
  rewired.
- **F. Entry-side assumptions are typed.** Room and boss logic that behaves
  differently by entry reads a typed `EntrySide`/`DoorwayId` from the Session,
  never a camera position or a raw door word. Then a non-vanilla entry is just
  another value. Whether the vanilla bug is kept (Kraid's left-screen clamp,
  Crocomire's scrolls) is a profile option (section 9).
- **F. Per-arrival hooks are data.** Map reveal on arrival, the sand X-clamp range
  and the boss-exit cleanup are typed properties of a doorway or connection,
  interpreted by the transition module. They are not generated code.
- **F. Door kinds** (normal, sand, elevator; horizontal, vertical) are typed on
  the doorway, and validation checks them. The engine doesn't enforce
  generator-level constraints like strong connectivity.
- **L.** The specific race fixes: options in the Map Rando profile (section 9).
  Our engine may not have the race at all. Where it doesn't, the option is a no-op
  and is recorded as such.

**Vanilla-exact.** The transition itself: its phases, scroll speed, fade timing,
when PLMs and enemies are initialized, and how Samus' position and camera are
computed from the arrival data. These must be the vanilla rules applied to
whatever pair is connected. Cross-room strats rely on them (section 13).

### 3. Two kinds of area

**What changes.** Map Rando keeps the vanilla area byte in each room header. It
adds a *map area* in an extended room record (`extra_room_data.map_area`, read
through the room state's unused special-X-ray word; `write_extra_room_data`). Only
the map consumers are moved to the map area (`map_area.asm`): the explored-map
mirror, the pause map and its scroll limits, the HUD minimap, map-station
activation and the area label. Boss bits, save-station tables and every other
area-indexed table stay on the vanilla area. Area-themed music, palettes and
tiles follow the map area and sub-area.

**Foundation must offer.**

- **F.** Two typed notions: the room's *home area* (part of its identity; it keys
  boss and event state and the vanilla area-indexed tables) and its *map area*
  (part of its placement; it keys the map page, explored bits, map station,
  area-transition markers and the area themes). In the vanilla world the two are
  equal. Every area consumer in the engine names which of the two it reads.
- **F.** Sub-area (and sub-sub-area) is placement data available to presentation
  rules (music, themes). The engine gives it no gameplay meaning.

**Vanilla-exact.** Boss and event flag layout and its save representation, keyed
by home area.

### 4. Map presentation

**What changes.** The whole pause map and minimap are regenerated. Per-room map
tile graphics, a per-room tilemap and dynamic item-dot tiles are written to a new
bank (`apply_map_tile_patches`, `patch/map_tiles.rs`). The game composes area
pages from the per-room tiles of whichever rooms are placed in each area. There
are several additions:

- markers for doors and locks, area-transition letters or arrows, item-dot
  tiers, heat and water shading, map/refill/save/objective icons;
- room names;
- an expanded HUD and map palette scheme (`map_area.asm`, `hud_expansion_opaque.asm`);
- explored and revealed state that survives death and reload (`map_progress_maintain.asm`).

**Foundation must offer.**

- **F.** Map tiles are per-room content relative to the room's origin. The area
  map is composed at runtime from placements. Importing vanilla breaks the
  vanilla area pages into per-room tiles, and the vanilla world composes them
  back to identical pages, which can be checked.
- **F.** Explored and revealed state is keyed by (map area, map cell) and owned by
  the Session, so it survives transitions to rooms that were never neighbours.
- **L.** The particular markers, icons, letters, dot tiers and room names: map
  presentation options, all media-only.

**Vanilla-exact.** Explored-bit semantics and the vanilla map graphics in the
vanilla world.

### 5. Per-room presentation: music, palettes, tiles, FX

**What changes.**

- **Music**: each room state's song (songset and track) is replaced by an area and
  sub-area song. Boss tracks and a few rooms are kept (`use_area_based_music`).
  The tracks then react to events: Zebes asleep, Phantoon dead, PBs held
  (`zebes_asleep_music.asm`). Some sound effects depend on instruments the current
  songset loads, so `sound_effect_disables.asm` turns them off. MSU-1 can stream
  replacement tracks (`msu1.asm`).
- **RNG coupling**: in vanilla, the game RNG keeps advancing while a new track
  loads. Rooms with RNG-reseeding enemies therefore see different values when a
  music change is added or removed. `rng_fix.asm` re-seeds to the vanilla value
  from a table per room/door.
- **Palettes**: area-themed, area-shuffled, scrambled or constant palettes,
  indexed per room (`customize/room_palettes.rs`, `Area Palettes.ips`, `Area
  Palette Glows.ips`).
- **Tile themes** ("Mosaic"): BPS patches retile each room's level data and
  tilesets per theme, without changing collision (`customize/retiling.rs`,
  `in_place_level_data.asm`). Splash and dust FX are disabled where they would
  not match (`mosaic_fx_fix.asm`).
- **Statues hallway**: rooms next to Tourian can be retiled and re-scored.

**Foundation must offer.**

- **F.** Room-state presentation is a set of replaceable references to media:
  tileset (tiles, tile table, palette), level graphics layer, music cue, FX
  appearance. A world or media profile can swap any of them without touching
  collision, BTS, objects or enemies. The split between collision/BTS and
  graphics must be exact, because the tile themes replace graphics while keeping
  collision.
- **F.** Music is a typed cue `(songset, track)` per room state. Event-driven
  overrides are presentation rules. The sound engine models the coupling between
  sound effects and the loaded songset as explicit data, not as accident.
- **F.** Any gameplay effect of loading content (load time, RNG ticks during a
  music upload, decompression time) comes from the content being loaded and the
  transition rules. It must not come from a cached vanilla neighbour. The vanilla
  profile must reproduce it, because RNG and enemy behaviour observe it. The Map
  Rando profile's `rng_fix` then becomes "re-seed to the value vanilla would have
  had" on its vanilla connection. That value can be computed by running our
  own vanilla rules over the vanilla world, so no imported table is needed.
- **L.** Theme catalogues, palette sets, MSU-1, per-area music tables.

**Vanilla-exact.** The time and RNG cost of each load in the vanilla world; the
sound effects available under each songset.

### 6. Door locks and door kinds

**What changes.**

- Door colours are randomized per *pair*: both sides get the same lock, opening
  one side opens both (`LockedDoor { src, dst, door_type, bidirectional }`;
  `apply_locked_doors`).
- New beam doors (Charge, Ice, Wave, Spazer, Plasma), limited to one per room by
  VRAM (`beam_doors.asm`), and walls in place of doors (`wall_doors.asm`).
- The gray door rule changes: a gray door doesn't close on entry, but on item
  pickup, boss damage or room clear (`gray_doors.asm`, about.html "Doors"). A
  gray door is added on the right of Metal Pirates Room, and the bottom of Spore
  Spawn Room becomes gray. During the escape, a gray door spawns in whichever
  room connects to Mother Brain's.
- Hazard markers on doors leading to danger.
- `DoorType` is `Blue | Red | Green | Yellow | Gray | Beam(BeamType) | Wall`.

**Foundation must offer.**

- **F.** A door lock is content attached to a doorway, with a lock-state identity
  that a world can share between both doorways of a connection. Opened-lock state
  is keyed by that identity, not by a vanilla PLM slot.
- **F.** Door-cap kinds are an extensible enum. Vanilla variants keep vanilla
  rules, and new kinds (beam, wall) slot in as variants with their own rules and
  media. The renderer imposes no "one per room" limit.
- **F.** Gray-door closing is a rule with typed conditions, so a profile can
  change it.
- **F.** Event-placed objects can name a doorway reached through the topology
  (the escape gray door goes "in the room connected to Mother Brain's
  Room"), not a fixed room.
- **L.** Beam doors, walls, hazard markers and lock colours on the map.

**Vanilla-exact.** Vanilla lock behaviour per side, door-cap animation, shot
response and timing.

### 7. Items

**What changes.**

- Each of the 100 locations gets an item. Map Rando changes the PLM's item while
  keeping its style (visible, Chozo orb, hidden in a shot block)
  (`place_items`, `item_to_plm_type`).
- New items: Wall-Jump Boots, Spark Booster and Blue Booster (split Speed Booster),
  and Nothing. Their PLMs, graphics and message boxes come from `walljump_plm`,
  `split_speed_plm`, `nothing_item` and `extended_msg_boxes`.
- Tank sizes are configurable: missile, super and PB packs, E-tank and reserve
  multiples. These are written into PLM arguments and the credits divisors.
- All items can be made to spawn from the start (`all_items_spawn`): the Morph
  Ball and Pit Room items are copied into their other states.
- Starting items, and item-collection stats and percentage (`item_count`).

**Foundation must offer.**

- **F.** An item location is a stable `ItemLocationId` with its presentation style.
  The item placed there is world data.
- **F.** Item kinds and equipment are typed enums and sets. Vanilla is a subset,
  and adding a kind doesn't change the vanilla save fields. Abilities that Map
  Rando gates behind a new item (wall jump, blue speed, shinespark) are separate
  capabilities in the movement rules. In vanilla a fixed item grants them.
- **F.** Pickup amounts are per-item-kind data in the gameplay profile, not
  constants in pickup code.
- **F.** Which items exist in which room state is world data, so "all items
  spawn" is a content edit.
- **L.** The new items' graphics and messages, item stats.

**Vanilla-exact.** Pickup sequence, fanfare and message-box timing (unless the
profile changes them), and item-bit semantics.

### 8. Objectives, bosses, escape, start

**What changes.**

- **Objectives** are a chosen set of 0 to 19 flags: bosses, minibosses, the four
  Metroid rooms, the Bowling and Acid Chozo statues, and pirate rooms. Mother
  Brain's room gets barriers, one per objective (`mb_barrier_clear`,
  `write_objective_data`), and a pause screen lists them (`pause_menu_objectives`).
  The Statues Room is open from the start.
- **Mother Brain**: Vanilla, Short (ends after phase 2) or Skip. Supers can do
  double damage. Big Boy and Mother Brain cutscenes can be faster.
- **Escape**: Hyper Beam can grant all movement items, and destroys blocks and
  opens gates and doors (`escape.asm`, `escape_items`). The escape timer comes from
  the seed. There are options to save the animals, clear enemies, refill on
  escape and auto-save (`escape_autosave`).
- **Start**: the intro and Ceres are skipped (`new_game.asm`, always applied).
  The start is any of 328 locations, or the escape itself with all bosses dead
  (`set_start_location`, which overwrites a Crateria load station). Starting items
  and flags are set.

**Foundation must offer.**

- **F.** Game flags and events are a typed enum with stable names, mappable to
  sm-json-data flag names. The Tourian gate reads an objective set from world
  data.
- **F.** A start is `(RoomId, position, camera, arrival DoorwayId for scroll
  setup, starting inventory and flags)`. It is not a vanilla load-station slot or
  a door header. The new-game flow (intro, Ceres, landing) is a sequence the
  profile can skip.
- **F.** Boss fight variants (MB short/skip) and the escape's rules (timer value,
  Hyper Beam powers, enemy clearing) are profile parameters of the modules that
  own those rules.
- **L.** Objective screen, barrier graphics, escape timer computation (an
  offline tool).

**Vanilla-exact.** Each boss's behaviour within a vanilla phase, and the escape
sequence.

### 9. ROM patches by group

Map Rando applies 109 ASM sources and 119 IPS files (`patches/src`,
`patches/ips`). It also makes direct ROM writes in `patch.rs` and `customize.rs`.
"Always" means applied to every seed (the base list in `apply_ips_patches` and
`apply_orig_ips_patches`). Everything else is selected by a setting.

| Group | Always | Optional (setting) |
| --- | --- | --- |
| **Topology support** (needed because neighbours change) | `shaktool`, `mb_left_entrance`, `sand_clamp`, `transition_reveal`, `rng_fix`, `load_plms_early`, `boss_exit`, `fix_kraid_door`, `fix_kraid_hud`, `fix_kraid_vomit`, `fix_transition_bad_tiles`, `fix_horiz_doors`, `horizontal_door_fix`, `door_irq_fix`, `fix_choot`, `extra_setup`; door FX, save-station and Shaktool scroll writes | `vanilla_camerafixes` and Kraid/Crocomire/Spore Spawn camera writes (`camera_fixes`) |
| **Gameplay rules** | `complementary_suits` (Gravity alone gives only its increment over Varia), `gray_doors`, `escape`, `mb_barrier_clear`, `nothing_item`, `beam_doors`, `wall_doors`, `vanilla_bugfixes` (includes the lava trap on equipped Speed Booster); layout edits (Main Street tunnel, Aqueduct floor, MB room slopes, Upper Tourian save→map, Statues open, Spore Spawn and Metal Pirates gray doors) | `respin`, `momentum_conservation`, lenient space jump (`infinite_space_jump`), `walljump_plm`, `split_speed`(+`_plm`), `energy_free_shinesparks`, `everything_respawns`, `remove_spikesuit`, `remove_bluesuit`, `disable_major_glitches` + `oob_death`, `etank_refill_disabled`/`_full`, `energy_station_reserves`, `disableable_etanks`, `reserve_backward_fill`, `missile_refill_all`, `no_drops`/`buffed_drops`, `all_items_spawn`, `escape_items`, acid Chozo without Space Jump, Climb lava removal, supers double, MB short/skip (`mb_short_samus_standup`, `fix_hyper_slowlock`), save animals, escape enemies cleared, escape refill, tank sizes, `load_flash_suit`, `load_blue_suit`, `fix_blue_echoes` |
| **QoL: speed and flow** | `new_game`, `fast_reload`, `saveload` (rolling saves, spawn handling), `map_progress_maintain`, `spin_lock`, `aim_anything` | `fast_doors`, `elevators_speed`, `fast_pause_menu`, `fast_saves`, `decompression`, `fast_big_boy_cutscene`, `fast_mother_brain_cutscene`, `escape_autosave`, fanfares (`trimmed_fanfare`, `itemsounds`, `vanilla_fanfare_stop_sounds`), `savestate`, crash handling (`crash_handle_base` and five specific handlers; each Death, Warn, Silent or Crash) |
| **Presentation** | `map_area`, `fix_water_fx_bug`, `fix_dust_torizo`, `disable_map_icons`, `item_dots_disappear`, `hud_expansion_opaque`, `max_ammo_display_fast`, `resource_collection_hudcap`, `escape_timer`, `title_map_animation`, `tourian_map`, `no_explosions_before_escape`, `intro_song`, `sound_effect_disables`, `seed_hash_display`, `credits`, `stats`, `item_count`; MB flashing removed | `animated_map_tiles`, `hazard_markers`, `special_door_reveal`, `pause_menu_objectives`, `zebes_asleep_music`; customization layer (section 11) |
| **Platform** | `sram_check_disable`, `self_check`, `msu1`, `samus_tiles_optim_animated_tiles_fix`; demo disabled | none |

`fast_doors` runs the door transition twice per frame but leaves the fades
alone, so enemy damage during the fade, and R-mode and G-mode setups, keep
working (about.html). Several QoL options are therefore *timing* changes to the
transition and pause phases. That makes them gameplay-affecting under our
fingerprint rule ([CONTRACTS](../target-architecture/CONTRACTS.md), "Content
revisions and fingerprints").

**Foundation must offer.**

- **F.** Options are typed profiles, not patches. A `GameplayProfile` is a struct
  of enums and values, each owned by the module whose rule it changes:
  - suits (damage table);
  - transition (scroll speed multiplier, fade policy, PLM init point);
  - movement (respin, momentum conservation, space jump leniency, wall-jump and
    speed capabilities, blue suit/spike suit/flash suit persistence);
  - shinespark (energy cost);
  - pickups (sizes);
  - stations (refill rules);
  - drops (drop tables);
  - enemies (respawn);
  - bosses (MB variant, damage);
  - escape;
  - glitch policy (out-of-bounds outcome, GT code, pause-menu beam glitch);
  - crash policy.

  Vanilla is the default value of every field.
- **F.** Original crash and softlock conditions are typed outcomes
  (`OriginalFailure::{SpringBallMenu, YappingMawSpark, AutoReservePause, XModeCollision,
  SpriteOverflow, ...}`). The crash policy maps each one to *reproduce*, *death*,
  *warn* or *continue*, which is Map Rando's `FixMode`. They are never panics.
- **F.** The always-on Map Rando set (rows marked "Always" in the gameplay and
  topology groups) is a named profile preset, "Map Rando base". Seeds are only
  beatable as their logic assumes under that preset, for example complementary suits.
- **L.** Every individual optional rule. The foundation needs only a place for
  each rule and a fingerprint entry for it.

### 10. Settings that change mechanics

From `rust/maprando/src/settings.rs`. `OtherSettings` holds: `wall_jump`
(Vanilla/Collectible), `speed_booster` (Vanilla/Split), `energy_free_shinesparks`,
`all_enemies_respawn`, `disable_spikesuit`, `disable_bluesuit`,
`enable_major_glitches`, `savestate` (No/Limited/Unlimited), `race_mode` and
`door_locks_size`. `QualityOfLifeSettings` holds the rest of the mechanics in the
table above: `respin`, `infinite_space_jump`, `momentum_conservation`,
`etank_refill`, `energy_station_reserves`, `disableable_etanks`,
`reserve_backward_transfer`, `enemy_drops`, `ammo_refill_all`, `persist_flash_suit`,
`persist_blue_suit`, `fast_doors`, `fast_elevators`, `fast_pause_menu`,
`fast_saves`, `fanfares`, `crash_fixes`, `mother_brain_fight`, `supers_double`,
the escape options, `all_items_spawn`, `acid_chozo`, `remove_climb_lava`,
`camera_fixes` and `fix_blue_echoes`.

Skill assumptions (`SkillAssumptionSettings`: tech and notable toggles, leniency
frames, boss proficiencies) and item-progression settings only steer the
generator. They don't change the game. The importer reads them as provenance,
not as rules.

**Foundation must offer.** **F.** The importer maps every mechanic setting to a
profile field or rejects the seed with a diagnostic, never silently. This
follows the existing CONTRACTS rule. Generator-only settings are recorded but
have no effect.

### 11. Cosmetic and input options

From `CustomizeSettings` (`customize.rs`), applied after the seed:

- **Samus**: 136 sprite sheets (`patches/samus_sprites/*.ips`, from SpriteSomething),
  each replacing Samus tile graphics and palettes in the vanilla pose layout, and
  the vanilla screw attack animation toggle.
- **HUD**: E-tank colour, reserve HUD style, beeping.
- **Map**: map theme (light/dark), item dot change (stay, fade, disappear),
  transition letters, save/boss/miniboss icons, room names.
- **World**: palette theme, tile theme, door theme (`alternate_door_colors`),
  statues hallway tiling and audio, music (area-themed or disabled).
- **Comfort**: screen shaking (vanilla, reduced, off) and flashing (vanilla,
  reduced), which uses `flashing_placebo` and `rust/data/reduced_flashing.json`.
- **Controller**: shot, jump, dash, item select/cancel and angle up/down on any
  button; buttons for spin lock, quick reload and save/load state; moonwalk.

**Foundation must offer.**

- **F.** Samus' appearance is a media asset keyed by stable pose and frame IDs,
  with a palette set keyed by suit and special state (speed, spark, screw, hyper,
  x-ray, damage, death). A replacement sheet has to cover that contract. Hitboxes
  and timing aren't part of it.
- **F.** A `PresentationProfile` (media-only: palettes, themes, door colours,
  sprite, map style, shaking and flashing levels, HUD style) sits outside the
  gameplay fingerprint. Shaking and flashing are presentation even though the
  original drives them from gameplay code. The Scene carries the shake and flash
  intent, and the renderer applies the profile.
- **F.** Input mapping lives in `sm_app`. Gameplay sees logical buttons. Quick
  reload, spin lock and save states are Session-level commands carried in
  recorded input, so replays reproduce them.
- **L.** Every catalogue (sprites, themes, palettes).

### 12. Saves and lifecycle

**What changes.** Death reloads straight to the last save. L+R+Select+Start
reloads, and repeating it walks back through older saves. Saves roll across
slots. Map progress persists across deaths, reloads and files. SRAM expands to
32 KB. Save states are optional (`saveload.asm`, `fast_reload.asm`,
`map_progress_maintain.asm`, `savestate.asm`).

**Foundation must offer.** **F.** The Session's persistent record (inventory,
flags, locks, explored map) is one typed value with an explicit split between
what reload restores and what survives reload. Map Rando moves the explored map
into the "survives" part. The save format is our own, versioned, and records the
world and profile identity. **L.** Rolling slots, quick reload, save states.

### 13. State carried across doors that were never neighbours

This is the core of Map Rando's difficulty (REVIEW item 9). In sm-json-data's
14,315 strats, the most common entrance conditions are:

| Entrance condition | Count |
| --- | ---: |
| `comeInWithGMode` | 1,354 |
| `comeInShinecharging` | 1,049 |
| `comeInShinecharged` | 818 |
| `comeInWithGrappleTeleport` | 796 |
| `comeInWithRMode` | 416 |
| `comeInRunning` | 227 |
| `comeInNormally` | 198 |
| `comeInGettingBlueSpeed` | 173 |
| `comeInJumping` | 156 |
| `comeInWithStoredFallSpeed` | 145 |
| `comeInStutterShinecharging` | 136 |
| `comeInWithSpark` | 114 |

The most common exit conditions are: `leaveWithRunway` 819, `leaveShinecharged`
794, `leaveWithSpark` 564, `leaveWithTemporaryBlue` 505, `leaveWithGrappleTeleport`
404, `leaveWithDamage` 345, `leaveWithGMode` 267 and `leaveWithMockball` 238. The
full list (`sjd/strats.md`, "Entrance conditions" and "Exit conditions") also
covers:

- spin, space jump, spring-ball bounce, mockball, speedball, super sink;
- knockback, damage boost, i-frames, door-stuck setup;
- X-mode, wall-jump/space-jump/platform below, side platform;
- grapple swing, grapple jump, Samus Eater teleport.

Two modifiers matter for topology: `comesThroughToilet` and `comesInHeated`.
Map Rando's logic joins an exit strat in one room to an entrance strat in
whatever room the map connects. Each join is a prediction that the engine moves
the state across faithfully.

**State that must carry, unchanged by transition code that vanilla doesn't reset:**

- **Movement**: Samus' pose and movement state (running, spinning, morph,
  spring ball, wall-jump eligibility), X/Y speed and subspeed, momentum, stored
  fall speed.
- **Speed**: dash counter and blue speed, the shinecharge timer, and temporary
  blue.
- **Damage**: i-frames, knockback timer and direction, damage-boost state,
  health, reserves and reserve mode (R-mode), and heat exposure state
  (`comesInHeated`).
- **Grapple and glitch modes**: grapple state, including the block position that
  grapple teleport turns into an absolute position in the next room. G-mode,
  direct and indirect: the x-ray and grapple state that changes how the next room's
  objects initialize. X-mode.
- **Equipment use**: charge counter, selected weapon, beam state.
- **Glitched suits**: flash suit, blue suit and spike suit (and their persistence
  options).
- **Global**: RNG, frame counters, escape timer, music and sound continuity, door
  direction.
- **Position across the doorway**: Y relative to a horizontal door frame, X
  relative to the screen for a vertical door, and the camera alignment (which
  Map Rando's camera fixes deal with).

**Foundation must offer.**

- **F.** One Session-owned state value carries across the transition. Transition
  code resets only a documented, typed list of room-scoped state (actors,
  projectiles, objects, scrolls, BG). The position and camera mapping is a pure
  function of (exit doorway, arrival doorway, carried state). Nothing in it
  depends on which room the exit led to in vanilla.
- **F.** `Session::open` accepts a *constructed arrival*: a world, a doorway and
  a typed carried state ("arrives shinecharged with N frames", "arrives in G-mode
  with x-ray held", "falling with stored speed S"). Scenarios need it to test
  strat joins between rooms that are never adjacent in vanilla, without playing a
  movie through the vanilla neighbour.

**Vanilla-exact.** All of it, in every profile, unless an option names a change.
These behaviours are not QoL.

### 14. What sm-json-data gives the scenario harness

- **Rooms**: 261 rooms, each with a stable `id`, `roomAddress`, area, sub-area and
  environment (heated), map tile mask, obstacles and enemies.
- **Nodes**: 950 nodes with a stable `id` within the room. Their type is door,
  entrance, exit, item, junction or utility. A subtype gives the door colour,
  sandpit, elevator, save, refill, map, G-mode or R-mode. A door node has
  `nodeAddress` (the vanilla exit header), orientation, locks, door environments
  (water) and twin door addresses.
- **Strats**: 14,315 strats, keyed `(room id, strat id)` (`nextStratId` keeps strat
  ids stable). Each has from and to nodes, an optional entrance and exit condition,
  requirements, obstacles cleared or reset, flags set, items collected and doors
  unlocked.
- **Notables**: 457, keyed `(room id, notable id)`.
- **Tech**: 244 entries in 10 categories, with numeric tech ids.
- **Requirements** (`logicalRequirements.md`): items and tech, and ammo, energy,
  heat/lava/acid frames, shinespark frames, `canShineCharge` with runway geometry,
  `getBlueSpeed`, enemy kills and damage, spike hits, resource capacity, refill,
  flags, obstacles, `doorUnlockedAtNode`, `itemCollectedAtNode`, flash/blue-suit
  gain and use, `autoReserveTrigger`, `resetRoom`.
- **Connections** (`connection/*.json`): the vanilla door graph by
  `(room id, node id)`.
- **Items and flags** (`items.json`): the start (Ceres Elevator Room, node 1),
  item and flag names, and item data bytes.

**What the harness needs from the foundation.**

- **F.** The world model can resolve an sm-json-data `(room id, node id)` to a
  `DoorwayId`, an `ItemLocationId` or a position. The mapping table is generated
  by the importer from a pinned sm-json-data commit and validated against our
  content (door `nodeAddress` against our compatibility keys). It is not
  committed as data (REVIEW item 9 and PLAN's data rule).
- **F.** The Session exposes typed observations for the facts a strat states:
  resources, equipment, flags, obstacle state, the current node or region, and
  the carried movement state at an exit. A scenario's end condition is then a
  set of named facts.

## Decisions this forces on the foundation

1. **Stable IDs are ours, with mappings at the edge.** `RoomId`, `DoorwayId`,
   `ItemLocationId`, `FlagId`/event enum, `StartId`, `LockId`, `PoseId` and
   `AssetId` are our own stable identities. Vanilla addresses, Map Rando
   `DoorPtrPair`s and sm-json-data `(room, node)` pairs resolve to them through
   validated mapping tables, built by `sm_tools` at import. No engine logic sees
   an address.
2. **Content is room plus placement plus topology, kept as three records.**
   *Room content* (states, level data, collision, objects, enemies, presentation
   references, doorways with their arrival data) is independent of where the room
   sits. *Placement* (map position, map area, sub-area, inclusion) and *topology*
   (doorway connections, one-way arrivals, walls, lock assignments) form the
   world. The vanilla world is one instance and must rebuild today's bytes
   exactly.
3. **A door's destination is topology, and its arrival behaviour belongs to the
   destination doorway.** Flags that depend on the connection (area change, CRE
   reload) are computed from the pair. Everything keyed by "how you came in"
   keys on the arrival `DoorwayId`. This changes today's
   `ExitDefinition`/`ArrivalDefinition`/`MatchExitTransfer`/`StartDefinition.exit`
   shapes.
4. **Two areas.** Home area (identity) and map area (placement). Each consumer is
   typed to one of them.
5. **Presentation is replaceable per room state**, with collision kept apart.
   Tileset, palette, level graphics, music cue, FX appearance and map tiles are
   media references. Their *load cost* is a gameplay input under the vanilla
   timing model.
6. **Options are three typed profiles.** `GameplayProfile` is part of the
   fingerprint; it has one field per rule change, vanilla defaults, and named
   presets ("Vanilla", "Map Rando base"). `PresentationProfile` is media-only.
   `InputProfile` lives in the app. Timing changes (fast doors, fast pause, fast
   elevators, fanfares) are gameplay fields. Original crash conditions are typed
   outcomes under a crash policy.
7. **Extensible enums with a vanilla subset.** Item kinds, equipment
   capabilities, door-cap kinds and objective kinds are enums. Vanilla save and
   observation encoding stays byte-exact for the vanilla subset.
8. **Transitions carry the whole Session.** There is an explicit, typed reset
   list. The position mapping is a pure function of the doorway pair and the
   carried state. `Session::open` accepts constructed arrivals.
9. **Seeds import from Map Rando's typed records**, not from its patched ROM:
   `settings.json` and `randomization.json` (with `map.json`) at a pinned Map
   Rando version, through a versioned `sm_tools` adapter that rejects unknown
   fields. Customization settings are imported into the presentation and input
   profiles.

## Can be added later without changing the foundation

If the decisions above hold, each of these is an ordinary rule module change,
media pack or tool:

- every optional QoL and gameplay rule in sections 9 and 10;
- beam doors, walls and hazard markers;
- the objective screen and Mother Brain barriers;
- map enhancements and room names;
- Samus sprite sheets, tile themes and palette themes;
- MSU-1 tracks;
- rolling saves, quick reload and save states;
- the Toilet overlap transform;
- escape timer computation;
- the spoiler map;
- the Map Rando seed importer itself.

## Open questions for Jim

1. **The oracle for the Map Rando profile.** Should a Map Rando-patched ROM on
   the emulator be the authority for "Map Rando base" and its options, the way
   the vanilla ROM is for vanilla? That would mean generating seeds in CI with
   Map Rando's MIT crate and a user-supplied ROM. The alternative is to accept
   our own rules plus interaction checks, with no oracle for options.
2. **Profile scope for the rebuild.** Is "Map Rando base" (complementary suits,
   gray-door rule, escape rules, entry-side fixes, `load_plms_early`, `rng_fix`) a
   foundation milestone, or does the first acceptance target only the vanilla
   world rewired, with vanilla rules? REVIEW item 9 defers "suit damage, door
   timing". This report finds that complementary suits are always on in Map Rando,
   so seeds assume them.
3. **Crash policy.** Should the vanilla profile *reproduce* original crashes and
   softlocks (spring ball menu, Yapping Maw spark, X-mode, sprite overflow,
   auto-reserve pause) as typed fatal outcomes? Or should the engine always
   recover, with the outcome listed in LEGACY?
4. **Load-time fidelity.** Does the vanilla profile model content-dependent load
   time (decompression, music upload) and the RNG ticks that happen during it?
   Map Rando's `rng_fix` and its fast-decompression compensation both show that
   players' strats observe it. It decides whether the transition module needs a
   timing model of the original loaders.
5. **ID mapping storage.** May the importer-generated mapping from sm-json-data
   and Map Rando IDs to our IDs be committed as `.rs` or `.toml` (it is
   derived from CC BY and MIT sources, and the IDs are facts), or must it be
   regenerated from pinned upstream commits at build time like ROM data?
6. **Supported Map Rando versions.** Map Rando's settings carry a `version`
   number, and its records change between releases. Should we support one pinned
   version at a time, or a range with per-version adapters?
7. **Twins and the Toilet.** Should twin rooms (Pants/East Pants, West
   Ocean/Homing Geemer) become one room with two states in our model, or stay
   two rooms in one placement unit? And is the Toilet overlap in scope for the
   first acceptance?

[mr]: https://github.com/blkerby/MapRandomizer/tree/5cb618b8a64157d4164f02575e88a340537b8c9a
[sjd]: https://github.com/vg-json-data/sm-json-data/tree/f4ae6709bea3ee15713bbfd84363ea52361f1b2a
[site]: https://maprando.com/about
