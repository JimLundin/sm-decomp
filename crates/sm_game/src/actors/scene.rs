//! Actor visual selection and compatible resources, separate from gameplay collision.
#![forbid(unsafe_code)]
use super::{
    content::{ActorLiveRecord, PoseId},
    media::{AppearanceId, FrameId, MaterialId, VisualResourceId},
};
use crate::geometry::Point;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorDrawMode {
    Base,
    Extended,
    ExtendedOffscreen,
}
/// Opaque adapter descriptor. Acquisition remains at its original effect boundary.
pub struct ActorVisualSource {
    pub bank: u8,
    pub address: u16,
    pub mode: ActorDrawMode,
}
pub enum ActorVisualSelection {
    Prepared(ResolvedActorVisual),
    Compatibility(ActorVisualSource),
    Unresolved {
        appearance: AppearanceId,
        reason: UnresolvedActorVisual,
        compatibility: ActorVisualSource,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnresolvedActorVisual {
    UnknownPose,
    UndeclaredPart { pose: PoseId, part: usize },
    IncompatibleVisualForm,
    Content(super::content::ActorContentError),
}
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedActorVisual {
    pub appearance: AppearanceId,
    pub pose: PoseId,
    pub form: ActorVisualForm,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ActorVisualForm {
    Sprites(Vec<SpritePiece>),
    Tilemap(TilemapVisual),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorPaletteMode {
    Normal,
    Flash,
    Frozen,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SpritePiece {
    pub frame: FrameId,
    pub resource: VisualResourceId,
    pub source_origin: Point,
    pub source_extent: Point,
    pub offset: Point,
    pub flip_x: bool,
    pub flip_y: bool,
    pub material: MaterialId,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TilemapVisual {
    pub resource: VisualResourceId,
    pub extent: Point,
    pub offset: Point,
    pub material: MaterialId,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ActorSceneRecord {
    pub visual: ResolvedActorVisual,
    pub position: Point,
    pub layer: u16,
    pub order: u64,
    pub palette: ActorPaletteMode,
}
pub struct ActorDrawRequest {
    pub visual: ActorVisualSelection,
    pub position: Point,
    pub palette: u16,
    pub tile: u16,
    pub palette_mode: ActorPaletteMode,
    pub mode: ActorDrawMode,
}
pub enum ActorTilemapSource {
    Compatibility {
        bank: u8,
        address: u16,
    },
    Prepared(TilemapVisual),
    Unresolved {
        appearance: AppearanceId,
        reason: UnresolvedActorVisual,
        bank: u8,
        address: u16,
    },
}
pub struct ActorTileChunkSource {
    pub record: ActorLiveRecord,
}
pub struct ActorTileDestination(pub u16);

pub fn select_visual(
    world: &crate::room_entry::world::WorldBinding,
    appearance: Option<AppearanceId>,
    bank: u8,
    map: u16,
    mode: ActorDrawMode,
) -> Result<ActorVisualSelection, super::content::ActorContentError> {
    let fallback = ActorVisualSource {
        bank,
        address: map,
        mode,
    };
    let Some(appearance) = appearance else {
        return Ok(ActorVisualSelection::Compatibility(fallback));
    };
    let Some(pose) = world.actors().pose_for_key(bank, map) else {
        return Ok(ActorVisualSelection::Unresolved {
            appearance,
            reason: UnresolvedActorVisual::UnknownPose,
            compatibility: fallback,
        });
    };
    let definition = &world
        .actors()
        .definitions()
        .expect("resolved prepared pose")
        .poses[&pose];
    world
        .actor_media()
        .resolve(appearance, pose, definition)
        .map(ActorVisualSelection::Prepared)
}

/// Extended drawing keeps the captured root pose and ordinal while each legacy part
/// reacquires its own map. The final position already includes the gameplay part offset.
pub fn select_part_visual(
    world: &crate::room_entry::world::WorldBinding,
    appearance: Option<AppearanceId>,
    pose: Option<PoseId>,
    part: usize,
    bank: u8,
    map: u16,
    mode: ActorDrawMode,
) -> Result<ActorVisualSelection, super::content::ActorContentError> {
    let fallback = ActorVisualSource {
        bank,
        address: map,
        mode,
    };
    let Some(appearance) = appearance else {
        return Ok(ActorVisualSelection::Compatibility(fallback));
    };
    let Some(pose) = pose else {
        return Ok(ActorVisualSelection::Unresolved {
            appearance,
            reason: UnresolvedActorVisual::UnknownPose,
            compatibility: fallback,
        });
    };
    let definition = world
        .actors()
        .definitions()
        .and_then(|definitions| definitions.poses.get(&pose))
        .ok_or(super::content::ActorContentError::MissingPose(pose))?;
    let Some(part_definition) = definition.parts.get(part) else {
        // Compatibility mutation can expose additional live parts without authored roles.
        return Ok(ActorVisualSelection::Unresolved {
            appearance,
            reason: UnresolvedActorVisual::UndeclaredPart { pose, part },
            compatibility: fallback,
        });
    };
    world
        .actor_media()
        .resolve_part(appearance, pose, part_definition.role, Point { x: 0, y: 0 })
        .map(ActorVisualSelection::Prepared)
}

pub fn pose_identity(source: &super::content::ActorPoseCapture) -> Option<PoseId> {
    if let Some(pose) = source.position {
        return Some(pose);
    }
    let keys = source.world.actors().keys()?;
    let mut matches = keys.poses.iter().filter_map(|(&pose, &(bank, offset))| {
        let address = (u32::from(bank) << 16) | u32::from(offset);
        (sm_data::cartridge::lorom_offset(address) as usize == source.record.base).then_some(pose)
    });
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}
impl ActorTileChunkSource {
    pub fn from_record(record: ActorLiveRecord) -> Self {
        Self { record }
    }
}
impl ActorTileDestination {
    pub fn from_legacy(destination: u16) -> Self {
        Self(destination)
    }
}

/// Tilemap copies keep the same appearance decision as sprite emission. An
/// unsupported live form remains explicit while preserving the compatible copy.
pub fn select_tilemap_source(
    world: &crate::room_entry::world::WorldBinding,
    appearance: Option<AppearanceId>,
    pose: Option<PoseId>,
    part: usize,
    bank: u8,
    address: u16,
) -> ActorTilemapSource {
    match select_part_visual(
        world,
        appearance,
        pose,
        part,
        bank,
        address,
        ActorDrawMode::Extended,
    ) {
        Ok(ActorVisualSelection::Compatibility(_)) => {
            ActorTilemapSource::Compatibility { bank, address }
        }
        Ok(ActorVisualSelection::Unresolved {
            appearance, reason, ..
        }) => ActorTilemapSource::Unresolved {
            appearance,
            reason,
            bank,
            address,
        },
        Ok(ActorVisualSelection::Prepared(visual)) => match visual.form {
            ActorVisualForm::Tilemap(tilemap) => ActorTilemapSource::Prepared(tilemap),
            ActorVisualForm::Sprites(_) => ActorTilemapSource::Unresolved {
                appearance: visual.appearance,
                reason: UnresolvedActorVisual::IncompatibleVisualForm,
                bank,
                address,
            },
        },
        Err(error) => match appearance {
            Some(appearance) => ActorTilemapSource::Unresolved {
                appearance,
                reason: UnresolvedActorVisual::Content(error),
                bank,
                address,
            },
            None => ActorTilemapSource::Compatibility { bank, address },
        },
    }
}
