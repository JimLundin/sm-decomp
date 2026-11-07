//! The importer: Samus's sprite sheet and clips from the ROM, the only code that knows the
//! original's layout for them.
//!
//! It reads the decoded pose tables (her pose definitions, animation delay lists, half
//! spritemap tables and tile uploads) and bakes, frame by frame, what each of her poses
//! draws.

pub mod map;

use std::collections::BTreeMap;
use std::collections::HashMap;

use super::{
    Chest, Clip, ClipEnd, Frame, FrameNumber, FrameTiles, Hold, SamusAssets, SpriteId, SpriteSheet,
    TileUpload, TilesId, Unkeyed,
};
use crate::pose::animation::{
    AnimationDelays, AnimationError, AnimationFrame, DelayItem, DelayList, Instruction,
};
use crate::pose::spritemap::{Half, PositionOffsets, SamusSpritemaps, SpritemapError};
use crate::pose::tiles::{GRAPHICS_BANKS, Graphics, SamusTiles, TileDefinitionId};
use crate::pose::{MovementType, POSE_COUNT, PoseError, PoseId, Poses};
use crate::rom::{AddressError, Lorom};
use crate::spritemap::{SpriteAttributes, Spritemap};

/// The decoded tables the importer reads.
#[derive(Clone, Copy, Debug)]
pub struct Tables<'a> {
    /// The pose definitions.
    pub poses: &'a Poses,
    /// The animation delay lists.
    pub delays: &'a AnimationDelays,
    /// The half spritemap tables.
    pub spritemaps: &'a SamusSpritemaps,
    /// The tile uploads.
    pub tiles: &'a SamusTiles,
    /// The graphics they upload.
    pub graphics: &'a Graphics,
}

/// Why the assets don't import.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// The pose definitions don't decode.
    Poses(PoseError),
    /// The delay lists don't decode.
    Animation(AnimationError),
    /// The spritemaps don't decode.
    Spritemaps(SpritemapError),
    /// A table or the graphics aren't in the image.
    Address(AddressError),
    /// One of her poses has no definition, or no delay list.
    Missing(PoseId),
    /// One of her poses's delay list has no end a clip can name.
    Unended(PoseId),
    /// A frame uploads graphics that aren't in the image.
    Graphics(TileDefinitionId),
}

// The chest sprite is an immediate of the power suit's facing-forward bottom-half handler
// (bank `$90` log, `$90:868D`): tile $21, palette 4, priority 3, 7 pixels left of her and
// 17 up.
const CHEST: Chest = Chest {
    left: 7,
    up: 17,
    attributes: SpriteAttributes::from_word(0x3821),
};

// The poses the drawing rules name, by the original's numbers.
/// The facing-the-screen poses, first suit and the others.
const FACING_SCREEN: [u16; 2] = [0x00, 0x9B];
/// The first landing pose; the landings run on for four.
const FIRST_LANDING: u16 = 0xA4;
/// The first change-of-shape pose; they run on for twelve.
const FIRST_TRANSITION: u16 = 0x35;

/// Samus's pose assets, from the ROM.
///
/// # Errors
///
/// [`ImportError`] if a table doesn't decode or the graphics aren't in the image.
pub fn import(rom: Lorom<'_>) -> Result<SamusAssets, ImportError> {
    let poses = Poses::load(rom).map_err(ImportError::Poses)?;
    let delays = AnimationDelays::load(rom, &poses).map_err(ImportError::Animation)?;
    let spritemaps = SamusSpritemaps::load(rom).map_err(ImportError::Spritemaps)?;
    let tiles = SamusTiles::load(rom, &delays, &poses).map_err(ImportError::Address)?;
    let graphics = Graphics::load(rom, GRAPHICS_BANKS).map_err(ImportError::Address)?;
    from_tables(Tables {
        poses: &poses,
        delays: &delays,
        spritemaps: &spritemaps,
        tiles: &tiles,
        graphics: &graphics,
    })
}

