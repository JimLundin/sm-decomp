//! Everything a room transition resets, in one struct (D4): replacing it is the reset.

use canonical::Canonical;
use data::id::{DoorwayId, RoomId, SpeciesId};
use data::room::{RoomState, Screens};

pub use data::level::{
    Block, BlockKind, BombableBts, Bts, DoorIndex, Offset, PlmChoice, ScrollArea, Slope, Tile,
    TileNumber,
};

use super::samus::{Fixed, Point};
use super::slots::{Cursor, Flag, FlagSet, Frames, SlotStore};

/// PLM slots (bank $84 log: the PLM tables hold 40 entries).
pub const PLM_SLOTS: usize = 40;
/// Enemy slots (bank $A0 log: the enemy table holds 32 entries).
pub const ENEMY_SLOTS: usize = 32;

/// A block's place in the room's level data, row-major from the top left.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockIndex(pub u16);

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
// open: the meanings of bits 1 and 3 to 14; their readers name them.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyExtraProperty {
    /// Paralysed by the grapple beam: the main enemy routine runs no AI for it (bank $A0
    /// log, `$A0:904F`).
    Paralysed,
    /// Bit 1, open.
    Unnamed1,
    /// Its spritemaps are in the extended format (bank $A0 log, enemy drawing).
    ExtendedSpritemap,
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
    /// Bit 8, open.
    Unnamed8,
    /// Bit 9, open.
    Unnamed9,
    /// Bit 10, open.
    Unnamed10,
    /// Bit 11, open.
    Unnamed11,
    /// Bit 12, open.
    Unnamed12,
    /// Bit 13, open.
    Unnamed13,
    /// Bit 14, open.
    Unnamed14,
    /// The instruction list moved on to a new spritemap this frame: processing the
    /// instructions sets it when the instruction timer runs out and clears it otherwise
    /// (bank $A0 log, `$A0:C2A6`, `$A0:C2B1`); extended-spritemap drawing reads it.
    GraphicUpdated,
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

impl Flag for EnemyProperty {
    fn bit(self) -> u32 {
        let place = match self {
            Self::Unnamed0 => 0,
            Self::Unnamed1 => 1,
            Self::Unnamed2 => 2,
            Self::Unnamed3 => 3,
            Self::Unnamed4 => 4,
            Self::Unnamed5 => 5,
            Self::Unnamed6 => 6,
            Self::Unnamed7 => 7,
            Self::Invisible => 8,
            Self::Deleted => 9,
            Self::Intangible => 10,
            Self::ProcessedOffScreen => 11,
            Self::BlocksPlasmaBeam => 12,
            Self::ProcessesInstructions => 13,
            Self::RespawnsIfKilled => 14,
            Self::SolidToSamus => 15,
        };
        1 << place
    }
}

impl Flag for EnemyExtraProperty {
    fn bit(self) -> u32 {
        let place = match self {
            Self::Paralysed => 0,
            Self::Unnamed1 => 1,
            Self::ExtendedSpritemap => 2,
            Self::Unnamed3 => 3,
            Self::Unnamed4 => 4,
            Self::Unnamed5 => 5,
            Self::Unnamed6 => 6,
            Self::Unnamed7 => 7,
            Self::Unnamed8 => 8,
            Self::Unnamed9 => 9,
            Self::Unnamed10 => 10,
            Self::Unnamed11 => 11,
            Self::Unnamed12 => 12,
            Self::Unnamed13 => 13,
            Self::Unnamed14 => 14,
            Self::GraphicUpdated => 15,
        };
        1 << place
    }
}

impl Flag for AiHandler {
    fn bit(self) -> u32 {
        let place = match self {
            Self::Grapple => 0,
            Self::Hurt => 1,
            Self::Frozen => 2,
        };
        1 << place
    }
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

/// The elevator platform's AI words (bank $A3 log, `$A3:94E6` to `$A3:9612`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Elevator {
    /// Where it stops on entering the room: its placed Y, which its init keeps (`$A3:9501`).
    pub target_y: u16,
    /// The other five words, which its AI never reads or writes: they keep what the spawn
    /// left, in slot order.
    pub unused: [u16; 5],
}

/// Which way an elevator goes when Samus calls it: its first parameter, which its init
/// doubles into an index, is zero for down (bank $A3 log, `$A3:94E2`, `$A3:9579`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ElevatorDirection {
    /// Down to the room below.
    #[default]
    Down,
    /// Up to the room above.
    Up,
}

/// Where an elevator ride is (bank $A3 log, the elevator status, `$A3:9540`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ElevatorStatus {
    /// No ride: the platform waits for the call button.
    #[default]
    Inactive,
    /// Carrying Samus out of the room.
    LeavingRoom,
    /// The door transition between the rooms.
    DoorTransition,
    /// Carrying Samus into the room, to the platform's target.
    EnteringRoom,
}

/// What lets the elevator AI run while no ride is on: the elevator properties word (bank $A3
/// log, `$A3:952A`, which tests it for zero).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ElevatorArming {
    /// Nothing: the platform waits.
    #[default]
    Disarmed,
    /// Samus stands in the elevator's doorway, a door with no destination, which sets it to 1
    /// while her pose is one of the first nine (bank $82 log, the door transition check).
    InDoorway,
    /// Samus came in by an elevator door: door loading keeps the door's elevator property
    /// bit, $80.
    ElevatorDoor,
}

/// The elevator ride, which outlives a room: the next room's elevator reads it (bank $A3
/// log, `$A3:9507`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ElevatorRide {
    /// Where the ride is.
    pub status: ElevatorStatus,
    /// What lets the elevator AI run while no ride is on; an idle frame without the call
    /// button disarms it (`$A3:9575`), and so does the ride's end (`$A3:95FB`).
    pub armed: ElevatorArming,
    /// The way the last departure went, which the door transition reads (`$A3:9581`).
    pub direction: ElevatorDirection,
}

/// The six words the original gives each enemy's AI.
// open: each species' names for them come with its AI lane (C8: the eye, and the face
// blocks); until then they are the original's words in slot order.
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
    Elevator(Elevator),
    /// A Blue Brinstar face block.
    FaceBlock {
        /// Its AI words.
        variables: AiVariables,
    },
}

/// A room's width in blocks, 16 a screen.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockColumns(pub u16);

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
    /// How many blocks across the room is: the level's row length.
    pub width: BlockColumns,
    /// One per screen, row-major.
    pub scroll_areas: Vec<ScrollArea>,
    /// The camera.
    pub camera: Camera,
    /// The PLMs.
    pub plms: SlotStore<PlmCommon, PlmBody, PLM_SLOTS>,
    /// The enemies.
    pub enemies: SlotStore<EnemyCommon, EnemyBody, ENEMY_SLOTS>,
}

impl RoomScoped {
    /// Room `room`, `size` screens, as it loads in `state`: its level's layer 1 blocks, its width
    /// in blocks and its scroll areas, with no PLMs or enemies yet and the camera at the top left.
    // open: the door's arrival, the PLM and enemy populations and the camera's start join with
    // the room transition (bank $82 log, `$82:E7D3`).
    #[must_use]
    pub fn load(room: RoomId, size: Screens, state: &RoomState) -> Self {
        Self {
            room,
            blocks: state.layers.layer1.clone(),
            width: BlockColumns(u16::from(size.width).wrapping_mul(16)),
            scroll_areas: state.scroll_areas.clone(),
            ..Self::default()
        }
    }
}
