# Domain plan: the rest of Super Metroid as clean-room domains

Planner, 2026-11-08, read-only, at spine `e458a3d5d`. Sizes are `tools size` weighted scores
(the wire-cam `size.dll`, the same counter as the specs' baselines). Raw per-file output:
`$TEMP/collab/reviews/sz_all.txt` (today's engine) and `sz_old.txt` (the whole-game port at
`archive/old-engine`, `crates/sm_game/src`, verification and tests excluded); per-module sums
`mods.txt` and `oldmods.txt`.

## 0. The answer in three lines

- Done: about 5% of the game's behaviour (samus-movement 7,160 + camera 1,506 weighted, about
  250 spec units). The remaining clean model is likely 120-170k weighted, about 3,500-4,500
  spec units at k = 36.
- That is about 140-150 clean-room pipeline rounds (spec, adversary, implement, wire, gaps)
  plus 30-40 platform lanes (encoders, room loading, timing, renderer, audio backend, tooling).
  With 4-6 lanes in parallel at about 2.5 h a round it is 70-100 h of pipeline wall-clock in
  the best case. With review, Jim's gates and one-at-a-time landing, expect several weeks.
- The order is toward the headline milestone (MODEL §1 decision 14), the engine replaying a
  whole movie alone, first power-on through Ceres (§4): finish Samus and the world she touches
  on the 32 cached rows; then doors, rooms and the fight (so the engine runs past slice 0 and
  through Ceres's rooms); then Ceres itself and the opening around it (Ceres Ridley and the
  escape, the title, intro and file select, the cinematics, audio, Mode 7, slowdown and loading
  time). Then the movie's next stretches: items, enemies and bosses; the rest last.

## 1. The engine today

The engine runs **slice 0 only**: the rooms `data::room` loads for `area-brinstar` (the
elevator ride into Brinstar and a short walk) and `item-morph-ball` (the Morph Ball Room, with
its item, message box and about 370 lag Ticks). Free-run gate green pairs: area-brinstar
vram/cgram/oam/sram/ppu/timing, item-morph-ball sram/timing. Lockstep: area-brinstar 476/476,
item-morph-ball 1372/1427 Ticks equal (54 video, 1 each elevator/projectiles/room). No doors,
no second room in a run. `app` is an empty `main` (2.5). The **domain harnesses** (game-time,
samus-movement, camera) run on oracle frames of any room, so the clean domains are proven far
beyond the slice (32 rows, 18,331 frames), but only per frame, re-seeded.

Totals: `game` 36,856, `data` 25,412, `runtime` 8,351, clean models 8,666, `types` 467.

### `crates/game`