/// Samus's pose assets, from the decoded tables.
///
/// # Errors
///
/// [`ImportError`] if one of her poses has no definition or delay list, or a frame's
/// graphics aren't there.
pub fn from_tables(tables: Tables<'_>) -> Result<SamusAssets, ImportError> {
    let mut pools = Pools::default();
    let clips = map::MAP
        .iter()
        .map(|entry| {
            let frames = frames(tables, entry.id, &mut pools)?;
            Ok(Clip {
                key: entry.key,
                worn: entry.worn,
                frames,
                end: end(
                    tables
                        .delays
                        .of(entry.id)
                        .ok_or(ImportError::Missing(entry.id))?,
                    entry.id,
                )?,
            })
        })
        .collect::<Result<_, ImportError>>()?;
    // The poses outside her domain that have a definition and a delay list to draw from.
    let unkeyed = (0..POSE_COUNT)
        .filter_map(|id| u16::try_from(id).ok().map(PoseId))
        .filter(|&id| {
            map::key_of(id).is_none()
                && tables.poses.get(id).is_some()
                && tables.delays.of(id).is_some()
        })
        .map(|id| {
            Ok(Unkeyed {
                id,
                frames: frames(tables, id, &mut pools)?,
            })
        })
        .collect::<Result<_, ImportError>>()?;
    Ok(SamusAssets {
        sheet: SpriteSheet {
            sprites: pools.sprites,
            uploads: pools.uploads,
        },
        clips,
        unkeyed,
        running: holds(tables.delays.running()).collect(),
        chest: CHEST,
    })
}

/// The sprite lists and uploads gathered so far, each once.
#[derive(Default)]
struct Pools {
    sprites: Vec<Spritemap>,
    sprite_ids: HashMap<Spritemap, SpriteId>,
    uploads: Vec<TileUpload>,
    upload_ids: BTreeMap<TileDefinitionId, TilesId>,
}

impl Pools {
    fn sprite(&mut self, map: &Spritemap) -> SpriteId {
        if let Some(&id) = self.sprite_ids.get(map) {
            return id;
        }
        let id = SpriteId(u16::try_from(self.sprites.len()).unwrap_or(u16::MAX));
        self.sprites.push(map.clone());
        self.sprite_ids.insert(map.clone(), id);
        id
    }

    fn upload(
        &mut self,
        tables: Tables<'_>,
        definition: TileDefinitionId,
    ) -> Result<TilesId, ImportError> {
        if let Some(&id) = self.upload_ids.get(&definition) {
            return Ok(id);
        }
        let upload = upload(tables, definition).ok_or(ImportError::Graphics(definition))?;
        let id = TilesId(u16::try_from(self.uploads.len()).unwrap_or(u16::MAX));
        self.uploads.push(upload);
        self.upload_ids.insert(definition, id);
        Ok(id)
    }
}

/// The bytes a tiles definition uploads: part 1's, a size of 0 being the DMA's whole
/// 10000h, then part 2's.
fn upload(tables: Tables<'_>, definition: TileDefinitionId) -> Option<TileUpload> {
    let definition = tables.tiles.definition(definition)?;
    let first = match definition.first {
        0 => 0x1_0000,
        size => usize::from(size),
    };
    let second = usize::from(definition.second);
    Some(TileUpload {
        first: tables.graphics.bytes(definition.source, first)?.to_vec(),
        second: if second == 0 {
            Vec::new()
        } else {
            tables
                .graphics
                .bytes(definition.second_source(), second)?
                .to_vec()
        },
    })
}

/// A delay list's holds, one per byte.
fn holds(list: &DelayList) -> impl Iterator<Item = Hold> + '_ {
    (0..list.len()).filter_map(|at| {
        let frame = AnimationFrame(u16::try_from(at).ok()?);
        Some(match list.at(frame)? {
            DelayItem::Delay(frames) => Hold::Shows(frames),
            DelayItem::Instruction(_) => Hold::Control(list.byte(frame)?),
        })
    })
}

/// What a clip does after its last shown frame: the instruction that ends its delay list,
/// walked from the start as the list is laid out, an instruction and its operands at a time.
fn end(list: &DelayList, id: PoseId) -> Result<ClipEnd, ImportError> {
    let mut at = 0_u16;
    while let Some(item) = list.at(AnimationFrame(at)) {
        let (size, end) = match item {
            DelayItem::Delay(_)
            | DelayItem::Instruction(
                Instruction::NoOp(_)
                | Instruction::LoopIfHealthy
                | Instruction::DrainedFalling
                | Instruction::WallJump,
            ) => (1, None),
            DelayItem::Instruction(Instruction::Loop) => (
                1,
                Some(ClipEnd::Loop {
                    from: FrameNumber(0),
                }),
            ),
            DelayItem::Instruction(Instruction::GoBack(back)) => (
                2,
                Some(ClipEnd::Loop {
                    from: FrameNumber(at.wrapping_sub(u16::from(back))),
                }),
            ),
            DelayItem::Instruction(
                Instruction::Transition(_) | Instruction::AutoJumpTransition(_),
            ) => (2, Some(ClipEnd::HandOver)),
            DelayItem::Instruction(Instruction::BySpeed(_)) => (3, Some(ClipEnd::HandOver)),
            DelayItem::Instruction(Instruction::ByItems { .. }) => (5, Some(ClipEnd::HandOver)),
            DelayItem::Instruction(Instruction::ByItemsAndSpeed { .. }) => {
                (7, Some(ClipEnd::HandOver))
            }
        };
        at = at.wrapping_add(size);
        if let Some(end) = end
            && usize::from(at) == list.len()
        {
            return Ok(end);
        }
    }
    Err(ImportError::Unended(id))
}

