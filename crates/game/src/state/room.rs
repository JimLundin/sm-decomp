//! Everything a room transition resets, in one struct (D4): replacing it is the reset.

use canonical::Canonical;
use data::id::{DoorwayId, RoomId, SpeciesId};

use super::samus::{Fixed, Point};
use super::slots::{Cursor, Frames, SlotStore};

/// PLM slots (bank $84 log: the PLM tables hold 40 entries).
pub const PLM_SLOTS: usize = 40;
/// Enemy slots (bank $A0 log: the enemy table holds 32 entries).
pub const ENEMY_SLOTS: usize = 32;

/// A block's place in the room's level data, row-major from the top left.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockIndex(pub u16);

/// What a block does when something touches it.
// open: the full set of the original's sixteen block types and what their BTS means for
// each (bank $94 block collision and the BTS tables); the decoder's level codec decides it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlockKind {
    /// Nothing to collide with.
    #[default]
    Air,
    /// Solid.
    Solid,
    /// A slope, its shape in the BTS.
    Slope,
    /// A door's edge, the doorway in the BTS.
    Door,
    /// Answers to the next block over, the offset in the BTS.
    Extend,
}

/// One block of level data.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Block {
    /// Its behaviour.
    pub kind: BlockKind,
    /// The tilemap block it draws, flips included.
    pub tile: u16,
    /// The block's argument: a slope shape, a doorway, an offset.
    pub bts: u8,
}

/// A screen's scroll permission.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollArea {
    /// The camera may not enter.
    #[default]
    Red,
    /// The camera may enter; the HUD hides its bottom.
    Blue,
    /// The camera may enter freely.
    Green,
}

/// The camera: the playfield's top-left corner in the room.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Camera {
    /// Rightward.
    pub x: Fixed,
    /// Downward.
    pub y: Fixed,
}

/// The PLM family marker.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Plm;

/// The fields every PLM slot has, whatever occupies it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlmCommon {
    /// The block it sits on.
    pub block: BlockIndex,
    /// Its instruction list.
    pub cursor: Cursor<Plm>,
    /// Its room argument.
    pub argument: u16,
}

/// A PLM's typed body.
// open: the kinds slice 0's rooms place, as the decoder reads the Morph Ball Room's PLM
// population; each kind's fields come from its bank $84 setup and instructions.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlmBody {
    /// A door cap.
    DoorCap {
        /// Shots taken, for caps that need more than one.
        hits: u8,
    },
    /// An item.
    Item {
        /// Whether it hides in a shot block until revealed.
        hidden: bool,
    },
}

/// The enemy family marker.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Enemy;

/// The fields every enemy slot has, whatever occupies it.
// open: the enemy property and extra-property flag sets, decided by the bank $A0 enemy
// processing that reads them.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnemyCommon {
    /// Its species.
    pub species: SpeciesId,
    /// Its centre.
    pub position: Point,
    /// Half its width.
    pub radius_x: u16,
    /// Half its height.
    pub radius_y: u16,
    /// Health left.
    pub health: u16,
    /// Frames of the hit flash and invincibility left.
    pub invincibility: Frames,
    /// Frames frozen left.
    pub frozen: Frames,
    /// Its instruction list.
    pub cursor: Cursor<Enemy>,
}

/// Which way a crawler goes round its surface.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Turn {
    /// Clockwise.
    #[default]
    Clockwise,
    /// Anticlockwise.
    Anticlockwise,
}

/// A species' AI state (D14): one variant per species slice 0 runs.
// open: the species in slice 0's rooms (the decoder's enemy population for them) and each
// species' states, from its AI bank's log.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyBody {
    /// A wall crawler.
    Crawler {
        /// Its direction round the surface.
        turn: Turn,
    },
}

/// Everything room-scoped.
// open: FX, enemy projectiles, sprite objects, Samus's projectiles and the visual effects
// (HDMA objects, palette FX, animated tiles, layer blending) join with their domains.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct RoomScoped {
    /// The room.
    pub room: RoomId,
    /// The doorway Samus came in by, if she came in by one.
    pub entered_by: Option<DoorwayId>,
    /// The level data, as PLMs and collisions have changed it.
    pub blocks: Vec<Block>,
    /// One per screen, row-major.
    pub scroll_areas: Vec<ScrollArea>,
    /// The camera.
    pub camera: Camera,
    /// The PLMs.
    pub plms: SlotStore<PlmCommon, PlmBody, PLM_SLOTS>,
    /// The enemies.
    pub enemies: SlotStore<EnemyCommon, EnemyBody, ENEMY_SLOTS>,
}
