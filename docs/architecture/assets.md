# Assets: our own format, imported from the ROM

MODEL §1 decision 4: the game's content gets a format of our own that names things by what
they are, and an importer at the edge reads the ROM's layout once and writes it. Decision 6:
the format looks like what a developer of a 2D pixel-art game expects, a **sprite sheet** and
**animation clips**, the shape an Aseprite export with tags or a texture atlas gives. This note
sets the format for Samus, the first asset; enemies, projectiles and rooms follow it later.

## The format

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

**The key** is the pose as the player sees it (`docs/behaviours/samus-motion.md` §2):

| Record | Holds |
| --- | --- |
| `Pose` | body, action (with the turn's kind and what a landing touched down from), facing, aim, gun, legs |
| `Dress` | what the sprite shows that the pose doesn't decide: how her spin is drawn (`SpinSprite`), her suit facing the screen (`Suit`) |
| `Worn` | which dress a clip is the variant for: any, one spin sprite, or one suit |

The key is the clean room's `clean_types::Pose`, which carries every part §2 lists;
`data::assets::samus::key` re-exports it beside the dress, so `data` depends on `clean_types`
(`tools/check_crate_rules.sh`).

`SamusAssets::clip(key, dress)` finds a pose's clip. A pose that has no sprites of its own is
shown by its nearest (the air turns' one sprite for down-forward and down is the down one,
the crouched turn's is the down-forward one, spec §2 "Aim in a turn"); the lookup applies
that rule, so the format holds each clip once.

The palette a sprite draws with is its entries' palette field; which colours that palette
holds (the suit's palette, the visor's cycle) is not part of Samus's assets yet.

A clip keeps one frame per step of the animation, the control steps included, because the
animation's frame number is still the engine's state: frame `n` of a clip is the engine's
animation frame `n`. Which pose a `HandOver` goes to stays in `data::pose::animation` for
now: it is motion's, and moves with the pose transitions.

## The importer

`data::assets::samus::import` is the only code that knows the ROM's layout for these records.
It reads the decoded tables of `data::pose` (they keep their bank and address comments) and
writes the format:

| Format | From |
| --- | --- |
| the key and dress variant of each clip | the pose map (the 164 ids the domain reaches, decode notes "The pose map"), held once here; nothing in the ROM names a pose by its parts |
| frames, holds and the clip's end | the pose's animation delay list (`data::pose::animation`), one frame per byte the animation reaches; the end from the instruction that ends the list |
| top and bottom sprites | the top and bottom half spritemap tables (`data::pose::spritemap`), first index by pose plus the frame |
| whether the bottom half is drawn | the bottom-half rule of draw Samus, by movement type, pose and frame, baked per frame |
| the origin's Y offset | the pose definition's Y offset, or the landing and transition offset tables, by the spritemap position rule, baked per frame |
| uploads | the animation definitions and tiles definitions (`data::pose::tiles`), with the bytes from the graphics banks |
| `chest` | the facing-forward drawing routine's immediate sprite, held here |
| `running` | the running delay list |

The importer also answers the bridge's two questions while the engine still keeps a pose id
(`import::map::key_of`, `import::map::id_of`):
the key of an id, and the id that shows a key in a dress.

Assets come from the player's ROM at load and are never committed.

## Out of scope for now

- Other sprites: enemies, projectiles, the HUD, PLMs. Their assets reuse `SpriteSheet`,
  `Clip`'s shape and `Hold`; each gets its own key type.
- Rooms, tilesets, palettes and sound.
- The poses outside the domain (shinespark, grapple, knockback, drained, elevator): no key
  yet, so no clip.
- The arm cannon, the charge flare and the speed echoes drawn with her.
