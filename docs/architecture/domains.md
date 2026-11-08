# Domain plan: the rest of Super Metroid as clean-room domains

Planner, 2026-11-08, read-only, at spine `e458a3d5d`. Sizes are `tools size` weighted scores
(the wire-cam `size.dll`, the same counter as the specs' baselines). Raw per-file output:
`$TEMP/collab/reviews/sz_all.txt` (today's engine) and `sz_old.txt` (the whole-game port at
`archive/old-engine`, `crates/sm_game/src`, verification and tests excluded); per-module sums
`mods.txt` and `oldmods.txt`.

## 0. The answer in three lines

- Done: about 5% of the game's behaviour (samus-motion 7,160 + camera 1,506 weighted, about
  250 spec units). The remaining clean model is likely 120-170k weighted, about 3,500-4,500
  spec units at k = 36.
- That is about 140-150 clean-room pipeline rounds (spec, adversary, implement, wire, gaps)
  plus 30-40 platform lanes (encoders, room loading, timing, renderer, audio backend, tooling).
  With 4-6 lanes in parallel at about 2.5 h a round it is 70-100 h of pipeline wall-clock in
  the best case. With review, Jim's gates and one-at-a-time landing, expect several weeks.
- The order: finish Samus and the world she touches on the 32 cached rows, then doors (so the
  engine runs past slice 0), then combat and enemies, bosses smallest first, menus and
  cinematics last. Audio and slowdown run as separate tracks, each with a goal question for Jim.

## 1. The engine today

The engine runs **slice 0 only**: the rooms `data::room` loads for `area-brinstar` (the
elevator ride into Brinstar and a short walk) and `item-morph-ball` (the Morph Ball Room, with
its item, message box and about 370 lag Ticks). Free-run gate green pairs: area-brinstar
vram/cgram/oam/sram/ppu/timing, item-morph-ball sram/timing. Lockstep: area-brinstar 476/476,
item-morph-ball 1372/1427 Ticks equal (54 video, 1 each elevator/projectiles/room). No doors,
no second room in a run. `app` is an empty `main` (2.5). The **domain harnesses** (game-time,
samus-motion, camera) run on oracle frames of any room, so the clean domains are proven far
beyond the slice (32 rows, 18,331 frames), but only per frame, re-seeded.

Totals: `game` 36,856, `data` 25,412, `runtime` 8,351, clean models 8,666, `clean_types` 467.

### `crates/game` (36,856)

| Module | Score | What it does, what it covers |
| --- | ---: | --- |
| `state` | 3,984 (+7,089 types) | the State schema: samus, room (D4 reset struct), slots (D12, cursor D13), strand, video (D5), sram, hud, projectile, atmospheric |
| `plm` (+crumble, draw, item_gfx) | 4,705 | spawning a room's PLMs; scroll PLMs, block extensions, items (hand-coded `ItemStep`, a review stop per D13), the 1x1 shot block crumble, PLM draw into BG1, item GFX load. Slice 0's PLMs only; others are `PlmKind::Unmodelled` |
| `enemy` (+drawing, eye, face_block, instructions) | 4,567 | processing window and per-slot bookkeeping, the instruction-list runner, drawing by layer; species: the elevator, the Morph Ball Room's eye, the Blue Brinstar face block. 3 of about 170 species |
| `samus_world` (bridge) | 2,769 | the samus-motion model's `World`, state map and step (integrator's; reported apart) |
| `projectile` | 2,551 | Samus's projectiles: cooldown, the power beam's flight and block checks, instruction lists. No other beam, missiles, bombs |
| `tick` | 2,384 | main-strand dispatcher, phases in the original's order |
| `message_box` (+video) | 1,925 | the item pickup box as a strand frame with its lag waits |
| `bg_graphics` | 1,803 | building tilemap columns/rows as layers scroll |
| `animation` | 1,246 | Samus's animation delay lists |
| `commit` | 1,203 | the NMI's video commit points |
| `mini_map` | 1,146 | explored cells, the 5x3 HUD map |
| `atmospheric` | 1,082 | splashes, bubbles, dust (4 slots) |
| `hud` | 1,058 | HUD tilemap redraw (energy, tanks, ammo, selection, auto reserve) |
| `collision` (+inside, solid_enemy) | 1,028 | the room she meets; block-inside reactions; solid enemies |
| `pose` | 917 | what's left of the pose input / change outside the model |
| `tilemap_dma` | 764 | the NMI's scrolling DMAs |
| `session` | 597 | the Session, Tick begin/end |
| `samus_drawing` | 605 | her clip's frame into the sprite buffer |
| `camera_world` (bridge) | 562 | the camera model's contact |
| `nmi` | 533 | commit or lag, pad latch, counters |
| `oam` | 525 | sprite buffer frame (clear, hide undrawn) |
| `scrolling` | 252 | BG scrolls follow the camera, layer 2 at the room's rates |
| `samus_palette` | 183 | visor only |
| `game_time` | 147 | play time |
| `rng` | 120 | the RNG word |
| `lines` | 107 | the scan-out skeleton (HUD split, HDMA not yet) |
| `model`, `lib` | 98 | |

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

`clean_model/samus-motion` 7,160 (+8,118 data), `clean_model/camera` 1,506 (+535 data).

## 2. What the real game has that the engine lacks

