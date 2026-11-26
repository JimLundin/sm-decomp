//! Convert the two standing clips into body pieces and references to their imported tiles.

use std::collections::BTreeMap;

use data::assets::samus::map;
use data::pose::animation::{AnimationDelays, AnimationFrame, DelayItem};
use data::pose::spritemap::{Half, SamusSpritemaps};
use data::pose::tiles::{GRAPHICS_BANKS, Graphics, SamusTiles, TileDefinition};
use data::pose::{PoseId, Poses};
use data::rom::Lorom;
use data::spritemap::{SpriteSize, Spritemap};
use samus_appearance::Step;
use samus_appearance::{
    ArtDisplacement, BodyHalf, BodyPiece, ElevatorArt, ObjectSize, ObjectStyle, StandingLook,
    StandingPoseArt, StandingStepArt,
};
use types::{Pair, Side};

use crate::state::video::{RomWords, StandingStrips};

/// ROM resources for the locked standing picture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Art {
    pub(crate) pictures: ElevatorArt<RomWords>,
    pub(crate) graphics: Graphics,
}

/// The required standing art is unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Pose, delay, sprite or upload tables failed to decode.
    Tables,
    /// A displayed step is missing one of its body halves or a valid tile strip.
    Picture,
}

impl Art {
    /// Read the picture and tile assets consumed by the standing rule.
    ///
    /// # Errors
    ///
    /// The player's ROM must have the two forward standing clips and all their visible art.
    pub fn load(rom: Lorom<'_>) -> Result<Self, Fault> {
        let poses = Poses::load(rom).map_err(|_| Fault::Tables)?;
        let delays = AnimationDelays::load(rom, &poses).map_err(|_| Fault::Tables)?;
        let maps = SamusSpritemaps::load(rom).map_err(|_| Fault::Tables)?;
        let tiles = SamusTiles::load(rom, &delays, &poses).map_err(|_| Fault::Tables)?;
        let graphics = Graphics::load(rom, GRAPHICS_BANKS).map_err(|_| Fault::Tables)?;
        let standing = |side| {
            let id = map::MAP
                .iter()
                .find(|entry| {
                    StandingLook::from_pose(entry.key, Step(0))
                        .is_some_and(|look| look.side == side)
                })
                .ok_or(Fault::Picture)?
                .id;
            let pose = poses.get(id).ok_or(Fault::Picture)?;
            let list = delays.of(id).ok_or(Fault::Picture)?;
            let steps = list
                .steps()
                .enumerate()
                .filter(|(_, (_, item))| matches!(item, DelayItem::Delay(_)))
                .map(|(index, (frame, _))| {
                    let step = Step(u8::try_from(index).map_err(|_| Fault::Picture)?);
                    let art =
                        step_art(id, frame, &maps, &tiles, &graphics).ok_or(Fault::Picture)?;
                    Ok((step, art))
                })
                .collect::<Result<BTreeMap<_, _>, Fault>>()?;
            if steps.is_empty() {
                return Err(Fault::Picture);
            }
            Ok(StandingPoseArt {
                displacement: ArtDisplacement(i32::from(pose.y_offset).wrapping_neg()),
                steps,
            })
        };
        Ok(Self {
            pictures: ElevatorArt {
                left: standing(Side::Left)?,
                right: standing(Side::Right)?,
            },
            graphics,
        })
    }

    pub(crate) fn accepts(&self, strips: StandingStrips) -> bool {
        if strips.upper_first.is_none() || strips.lower_first.is_none() {
            return false;
        }
        [
            strips.upper_first,
            strips.upper_second,
            strips.lower_first,
            strips.lower_second,
        ]
        .into_iter()
        .flatten()
        .all(|strip| {
            (1..=128).contains(&strip.words)
                && self
                    .graphics
                    .bytes(strip.from, usize::from(strip.words).wrapping_mul(2))
                    .is_some()
        })
    }
}

fn step_art(
    pose: PoseId,
    frame: AnimationFrame,
    maps: &SamusSpritemaps,
    tiles: &SamusTiles,
    graphics: &Graphics,
) -> Option<StandingStepArt<RomWords>> {
    let definitions = tiles.frame(pose, frame)?;
    let half = |kind, definition| {
        let map = maps.spritemap(maps.index(kind, pose, frame)?)?;
        half_art(map, *tiles.definition(definition)?, graphics)
    };
    Some(StandingStepArt {
        upper: half(Half::Top, definitions.top)?,
        lower: half(Half::Bottom, definitions.bottom?)?,
    })
}

fn half_art(
    map: &Spritemap,
    definition: TileDefinition,
    graphics: &Graphics,
) -> Option<BodyHalf<RomWords>> {
    let strip = |from, bytes: u16| {
        if bytes == 0 || bytes > 256 || !bytes.is_multiple_of(2) {
            return None;
        }
        graphics.bytes(from, usize::from(bytes))?;
        Some(RomWords {
            from,
            words: bytes / 2,
        })
    };
    let pieces = map
        .entries
        .iter()
        .map(|entry| {
            let attributes = entry.attributes;
            BodyPiece {
                position: Pair {
                    x: i32::from(entry.x),
                    y: i32::from(entry.y.cast_signed()),
                },
                style: ObjectStyle {
                    tile: attributes.tile,
                    palette: attributes.palette,
                    size: match entry.size {
                        SpriteSize::Small => ObjectSize::Small,
                        SpriteSize::Large => ObjectSize::Large,
                    },
                    priority: attributes.priority,
                    flips: Pair {
                        x: attributes.flip_x,
                        y: attributes.flip_y,
                    },
                },
            }
        })
        .collect();
    Some(BodyHalf {
        pieces,
        first_strip: strip(definition.source, definition.first)?,
        second_strip: if definition.second == 0 {
            None
        } else {
            Some(strip(definition.second_source(), definition.second)?)
        },
    })
}
