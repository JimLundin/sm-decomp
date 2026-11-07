# Assets: our own format, imported from the ROM

MODEL §1 decision 4: the game's content gets a format of our own that names things by what
they are, and an importer at the edge reads the ROM's layout once and writes it. This note
sets the format for Samus's poses, the first asset; enemies, projectiles and rooms follow it
later.

## The format

`data::assets::samus` holds the records. Nothing in them is a ROM address, a pose id or a
table index of the original's.

**The key** is the pose as the player sees it (`docs/behaviours/samus-motion.md` §2):

| Record | Holds |
| --- | --- |
| `PoseKey` | body, action (with the turn's kind and what a landing touched down from), facing, aim, gun, legs |
| `Dress` | what the sprite shows that the pose doesn't decide: how her spin is drawn (`SpinSprite`), her suit facing the screen (`Suit`) |
| `Worn` | when an asset is the one shown: whatever she wears, with one spin sprite, or in one suit |

`PoseKey` is defined once, in `data::assets::samus::key`, mirroring §2. It is replaced by
`clean_types::Pose` when the clean room's round 6 lands. `data` depends on `canonical` alone
today (`tools/check_crate_rules.sh`), so that switch needs the crate rule to let `data`
depend on `clean_types`; it is not made yet.

`SamusAssets::pose(key, dress)` finds a pose's asset. A pose that has no sprite of its own is
shown by its nearest (the air turns' one sprite for down-forward and down is the down one,
the crouched turn's is the down-forward one, spec §2 "Aim in a turn"); the lookup applies
that rule, so the format holds each sprite once.

**What is drawn**, by pose:

| Record | Holds |
| --- | --- |
| `PoseAsset` | its key and `Worn`, and its frames in animation order |
| `Frame` | how long it holds (`Hold`), the top half's sprite, the bottom half's, whether the chest sprite goes before the bottom half, the origin's Y offset from her position, and the tile upload |
| `Hold` | `Shows(n)`: shown `n` frames; `Control(byte)`: a step of the animation's control (loop, go back, hand over to another pose), which motion runs; the byte is what the timer is set from |
| `SpriteId` → `Spritemap` | a pool of sprite lists, each list's sprites with their offsets, size, tile, palette, priority and flips; frames share them |
| `TilesId` → `TileUpload` | a pool of the graphics a frame uploads, its two parts' bytes |
| `chest` | the one sprite the first suit's facing-the-screen pose adds, and where it sits from her position |
| `running` | the holds a run uses instead of its own while she has running momentum |

The palette a sprite draws with is its entries' palette field; which colours that palette
holds (the suit's palette, the visor's cycle) is not part of the pose asset yet.

Frames are kept one per step of the animation, the control steps included, because the
animation's frame number is still the engine's state: frame `n` of an asset is the engine's
animation frame `n`. Where the animation hands over to another pose, the target stays in
`data::pose::animation` for now: it is motion's, and moves with the pose transitions.

## The importer

`data::assets::samus::import` is the only code that knows the ROM's layout for these records.
It reads the decoded tables of `data::pose` (they keep their bank and address comments) and
writes the format:

| Format | From |
| --- | --- |
| the key of each pose | the pose map (the 164 ids the domain reaches, decode notes "The pose map"), held once here; nothing in the ROM names a pose by its parts |
| frames and holds | the pose's animation delay list (`data::pose::animation`), one frame per byte the animation reaches |
| top and bottom sprites | the top and bottom half spritemap tables (`data::pose::spritemap`), first index by pose plus the frame |
| whether the bottom half is drawn | the bottom-half rule of draw Samus, by movement type, pose and frame, baked per frame |
| the origin's Y offset | the pose definition's Y offset, or the landing and transition offset tables, by the spritemap position rule, baked per frame |
| tiles | the animation definitions and tiles definitions (`data::pose::tiles`), with the bytes from the graphics banks |
| `chest` | the facing-forward drawing routine's immediate sprite, held here |
| `running` | the running delay list |

The importer also answers the bridge's two questions while the engine still keeps a pose id:
the key of an id, and the id that shows a key in a dress.

Assets come from the player's ROM at load and are never committed.

## Out of scope for now

- Other sprites: enemies, projectiles, the HUD, PLMs. Their assets reuse `Spritemap`,
  `TileUpload` and `Hold`; each gets its own key type.
- Rooms, tilesets, palettes and sound.
- The poses outside the domain (shinespark, grapple, knockback, drained, elevator): no key
  yet, so no asset.
- The arm cannon, the charge flare and the speed echoes drawn with her.
