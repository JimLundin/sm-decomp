# Game data audit (#253)

Research lane `research/data-audit`, based on `origin/main` at `3b37340`. This
report changes no engine code. It answers #253 against REVIEW decision 6:
"Values the ROM provides are read from the ROM at load time. Source holds
structure (types and tables of meaning) and behaviour."

## How this was measured

- **Items.** I used the `sm_pipeline volume` lexer and its pure-data rule
  (docs/VERIFY_PIPELINE.md, "Code volume") unchanged, and added a per-item
  listing in a throwaway copy, kept out of the repository. That listing gives
  each item's path, line, name, tokens, entries, type and literal counts. Its
  totals match `sm_pipeline volume` exactly: 4,434 engine data items and
  52,837 engine data tokens (the issue quoted 52,807 at `a5ecb27`).
- **Scope.** The engine is crates/sm_data, sm_game, sm_runtime and sm_app. The
  oracle row is crates/sm_host plus sm/src outside `verification`.
  Verification directories are counted, but they are not the subject here.
  Non-pure constants (calls, casts, macros) are not data by the volume rule.
  The largest of them are all `const _: () = assert!(..)` blocks, so the
  measure leaves no large table uncounted. Apart from source files embedded
  for build hashes in sm_app, nothing uses `include_bytes!`.
- **Classification.** Every item gets one of the classes below. The 48
  largest items were checked by hand, and so was every item whose doc names a
  bank address. The others were sorted by rules on type and value: a 24-bit
  value in banks $80 and up is a ROM address; a `u16` from $8000 to $FEFF is
  a bank-local pointer; `usize` values are sizes or RAM offsets; and a numeric
  array with no stated source is a probable ROM table. Spot checks of 25
  random items per scalar class found a few misfiles. For example, some
  values under $8000 that are really PLM or instruction ids landed in "code
  immediate". None of them is a table, so they don't change the conflict list.
- **Bank logs.** I read banks $80, $82, $88, $8B, $90, $91, $94 and $A0 at
  patrickjohnston.org/bank to confirm the sources of the unattributed tables
  below.
- **No ROM.** There is no ROM here, so nothing was diffed against ROM bytes.
  The section "Checks that need the ROM" lists what CI would have to compare.

| Class | Meaning | Conflicts with decision 6? |
| --- | --- | --- |
| authored | Types, enumerations of typed paths, power-on zero records, platform data (GL names) | no |
| ram-layout | WRAM addresses, offsets and ranges (the save/Session layout) | no: structure |
| size/count | Table lengths and counts | no: structure |
| hardware | SNES PPU/DMA/DSP definitions, DSP Gaussian table, SPC700 IPL ROM | no game data, but see Q3 |
| rom-address | Where the original's code and data live: import addresses, routine and instruction-list pointers used as state identities | no for values; their raw use as state words is an AGENTS.md typing issue (#251), not #253 |
| code-immediate | Small constants the original code uses as instruction operands: speeds, timers, sound ids, thresholds | open, see Q1 |
| formula | ROM data that a stated formula reproduces exactly | yes, unless it is generated or imported |
| rom-table | A table copied from the ROM's data | **yes** |
| patch-code | 65816 bytes the oracle writes over the cartridge image | oracle only |

## Totals

Engine data tokens by class, with the item count in parentheses:

| Package | Total | rom-table | formula | rom-address | code-immediate | size/count | ram-layout | hardware | authored |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| sm_game | 44,987 | 12,694 (297) | 844 (1) | 19,339 (2,382) | 6,490 (889) | 1,512 (204) | 1,211 (70) | 736 (81) | 2,161 (74) |
| sm_runtime | 3,989 | 1,311 (12) | | | 125 (16) | 36 (5) | 7 (1) | 1,176 (3) | 1,334 (14) |
| sm_data | 3,407 | 406 (5) | | 1,672 (209) | 45 (6) | 1,157 (145) | 50 (6) | | 77 (4) |
| sm_app | 454 | | | | | | | | 454 (10) |
| **engine** | **52,837** | **14,411 (314)** | **844 (1)** | **21,011 (2,591)** | **6,660 (911)** | **2,705 (354)** | **1,268 (77)** | **1,912 (84)** | **4,026 (102)** |
| share | 100% | 27.3% | 1.6% | 39.8% | 12.6% | 5.1% | 2.4% | 3.6% | 7.6% |
| oracle (sm_host + sm/src) | 7,533 | 229 (9) | | 856 (8) | 158 (18) | 46 (6) | 361 (13) | 4,081 (12) | 285 (22) + patch-code 1,517 (3) |

How sure each part of the engine count is:

| Basis | Tokens | Share |
| --- | ---: | ---: |
| Checked by hand, or the bank log was read | 10,390 | 19.7% |
| A bank address named in the item's own doc | 1,128 | 2.1% |
| Rules with a clear signal (scalar type and value, typed paths only, platform) | 33,082 | 62.6% |
| Rule: a numeric table with no stated source, so probably from the ROM | 8,237 | 15.6% |

So 84% of engine data tokens have a direct classification, and 100% have a
class. The 15.6% "no stated source" group is the least certain. I opened
four of them against the bank logs (MANUAL_COOLDOWN, CENTISECOND_DECREMENTS,
ROOM_SHAKES, SLOPE_HEIGHTS), and all four are ROM tables. Ten tables in that
group are constants or arithmetic progressions, 325 tokens in all, such as the
Ridley inertia tables 16, 15, ..., 1. They are derivable, but they still
come from the ROM.