/// Pose `id`'s frames, one per byte of its delay list.
fn frames(tables: Tables<'_>, id: PoseId, pools: &mut Pools) -> Result<Vec<Frame>, ImportError> {
    let definition = tables.poses.get(id).ok_or(ImportError::Missing(id))?;
    let list = tables.delays.of(id).ok_or(ImportError::Missing(id))?;
    holds(list)
        .enumerate()
        .map(|(at, hold)| {
            let frame = AnimationFrame(u16::try_from(at).unwrap_or(u16::MAX));
            let mut sprite = |half| {
                tables
                    .spritemaps
                    .index(half, id, frame)
                    .and_then(|index| tables.spritemaps.spritemap(index))
                    .map(|map| pools.sprite(map))
            };
            let top = sprite(Half::Top);
            let lower = bottom_half(definition.movement, id, frame);
            let bottom = match lower {
                BottomHalf::Drawn | BottomHalf::WithChest => sprite(Half::Bottom),
                BottomHalf::Hidden => None,
            };
            let tiles = match tables.tiles.frame(id, frame) {
                Some(uploads) => Some(FrameTiles {
                    top: pools.upload(tables, uploads.top)?,
                    bottom: uploads
                        .bottom
                        .map(|bottom| pools.upload(tables, bottom))
                        .transpose()?,
                }),
                None => None,
            };
            Ok(Frame {
                hold,
                top,
                bottom,
                chest: lower == BottomHalf::WithChest,
                origin_y: origin_y(
                    definition.movement,
                    id,
                    frame,
                    definition.y_offset,
                    &tables.spritemaps.offsets,
                ),
                tiles,
            })
        })
        .collect()
}

/// A frame number the original compares as a signed word.
const fn signed(frame: AnimationFrame) -> i16 {
    i16::from_le_bytes(frame.0.to_le_bytes())
}

// Calculate Samus spritemap position (bank `$90` log, `$90:8C1F`) dispatches on the
// movement type (`$90:8C5C`): standing `$90:8CC3`, the transitions `$90:8D3C`, shinespark
// and drained `$90:8D98`, the others `$90:8C94`.
/// Pixels the spritemaps' origin sits below her position, for `pose` at `frame`.
fn origin_y(
    movement: MovementType,
    pose: PoseId,
    frame: AnimationFrame,
    y_offset: i8,
    offsets: &PositionOffsets,
) -> i16 {
    let at = usize::from(frame.0);
    let usual = i16::from(y_offset).wrapping_neg();
    match movement {
        MovementType::Standing if FACING_SCREEN.contains(&pose.0) => {
            if signed(frame) >= 2 {
                -1
            } else {
                usual
            }
        }
        MovementType::Standing if (FIRST_LANDING..FIRST_LANDING + 4).contains(&pose.0) => {
            let row = usize::from(pose.0.wrapping_sub(FIRST_LANDING)) * 4;
            offsets
                .landing
                .get(row.wrapping_add(at))
                .map_or(usual, |&up| i16::from(up).wrapping_neg())
        }
        MovementType::CrouchingTransition
            if (FIRST_TRANSITION..=FIRST_TRANSITION + 0xB).contains(&pose.0) =>
        {
            let row = usize::from(pose.0.wrapping_sub(FIRST_TRANSITION)) * 2;
            offsets
                .transition
                .get(row.wrapping_add(at))
                .map_or(usual, |&down| i16::from(down))
        }
        MovementType::Shinespark if matches!(pose.0, 0xE8 | 0xE9) => offsets
            .drained
            .get(at)
            .map_or(usual, |&down| i16::from(down)),
        MovementType::Shinespark if matches!(pose.0, 0xEA | 0xEB) && signed(frame) >= 5 => -3,
        MovementType::Standing
        | MovementType::Running
        | MovementType::NormalJumping
        | MovementType::SpinJumping
        | MovementType::MorphBallOnGround
        | MovementType::Crouching
        | MovementType::Falling
        | MovementType::Unused07
        | MovementType::MorphBallFalling
        | MovementType::Unused09
        | MovementType::KnockbackOrCrystalFlashEnding
        | MovementType::Unused0B
        | MovementType::Unused0C
        | MovementType::Unused0D
        | MovementType::TurningAroundOnGround
        | MovementType::CrouchingTransition
        | MovementType::Moonwalking
        | MovementType::SpringBallOnGround
        | MovementType::SpringBallInAir
        | MovementType::SpringBallFalling
        | MovementType::WallJumping
        | MovementType::RanIntoWall
        | MovementType::Grappling
        | MovementType::TurningAroundJumping
        | MovementType::TurningAroundFalling
        | MovementType::DamageBoost
        | MovementType::GrabbedByDraygon
        | MovementType::Shinespark => usual,
    }
}