Sized by the whole-game port at `archive/old-engine` (`sm_game`, 1.04 M weighted, verbose and
partly transpiled; it is an upper bound). Calibration: samus-motion's clean model is about 0.13
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
| S2 | **Samus in water, lava and sand; the speed booster; pushes** (samus-motion rounds) | other speeds and gravity in liquids, the booster's dash cap and higher jump, quicksand, conveyors, Ceres pushes, the steep blocks' corners | reads the liquid's level (R3) as an input; same speed quantities, so it extends samus-motion | in `samus_world`/model already; old port movement | cached (chain-0220/0224/0232, area-maridia, terrain-quicksand, samus-ceres-push) | S1 | 4 |
| S3 | **The shinespark and echoes** (samus-motion rounds) | charge store, the spark's directions, its end, the echoes' positions | exports the echoes to S5 | none | cached (samus-shinespark-*) | S2 | 2 |
| S4 | **Hurt and knockback; bomb jumps** (samus-motion rounds) | knockback motion and its turn of the camera lead (camera C4), bomb-jump launch | reads a hit event (C1) and a bomb blast (W1) | none | cached (chain-0106, samus-bomb-jump-vertical) | S2 | 2 |
| S5 | **How Samus looks** | animation timing (`AnimationEnds` becomes an output), her sprite (clips), palette (suits, visor, charge flash, screw attack, speed booster glow, hurt flash), atmospheric splashes and dust, the i-frame flicker | reads her pose; exports `SpriteDraw`, palette intents; samus-motion reads its "animation ended" | `animation` 1,246, `samus_drawing` 605, `samus_palette` 183, `atmospheric` 1,082 (3,116) | cached, OAM and CGRAM streams | E1 | 3 |
| S6 | **The grapple beam** | aim, fire, latch, swing (angle, length), release hand-off, grapple blocks | owns her position on attached frames; reads grapple block kinds; exports release speeds | none (old port 11k) | cached (samus-grapple-*); boss-draygon-grapple-* | S2, W2 | 3 |
| S7 | **The x-ray scope** | the cone, its sweep, time stopped (camera's `TimeStopped`), revealed blocks | exports time-stopped and the cone to R4 | none | cache `xray-scan-*` (4 rows) | E2 | 2 |

### Her weapons and the world she touches

| # | Domain | Scope | Reads / exports | Old code, baseline | Rows | Depends | Rounds |
| --- | --- | --- | --- | --- | --- | --- | ---: |
| W1 | **Beams, missiles and bombs** | beam combinations, charge, cooldowns, flight shapes (wave, spazer, plasma, ice), missiles, supers, bombs and their blasts, power bombs and their expanding window, crystal flash | reads equipment, her pose; exports projectile hits on blocks (W2) and enemies (C1), blasts (S4) | `projectile` 2,551 | cache `special-beam-*`, `beam-reflect`, `power-bomb-*`, `samus-crystal-flash`; chains | E1 | 5 |
| W2 | **Blocks that react** | shot, bomb, power bomb, super and speed blocks, crumble, respawn, grapple blocks, spikes (as a hurt event), the scroll cells that open and close screens (camera round 2), door caps as blocks | reads her touched cells (samus-motion `TouchedCells`) and W1's hits; writes the level's blocks; needs D13's `Program<F>` with PLM lists as imported assets | `plm` 4,705 (part), `collision/inside` | cached chains (crumble, scroll PLMs); more chain rows | A1 | 5 |
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
| B8 | Ceres Ridley and the Ceres escape | 4 | `boss-ceres-ridley*`, `area-ceres` |
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
| F2 | **Title, intro and file select; options** | boot, title, the intro's text and pages, file select (copy, clear), options and controls, demos (a recorded input source) | none | `options-menu-tour`, `file-copy-clear`, the `power-on` row; **new** for the attract demos and the intro | E2, E4 | 8 |
| F3 | **Game over** | death's screen, continue | none | `death` | H1 | 1 |
| F4 | **Cinematics and credits** | the gunship's landing and take-off, Samus's arrival, the planet's explosion, the ending, credits | none | `gunship-bobbing-cycle`, `credits*` (14 rows, 67k frames) | E4 | 5 |

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

## 4. Order

Each wave's domains are independent specs and run in parallel; a wave starts when what it
depends on has its model shape (decision 2).

**Wave 0, now (small, queued):**
- S1 samus-motion follow-ups: `rising`, C2, C3. 1 round.
- Camera round 2: C5's masks named in the harness note; the lead settings' vectors; the
  closed-screen cells' interface ready for W2. 1 round. (C4 waits for S4/H1/S7.)
- V1 tooling: the domain free-run harness (the model fed its own State across a row, other
  domains' outputs from the records), so every domain has MODEL §4's primary proof before the
  engine runs past slice 0; the RNG consumer split; a compressed or WRAM-only cache tier.

**Wave 1 (on the cached rows, 4-5 lanes):** S2 (4), S5 (3) with E1, H2 (2) with E2, W2 (5)
with A1, P1 (1). About 15 rounds plus 8 platform lanes.

**Wave 2:** R1 (6) with R2, W1 (5), W3 (2), S3 (2), S6 (3), H3 (2). This is where the engine
first runs a chain row across a door: value jumps, since every later domain is then proven in
the engine's free run on hundreds of chain rows, not only the harness. About 20 rounds.

**Wave 3:** C1 (6) with A2, then S4 (2) and H1 (2) (camera C4 closes), S7 (2), R3 (4), R4 (3)
with E3, P2 (3), F3 (1). About 23 rounds.

**Wave 4 (widest):** the enemy families C2-C11 (25-30), up to 6 in parallel; B1+B2 together to
fix the phase table shape (5), then the other bosses in size order (B3-B10, 34); F1 (5).
About 70 rounds.

**Wave 5:** F2 (8), F4 (5) with E4, U1, T1 continuing. About 15 rounds plus the tracks.

**Total:** about 145 clean-room rounds and 30-40 platform lanes, against about 13 done so far
(samus-motion's 12, the camera's 1). Units per round need to average about 30 (the camera did
42 in one round; samus-motion 17 a round while the method was settling). If the round's fixed
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