## SINE_COSINE

`crates/sm_game/src/projectile_geometry.rs:26`, `[i16; 320]`, 844 tokens.

- **Source.** It is the ROM's $A0:B3C3..$A0:B642: the bank log's "8-bit
  negative cosine, sign-extended" at $A0:B3C3, followed by "8-bit sine,
  sign-extended" at $A0:B443, and so on, all under "$B143: Sine/cosine
  tables". The log's first words, FF00, FF01, FF01, FF01, FF02, FF02, FF03,
  FF04, FF05, FF07, are the table's first values (-256, -255, ...).
- **Formula.** A throwaway Rust test in my scratch directory, not in the
  repository, tried candidate formulas against the 320 values in source:

  | Formula | Mismatches |
  | --- | ---: |
  | `(-256.0 * (TAU * i / 256.0).cos()).trunc()`, f64 | 0 / 320 |
  | the same in f32 | 0 / 320 |
  | `.round()` instead of `trunc` | 150 / 320 |

  So the generator is truncation, which matches the bank log's own note,
  `0x100 * math.sin(i * math.pi / 0x80)`. The closest any non-integer sample
  comes to an integer is 0.0193, far more than any libm error, so the result
  holds on any platform. The table repeats every 256 entries and has the
  usual quarter-wave symmetry.
- **What this does not prove.** It proves the values in source are formula
  values. It does not prove those values equal the ROM bytes. That needs the
  ROM check below, although the first ten values match the log.
- **The way out.** Either generate the table, or import it as
  `ImportedWords<0xa0_b3c3, 320>`, as `gameplay_tables::ENEMY_SINE` already
  does for the 16-bit sine at $A0:B1C3. `f64::cos` is not a `const fn`, so a
  generated table must be built at start-up (`LazyLock`). Importing keeps a
  modified ROM authoritative. That choice is Q2.

## Inventory

### Engine items of 44 tokens or more (133 items, 15,432 tokens, 29% of engine data tokens)

The other 4,301 engine items are mostly scalars. 4,088 items have a single
entry and hold 33,177 tokens, 63% of the total. The class totals above cover
all of them, and the scalar breakdown follows this table.