/// Whether a frame draws her bottom half.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BottomHalf {
    Drawn,
    WithChest,
    Hidden,
}

// Draw Samus's bottom-half handler by movement type (bank `$90` log, `$90:864E`); the
// handlers' own rules are noted by address.
/// Whether `pose` at `frame` draws her bottom half, and the chest sprite before it.
fn bottom_half(movement: MovementType, pose: PoseId, frame: AnimationFrame) -> BottomHalf {
    let shown = |drawn: bool| {
        if drawn {
            BottomHalf::Drawn
        } else {
            BottomHalf::Hidden
        }
    };
    let frame_number = signed(frame);
    match movement {
        // `$90:868D`.
        MovementType::Standing if pose.0 == FACING_SCREEN[0] => BottomHalf::WithChest,
        MovementType::Standing
        | MovementType::Running
        | MovementType::NormalJumping
        | MovementType::Crouching
        | MovementType::Falling
        | MovementType::Unused0B
        | MovementType::Unused0C
        | MovementType::TurningAroundOnGround
        | MovementType::Moonwalking
        | MovementType::RanIntoWall
        | MovementType::Grappling
        | MovementType::TurningAroundJumping
        | MovementType::TurningAroundFalling
        | MovementType::GrabbedByDraygon => BottomHalf::Drawn,
        MovementType::MorphBallOnGround
        | MovementType::Unused07
        | MovementType::MorphBallFalling
        | MovementType::Unused09
        | MovementType::SpringBallOnGround
        | MovementType::SpringBallInAir
        | MovementType::SpringBallFalling => BottomHalf::Hidden,
        // `$90:86C6`: the spin's first frame, its wind-down, and the poses that don't spin.
        MovementType::SpinJumping => shown(
            matches!(pose.0, 0x81 | 0x82 | 0x1B | 0x1C) || frame_number == 0 || frame_number >= 11,
        ),
        // `$90:86EE`.
        MovementType::KnockbackOrCrystalFlashEnding => {
            shown(!matches!(pose.0, 0xD7 | 0xD8) || frame_number >= 3)
        }
        // `$90:874C`.
        MovementType::Unused0D => shown(!matches!(pose.0, 0x65 | 0x66) || frame_number < 1),
        // `$90:870C`: of the transitions, only crouching and standing keep a bottom half.
        MovementType::CrouchingTransition => shown(if pose.0 >= 0xF1 {
            true
        } else if pose.0 >= 0xDB {
            frame_number == if pose.0 < 0xDD { 0 } else { 2 }
        } else {
            matches!(pose.0, 0x35 | 0x36 | 0x3B | 0x3C)
        }),
        // `$90:8768`: not while she spins off the wall.
        MovementType::WallJumping => shown(!(3..13).contains(&frame_number)),
        // `$90:877C`.
        MovementType::DamageBoost => shown(!(2..9).contains(&frame_number)),
        // `$90:8790`.
        MovementType::Shinespark => shown(if pose.0 < 0xCF {
            !matches!(pose.0, 0xCB | 0xCC)
        } else {
            !matches!(pose.0, 0xE8 | 0xE9) || frame_number >= 2
        }),
    }
}

#[cfg(test)]
mod tests;