The modules as they are now. The scores above are the plan's, at its spine; the engine's size
today, as code still to be rebuilt, bridges and rebuilt models, is in
[the rebuild report](../REPORT.md#engine-not-yet-rebuilt).

| Module | What it does, what it covers |
| --- | --- |
| `state` | the State schema: samus, room (D4 reset struct), slots (D12, cursor D13), strand, video (D5), sram, hud, projectile, atmospheric |
| `tick`, `session`, `nmi` | the main strand's dispatcher in the original's order; the Session and Tick begin and end; the NMI's commit or lag, pad latch and counters |
| `intent` | the per-Tick intents: `SpriteDraw`, `TileWrite` (overlay rows, block rows), `LayerExposure` |
| `exact` (`oam`, `tilemap`, `ppu`) | the encoder from intents to the original's bytes: the OAM buffer; the VRAM write queue, the scrolling DMAs and the message box's direct overlay writes; the message box's PPU set-up |
| `commit` | the NMI's video commit points |
| `lines` | the scan-out skeleton (HUD split, HDMA not yet) |
| `samus_movement_bridge`, `world_camera_bridge`, `world_blocks_bridge`, `interface_hud_bridge`, `run_progress_bridge` (bridges) | each clean model's contact with the engine's State |
| `plm` (+draw, item_gfx) | spawning a room's PLMs, the items' triggers and the PLM pass; the PLM draw as `TileWrite` block rows; item GFX load. Slice 0's PLMs only |
| `enemy` (+drawing, eye, face_block, instructions) | processing window and per-slot bookkeeping, the instruction-list runner, drawing by layer; species: the elevator, the Morph Ball Room's eye, the Blue Brinstar face block |
| `projectile` | Samus's projectiles: cooldown, the power beam's flight and block checks, instruction lists. No other beam, missiles, bombs |
| `message_box` (+video) | the item pickup box as a strand frame with its lag waits; its rows as overlay writes; its BG3 scroll HDMA channel |
| `scrolling` | the BG scrolls follow the camera, and the block lines they bring into view (`LayerExposure`) |
| `bg_graphics` | the level lookup: each exposed or changed block's look |
| `collision` (+inside, solid_enemy) | the room she meets; block-inside reactions; solid enemies |
| `pose`, `animation` | what's left of the pose input and change outside the model; her animation delay lists |
| `samus_drawing`, `samus_palette` | her clip's frame into the sprite buffer; the visor |
| `atmospheric` | splashes, bubbles, dust (4 slots) |
| `game_time`, `rng`, `model` | play time; the RNG word; the pixel and subpixel numbers motion counts in |

No sound (only CPU-side hooks named), no doors, no pause, no front end, no damage, no HDMA.

### `crates/data` (25,412)

`assets` 6,431 (Samus's sheet and clips importer, decision 4), `pose` 4,381, `enemy` 3,825
(headers and instruction-list decode), `room` 2,022 and `level` 1,915 (rooms of the slice,
level data, slopes), `map` 1,043, `rom` 998, `span` 947, `plm` 798, `compression` 785,
`message` 688, `tileset` 544, `spritemap` 374, `digest` 191, the rest under 110. The decoders
are general where cheap (compression, level, spritemaps); the content loaded is slice 0's.

### `crates/runtime` (8,351)

The line compositor (D17): backgrounds (mode 1, mode 7 code present), sprites, colour math,
windows; between-line changes only. No sound backend, no storage.

### The clean models (8,666)

`clean/samus_movement` 7,160 (+8,118 data), `clean/world_camera` 1,506 (+535 data).

## 2. What the real game has that the engine lacks

Sized by the whole-game port at `archive/old-engine` (`sm_game`, 1.04 M weighted, verbose and
partly transpiled; it is an upper bound). Calibration: samus-movement's clean model is about 0.13
of that port's player movement, collision and transitions (57k), the camera 0.19 of its room
scrolls (7.8k). So **clean ≈ 0.12-0.2 × old port**. Banks from the bank-log index.

| System | Banks | Old port | Clean estimate | In the engine |
| --- | --- | ---: | ---: | --- |
| Bosses (Spore Spawn, Torizos, Kraid, Crocomire, Phantoon, Botwoon, Draygon, Ridley x2, Mother Brain, baby metroid, zebetites, statues) | A4-A7, A9, AA, B3, AD | ~175k | 25-35k | none |
| Enemy projectiles (most belong to bosses) | 86, 8D | 65k | 8-13k | none |
| Enemy species (~170) and the framework (spawn, process, contact, vulnerability, freeze, death, drops, corpses) | A0-A3, A8, B2-B4 | ~150k + 35k | 25-35k | 3 species, the framework partly |
| Front end and menus (boot, title, intro, file select, options, pause map and equipment, game over, demos) | 81, 82, 8B, 8C, 8E, B5, B6 | ~110k (menus 49k, scene part) | 15-22k | none |
| Cinematics (Ceres, the escape, gunship, ending, credits, Mode 7) | 8B, 8C, 95-99 | ~50k | 8-10k | none |
| Doors, elevators and room loading | 82, 83, 8F | ~40k | 5-8k | none (elevator ride only) |
| Block reactions and PLMs | 84, 94 | ~30k | 5-6k | slice 0's |
| Samus's beams and weapons, power bomb | 90, 93, 94, 88 | ~37k | 5-7k | power beam |
| Room FX (liquids' levels, rain, fog, heat, animated tiles, palette FX, HDMA objects, x-ray cone, sky scroll) | 87, 88, 89, 8A, 8D | ~48k | 7-9k | none |
| Samus remaining (liquids, booster, shinespark, grapple, knockback, damage, death, palette, look) | 90, 91, 94, 9B | ~30k | 5-7k | animation, drawing, visor |
| HUD, message boxes, dialogue | 80, 85 | ~15k | 2-3k | slice 0's |
| Saves, progress, inventory | 81, 82 | ~9k | 1.5k | SRAM image |
| Audio (CPU side; the old port emulated the SPC) | 80, CF-DF, the SPC driver | 3k CPU side | CPU 1k; driver a separate question | none |
| Slowdown and loading time (the cost model, D9) | all | n/a | table-driven | recorded NMI timing |

Infrastructure in the old port (encoding, checkpoints, owners, RAM views, ~110k) is not
behaviour and has no counterpart here.

## 3. The domains

Rules used to cut them (MODEL §4): a domain is verifiable alone. Two corollaries that keep the
list from collapsing into one domain:

- **Modes partition frames.** When one domain hands Samus over to another for whole frames
  (grapple swing, shinespark, elevator, cutscene control), each is verified on its own frames
  and the other masks them by a named reason that doesn't depend on the outcome.
- **Another domain's output may be an input** read from the record when it doesn't depend on
  this domain's output in the same frame (the camera reads her position after her moves).
  Where it does (a contact this frame changes her motion this frame) the two merge or the
  boundary moves to an event the earlier one exports.

"Rows" says whether the 32 cached rows (`$TEMP/collab/more-caches/cache`) exercise it,
or which rows of `scenarios/locks/lsnes.toml` (635 rows, 445,583 frames, 520 chain rows
over 197 rooms of the 100% and any% routes) must be cached. "New" means no lock row covers it.
Rounds are pipeline rounds (observer spec + adversary + implementer + integrator + gap answer).

### Samus

| # | Domain (player terms) | Scope | Reads / exports | Old code, baseline | Rows | Depends | Rounds |
| --- | --- | --- | --- | --- | --- | --- | ---: |
| S1 | **Samus's motion: follow-ups** | export `rising`; C2 (`camera_from` under a block-inside push); C3 (chain-0059-A293 400) | exports `rising` to the camera | none (fixes) | cached | none | 1 |
| S2 | **Samus in water, lava and sand; pushes** (samus-movement rounds) | other speeds and gravity in liquids, the speed booster's dash cap and higher jump (the booster's stages are S5's), quicksand, conveyors, Ceres pushes, the steep blocks' corners | reads the liquid's level (R3) as an input; same speed quantities, so it extends samus-movement | in `samus_movement_bridge`/model already; old port movement | cached (chain-0220/0224/0232, area-maridia, terrain-quicksand, samus-ceres-push) | S1 | 4 |
| S3 | **The shinespark and echoes** (samus-movement rounds) | charge store, the spark's directions, its end, the echoes' positions | exports the echoes to S5 | none | cached (samus-shinespark-*) | S2 | 2 |
| S4 | **Hurt and knockback; bomb jumps** (samus-movement rounds) | knockback motion and its turn of the camera lead (camera C4), bomb-jump launch | reads a hit event (C1) and a bomb blast (W1) | none | cached (chain-0106, samus-bomb-jump-vertical) | S2 | 2 |
| S5 | **How Samus looks** | animation timing (`AnimationEnds` becomes an output; the run's pace and the speed booster's stages up to boosting, the clock the shinespark, the echoes, the glow, the blocks and the enemies read), her sprite (clips), palette (suits, visor, charge flash, screw attack, speed booster glow, hurt flash), atmospheric splashes and dust, the i-frame flicker | reads her pose; exports `SpriteDraw`, palette intents; samus-movement reads its "animation ended" | `animation` 1,246, `samus_drawing` 605, `samus_palette` 183, `atmospheric` 1,082 (3,116) | cached, OAM and CGRAM streams | E1 | 3 |
| S6 | **The grapple beam** | aim, fire, latch, swing (angle, length), release hand-off, grapple blocks | owns her position on attached frames; reads grapple block kinds; exports release speeds | none (old port 11k) | cached (samus-grapple-*); boss-draygon-grapple-* | S2, W2 | 3 |
| S7 | **The x-ray scope** | the cone, its sweep, time stopped (camera's `TimeStopped`), revealed blocks | exports time-stopped and the cone to R4 | none | cache `xray-scan-*` (4 rows) | E2 | 2 |

### Her weapons and the world she touches

| # | Domain | Scope | Reads / exports | Old code, baseline | Rows | Depends | Rounds |
| --- | --- | --- | --- | --- | --- | --- | ---: |
| W1 | **Beams, missiles and bombs** | beam combinations, charge, cooldowns, flight shapes (wave, spazer, plasma, ice), missiles, supers, bombs and their blasts, power bombs and their expanding window, crystal flash | reads equipment, her pose; exports projectile hits on blocks (W2) and enemies (C1), blasts (S4) | `projectile` 2,551 | cache `special-beam-*`, `beam-reflect`, `power-bomb-*`, `samus-crystal-flash`; chains | E1 | 5 |
| W2 | **Blocks that react** | shot, bomb, power bomb, super and speed blocks, crumble, respawn, grapple blocks, spikes (as a hurt event), the scroll cells that open and close screens (camera round 2), door caps as blocks | reads her touched cells (samus-movement `TouchedCells`) and W1's hits; writes the level's blocks; needs D13's `Program<F>` with PLM lists as imported assets | `plm` 4,705 (part), `collision/inside` | cached chains (crumble, scroll PLMs); more chain rows | A1 | 5 |
| W3 | **Items to pick up** | item blocks, chozo balls, hidden items, the collected bit, what each item gives, reserve tanks' auto refill | reads W2's reach; exports the message (H2) and equipment | `plm` item path, `ItemStep` | cached (item-morph-ball); cache `item-wave`, chain rows with items | W2 | 2 |

### Rooms

| # | Domain | Scope | Reads / exports | Old code | Rows | Depends | Rounds |
| --- | --- | --- | --- | --- | --- | --- | ---: |
| R1 | **Doors and elevators between rooms** | the door's passage (the screen scrolling over, the fade, the new room's placement of Samus and the view, the camera's `ScreensRunOn`), elevators between areas, the room state chosen by events (P1), loading time as a Timing behaviour | reads the door it entered; exports the new room, her and the view's placement, the spawn calls to each family | none in the clean sense; `state/room` reset | cached chains (each crosses a door), `door-transition`, `door-loading-lag-nmi`, `elevator-traversal` | E2, P1 | 6 |
| R2 | **Room loading** (platform, not clean-room) | room headers, level decompression, tileset and palette upload, the population lists, as imported assets; the VRAM upload sequence and its cost | ROM → assets; outputs video writes | `data::room`, `level`, `compression` | as R1 | E2 | 4 lanes |
| R3 | **Liquids and room weather** | the liquid's level, its rise and fall (Tourian acid, lava quakes), rain, fog, heat, the room's quakes and the room code that moves the view (camera's "room rules") | exports the liquid level (S2), the view's shake | none | cache `area-maridia`, `volcano-room-lavaquake`, `terrain-*-hand`, chains | R1 | 4 |
| R4 | **Room ambience** | animated tiles, palette cycles (palette FX objects), the sky's scroll, layer 2's own motion, HDMA visuals | output only (tile, palette, line intents) | `scrolling` 252, `bg_graphics` (part) | chains | E2, E3 | 3 |

### Combat and enemies

| # | Domain | Scope | Reads / exports | Old code | Rows | Depends | Rounds |
| --- | --- | --- | --- | --- | --- | --- | ---: |
| C1 | **Fighting enemies** (the framework) | which enemies run (the window), contact with her (hurt, solid), with her projectiles (hit, vulnerability table, freeze with ice, frozen as platform), death explosion, drops (drop chance and RNG), corpses, respawn | reads enemy hitboxes and positions (species), W1's projectiles; exports hits to H1 and S4 and species | `enemy` 4,567 (part), `collision/solid_enemy` | cached chains (zoomers, wavers, fish); cache `enemy-*` (19 rows) | W1, A1, A2 | 6 |
| H1 | **Energy, reserves and death** | energy, tanks, reserves (auto and manual), suit damage reduction, i-frames, low-energy alarm, death and its sequence, game over hand-off | reads hits (C1, W2 spikes, R3 lava); exports values to H2, the death to F3 | none | cached (chain-0106 hits); cache `death`, `samus-square-slope-reserve` | C1 | 2 |
| C2-C11 | **Enemy families**, about 10 domains by movement shape: crawlers (zoomer, geemer, wall crawlers), hoppers (sidehopper, dessgeega), fliers (waver, skree, mellow, fireflea), swimmers (Maridia fish, skultera, choot), turrets and spawners (pipes, rinka, cannons), space pirates and ki-hunters, metroids (latch, drain, ice shatter), friendly animals (etecoons, dachora), Wrecked Ship robots and shutters, room hazards (boulders, magdollite, lava-dwellers) | each its AI and its own projectiles; reads her position, the level | species `eye`, `face_block` | cache `enemy-*`, `coverage-metroid-*`, chain rows per area | C1, A1 | 25-30 total |

### Bosses (D14 phase tables)

| # | Domain | Rounds | Rows |
| --- | --- | ---: | --- |
| B0 | **Boss phase tables**, designed with Spore Spawn and Bomb Torizo together (D14) | in B1+B2 | |
| B1 | Spore Spawn | 2 | `boss-spore-spawn`, `spore-spawn-parameter` |
| B2 | Bomb Torizo, Golden Torizo | 3 | chain rows |
| B3 | Botwoon | 2 | `boss-botwoon`, `coverage-botwoon-spit` |
| B4 | Crocomire | 4 | chain rows, `scenario-one-energy-crocomire` (2) |
| B5 | Kraid (and its room keeping her on its first screen; mini Kraid) | 4 | chain rows |
| B6 | Phantoon | 4 | `boss-phantoon*`, `phantoon-exit-spawn-parameter` |
| B7 | Draygon | 3 | `boss-draygon*` (8 rows) |
| B8 | Ceres Ridley and the Ceres escape | 4 | `boss-ceres-ridley*`, `area-ceres`, `chain-any-0006-E0B5` to `chain-any-0011-DF45` |
| B9 | Ridley | 5 | `boss-ridley*` |
| B10 | Mother Brain, the glass, zebetites, rinkas, the baby metroid, the escape | 8 | `boss-mother-brain`, `mother-brain-*` (7), `enemy-baby-metroid-remorse` |

Each boss reads her position, her projectiles' hits (C1) and the camera; exports its parts'
positions, its hits, its effects (shakes, palette, the camera's lead setting).

### HUD, menus, saves, progress

| # | Domain | Scope | Old code | Rows | Depends | Rounds |
| --- | --- | --- | --- | --- | --- | ---: |
| H2 | **The HUD and the mini-map** | what the HUD shows (values, item selection and its cycling, auto reserve sign), the mini-map and exploration | `hud` 1,058, `mini_map` 1,146 | cached | E2 | 2 |
| H3 | **Message boxes** | item texts, the save prompt's yes and no, their lag waits (C5's waits) | `message_box` 1,925 | cached (item-morph-ball, samus-square-slope-reserve) | E2 | 2 |
| P1 | **Progress** | events, boss flags, collected items, map stations, the percentage; which room state a room loads | none | chains | none | 1 |
| P2 | **Saving and loading; stations** | save stations, the SRAM file and checksums, the load flow, energy and missile refill stations, map stations | `state/sram` | cache `save-memory`, `samus-gunship-save`, `file-copy-clear` | P1, H3 | 3 |
| F1 | **The pause screen** | map screen (scrolling, areas, explored), equipment screen (toggles), its fades and the unpause | none (old 49k with file select) | **new recording** (no `pause-menu-tour` in the lock; the 100% chains may pause for toggles: check) | P1, E2 | 5 |
| F2 | **Title, intro and file select; options** | boot, title, the intro's text and pages, file select (copy, clear), options and controls, demos (a recorded input source) | none | `options-menu-tour`, `file-copy-clear`, `any-power-on`, the intro's `any-intro-*` (four rows); **new** for the attract demos | E2, E4 | 8 |
| F3 | **Game over** | death's screen, continue | none | `death` | H1 | 1 |
| F4 | **Cinematics and credits** | the gunship's landing and take-off, Samus's arrival, the planet's explosion, the ending, credits | none | `gunship-bobbing-cycle`, `credits*` (14 rows, 67k frames), `any-fly-to-ceres`, `chain-any-0011-DF45` (the explosion and the flight to Zebes) | E4 | 5 |

### Platform and tracks (not clean-room; hardware formats or the original's timing)

| # | Work | Lanes |
| --- | --- | ---: |
| E1 | `SpriteDraw` + `exact::oam` (MODEL order 3) | 2 |
| E2 | `TileWrite` + `exact::tilemap`, absorbing `plm/draw`, `bg_graphics`, `tilemap_dma`; HUD and message box video | 3 |
| E3 | The line program (B6): HUD split, HDMA channels, windows | 3 |
| E4 | Mode 7 and the renderer's remaining modes, mid-line writes | 2 |
| A1 | `Program<F>` generalised from `enemy/instructions` and the instruction lists as imported assets (PLM, enemy, sprite object, HDMA, palette FX, animated tiles) | 3 |
| A2 | The asset importers for enemies (headers, spritemaps, hitboxes, vulnerability, drops), rooms (R2) | 3 |
| T1 | **Slowdown and loading time** (D9): the cost model that replaces recorded NMI timing | 6+ |
| U1 | **Audio**: CPU side as a clean domain (sound requests, the three libraries' handshakes and drops, the music queue, per-room tracks; 3 rounds, `apu` stream and `apu_log` in every record); the driver: see risks | 3 rounds + ? |
| V1 | Tooling: a domain free-run harness; RNG draws per consumer from `hits`; a compressed record cache tier | 3 |
| V2 | The whole-movie run: the engine from power-on through a movie's inputs alone, compared at sparse checkpoints, then at dense ones over the range they narrow a divergence to | 2 |

Two rules hold for the platform work whatever lane carries it:

- **D16, the display snapshot.** What the renderers read (the Tick's intents, D16's view) is
  latched at the boundary where the hardware latches the picture, the NMI's commit at the end of
  the frame: a lag frame shows the previous snapshot, as the screen does. Rendering reads the
  snapshot and never advances the game; drawing a frame twice, or at the host's own rate,
  changes no State.
- **T1, one coordinator of time.** One coordinator advances the game by whole original frames,
  each with the lag the cost model gives it. A host that falls behind catches up by running
  whole frames, at most a capped number per host frame, and drops the rest of the debt rather
  than run ahead in a burst. Neither the renderer nor the audio backend advances the game.

### E1: where sprite drawing stands, and its second lane

Every sprite the engine draws is a `game::intent::SpriteDraw`: pieces, each with its place on
the screen at full precision (`ScreenPoint`, `i32`), its look (picture, palette, priority,
flips: `data::spritemap::SpriteAttributes`) and its placing (`Mapped(size)`, a spritemap's
piece, or `Lone`, a small sprite placed by itself). `exact::oam` is the one encoder: X to nine
bits and Y to eight, the high table's bits, the 128-entry wrap for mapped pieces, the lone
sprite's low table only and no wrap, and the frame's clear and hide. Its three producers are
Samus's body (`samus_drawing`), the atmospheric graphics and the enemies (`enemy::drawing`);
each builds the draw and the caller writes it where the original wrote its sprites, so the
order of the entries is the original's. The enemy's tile and palette base (`SpriteBase`, the
carry from the tile into the palette) stays with the enemy: the piece holds the look it gives.

The second lane carries, in this order:

1. **The intent view (D16).** The Tick's draws, kept where `NmiBatch::draw_sprites` encodes
   them, go out with the Tick (`TickOutput`, or `game::view::describe`), and a check holds that
   the kept draws, encoded again from an empty buffer, give the frame's OAM. *Risk:* draws are
   made per main-loop pass, not per Tick: a lag Tick has none and shows the previous; the view
   must say which pass a picture is from.
2. **Samus-appearance part 2's bridge (S5).** The clean model says which clip frame shows and where
   (and the flicker); the bridge turns it into the draw through the assets, as
   `samus_drawing::body` does now, and `samus_drawing` goes. No sprite type joins `types`:
   a model names clips and frames, never sprites ([DESIGN](../cleanroom/DESIGN.md)). *Risk:*
   the chest sprite is her body's one `Lone` piece (`SamusAssets::chest`); the bridge keeps it
   between the top half and the bottom half, where the `oam` stream sees its entry.
3. **The writers not yet drawn**, each with its domain: Samus's projectiles and their trails
   (the `projectiles` field group and the `oam` stream of item-morph-ball), the arm cannon, the
   charge flare and the echoes, then enemy projectiles and sprite objects. Each is a producer of
   `SpriteDraw`; none adds an encoder rule unless the original writes OAM another way.
4. **The encoder's open rules**, each a named rule in `exact::oam` or a LEGACY entry: a lone
   sprite past the table's end (the original writes it into the high table; here it is
   dropped), the enemies' extended spritemaps and graphical offsets (with A2), and margin draws
   for the wide renderer (a producer that culls to the screen, as the atmospheric graphics do,
   draws past it on request; the encoder never sees those pieces).

### E2: where tile writing stands, and what is left

A write to a background layer is a `game::intent::TileWrite`: where it lands, in the layer's
own terms, and the looks it writes there. Each cell's look is a `TilemapEntry` (picture,
palette, priority, flips), the background's counterpart of a sprite's `SpriteAttributes`.
`Overlay { row, cells }` is whole rows of the overlay, the screen-fixed layer the HUD and the
message box show on. `Blocks { at, looks }` is a row of the room's blocks on layer 1, placed in
room blocks, each with its `BlockLook` (its four cells). A level write is in room terms because
the tilemap is a ring of 32 block columns around the camera: the encoder, not the producer,
says whether the ring holds a block now and where. The wide renderer reads the same write
without the ring.

`exact::tilemap::TileQueue` is the encoder for the VRAM write queue. The overlay's rows are one
upload read from the HUD's buffer. A block row is clipped to the blocks the ring holds and
placed in BG1's two screens, with the screen swap BG1's scroll offset makes. Its top cells are
queued before its bottom cells, and a row that runs off its screen is split onto the other (the
screen split of tilemap rows). It is read from the PLM draw's buffer, which a pass fills from
its start. The queue's capacity drops a row whole. Its producers are the HUD (`interface_hud_bridge`, the
model's grid as the overlay's rows 1 to 3) and the PLM pass (`plm::draw::Drawing`, a changed
block's look from `bg_graphics::block_look`). Each caller writes where the original queued, so
the queue keeps the original's order.

A line of blocks the camera brings into view is a `game::intent::LayerExposure`: the layer, a
column (16 blocks down) or a row (17 across), its first block in the layer's level and the
background's block it lands on. The two differ because the BG scroll adds the scroll offset to
the layer's position, so the intent carries both. `scrolling::exposures` decides the lines from
the layers' blocks and the ones they last exposed (`camera.drawn`), layer 2 only on an axis where
BG2 shows its level. `bg_graphics` is only the level lookup (`exposed_looks`, `block_look`): the
original's index multiply, the level buffer's tail and fill, the item GFX entries and the flips.
`exact::tilemap::expose` places a line on the ring (each BG's two screens, BG2's the room's
tilemap gap before BG1's, the column part that wraps to the screen's top and the row part that
wraps to the other screen) and builds it into the layer's column or row buffer
(`video.batch.level`), counting a request; `exact::tilemap::commit_exposures` is the NMI's
scrolling DMAs. The buffers stay State decoded from work RAM, so a lag frame carries them and
their request counts to the next NMI.

What is left of scrolling:

1. **The lookup's stale reads** (docs/LEGACY.md: a block past the level data, a tile past the
   tile tables) stay with the lookup, not the encoder.
2. **The exposures in the intent view (D16)**, with E1's second lane: the Tick's exposures go
   out with it, and the wide renderer draws a line from its `level` without the ring.
3. **Who decides the lines.** `scrolling::exposures` is engine code over the engine's camera
   State; the camera's clean model exports layer 1's position and nothing about lines. When R4
   brings layer 2's own motion and the BG scrolls into a domain, the exposure goes with it as an
   output, and the camera's interface gains it then, not before.
4. **Mode 7 and time frozen** (`scrolling`'s open notes), with the rooms that use them, and the
   room load's whole-screen draw with R1, as more producers of `LayerExposure` or a
   screen-sized shape of it.

The message box writes the PPU itself while the NMI lags, around the batch. Its blank box, its
message's rows (the borders around the message, with the special button's letter) and its
restore are each an `OverlayRows` write, the overlay's shape of `TileWrite`, through the
encoder's direct entry: `exact::tilemap::write_now` places the rows on the overlay at once and
leaves DMA 1's registers as the box's own DMA does (its buffer's address run on past the rows,
the indirect fields cleared), since the next NMI's uploads start from them. The backup is a
read-back (D5), `exact::tilemap::read_now`, made where the box makes it. The restore writes the
backup's words as cells, and a cell gives its word back exactly, so cells VRAM held stale come
back stale. `exact::ppu` holds the box's PPU set-up and the way back: the text colours, the
playfield it shows over, the black backdrop and BG3's scroll.

What is left of the message box's video: its BG3 Y scroll HDMA channel (`message_box::video`),
its scroll table and HDMA's enable and restore, which go with E3's HDMA channels; and the save
prompt's yes and no, with H3.

R1 and R4 export tile writes onto the ring the scrolling lane settled. The PLM draw's open
shapes go with W2 and A1, each as more `TileWrite` shapes or the row shape repeated: the draw
instructions that draw a column or more than one row, and the buffer's index restarting at a
pickup's resume (`plm::resume`'s open note).

## 4. Order

The waves are ordered toward the **headline milestone** (MODEL §1 decision 14): the engine
replays a whole movie from one start state with nothing fed back. The first target is power-on
through Ceres: the any% movie from power-on to its frame 15,196, where seed `any-opening:7` falls
after the flight to Zebes and before the Landing Site loads (Jim, 2026-11-12); the Landing Site
starts the next milestone. A divergence is localised by sparse checkpoints over the run, then
dense ones over the range they narrow it to.

**What the stretch holds.** Observed, not read off the game: lsnes played the any% movie from
power-on to frame 15,500 and logged, per frame, the game state, the cinematic step, the room,
each object list (enemies, enemy projectiles, PLMs, palette FX, animated tiles, HDMA objects,
cinematic objects), the BG mode, the quake, the FX type, the music and sounds, the timer, the
message box, energy, SRAM writes and the APU port writes; the bank logs name what each id is.
A frame is a video frame of the movie, lag frames included; the stretch is frames 0 to 15,195,
and the table runs on to seed `any:12` (15,428) for the Landing Site's load that follows it.

| Frames | What plays | What runs (evidence) | Lock rows (`lsnes.toml`) |
| --- | --- | --- | --- |
| 0-270 | power-on and boot | the sound driver's upload (115,715 APU port writes); SRAM checked and written (263-270); 99 lag frames | `any-power-on` |
| 271-567 | the title, file select (a new file, SRAM written at 432), the start menu | mode 7 and its objects in the title; HDMA; palette FX objects (the title's tube light and displays); the title music's upload; 41 lag frames | `any-power-on` |
| 568-7,511 | the intro: its text pages, the old Mother Brain fight, the baby metroid's discovery | cinematic text, sprite and BG objects; Samus played by demo input in two scenes (2,219-2,738, 3,517-4,322): her motion, looks, missiles (899 to 896) and a knockback, rinkas and Mother Brain as cinematic objects; a palette FX object; the intro's music uploads | `any-intro-0568`, `any-intro-2219`, `any-intro-4323`, `any-intro-6012` |
| 7,512-8,314 | the flight to Ceres | mode 7; cinematic objects; palette FX objects (the gunship's engine, Ceres's lights) | `any-fly-to-ceres` |
| 8,315-8,537 | the new game: SRAM written (8,315), the elevator shaft loaded and faded in | room loading; the elevator pad (two enemy projectiles); the shaft's mode 7 by HDMA; the Ceres haze; 32 lag frames | `any-ceres-arrival` |
| 8,538-10,160 | Ceres's rooms 1 to 5 and the five doors down to Ridley's room | her motion and looks, the view, the HUD; the Ceres doors (an enemy) in every room; FX: the elevator's mode switch, the haze; door transitions with their lag | `chain-any-0001-DF45` to `chain-any-0005-E06B` |
| 10,161-11,771 | Ceres Ridley's room | Ceres Ridley: five contact hits (energy 99 to 24 by 15, five knockbacks), her power beam's hits; the mini-map off; the getaway in mode 7; quakes; the escape timer starts; no fireballs | `chain-any-0006-E0B5` |
| 11,772-13,235 | the escape back up through rooms 5 to 1 | the timer; quakes in every room; steam (an enemy) in rooms 3 to 1; falling debris (enemy projectiles) in room 2, dust in rooms 2 and 1; the rooms' boss-dead states; the shaft rotating | `chain-any-0007-E06B` to `chain-any-0010-DF8D`, `chain-any-0011-DF45` |
| 13,236-15,195 | made it to the elevator, the blackout, Ceres goes boom (SRAM written at 13,312), the flight to Zebes; at 15,195 the game turns to loading the next area | mode 7; cinematic objects; HDMA; palette FX objects (the planet's name fading); music uploads | `chain-any-0011-DF45` |
| 15,196-15,427 | after the milestone: the Landing Site loaded and faded in | room loading; its door caps (yellow, green) and scroll PLM; the rain FX, rain tiles, lightning and the gunship's glow; the gunship (an enemy); the Crateria music's upload; 41 lag frames | `any-zebes-arrival` |

What the stretch never shows: a block that reacts to her (Ceres's rooms have no PLMs, so no
door caps, crumble, shot or scroll blocks), animated tiles or palette FX in Ceres's rooms, a
liquid, an item, a message box (its index stays 0), a pause, a death, a sprite object, an enemy
outside Ceres's three species. 413 frames lag, 141 of them before the intro.

**What the milestone needs**, each item for what the stretch shows of it:

| Item | For |
| --- | --- |
| S1, S5 (samus-movement, samus-appearance) | her motion and looks in every room and in the intro's two demo scenes |
| S2 | Ceres Ridley's getaway pushes her when she stands in his way (the 100% movie's `samus-ceres-push`); no liquid in the stretch |
| S4 | her knockback: Ridley's five hits, the intro's one; no bomb |
| CAM | the view in every room; while the mode 7 flag is set (the shaft, Ridley's getaway) the BG scroll doesn't follow it |
| W1 | her power beam at Ridley, her missiles in the intro's Mother Brain fight |
| W2 | no reacting block: only the PLM runner's pieces the stretch spawns: the timer that enables sounds and the escape shaft's door turned to solid blocks (its boss-dead state's set-up) |
| C1, A2 | the enemy framework (process, draw, contact, her shots' hits) and the enemy projectile runner, over Ceres's species and projectiles; their importers |
| B8 | Ceres's own species and projectiles: Ceres Ridley and the getaway, the Ceres doors (and the shaft's platform), steam, the elevator pad, the falling debris, the dust; the shaft's rotation |
| H1, H2 | her energy as Ridley hurts her; the HUD, the mini-map switched off, the escape's timer |
| P1, P2 | Ceres's boss defeated and the boss-dead room states; SRAM at boot, the new file, the new game and Ceres's end |
| R1, R2 | ten door passages and their lag; loading every Ceres room, the shaft at the new game included; no elevator ride |
| R3 | quakes (Ridley's room, every escape room); the FX types of the shaft, Ridley's room and the haze |
| R4 | the palette FX runner, which the title, the intro and both flights use; no animated tiles, and Ceres's rooms have no palette FX |
| F2 | boot, the title, file select, the start menu and the intro, its demo input included |
| F4 | the flight to Ceres, the elevator's end, Ceres goes boom, the flight to Zebes |
| U1 | the `apu` stream holds the CPU side of every exchange, the driver's uploads included (411,621 port writes over the stretch), so the driver's replies must be reproduced, not only the requests |
| E1, E2, A1 | sprites, tile writes, instruction lists as assets (PLM, enemy, projectile, palette FX, HDMA and cinematic object lists) |
| E3, E4 | HDMA objects in every stretch but boot, the shaft's and Ridley room's mode switch per line; mode 7 in the title, both flights, the shaft, Ridley's getaway and the explosion |
| T1 | 413 lag frames: with nothing fed back, the recorded NMI timing can't stand in |
| V2 | the whole-movie run: one free run from power-on, compared at sparse checkpoints, then dense ones |

Not needed for it: S3, S6, S7, W3, H3, F1, F3, the enemy families and the other bosses (Ceres
Ridley shares Ridley's code; B9 may reuse B8's model, not the reverse). The Landing Site's
content is the next milestone's: W2's door caps and scroll PLM, R3's rain, R4's rain tiles,
lightning and the gunship's glow, F4's gunship. `tools report` shows the milestone as the
README's first row, from `[milestone]` in [domains.toml](domains.toml): the stretch's frames, how
many the engine plays alone today and which of these items are not done.

**The milestone's order of work.** Each night picks the first step whose prerequisites hold.

1. Rows for every frame: the opening (0-8,537) and the escape's end (13,072-15,195) are recorded
   and locked (the `any-*` rows and `chain-any-0011-DF45`); with Ceres's chain rows the lock
   covers the whole stretch, one row run from its own seed at a time. `any-zebes-arrival`
   (15,196-15,427) locks the Landing Site's load, the next milestone's first row. Their seeds
   and pads are in the relock bundle and their records cached with floors, but the engine's
   decoder starts only main gameplay in rooms it knows, so the free-run gate starts none of
   them yet and lockstep compares none of their Ticks.
2. Ceres's play, 8,538-13,235 (4,698 frames, 31%): the in-progress Samus, camera, HUD and
   progress work; then R1 with R2, A1 and A2, C1 with its projectile runner, B8 (the Ceres doors
   first, in every room), H1 with S4, W1's power beam, R3's FX and quakes, E3's mode switch and
   E4's shaft. Each lands on Ceres's chain rows.
3. The run as a whole: T1's lag, U1's driver and uploads, V2's free run with checkpoints.
4. The cinematics and the front end, 0-8,537 and 13,236-15,195 (10,498 frames, 69%; the intro
   alone is 46%): F4's flights and explosion, F2's boot, title, file select and intro (its demo
   scenes reuse S1, S5 and W1), R4's palette FX runner, P2's four SRAM writes.

Each wave's domains are independent specs and run in parallel; a wave starts when what it
depends on has its model shape (decision 2).

**Wave 0, now (small, queued):**
- S1 samus-movement follow-ups: `rising`, C2, C3. 1 round.
- Camera round 2: C5's masks named in the harness note; the lead settings' vectors; the
  closed-screen cells' interface ready for W2. 1 round. (C4 waits for S4/H1/S7.)
- V1 tooling: the domain free-run harness (the model fed its own State across a row, other
  domains' outputs from the records), so every domain has MODEL §4's primary proof before the
  engine runs past slice 0; the RNG consumer split; a compressed or WRAM-only cache tier.

**Wave 1 (on the cached rows, 4-5 lanes):** S2 (4), S5 (3) with E1, H2 (2) with E2, W2 (5)
with A1, P1 (1). About 15 rounds plus 8 platform lanes.

**Wave 2, the engine past slice 0, toward Ceres:** R1 (6) with R2, W1 (5), C1 (6) with A2, then
H1 (2) and S4 (2) (camera C4 closes); V2, the whole-movie run. This is where the engine first
runs a chain row across a door: value jumps, since every later domain is then proven in the
engine's free run on hundreds of chain rows, not only the harness. About 21 rounds.

**Wave 3, Ceres and the opening; the milestone closes with it:** B8 (4), F2 (8), F4 (5), R3 (4),
R4 (3) with E3, P2 (3), U1 (3); E4 and T1 alongside. F4's Ceres scenes come first; its ending
and credits can wait for the movie's end. About 30 rounds.

**Wave 4 (widest), the movie's next stretches:** W3 (2) and H3 (2), the first things after
Ceres; the enemy families C2-C11 (25-30), up to 6 in parallel; B1+B2 together to fix the phase
table shape (5), then the other bosses in size order (B3-B7, B9, B10, 30). An enemy species keeps
its definition, its state, its AI and its animations together, in one place, as one family's
member; its animations are typed programs (A1's instruction lists, in steps named by behaviour)
checked against the ROM by the importer. About 66 rounds.

**Wave 5:** S3 (2), S6 (3), S7 (2), F1 (5), F3 (1). About 13 rounds.

**Total:** about 145 clean-room rounds and 30-40 platform lanes, against about 13 done so far
(samus-movement's 12, the camera's 1). Units per round need to average about 30 (the camera did
42 in one round; samus-movement 17 a round while the method was settling). If the round's fixed
cost (export, adversary twice, canaries, landing one at a time) doesn't fall, root integration
and review, not implementation, set the pace.

## 5. Risks and proposals

1. **Scale.** About 20 times what's done. The fixed cost per round is the bottleneck.
   *Proposal:* batch arms per round (target 40-60 units); let adversaries gate a family's
   spec once, with per-species deltas; integrators land per wave, not per round.
2. **Audio.** The music is the N-SPC driver: SPC700 code and DSP on the ROM. A clean-room spec
   of a sequencer from port logs and SPC snapshots is a project of its own, and a sample-exact
   check needs the DSP modelled. *Proposal:* the CPU side (requests, handshakes, queue) is a
   normal domain now, verified on the `apu` stream. The driver is a **goal question for Jim**:
   (a) a clean-room N-SPC sequencer over imported songs plus a DSP model in `runtime`, the
   thesis's way (perhaps 8-12 rounds plus a DSP), or (b) run the ROM's driver in a GPL SPC
   emulator as the vanilla `SoundBackend` (Jim's "GPL emulator OK as binary"), so the engine
   models only what the game observes (D18).
3. **Bosses and scripted behaviour.** Long instruction lists with many custom steps, multipart
   enemies, rare branches (Mother Brain's phases) and 65k of enemy-projectile code in the old
   port. Records reach few branches. *Proposal:* B1+B2 first to fix the phase table; record
   short segment rows per phase (D26 ladders) before each boss's spec; projectiles belong to
   their owner's domain, the slot and contact rules to C1.
4. **Tables of thousands of entries as assets (decision 4).** Instruction lists (PLMs,
   enemies, sprite objects, HDMA, palette FX, animated tiles), enemy headers, vulnerabilities,
   drops, rooms and their states, doors, FX, texts, maps, demos, credits. They must be imported,
   not specified as arms. *The leak risk:* an instruction-list step vocabulary that mirrors the
   original's opcode set is a shape leak. *Proposal:* the asset format names steps by behaviour
   (show a frame, move, play a sound, spawn, wait for a condition), the importer maps opcodes
   to them outside the clean room, and the adversary checks the vocabulary once per family (A1)
   before any family uses it.
5. **The engine can't free-run past slice 0 until doors land.** Every new domain is proven
   only per frame (re-seeded), which MODEL §4 calls insufficient, and "delete the old code"
   applies only to slice 0's modules. *Proposal:* V1's domain free-run now; R1+R2 early (wave 2).
6. **One RNG across domains.** Enemies, drops and bosses draw from one word in slot order; a
   domain's harness can't know its draws without the others'. *Proposal:* watch the RNG step in
   `hits` (the record already supports watched addresses) to give each consumer its draws;
   the order of consumers is an "order that matters" entry, owned by the Tick.
7. **Slowdown and loading time.** Frame counts depend on the original's CPU time (door loads,
   explosions, many enemies). A cost is a property of the original's code, which the clean
   room can't see. *Proposal:* T1 as an observer-fitted table of named costs per event,
   measured from the records' `bank_instructions` and overruns, verified on the `timing`
   stream; recorded NMI timing stays the Tick input until it holds.
8. **Within-frame feedback forces merges.** A contact this frame changing her motion this
   frame, by MODEL §4's rule, merges domains and could grow a "combat" mega-domain. *Proposal:*
   the two corollaries in §3 (modes partition frames, events as exports), and check each
   boundary's verifiability in the spec review, before the implementer starts.
9. **The budget has no baseline for new domains.** "Smaller than the old code" needs old code;
   most domains have none in today's engine. The archive's port is so verbose (0.12-0.2) that
   it is a weak bar. *Proposal:* k × units only (k frozen at 36), with the archive's score
   reported as a ceiling; a goal question only if Jim wants a stricter bar.
10. **Disk.** A cached frame is about 206 KB; the whole lock would be about 92 GB. *Proposal:*
    cache per domain (20-40 rows each), and V1's compressed tier (WRAM deltas compress well).
11. **Recordings missing.** The pause screen, the attract demos and the intro have no lock row;
    the 100% chains may cover pausing (check before recording). Rare boss phases need segment
    rows.

The plan's domains, their waves, their progress and the rounds landed are data in
[domains.toml](domains.toml), from which `verify::report` writes the README's status.