| Item | Path:line | Tokens | Entries | Class | Evidence |
| --- | --- | ---: | ---: | --- | --- |
| `SLOPE_HEIGHTS` | sm_game/src/projectile_geometry.rs:2 | 1037 | 512 | rom-table | $94:8B2B 'Slope definitions' (bank log, first row 08x16 matches), 512 bytes; content_records.rs:398 and enemy_projectiles/collision.rs:175 already read the same table from the image at 0x948b2b |
| `GAUSS` | sm_runtime/src/audio/dsp.rs:661 | 1036 | 512 | hardware | S-DSP's Gaussian interpolation ROM (console silicon, not the cartridge); 'from the retained DSP reference' |
| `SINE_COSINE` | sm_game/src/projectile_geometry.rs:26 | 844 | 320 | formula | $A0:B3C3..$A0:B642, bank log 'Sine/cosine tables' (8-bit negative cosine, sign-extended, then sine); reproduced exactly by trunc(-256*cos(2*pi*i/256)), 320/320 |
| `HDMA_PRE_ACTIONS` | sm_game/src/hdma_dispatch.rs:120 | 489 | 59 | rom-address | bank address -> typed enum dispatch map, in the original's arm order |
| `ROOM_SETTINGS` | sm_game/src/layer_blending.rs:30 | 472 | 27 | rom-table | the room layer-blending configurations (source names no address), re-encoded as (field, byte) changes after the defaults |
| `INITIAL` | sm_runtime/src/audio/driver.rs:321 | 370 | 1 | authored | power-on zero records |
| `SLIME_Y_FAST` | sm_game/src/cinematic_states.rs:66 | 342 | 138 | rom-table | $8B:A9xx-$8B:AB9x 16.16 velocity tables (doc on PARTICLE_X) |
| `SLIME_Y_SLOW` | sm_game/src/cinematic_states.rs:56 | 304 | 124 | rom-table | $8B:A9xx-$8B:AB9x 16.16 velocity tables |
| `ROOM_SHAKES` | sm_game/src/gameplay_rules.rs:61 | 300 | 144 | rom-table | $A0:872D 'BG shake displacements' (bank log; BG1X,BG1Y,BG2X,BG2Y rows match) |
| `INITIAL` | sm_runtime/src/audio/driver.rs:81 | 298 | 1 | authored | power-on zero records |
| `PROCEDURES` | sm_app/src/platform/glsl_shader.rs:316 | 281 | 42 | authored | OpenGL entry-point names (sm_app platform) |
| `CENTISECOND_DECREMENTS` | sm_game/src/escape_countdown.rs:138 | 268 | 128 | rom-table | $80:9EEC (bank log 'Decrement timer': sum 213 = 128*100/60, values match); not floor((i+1)*100/60) (first mismatch at frame 12) |
| `SFX2_INSTRUCTION_LISTS` | sm_runtime/src/audio/driver.rs:662 | 266 | 127 | rom-table | SPC program's sound library 2 pointer table |
| `SFX2_CONFIG` | sm_runtime/src/audio/driver.rs:675 | 266 | 127 | rom-table | SPC program's per-sound channel configuration |
| `GAME_STATE_ENTRIES` | sm_game/src/gameplay_frame.rs:179 | 257 | 45 | authored | typed game-state dispatch records, paths only |
| `MIRRORS` | sm_game/src/display/preparation.rs:75 | 229 | 54 | ram-layout | register mirrors in RAM order ($51..$86): typed field list |
| `COMMON_RANGES` | sm_game/src/actors/codec.rs:90 | 214 | 33 | ram-layout | RAM ranges the Session codec owns |
| `ALL` | sm_game/src/save_file.rs:79 | 205 | 48 | authored | paths only: typed enumeration or structure |
| `SELECTION` | sm_data/src/message.rs:21 | 204 | 96 | rom-table | message box selection tilemap row (BG3 tile words $0E/$28xx) |
| `HDMA_CALLS` | sm_game/src/hdma_dispatch.rs:35 | 185 | 21 | rom-address | bank address -> typed enum dispatch map |
| `PARTICLE_Y` | sm_game/src/cinematic_states.rs:47 | 176 | 82 | rom-table | $8B:A9xx-$8B:AB9x 16.16 velocity tables |
| `OWNERS` | sm_game/src/demo_clear.rs:103 | 176 | 70 | ram-layout | owner spans in address order |
| `INITIAL` | sm_runtime/src/audio/dsp.rs:44 | 160 | 1 | authored | power-on zero records |
| `INITIAL` | sm_runtime/src/audio/dsp.rs:116 | 156 | 1 | authored | power-on zero records |
| `SHELL_HEIGHT` | sm_game/src/actors/tatori.rs:62 | 150 | 48 | rom-table | $A2:8E80 (doc) |
| `INITIAL` | sm_runtime/src/audio/driver.rs:164 | 148 | 1 | authored | power-on zero records |
| `SFX1_INSTRUCTION_LISTS` | sm_runtime/src/audio/driver.rs:649 | 144 | 66 | rom-table | SPC program's sound library 1 pointer table (ARAM addresses), uploaded from the ROM's audio data |
| `SFX1_CONFIG` | sm_runtime/src/audio/driver.rs:657 | 144 | 66 | rom-table | SPC program's per-sound channel configuration |
| `ALL` | sm_game/src/menu_window_state.rs:32 | 137 | 19 | authored | enumeration |
| `ALL` | sm_game/src/controller_state.rs:111 | 125 | 28 | authored | paths only: typed enumeration or structure |
| `ALL` | sm_game/src/game_loop.rs:109 | 117 | 26 | authored | paths only: typed enumeration or structure |
| `INITIAL` | sm_runtime/src/audio/driver.rs:215 | 106 | 1 | authored | power-on zero records |
| `SFX3_INSTRUCTION_LISTS` | sm_runtime/src/audio/driver.rs:681 | 106 | 47 | rom-table | SPC program's sound library 3 pointer table |
| `SFX3_CONFIG` | sm_runtime/src/audio/driver.rs:687 | 106 | 47 | rom-table | SPC program's per-sound channel configuration |
| `DESTINATION_OFFSETS` | sm_game/src/display/hdma.rs:3 | 96 | 8 | hardware | SNES DMA/HDMA B-bus transfer patterns by mode |
| `REWARDS` | sm_game/src/drop_rules.rs:95 | 91 | 7 | rom-table | drop rewards by code, typed (Resource, amount, sound) |
| `MANUAL_COOLDOWN` | sm_game/src/beam_definitions.rs:16 | 89 | 38 | rom-table | $90:C254 'Projectile cooldowns' (bank log: 0F per uncharged beam, 1E charged) |
| `DRAINED` | sm_game/src/scene/samus_position_tiles.rs:52 | 87 | 32 | rom-table | numeric table, no stated source (likely ROM) |
| `ATTACK_STRATEGY_BY_SAMUS_MOVEMENT` | sm_game/src/actors/mother_brain_fight.rs:1084 | 86 | 28 | rom-table | numeric table, no stated source (likely ROM) |
| `CROCOMIRE_SPIKE_WALL_PIECES_TAB3` | sm_game/src/enemy_projectiles/tables.rs:10 | 85 | 36 | rom-table | numeric table, no stated source (likely ROM) |
| `DIRECTIONS` | sm_game/src/player_projectiles/lifecycle.rs:4 | 83 | 10 | rom-table | projectile direction unit vectors |
| `SLOPE_FACTORS` | sm_game/src/projectile_geometry.rs:78 | 77 | 32 | rom-table | numeric table, no stated source (likely ROM) |
| `LARGE_BORDER` | sm_data/src/message.rs:9 | 76 | 32 | rom-table | message box border tilemap row |
| `SMALL_BORDER` | sm_data/src/message.rs:15 | 76 | 32 | rom-table | message box border tilemap row |
| `SCROLL_SPEEDS` | sm_game/src/menus/map_menu.rs:961 | 76 | 32 | rom-table | numeric table, no stated source (likely ROM) |
| `RESERVE_ARROW_COLORS` | sm_game/src/menus/menu_tilemap.rs:469 | 76 | 32 | rom-table | numeric table, no stated source (likely ROM) |
| `RESERVE_ARROW_SHADOWS` | sm_game/src/menus/menu_tilemap.rs:474 | 76 | 32 | rom-table | numeric table, no stated source (likely ROM) |
| `RATES` | sm_runtime/src/audio/dsp.rs:497 | 76 | 32 | hardware | S-DSP envelope rate periods (hardware) |
| `PALETTE` | sm_game/src/actors/work_robot.rs:12 | 75 | 31 | rom-table | bank address in doc: $A8:CC67 |
| `EFFECT_LENGTH` | sm_runtime/src/audio/driver.rs:1542 | 74 | 31 | rom-table | SPC program's track-command operand lengths |
| `POSES` | sm_game/src/player_transitions/demo.rs:93 | 73 | 4 | rom-table | numeric table, no stated source (likely ROM) |
| `CEILING_PLATFORMS` | sm_game/src/actors/kraid.rs:297 | 70 | 9 | rom-table | numeric table, no stated source (likely ROM) |
| `ALL` | sm_game/src/hdma.rs:95 | 69 | 14 | authored | paths only: typed enumeration or structure |
| `GRAPPLE_BANNED_FOR_MOVEMENT_TYPE` | sm_game/src/grapple_swing.rs:811 | 68 | 28 | rom-table | numeric table, no stated source (likely ROM) |
| `MOVEMENT_TYPE_FLAGS` | sm_game/src/actors/ridley.rs:1466 | 68 | 28 | rom-table | bank address in doc: $A6:BD04 |
| `FALLING_POSE_INPUT` | sm_game/src/player_collision/motion.rs:165 | 68 | 28 | rom-table | numeric table, no stated source (likely ROM) |
| `LANDED_POSE_INPUT` | sm_game/src/player_collision/motion.rs:168 | 68 | 28 | rom-table | numeric table, no stated source (likely ROM) |
| `CROUCHING` | sm_game/src/scene/samus_position_tiles.rs:46 | 68 | 24 | rom-table | numeric table, no stated source (likely ROM) |
| `ACTIONS` | sm_game/src/player_transitions/input.rs:89 | 68 | 28 | rom-table | numeric table, no stated source (likely ROM) |
| `POSTURES` | sm_game/src/player_transitions/xray.rs:12 | 68 | 28 | rom-table | numeric table, no stated source (likely ROM) |
| `TARGETS` | sm_game/src/player_transitions/actions.rs:235 | 68 | 28 | rom-table | numeric table, no stated source (likely ROM) |
| `TWO_SLOTS` | sm_game/src/player_transitions/atmosphere.rs:51 | 68 | 28 | rom-table | numeric table, no stated source (likely ROM) |
| `FRAMES` | sm_game/src/samus_death.rs:66 | 68 | 28 | rom-table | numeric table, no stated source (likely ROM) |
| `SPECIAL_OFFSETS` | sm_game/src/message_composition/native.rs:41 | 66 | 27 | rom-table | numeric table, no stated source (likely ROM) |
| `OFFSETS` | sm_game/src/actors/ridley.rs:1824 | 64 | 20 | rom-table | bank address in doc: $A6:C66E |
| `INTERVALS` | sm_game/src/actors/bull.rs:40 | 64 | 26 | rom-table | numeric table, no stated source (likely ROM) |
| `ROWS` | sm_game/src/hud.rs:253 | 64 | 3 | rom-table | numeric table, no stated source (likely ROM) |
| `WORD_86_B65_B` | sm_game/src/enemy_projectiles/tables.rs:41 | 64 | 20 | rom-table | numeric table, no stated source (likely ROM) |
| `SPRITE_SIZES` | sm_runtime/src/rendering/mod.rs:10 | 64 | 8 | hardware | SNES OBSEL sprite sizes |
| `BEAM_SPEEDS` | sm_game/src/launch_motion.rs:41 | 61 | 24 | rom-table | numeric table, no stated source (likely ROM) |
| `WALK_X_VELOCITIES` | sm_game/src/actors/torizo.rs:106 | 60 | 20 | rom-table | bank address in doc: $AA:C4BD |
| `MESSAGE` | sm_game/src/dialogue.rs:161 | 57 | 10 | authored | paths only: typed enumeration or structure |
| `ALL` | sm_game/src/fx_state.rs:34 | 57 | 11 | authored | paths only: typed enumeration or structure |
| `VERTICES` | sm_app/src/platform/opengl.rs:91 | 56 | 20 | authored | OpenGL quad vertices (sm_app) |
| `Y_RADII` | sm_game/src/actors/fire_pillar.rs:15 | 56 | 22 | rom-table | bank address in doc: $A6:8DE7 |
| `SHADES` | sm_game/src/samus_death.rs:184 | 56 | 22 | rom-table | grey ramp; derivable (steps of 0x842 per shade) but stored as the ROM's palette words |
| `SLOT_WORD_BASES` | sm_game/src/enemy_projectiles/table.rs:12 | 56 | 22 | ram-layout | RAM address of each slot word |
| `TABLE` | sm_runtime/src/audio/driver.rs:1697 | 56 | 22 | rom-table | SPC program's N-SPC pan curve |
| `ALL` | sm_game/src/presentation_workspace.rs:30 | 53 | 10 | authored | paths only: typed enumeration or structure |
| `SLOPE_QUADRANTS` | sm_game/src/projectile_geometry.rs:23 | 53 | 20 | rom-table | numeric table, no stated source (likely ROM) |
| `FILE_MENU_MISSILE_POSITIONS` | sm_game/src/sprite_output.rs:334 | 52 | 6 | rom-table | numeric table, no stated source (likely ROM) |
| `K_EPROJ_INIT_BOMB_TORIZO_STATUE_BREAKING_XPOS` | sm_game/src/enemy_projectiles/tables.rs:29 | 50 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `ESCAPE_DOOR_PARTICLE_X_VELOCITIES` | sm_game/src/enemy_projectiles/setup.rs:660 | 50 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `STAND_UP_X` | sm_game/src/actors/torizo.rs:436 | 49 | 16 | rom-table | bank address in doc: $AA:C3EE |
| `SIT_DOWN_X` | sm_game/src/actors/torizo.rs:438 | 49 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `WORDS` | sm_game/src/display/preparation.rs:588 | 49 | 9 | authored | paths only: typed enumeration or structure |
| `CROCOMIRE_SPIKE_WALL_PIECES_TAB2` | sm_game/src/enemy_projectiles/tables.rs:2 | 49 | 18 | rom-table | numeric table, no stated source (likely ROM) |
| `CROCOMIRE_SPIKE_WALL_PIECES_TAB1` | sm_game/src/enemy_projectiles/tables.rs:6 | 49 | 18 | rom-table | numeric table, no stated source (likely ROM) |
| `OFFSETS` | sm_game/src/actors/covern.rs:62 | 48 | 18 | rom-table | bank address in doc: $A8:9AA8 |
| `CENTRED` | sm_game/src/shooting_star_motion.rs:30 | 48 | 4 | rom-table | numeric table, no stated source (likely ROM) |
| `ESCAPE_DOOR_PARTICLE_X` | sm_game/src/enemy_projectiles/setup.rs:657 | 48 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `WATER_WAVE` | sm_game/src/liquid_effects.rs:8 | 48 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `SITES` | sm_app/src/platform/glsl_shader.rs:274 | 47 | 3 | authored | assert sites (sm_app) |
| `TORIZO_ROT_TILES` | sm_game/src/actors/corpses.rs:209 | 46 | 1 | rom-table | numeric table, no stated source (likely ROM) |
| `SOUNDS` | sm_game/src/room_visuals.rs:113 | 46 | 17 | rom-table | numeric table, no stated source (likely ROM) |
| `K_EPROJ_INIT_BOMB_TORIZO_EXPLOSIVE_SWIPE_TAB0` | sm_game/src/enemy_projectiles/tables.rs:23 | 46 | 11 | rom-table | numeric table, no stated source (likely ROM) |
| `PALETTE_DELAYS` | sm_game/src/actors/covern.rs:69 | 45 | 17 | rom-table | bank address in doc: $A8:9ACC |
| `GRADUAL_ACCELERATION_DIVISORS` | sm_game/src/actors/baby_metroid_encounter.rs:116 | 45 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `PREPARE` | sm_game/src/dialogue.rs:146 | 45 | 7 | authored | paths only: typed enumeration or structure |
| `RESTORE` | sm_game/src/dialogue.rs:175 | 45 | 7 | authored | paths only: typed enumeration or structure |
| `WORDS` | sm_game/src/display_scroll.rs:15 | 45 | 8 | authored | paths only: typed enumeration or structure |
| `X` | sm_game/src/player_projectiles/feedback.rs:197 | 45 | 13 | rom-table | numeric table, no stated source (likely ROM) |
| `ALL` | sm_game/src/shooting_star_motion.rs:61 | 45 | 8 | authored | paths only: typed enumeration or structure |
| `METROID_PAL_1` | sm_game/src/title_screen.rs:44 | 45 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `METROID_PAL_2` | sm_game/src/title_screen.rs:48 | 45 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `METROID_PAL_3` | sm_game/src/title_screen.rs:52 | 45 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `METROID_PAL_4` | sm_game/src/title_screen.rs:56 | 45 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `CRATERIA_LANDING_FLAGS` | sm_game/src/player_transitions/landing.rs:95 | 45 | 17 | rom-table | numeric table, no stated source (likely ROM) |
| `BYTE_8698_B4` | sm_game/src/enemy_projectiles/tables.rs:16 | 45 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `K_EPROJ_INIT_TOURIAN_STATUE_EYE_GLOW_COLORS` | sm_game/src/enemy_projectiles/tables.rs:48 | 45 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `GUNSHIP_GREY` | sm_game/src/ending_sequence.rs:110 | 44 | 16 | rom-table | bank address in doc: $8B:DE43 |
| `WALKING_ANIMATION_DELAYS` | sm_game/src/actors/mother_brain_fight.rs:1090 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `PALETTE` | sm_game/src/actors/covern.rs:71 | 44 | 16 | rom-table | bank address in doc: $A8:99AC |
| `INERTIA` | sm_game/src/actors/ridley.rs:2822 | 44 | 16 | rom-table | bank address in doc: $A6:D61F |
| `CERES_INERTIA` | sm_game/src/actors/ridley.rs:2844 | 44 | 16 | rom-table | bank address in doc: $A6:D712 |
| `WRECKED_SHIP_PALETTE_1` | sm_game/src/actors/statues.rs:85 | 44 | 16 | rom-table | bank address in doc: $AA:E31D |
| `WRECKED_SHIP_PALETTE_2` | sm_game/src/actors/statues.rs:89 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `LOWER_NORFAIR_PALETTE_1` | sm_game/src/actors/statues.rs:93 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `LOWER_NORFAIR_PALETTE_2` | sm_game/src/actors/statues.rs:97 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `COMMON_LOW` | sm_game/src/actors/torizo.rs:242 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `COMMON_HIGH` | sm_game/src/actors/torizo.rs:246 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `BOMB_START_LOW` | sm_game/src/actors/torizo.rs:250 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `BOMB_START_HIGH` | sm_game/src/actors/torizo.rs:254 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `BOMB_ACTIVE_LOW` | sm_game/src/actors/torizo.rs:258 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `BOMB_ACTIVE_HIGH` | sm_game/src/actors/torizo.rs:262 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `GOLDEN_START_LOW` | sm_game/src/actors/torizo.rs:266 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `GOLDEN_START_HIGH` | sm_game/src/actors/torizo.rs:270 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `GOLDEN_ACTIVE_LOW` | sm_game/src/actors/torizo.rs:274 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `GOLDEN_ACTIVE_HIGH` | sm_game/src/actors/torizo.rs:278 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `PARTIAL` | sm_game/src/menus/menu_tilemap.rs:805 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `CODE` | sm_game/src/menus/options_settings.rs:271 | 44 | 16 | rom-table | numeric table, no stated source (likely ROM) |
| `NMI_FIELDS` | sm_game/src/display/preparation.rs:600 | 44 | 8 | authored | paths only: typed enumeration or structure |
| `VOLUME` | sm_runtime/src/audio/driver.rs:2362 | 44 | 16 | rom-table | SPC program's N-SPC note velocity table |

