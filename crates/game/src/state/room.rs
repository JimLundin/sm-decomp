//! Everything a room transition resets, in one struct (D4): replacing it is the reset.

use canonical::Canonical;
use data::id::{DoorwayId, RoomId, SpeciesId};

use super::samus::{Fixed, Point};
use super::slots::{Cursor, FlagSet, Frames, SlotStore};

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

/// An enemy property (bank $A0 log, the slot's properties word), in bit order from bit 0.
// open: the low byte's meanings; the bank $A0 routines that read them name them.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyProperty {
    /// Bit 0, open.
    Unnamed0,
    /// Bit 1, open.
    Unnamed1,
    /// Bit 2, open.
    Unnamed2,
    /// Bit 3, open.
    Unnamed3,
    /// Bit 4, open.
    Unnamed4,
    /// Bit 5, open.
    Unnamed5,
    /// Bit 6, open.
    Unnamed6,
    /// Bit 7, open.
    Unnamed7,
    /// Not drawn.
    Invisible,
    /// Deleted: the next determine pass frees the slot.
    Deleted,
    /// Projectiles and Samus pass through it.
    Intangible,
    /// Processed while off screen.
    ProcessedOffScreen,
    /// Stops the plasma beam.
    BlocksPlasmaBeam,
    /// Runs its instruction list.
    ProcessesInstructions,
    /// Comes back when the room reloads after it was killed.
    RespawnsIfKilled,
    /// Solid to Samus.
    SolidToSamus,
}

/// An enemy extra property (the slot's extra properties word), in bit order from bit 0.
// open: the meanings other than the spritemap format; their readers name them.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyExtraProperty {
    /// Bit 0, open.
    Unnamed0,
    /// Bit 1, open.
    Unnamed1,
    /// Its spritemaps are in the extended format (bank $A0 log, enemy drawing).
    ExtendedSpritemap,
}

/// The AI routine the main loop runs for it instead of the main AI (bank $A0 log, the slot's
/// AI handler bits): none set runs the main AI.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiHandler {
    /// Grappled.
    Grapple,
    /// Hurt.
    Hurt,
    /// Frozen.
    Frozen,
}

/// An index into the sprite palettes, as the slot's palette word holds it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaletteIndex(pub u16);

/// An index into the enemy tiles in VRAM, as the slot's tiles word holds it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TilesIndex(pub u16);

/// The fields every enemy slot has, whatever occupies it (bank $A0 log: the slot's first
/// 48 bytes and its two parameters).
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
    /// Its properties.
    pub properties: FlagSet<EnemyProperty>,
    /// Its extra properties.
    pub extra_properties: FlagSet<EnemyExtraProperty>,
    /// The AI routines overriding its main AI.
    pub ai_handlers: FlagSet<AiHandler>,
    /// Health left.
    pub health: u16,
    /// A general timer its AI and instructions count.
    pub timer: u16,
    /// Its instruction list; the delay is the instruction timer.
    pub cursor: Cursor<Enemy>,
    /// Its palette.
    pub palette: PaletteIndex,
    /// Its tiles.
    pub tiles: TilesIndex,
    /// The layer it draws on.
    pub layer: u16,
    /// Frames of the hurt flash left.
    pub flash: Frames,
    /// Frames frozen left.
    pub frozen: Frames,
    /// Frames of invincibility left.
    pub invincibility: Frames,
    /// Frames of shaking left.
    pub shake: Frames,
    /// Frames since it spawned, wrapping.
    pub frame_counter: u16,
    /// The population's first species-defined parameter.
    pub parameter_1: u16,
    /// The population's second species-defined parameter.
    pub parameter_2: u16,
}

/// The six words the original gives each enemy's AI.
// open: each species' names for them come with its AI lane (C7: the elevator and the face
// blocks, C8: the eye); until then they are the original's words in slot order.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AiVariables {
    /// The slot's first AI word.
    pub a: u16,
    /// The second.
    pub b: u16,
    /// The third.
    pub c: u16,
    /// The fourth.
    pub d: u16,
    /// The fifth.
    pub e: u16,
    /// The sixth, which some species use for a pre-instruction.
    pub f: u16,
}

/// A species' AI state (D14): one variant per species slice 0 runs.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyBody {
    /// One of the Morph Ball Room eye's two parts.
    MorphBallEye {
        /// Its AI words.
        variables: AiVariables,
    },
    /// The elevator platform.
    Elevator {
        /// Its AI words.
        variables: AiVariables,
    },
    /// A Blue Brinstar face block.
    FaceBlock {
        /// Its AI words.
        variables: AiVariables,
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
