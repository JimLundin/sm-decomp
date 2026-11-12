# Assets: our own formats, imported from the ROM

MODEL §1 decision 4: every content type the engine and the runtime load gets a format of our
own that names things by what they are, and an importer at the edge reads the ROM's layout once
and writes it. Decision 6: the formats look like what a developer of a 2D pixel-art game
expects: tile sheets, palettes, sprite sheets with animation clips, tile maps with object layers
(Tiled's shape), timelines, a sound bank. The assets come from the player's ROM and are never
committed.

Samus's sheet and clips (§4) are in this format today, the blocks' reactions
([world-blocks.md §2](../behaviours/world-blocks.md)) are imported into the blocks model's own
records, and the rooms the engine loads come through the world's first records (§3.1).
Everything else is still read through the ROM's layout (`data::room`, `level`, `tileset`,
`enemy`, `plm`, `hud`, `map`, `message`, `palette`, `pose`, `spritemap`, `compression`), or
not decoded at all because the oracle's seed holds it (BG graphics,
palettes, palette FX, sound, scenes, Mode 7). The plan items AF0-AF5 and A3
([domains.md §3](domains.md#3-the-domains), the platform table) bring each family into this
format ahead of the domain that consumes it.

## 1. Principles every format keeps

1. **Named by what it is.** No ROM address, bank, pointer, table index or opcode in a record.
   Identity is a typed id we own (`RoomId`, `DoorId`, `ItemId`, `SpeciesId`, `SoundId`, ...),
   held in a committed registry that the importer assigns at the edge; the registry maps the
   original's places to our ids and holds no ROM bytes, as the pose map does. sm-json-data's
   names and ids (through sm-json-data-rs) are labels attached to our ids, for reading and for
   D26's strats, never the key, so an upstream rename changes only the label table. Inside the
   engine ids are dense (D15); checkpoints carry the content fingerprint.
2. **One shape per kind of thing** (§2). A new content type reuses a shape before it defines one.
3. **Lossless against what is observed.** Each format has one encoder in `game::exact` to the
   bytes the original puts in VRAM, CGRAM, OAM, WRAM or SRAM. The importer checks
   `encode(import(rom))` against the ROM's bytes for every record it writes and refuses the ROM
   if any differ (decision 15: the importer checks the content's shape); CI runs the check on the
   ci-data ROM. Only what is observed must round-trip: the compressed stream a level came from is
   never observed, so compression is knowledge of the edge alone. Sound is the exception (§3.7):
   it is compared on its timing, the game's sound requests and the sequencer's events, never on
   the original driver's bytes or the audio out.
4. **Quirks are named fields, not hidden layout.** Where the original reads past a record (a
   block past the level, a tile past the tile table, the level's tail in work RAM), the importer
   bakes what the original would read into a named field of the record, and LEGACY lists it. An
   edited record carries those fields with the defaults its own encoder produces.
5. **Program, not code.** Where the original runs a routine named by a pointer (room setup, room
   code, door code, an enemy's AI, a PLM's setup), the format holds an enum variant named by
   behaviour (`RoomRule::ShaftModeSwitch`, `SpeciesKind::CeresDoor`). Every vanilla pointer is
   known, so a pointer the importer doesn't know refuses the ROM; a tolerated invalid value is an
   `Unknown(..)` variant with a LEGACY line (AGENTS.md).
6. **Adjusting content is editing a record**: a room's cells, a sound's notes, a palette's
   colours. Rules and parameters stay in the model and the profile; the boundary is
   [world-blocks.md §2](../behaviours/world-blocks.md)'s: what a script shows, for how long and
   with which sound is content; what starts it and how its steps run are rules.
7. **Hardware formats stay in `exact`.** 4bpp planar tiles, BGR555 words, the OAM layout and the
   tilemap word are hardware encodings: a tile holds indexed pixels, and the planar bytes are
   `exact`'s encoding of them. A record holds a hardware payload only where that is the honest,
   lossless representation. A sample is plain PCM: the importer decodes the ROM's compressed
   samples once, a decode that loses nothing, so no SNES sound format reaches the content.

## 2. The shared shapes (AF0)

| Shape | Holds | A 2D developer recognises | Used by |
| --- | --- | --- | --- |
| `TileSheet`, `Tile`, `TileRef` | 8x8 indexed pixels at a depth of 2, 4 or 8 bits | an indexed PNG cut into 8x8 tiles; an Aseprite tileset | BG graphics, the HUD, pause and font, sprite tiles, Mode 7 characters |
| `Colour`, `Palette`, `PaletteSet` | a BGR555 colour; 16 (or 4) colours; named rows | a .gpl palette | tilesets, Samus's suits, species, the HUD, scenes |
| `Cell`, `Spritemap` | a tile ref, palette row, priority and flips; a list of placed sprites | a tile in a layer; a piece of an atlas frame | everything drawn |
| `SpriteSheet<K>`, `Clip<K>`, `Frame`, `Hold`, `ClipEnd` | §4's records, with a key type per owner | an Aseprite sheet with tags | Samus, species, projectiles, sprite objects, cinematic objects |
| `Figure` | a frame of pieces, each a spritemap at an offset with its hitboxes | an Aseprite frame with slices | extended spritemaps (bosses, multipart enemies) |
| `Grid<T>`, `TileLayer` | a width by height grid, row-major | a Tiled tile layer | levels, collision, backgrounds, the map, scene layers |
| `ObjectLayer<O>` | placed objects: a position and a typed `O` | a Tiled object layer | doors, PLMs, enemies, FX regions, scene objects |
| `Program<F>` (A1) | ARCHITECTURE D13's steps, named by behaviour (show, wait, move, play a sound, spawn, loop, call, end), with family steps per `F` | an animation timeline; a tracker's event list | PLM scripts, enemy lists, projectiles, palette FX, animated tiles, HDMA objects, scenes |
| `LineTable` | per-scanline values: runs of lines and a value | a raster effect table | HDMA objects, Mode 7 per-line matrices, the message box's scroll |
| `SoundBank` | samples, instruments, sequences (§3.7) | a tracker module | music and sound effects |
| `Registry<K>` | our typed ids for one content type, with their labels | asset names in an editor | every keyed record |

`ContentSet` is the whole: every record set, its fingerprint (the `Canonical` digest of its
encoding) and overlays by stable key (D15's packs).

## 3. The formats

### 3.1 The world: rooms, versions, levels, doors (AF1)

A room is a tile map with tile layers and object layers; its versions (the original's states)
are variants of it that share what they share.

| Record | Holds |
| --- | --- |
| `World` | the rooms by `RoomId`, the doorways, the topology (D15: leaving through a door is arriving at its partner) |
| `Room` | its id, `MapPlace { area, x, y }`, its size in screens, its scrollers, its versions: conditions in order, then the default |
| `Version` | its condition (progress's own `Condition`), and its level, tileset, music, effects, enemy layer, enemy graphics, object layer, layer 2 motion, scroll map, background, room rule and setup |
| `Level` | the front `TileLayer<BlockCell>`, the back (a layer or a background), and the collision `TileLayer<Collision>` |
| `BlockCell`, `Collision` | a block look and its flips; a block kind with its typed argument, door cells naming a `DoorId` |
| `LevelQuirks` | the bytes past the level the original's lookups reach, named |
| `ScrollMap` | a `Grid<ScrollArea>` per screen, plus the run past the room's screens the original keeps |
| `Doorway` | its id, room, side, arrival (cell, screen, distance, cap), kind (door, elevator, ...), area change and door rule |
| `RoomEffects` | per entrance: liquid, weather, layer blending, palette effects and tile animations to run |
| `Background` | a `TileLayer<Cell>` over the tileset's sheet, with its setup steps |
| object and enemy layers | placed reactions, items, door caps, scroll cells, stations; placed species with their init, and the kill quota |

Of these, `data::assets::world` holds today what the engine reads of a room: `World`, and `Room`
with its place, size, scrollers and doors, and its versions with the PLMs each places. Room ids
come from its registry, `rooms` (slice 0's room, Ceres's six and the rooms their doors lead to);
`import::world` maps the original's room headers to them and builds the records with
`data::room`'s decoders. Its doors, conditions and scroll changes hold no ROM address: a door
names its destination's `RoomId`, a version's condition is the world's own `Condition` in
progress's terms (a doorway by its `DoorwayId`), and a PLM population names its scroll change
lists by their place in it. The importer refuses a room, doorway or boss bit no id stands for.
Still to come: the world's `Condition` and progress's are two types until the records can name
progress's, a PLM no model runs keeps its ID and room argument (`PlmKind::Unmodelled`), and the
verifier's state-header lookup still decodes the room directly.

Collision is a second tile layer, not a tile property: in Super Metroid a cell's kind is the
cell's own, not its picture's. Room loading (R2) writes `Level` back into the original's work RAM
image, which encodes to the block words and BTS losslessly.

### 3.2 Tilesets and palettes (AF2)

| Record | Holds |
| --- | --- |
| `Tileset` | its id, a 4bpp `TileSheet`, its block looks (each four `Cell`s), its `PaletteSet`, whether it uses the shared elements |
| `SharedElements` | the common room elements' sheet and block looks, which fill the first blocks of every tileset that uses them |
| `TileQuirks` | the cells a lookup past the table reads |
| `PaletteEffect`, `TileAnimation` | `Program`s (A1) that set colours or show tiles, wait and loop |
| `SamusPalettes` | her palettes by suit, and colour sequences (visor, charge, screw attack, glow, hurt, shinespark) as holds of palettes |

A block look is a 2x2 metatile, as Tiled builds 16x16 tiles of 8x8.

### 3.3 Programs (A1)

A program set is `Registry<ProgramId>` to `Program<F>`, with steps named by behaviour
(domains.md §5 risk 4); the step vocabulary is the format, and the adversary checks it once per
family. The importer maps each opcode to a step, and a pointer argument to a typed target
(`Spawn(ProjectileId)`, `Sound(SoundId)`, `Goto(Label)`).

### 3.4 Enemies, projectiles and sprite objects (AF3)

| Record | Holds |
| --- | --- |
| `Species` | its id, `SpeciesKind` (the AI its model runs, D14), health, damage, hitbox, layer, sheet, palette, programs, vulnerability per weapon, drop weights, sounds, parts, boss |
| `EnemyGraphicsSet` | the species a room loads, in order, each with its palette row |
| `ProjectileKind` | its id, sheet, program, hitbox, damage and properties |
| `SamusProjectiles` | a `SpriteSheet<Shot>` keyed by weapon and direction, with trails |
| `EffectSprites` | explosions, smoke, splash and dust as clips |

A species is an Aseprite file (sheet, tags, slices for hitboxes) with its numbers beside it.
Figures keep the piece order the OAM stream sees.

### 3.5 HUD, maps, text and pause screens (AF4a, AF4b)

| Record | Holds |
| --- | --- |
| `HudArt` | a 2bpp sheet, palettes, the pictures (digits, item icons, the bar's frame) by name |
| `AreaMap` | the area's map cells (64 by 32), the cells a station reveals, the pictures past the last column; its sheet and palettes |
| `Font`, `Message`, `Borders` | glyphs for characters and button letters; a message's id, border size and rows of text with `Button(Action)` places; the borders by size |
| `PauseScreens` | the map and equipment screens as tile layers, with object layers for the cursor's places and item slots |

A message holds text, not tilemap words; `exact::tilemap` writes the words.

### 3.6 Scenes: title, intro, file select, cinematics (AF5)

| Record | Holds |
| --- | --- |
| `Scene` | its id (title, intro, file select, the flights, the explosion, ending, credits, game over, ...), tile layers, an optional Mode 7 plane, sheets, palettes, cinematic objects, its script as a `Program`, line tables |
| `TextPage` | the intro's and the credits' lines, where they sit, the typing pace and its sound |
| `Mode7Plane` | an 8bpp sheet of 256 tiles, a 128 by 128 map, a palette |
| `DemoInput` | a recorded pad, runs of buttons and frames, by scene and slot |
| `MenuLayout` | file select's and options' places, cursor routes and number pictures |

Mode 7's VRAM interleave is `exact`'s encoding, not the format's. HDMA tables are `LineTable`s
owned by the room effects, the scenes and the message box.

### 3.7 Sound (A3)

No code of the game plays sound (MODEL §1 decision 23). The sequencer is a clean-room domain
(U2): observers spec from the oracle's port logs and sound chip which sound and which note start
and stop when, and its model plays these records. A modern mixer in `runtime` (E6) turns its
events into audio. Nintendo's sound driver is never run, emulated, ported or shipped, not even as
an opaque blob, and the sound chip is not emulated.

| Record | Holds |
| --- | --- |
| `Sample` | its id, its PCM frames (16-bit, mono), where it loops from |
| `Instrument` | its sample, its envelope (attack, decay, sustain and release, or a gain curve), its tuning |
| `Sequence` | a song's tracks of typed events: notes (pitch, length, gate, velocity), rests, ties, instrument, volume, pan, tempo, transpose, vibrato, echo, loops, phrase calls |
| `SongSet` | the songs an area or scene loads together, with their instruments and samples |
| `SoundEffect` | its id in its library (1, 2 or 3), a one- or two-voice sequence, its priority |

The importer decodes each sample to PCM as the sound chip decodes it; where a loop's decode
depends on the samples before it, the record holds the loop as the chip repeats it. There is no
encoder back to the ROM's sample codec or the driver's upload blocks, and no image of the driver:
the engine makes none of the original's upload writes
([LEGACY](../LEGACY.md#the-engine-planned-differences-from-the-original)). Sound is compared on
its timing: the game's sound requests and the replies it reads through the ports, and the
sequencer's events per Tick. The audio out is not compared (LEGACY), so the mixer's
interpolation, envelopes, echo and output rate may be modern.

### 3.8 Saves

The SRAM file is not ROM content and has no importer. P2's model writes a `SaveFile` (progress,
inventory, map, the station, the play time); `exact::sram` encodes it to the original's three
slots, their checksums and complements, and our versioned `PlayerSave` (ARCHITECTURE §9.5)
holds the same record in our own file.

## 4. Samus's sheet and clips

`data::assets::samus` holds the records. Nothing in them is a ROM address, a pose id or a
table index of the original's.

**The sheet** (`SpriteSheet`) holds every piece once:

| Record | Holds |
| --- | --- |
| `SpriteId` → `Spritemap` | a sprite list: each sprite's offset, size, tile, palette, priority and flips |
| `TilesId` → `TileUpload` | the graphics a frame uploads for one half, its two parts' bytes |

**A clip** (`Clip`) is a pose's animation, one per pose and dress variant:

| Record | Holds |
| --- | --- |
| `Clip` | its key, its dress variant (`Worn`), its frames in order, and its end (`ClipEnd`) |
| `Frame` | how long it holds (`Hold`), the top half's sprite list, the bottom half's, whether the chest sprite goes before the bottom half, the origin's Y offset from her position, and the halves' uploads |
| `Hold` | `Shows(n)`: shown `n` frames; `Control(byte)`: a step where the animation's control runs, kept so the frame numbers stay the engine's; the byte is what the timer is set from |
| `ClipEnd` | `Loop { from }`: plays on from that frame; `HandOver`: another pose follows, which her motion picks |

Beside them, `SamusAssets` holds `running`, the holds a run uses instead of its clip's while
she has running momentum, and `chest`, the one sprite the first suit's facing-the-screen pose
adds and where it sits from her position.

`unkeyed` holds the frames of the poses outside her domain (shinespark, grapple, knockback,
drained, the elevator's: every pose of the original's with a definition and a delay list that
the map doesn't name), as `Unkeyed { id, frames }`. It is the one record that names the
original's pose id, so that the engine draws every pose from this format; an entry leaves it
for a clip when its domain gives the pose a key.

**The key** is the pose as the player sees it (`docs/behaviours/samus-movement.md` §2):

| Record | Holds |
| --- | --- |
| `Pose` | body, action (with the turn's kind and what a landing touched down from), facing, aim, gun, legs |
| `Dress` | what the sprite shows that the pose doesn't decide: how her spin is drawn (`SpinSprite`), her suit facing the screen (`Suit`) |
| `Worn` | which dress a clip is the variant for: any, one spin sprite, or one suit |

The key is the clean room's `types::Pose`, which carries every part §2 lists;
`data::assets::samus::key` re-exports it beside the dress, so `data` depends on `types`
(`tools/check_crate_rules.sh`).

`SamusAssets::clip(key, dress)` finds a pose's clip. A pose that has no sprites of its own is
shown by its nearest (the air turns' one sprite for down-forward and down is the down one,
the crouched turn's is the down-forward one, spec §2 "Aim in a turn"); the lookup applies
that rule, so the format holds each clip once.

The palette a sprite draws with is its entries' palette field; which colours that palette
holds (the suit's palette, the visor's cycle) is AF2's `SamusPalettes`.

A clip keeps one frame per step of the animation, the control steps included, because the
animation's frame number is still the engine's state: frame `n` of a clip is the engine's
animation frame `n`. Which pose a `HandOver` goes to stays in `data::pose::animation` for
now: it is motion's, and moves with the pose transitions.

The arm cannon, the charge flare and the speed echoes drawn with her are not in the format yet.

## 5. The importers

All importers live in one crate, `import`, which depends on `data`. `data` keeps the formats
and no ROM layout: `rom`, `compression`, `address_map` and the decoders move into `import`
family by family, private to it once their format lands. `tools`, `verify` and `app` (for the
player's ROM at startup) depend on `import`; `game` and `runtime` may not, and
`tools/check_crate_rules.sh` checks it, so "only the importer knows the layout" is a build
error rather than a review rule. Samus's importer is `import::samus`. The decoded tables of
`data::pose` it reads stay in `data` while the engine loads them too (`game::pose::Content`
holds her pose definitions, transitions, spritemaps and tiles); they move into `import` when
the engine moves and draws her from the format alone.

| Importer | Reads | Knowledge that never leaves `import` |
| --- | --- | --- |
| `world` (AF1) | room headers, states and conditions, level data, door lists and headers, scroll tables, FX lists, PLM and enemy populations, library backgrounds | the header and state layouts, condition codes, the compression format, the room, setup and door code pointers mapped to `RoomRule`, `RoomSetup` and `DoorRule` |
| `tileset` (AF2) | the tileset table, tile tables, BG and CRE graphics, palettes, palette FX and animated tile lists | the tileset pointers, the CRE's placement, the planar layout, palette compression, the FX opcodes |
| `program` (A1) | every instruction list a family runs | opcode tables, argument layouts, the pointer-to-label map (verification only) |
| `actors` (AF3) | species headers, extended spritemaps, hitboxes, vulnerability and drop tables, graphics sets, enemy projectiles, Samus's projectile and sprite object tables | the species header, AI pointers mapped to `SpeciesKind`, the spritemap format, the graphics set layout |
| `screens` (AF4a, AF4b) | HUD tiles and pictures, area maps and station bitmaps, messages and the font, pause tilemaps | the tilemap word, the map's page layout, the message layout |
| `scenes` (AF5) | title, intro, cinematic, credits and menu data, Mode 7 data, demo input | the cinematic object header, the intro text format, the demo input format, the Mode 7 interleave |
| `sound` (A3) | samples, the sample directory, instruments, sequences, the sound effect tables, out of the sound banks | the sample codec, the upload block format, the sound chip's addresses, the sequence command bytes |
| `samus` | Samus's pose tables | below |

Each importer decodes the ROM's layout, writes the format, re-encodes it to the observed bytes
and compares (§1 point 3, sound excepted), and checks the relations between fields (decision
15). It writes the address map beside the content, for verification only (K1). Its tests run on
hand-made images and, in CI, on the ci-data ROM. Its registries are committed: an id and the
original's place it stands for hold no ROM bytes.

Samus's importer (`import::samus`) is the only code that knows the ROM's layout of her
records. It reads the decoded tables of `data::pose` (they keep their bank and address comments) and writes the
format:

| Format | From |
| --- | --- |
| the key and dress variant of each clip | the pose map (the 164 ids the domain reaches, decode notes "The pose map"), held once in `data::assets::samus::map`; nothing in the ROM names a pose by its parts |
| frames, holds and the clip's end | the pose's animation delay list (`data::pose::animation`), one frame per byte the animation reaches; the end from the instruction that ends the list |
| top and bottom sprites | the top and bottom half spritemap tables (`data::pose::spritemap`), first index by pose plus the frame |
| whether the bottom half is drawn | the bottom-half rule of draw Samus, by movement type, pose and frame, baked per frame |
| the origin's Y offset | the pose definition's Y offset, or the landing and transition offset tables, by the spritemap position rule, baked per frame |
| uploads | the animation definitions and tiles definitions (`data::pose::tiles`), with the bytes from the graphics banks |
| `chest` | the facing-forward drawing routine's immediate sprite, held here |
| `running` | the running delay list |

The pose map is a registry (§1 point 1), so it is `data`'s, `data::assets::samus::map`: the
engine's bridges ask it two questions while the engine still keeps a pose id (`key_of`,
`id_of`), the key of an id and the id that shows a key in a dress, and `game` may not ask
`import`.

## 6. Loading at runtime

`ContentSet::load(rom)` builds the content in memory from the player's ROM at startup; that
build is the authority. A cache of it, written with the `Canonical` codec to the user's cache
directory and keyed by the ROM's SHA-256, the format's schema hash and the importer's revision,
only saves time: any change to one of the three rebuilds it, and the content fingerprint checks
it. The cache holds only the vanilla import; overlays (D15's packs) apply after it loads. It is
never committed, and the Quality check that refuses ROM bytes covers it.

Loading time (T1) stays exact for vanilla content without the compressed layout leaving the
edge: the importer gives each record a named load cost from the original's streams, and an
edited record's cost comes from T1's fitted rule over its size.