### Engine scalars and small items, by class

| Class | Items | Tokens | Examples |
| --- | ---: | ---: | --- |
| rom-address: bank-local `u16` pointers (instruction lists, routines, PLMs) | 1,489 | 11,626 | `spore_spawn::DESCENDED = 0xe6d5`, `botwoon::MOVING_AROUND = 0x9dc0` |
| rom-address: 24-bit bank addresses | 1,098 | 8,672 | `enemy_projectiles/addresses.rs` (`TORIZO_SONIC_BOOM = 0x86_ae6c`), and in sm_data every import address (`KRAID_NAIL_UPWARD_VELOCITIES = 0xa7_be3e`) |
| code-immediate | 911 | 6,660 | `etecoon::LOOK_DISTANCE = 0x40`, `torizo::EGG_RELEASED_SOUND = 0x34`, `title_sequence::TITLE_THEME = 5` |
| size/count | 354 | 2,705 | `gameplay_tables::ANGLES = 256`, `ceres_doors::DOOR_KINDS = 7` |
| authored (non-numeric or single typed records) | 67 | 1,884 | the audio `INITIAL` zero records |
| hardware | 80 | 640 | `display/registers.rs` (`CGADD = 0x2121`), DSP register indices |
| ram-layout | 71 | 559 | `display/preparation.rs::HUD_TILES = 0xc608` |
| rom-table (one entry, several literals) | 18 | 431 | small structs of ROM values |

The 1,672 rom-address tokens in sm_data are the import mechanism itself, which
is what decision 6 asks for.

### Oracle items of 60 tokens or more (sm_host and sm/src)

| Item | Path:line | Tokens | Entries | Class | Evidence |
| --- | --- | ---: | ---: | --- | --- |
| `BEFORE_AUDIO` | sm/src/rom_patches.rs:8 | 1306 | 78 | patch-code | 65816 patch bytes the port writes over the ROM (JSR/JMP hooks into free space, CLC carry hooks); port-authored, partly re-encoding relocated original instructions |
| `OPCODE_NAMES` | sm/src/tracing.rs:7 | 525 | 256 | hardware | 65816 mnemonics (CPU spec) |
| `SPC_OPCODE_NAMES` | sm/src/tracing.rs:84 | 525 | 256 | hardware | SPC700 mnemonics (CPU spec) |
| `OPCODE_TYPES` | sm/src/tracing.rs:73 | 524 | 256 | hardware | 65816 operand kinds |
| `SPC_OPCODE_TYPES` | sm/src/tracing.rs:150 | 524 | 256 | hardware | SPC700 operand kinds |
| `CYCLES_PER_OPCODE` | sm/src/snes/spc.rs:18 | 524 | 256 | hardware | CPU cycle counts |
| `CYCLES_PER_OPCODE` | sm/src/snes/cpu.rs:7 | 524 | 256 | hardware | CPU cycle counts |
| `PATCHED_CARRIES` | sm/src/cpu_infra.rs:130 | 326 | 155 | rom-address | patch-site bank addresses |
| `LAYERS` | sm_host/src/stubs/ppu_ports.rs:346 | 276 | 10 | hardware | PPU layers by BG mode |
| `PRIORITIES` | sm_host/src/stubs/ppu_ports.rs:358 | 276 | 10 | hardware | PPU layer priorities by BG mode |
| `AFTER_AUDIO` | sm/src/rom_patches.rs:135 | 172 | 9 | patch-code | 65816 patch bytes, as BEFORE_AUDIO |
| `LOCK_WRAM_MASK` | sm_host/src/lock_masks.rs:39 | 147 | 31 | ram-layout | RAM ranges |
| `FOLLOWS_CARRY` | sm_host/src/oracle_patch_policy.rs:36 | 140 | 62 | rom-address | patch-site bank addresses |
| `BOOT_ROM` | sm/src/snes/apu.rs:14 | 140 | 64 | hardware | the SPC700 IPL boot ROM (64 bytes of console firmware, not cartridge data) |
| `LOCK_SESSION_MASK` | sm_host/src/lock_masks.rs:6 | 123 | 25 | ram-layout | RAM ranges |
| `BIT_DEPTHS` | sm_host/src/stubs/ppu_ports.rs:370 | 116 | 10 | hardware | PPU bit depths by BG mode |
| `KEPT_ROM_PATCHES` | sm_host/src/oracle_patch_policy.rs:15 | 106 | 45 | rom-address | patch-site bank addresses |
| `SITES_BUG_HOOKS` | sm_host/src/oracle_patch_policy.rs:65 | 106 | 45 | rom-address | patch-site bank addresses |
| `B_BUS_OFFSETS` | sm_host/src/snes/dma.rs:11 | 96 | 8 | hardware | DMA B-bus patterns |
| `FOLLOWS_ROM_PATCHES` | sm_host/src/oracle_patch_policy.rs:57 | 82 | 33 | rom-address | patch-site bank addresses |
| `FOLLOWS_BUG_HOOKS` | sm_host/src/oracle_patch_policy.rs:47 | 70 | 27 | rom-address | patch-site bank addresses |

`BEFORE_AUDIO` (#253's example) is not ROM content. It is 65816 code the
oracle writes over the cartridge: JSR/JMP hooks into free space and runs of
`CLC` (0x18) at carry hooks. It came from the port the oracle descends from.
Some sequences re-encode relocated original instructions; for example
$82:F713 calls the original $80:9459, $82:8948 and $80:8338. It belongs to the
oracle and leaves with it. `BOOT_ROM` in `sm/src/snes/apu.rs:14` is the SPC700
IPL ROM: 64 bytes of console firmware, not game data, but it is copied
firmware (Q3).

## Items that conflict with decision 6

These are the 314 rom-table items and the one formula item in the engine:
15,255 tokens, 28.9% of engine data tokens, in 97 files. By file:

| File | Items | Tokens |
| --- | ---: | ---: |
| sm_game/src/projectile_geometry.rs | 4 | 2011 |
| sm_game/src/enemy_projectiles/tables.rs | 46 | 1527 |
| sm_runtime/src/audio/driver.rs | 12 | 1311 |
| sm_game/src/cinematic_states.rs | 6 | 915 |
| sm_game/src/actors/torizo.rs | 25 | 818 |
| sm_game/src/layer_blending.rs | 1 | 472 |
| sm_game/src/actors/mother_brain_fight.rs | 14 | 458 |
| sm_game/src/actors/ridley.rs | 14 | 439 |
| sm_data/src/message.rs | 4 | 382 |
| sm_game/src/gameplay_rules.rs | 1 | 300 |
| sm_game/src/escape_countdown.rs | 1 | 268 |
| sm_game/src/menus/menu_tilemap.rs | 5 | 236 |
| sm_game/src/scene/samus_position_tiles.rs | 3 | 198 |
| sm_game/src/menus/map_menu.rs | 6 | 196 |
| sm_game/src/player_projectiles/motion.rs | 7 | 184 |
| sm_game/src/title_screen.rs | 4 | 180 |
| sm_game/src/beam_definitions.rs | 4 | 179 |
| sm_game/src/actors/statues.rs | 4 | 176 |
| sm_game/src/actors/tatori.rs | 2 | 169 |
| sm_game/src/samus_death.rs | 4 | 166 |
| sm_game/src/player_projectiles/feedback.rs | 4 | 161 |
| sm_game/src/actors/corpses.rs | 4 | 158 |
| sm_game/src/grapple_swing.rs | 3 | 154 |
| sm_game/src/player_projectiles/placement.rs | 4 | 144 |
| sm_game/src/hud.rs | 4 | 142 |

The groups:

- **Collision and geometry.** `SLOPE_HEIGHTS` ($94:8B2B), `SLOPE_FACTORS`,
  `SLOPE_QUADRANTS` and `SINE_COSINE`. `SLOPE_HEIGHTS` is already read from
  the image at 0x948b2b in `actors/content_records.rs:398` and
  `enemy_projectiles/collision.rs:175`, so the literal duplicates a ROM read.
- **Enemy projectile setup.** `enemy_projectiles/tables.rs` holds 46 tables;
  its doc says "retained byte-for-byte". Their names carry their addresses
  (`BYTE_8698_B4`, `WORD_86_B65_B`).
- **Sound driver.** In sm_runtime, the SPC program's sound-library pointer
  and configuration tables, the N-SPC pitch, velocity and pan tables, and the
  command lengths. The game uploads all of these from the ROM into APU RAM.
- **Palettes.** Torizo, chozo statues, title metroid, gunship grey, Covern,
  death shades, reserve arrows. Many docs say "as the original's copy of
  $xx:xxxx".
- **Motion and boss tables.** Velocities, offsets, positions and timers in
  the actor modules, cinematic slime and particle velocities ($8B:A9xx), and
  the per-movement-type tables (28 entries, one per Samus movement type).
- **Room and screen tables.** `ROOM_SHAKES` ($A0:872D), `ROOM_SETTINGS`
  (layer blending, re-encoded as typed changes), `CENTISECOND_DECREMENTS`
  ($80:9EEC), and the message box border and selection rows in sm_data.

### How each can move behind `sm_data` import

The migration already has a pattern: `kraid_tables.rs` with `Imported<T>`,
`gameplay_tables.rs` with `ImportedWords<ADDRESS, ROWS>`, and the
`sm_data/tests/rom_import.rs` check.

1. **Plain word or byte tables**, most of the 314 items. In sm_data, add an
   address constant and a length (`pub const X: u32 = 0xbb_aaaa; pub const
   X_ROWS: usize = N;`). Add an `ImportedWords<X, X_ROWS>` field (or a byte
   equivalent) to the owning `Imported*` struct, filled from its
   `import(image)`. Read through the accessor, which falls back to the
   original indexed load on any other image. Add a `check!` in
   `rom_import.rs`, then delete the literal. Source keeps the address, the
   length and the meaning, as decision 6 says.
2. **Typed records**: `REWARDS`, `CEILING_PLATFORMS`, `ROOM_SETTINGS`, the
   `(x, y)` pairs. Add a typed importer in sm_data
   (`import_x(&[u8]) -> Result<T, _>`, as `sm_data::kraid` does) and wrap
   it in `Imported<T>`. Where the engine stores a re-encoding
   (`ROOM_SETTINGS`), the importer produces the re-encoding from the ROM
   bytes.
3. **Tables the engine already reads from the ROM**: `SLOPE_HEIGHTS`. Delete
   the literal and send the remaining callers through one import.
4. **`SINE_COSINE`**: import (`ImportedWords<0xa0_b3c3, 320>`) or generate
   (Q2).
5. **Sound driver tables.** sm_runtime already depends on sm_data. The tables
   live in the SPC program the game uploads, so the driver can read them from
   its `ApuRam` at their SPC addresses after the upload, or import them from
   the ROM's audio block at prepare time. Their ARAM addresses aren't written
   in source yet, so pinning them down needs the ROM.
6. **Derivable progressions** (10 tables, 325 tokens, such as the inertia
   tables 16..1). Import them like the rest, or write the formula with a
   `rom_import` check that it equals the ROM. Both satisfy decision 6.

On order: `enemy_projectiles/tables.rs`, `projectile_geometry.rs` and
`driver.rs` hold 4,849 tokens between them, a third of the conflict.
Palettes are the other easy group.

## Checks that need the ROM

These need `SM_ROM`, which only CI has from the encrypted ci-data. This lane
triggered no CI.

- Before deleting each literal, assert that it equals the bytes at its
  address. A one-off `rom_import.rs`-style test over the conflict list does
  this, and it also confirms the 253 "no stated source" items and finds
  their addresses.
- `SINE_COSINE` against the 320 words at $A0:B3C3. The formula already
  matches the source values exactly.
- The sound tables against APU RAM after the boot upload, which also gives
  their ARAM addresses.
- `GAUSS` is a hardware table, not cartridge data. A ROM can't check it. The
  check is the emulator's DSP, which is where it came from.

## Questions for Jim

1. **Code immediates.** 911 scalars, 6,660 tokens: speeds, timers, sound ids
   and thresholds that the original code uses as instruction operands. They
   are bytes in the ROM, but they are part of the code's behaviour. Does
   decision 6 cover only data tables, which is my reading, or immediates too?
2. **Exact formulas.** For `SINE_COSINE` and the derivable progressions,
   should a table that an exact formula reproduces be generated, with a ROM
   check, or imported? Importing keeps a modified ROM authoritative. That
   matters if Map Rando or other hacks change these tables, though none is
   known to change the sine table.
3. **Hardware tables.** Three are copied from console silicon, not the
   cartridge: `GAUSS` (S-DSP), `RATES` (DSP envelope periods) and the oracle's
   `BOOT_ROM` (SPC700 IPL). Should the README name them? The finished engine
   still needs `GAUSS` and `RATES` to be byte-exact.
4. **README wording.** Lines 33 and 129 say "No ROM or game data is
   included". A wording that is true today and stays true as the import
   proceeds:

   > This repository includes no ROM. The engine reads the game's content,
   > such as graphics, sound, rooms and the original's data tables, from your
   > ROM when it loads. The source holds the engine's types and behaviour,
   > the ROM addresses it reads from, and the constants the original code
   > uses directly. Some data tables are still copied in source while they
   > move to that import; docs/handoff/game-data-audit.md lists them. The
   > emulator used as the oracle contains the SNES sound CPU's 64-byte boot
   > program.

   Should the README name the remaining copies, or just say they are being
   moved?
5. **The rule conflict #253 names.** REVIEW's Nov 4 data-driven-boss ruling
   ("typed and stored in `.rs` or `.py` modules") and PLAN line 70 ("never
   commit ... derived game data") agree if the boss ruling's data means the
   state machines (structure and behaviour), while the boss's ROM values come
   in through import. Should that sentence go into REVIEW decision 6 or
   PLAN's standing rules?

## Reproducing

The item listing is the `sm_pipeline volume` lexer from
`crates/sm_tools/src/bin/sm_pipeline/volume.rs`, with one extra function that
prints each data item's path, line, name, tokens, entries, type and literal
counts. A short Python classifier applies the rules above. Both are throwaway
and not committed. Making the listing a `sm_pipeline volume --items` option
is a small change, if the audit should be repeatable.
