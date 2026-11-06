//! Everything a room transition resets, in one struct (D4): replacing it is the reset.

use canonical::Canonical;
use data::enemy::program::SpritemapId;
use data::id::{DoorwayId, RoomId, SpeciesId};
use data::plm::{ItemPlm, ScrollChangesId};
use data::room::{Room, RoomState};

pub use data::level::{
    Block, BlockKind, BombableBts, Bts, DoorIndex, Offset, PlmChoice, ScrollArea, Slope, Tile,
    TileNumber,
};

use super::samus::{Fixed, Point};
use super::slots::{Cursor, Flag, FlagSet, Frames, SlotIndex, SlotStore};
use super::video::Scroll;

/// PLM slots (bank $84 log: the PLM tables hold 40 entries).
pub const PLM_SLOTS: usize = 40;
/// Enemy slots (bank $A0 log: the enemy table holds 32 entries).
pub const ENEMY_SLOTS: usize = 32;
/// Enemy layers: the drawing loop's eight, drawn from 0 up (bank $A0 log, `$A0:884D`).
pub const ENEMY_LAYERS: usize = 8;

/// A block's place in the room's level data, row-major from the top left.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockIndex(pub u16);

/// The camera: layer 1's top-left corner in the room, layer 2's, and what the backgrounds
/// add to them (bank `$80` log, the scrolling variables).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Camera {
    /// Layer 1, rightward: the playfield's left edge.
    pub x: Fixed,
    /// Layer 1, downward: the playfield's top edge.
    pub y: Fixed,
    /// Layer 2's position, which follows layer 1 at the room's layer 2 rates.
    pub layer2: Scroll,
    /// What BG1's scroll adds to layer 1's position.
    pub bg1_offset: Scroll,
    /// What BG2's scroll adds to layer 2's position.
    pub bg2_offset: Scroll,
    /// How far ahead of Samus the camera aims.
    pub distance: CameraDistance,
    /// How far below the camera's top edge it aims Samus as she goes up or down.
    pub scrollers: Scrollers,
    /// How layer 2 follows layer 1 across and down.
    pub layer2_scroll: Layer2Scrolls,
}

/// How far to the left of Samus the camera aims its left edge: the camera distance index,
/// whose entry the horizontal scrolling reads from the tables at `$90:963F` (facing right)
/// and `$90:9647` (facing left).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CameraDistance {
    /// $60 facing right, $A0 facing left: the camera shows more of the way she faces.
    #[default]
    Leading,
    /// $40 facing right, $50 facing left.
    Near,
    /// $20 either way: she stays near the left edge.
    LeftEdge,
    /// $E0 either way: she stays near the right edge.
    RightEdge,
}

impl CameraDistance {
    /// The entries in the original's order.
    pub const ALL: [Self; 4] = [Self::Leading, Self::Near, Self::LeftEdge, Self::RightEdge];

    /// The camera's left edge's distance to the left of Samus: `(facing right, facing left)`.
    #[must_use]
    pub const fn lead(self) -> (u16, u16) {
        match self {
            Self::Leading => (0x60, 0xA0),
            Self::Near => (0x40, 0x50),
            Self::LeftEdge => (0x20, 0x20),
            Self::RightEdge => (0xE0, 0xE0),
        }
    }
}

impl TryFrom<u16> for CameraDistance {
    type Error = u16;

    /// The original's index, a word offset into the tables: 0, 2, 4 or 6.
    fn try_from(index: u16) -> Result<Self, u16> {
        Self::ALL
            .into_iter()
            .find(|&distance| u16::from(distance) == index)
            .ok_or(index)
    }
}

impl From<CameraDistance> for u16 {
    fn from(distance: CameraDistance) -> Self {
        match distance {
            CameraDistance::Leading => 0,
            CameraDistance::Near => 2,
            CameraDistance::LeftEdge => 4,
            CameraDistance::RightEdge => 6,
        }
    }
}

/// The room header's vertical scrollers, as room loading widens them into words: how far
/// below the camera's top edge the camera aims Samus.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Scrollers {
    /// While she isn't going up (`$90:966E`).
    pub up: u16,
    /// While she goes up (`$90:967A`).
    pub down: u16,
}

/// How layer 2 follows layer 1 on one axis: the room state's layer 2 scroll byte (bank `$80`
/// log, `$80:A2F9` and `$80:A33A`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layer2Scroll {
    /// 0: at layer 1's position.
    #[default]
    WithLayer1,
    /// 1: still, and BG2's scroll keeps what it has.
    Fixed,
    /// Any other byte: at layer 1's position times `rate` / 256.
    Scaled {
        /// The byte without bit 0: even and not zero.
        rate: u8,
        /// Bit 0 clear: BG2 shows the room's own layer 2 level data, whose columns and rows
        /// the scrolling uploads (`$80:A416`); set, the library background.
        custom: bool,
    },
}

impl From<u8> for Layer2Scroll {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Self::WithLayer1,
            1 => Self::Fixed,
            _ => Self::Scaled {
                rate: byte & 0xFE,
                custom: byte & 1 == 0,
            },
        }
    }
}

impl From<Layer2Scroll> for u8 {
    fn from(scroll: Layer2Scroll) -> Self {
        match scroll {
            Layer2Scroll::WithLayer1 => 0,
            Layer2Scroll::Fixed => 1,
            Layer2Scroll::Scaled { rate, custom } => rate | Self::from(!custom),
        }
    }
}

/// Layer 2's scroll rates across and down.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Layer2Scrolls {
    /// Across.
    pub x: Layer2Scroll,
    /// Down.
    pub y: Layer2Scroll,
}

/// The PLM family marker.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Plm;

/// The fields every PLM slot has, whatever occupies it (bank $84 log, the PLM tables).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlmCommon {
    /// The block it sits on (`$1C87`, which holds it times two).
    pub block: BlockIndex,
    /// Its instruction list; the delay is the instruction timer (`$7E:DE1C`).
    // open: the label stays at its default until the PLM programs decode (B7); a body that
    // runs a list keeps its place in it.
    pub cursor: Cursor<Plm>,
}

/// Where a scroll PLM is in its instruction list (bank $84 log, `$84:AF86`), past the debug
/// draw its setup skips (`$84:B366`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollStep {
    /// `$84:AF8A`: asleep until Samus touches its block.
    #[default]
    Sleep,
    /// `$84:AF8C`: touched, the scroll update runs when the timer next runs out.
    Update,
}

/// A scroll PLM: the block that changes scroll areas when Samus touches it.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollPlm {
    /// The changes it makes: its room argument.
    pub changes: ScrollChangesId,
    /// Touched and not yet updated (`$1E17`): a second touch before the update does nothing.
    pub triggered: bool,
    /// Its place in its instruction list.
    pub step: ScrollStep,
}

/// Where the 1x1 shot block is in its crumble (bank $84 log, its list `$84:CBB7`): each of
/// its draws holds for its frames, the last one before the delete.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Crumble {
    /// Spawned: its first run queues the crumble sound and draws the first frame.
    #[default]
    Spawned,
    /// The first frame drawn, air with tile `$53`, for 4 frames (`$84:A345`).
    First,
    /// The second, tile `$54`, for 4 frames (`$84:A34B`).
    Second,
    /// The third, tile `$55`, for 4 frames (`$84:A351`).
    Third,
    /// The block drawn empty, tile `$FF`, for 1 frame (`$84:A357`); its next run deletes it.
    Cleared,
}

/// A PLM's typed body: one variant per PLM slice 0's rooms run. An extension PLM deletes
/// itself in its setup, so no body holds one.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlmBody {
    /// The scroll PLM, `$84:B703`.
    Scroll(ScrollPlm),
    /// The scroll block touch PLM, `$84:B6FF`: its setup triggers the scroll PLM on the
    /// touched block, and its list deletes it (`$84:AAE3`).
    ScrollTouch,
    /// An item.
    // open: its instruction list, the collected check, drawing and the pickup (C12).
    Item {
        /// Which item.
        item: ItemPlm,
        /// Its collected bit's index, the room argument.
        collected_bit: u16,
    },
    /// The item collision detection PLM, `$84:EED3`, which touching or shooting an item's
    /// block spawns: its setup moves its own block to 0 and triggers the item on the touched
    /// block (`$84:EEAB`), and its list deletes it (`$84:AAE3`). It stays live while the
    /// item's pickup holds the PLM pass in its message box.
    ItemCollision,
    /// The 1x1 shot block, `$84:D074`, which shooting a shootable block with BTS 4 spawns:
    /// its setup makes the block solid (`$84:B3C1`), and its list crumbles it.
    ShotBlock {
        /// Its place in the crumble.
        crumble: Crumble,
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
    /// The spritemap its instruction list shows.
    pub spritemap: SpritemapId,
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

/// What the Morph Ball Room eye's AI does each frame: its function (bank $A8 log, the
/// function pointer the main AI jumps through, `$A8:90E2`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EyePhase {
    /// Closed, until Samus is near (`$A8:90F1`).
    #[default]
    WaitingForSamus,
    /// Opening, until the function timer runs out (`$A8:912E`).
    Activating,
    /// Open and following Samus while she stays near (`$A8:9160`).
    Active,
    /// Closing, until the function timer runs out (`$A8:91CE`).
    Deactivating,
    /// The mount: the eye's second part, which does nothing (`$A8:91DC`, an `RTL`).
    Mount,
}

/// The Morph Ball Room eye's AI words (bank $A8 log, `$A8:9058` to `$A8:91DC`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MorphBallEye {
    /// The first two words, which its AI never reads or writes: they keep what the spawn
    /// left, in slot order.
    pub unused: [u16; 2],
    /// Whether its beam is on: the beam sets it (its HDMA object) and turning away clears
    /// it (`$A8:9160`).
    pub activated: bool,
    /// The angle from it to Samus, 0 up and 40h right, as it last aimed (`$A8:912E`,
    /// `$A8:91A2`).
    pub angle: u16,
    /// What's left of opening or closing (`$A8:90F1`, `$A8:9160`).
    pub function_timer: Frames,
    /// What it does each frame.
    pub phase: EyePhase,
}

/// The Blue Brinstar face block's AI words (bank $A8 log, `$A8:E82E` to `$A8:E91B`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FaceBlock {
    /// Whether it has turned to look at Samus: once it has, it never turns again
    /// (`$A8:E8CD`, `$A8:E90F`).
    pub activated: bool,
    /// Samus's X less its X, as it last measured while she was within its range of rows
    /// (`$A8:E8DF`): its sign is the way it turns.
    pub samus_offset: u16,
    /// The last four words, which its AI never reads or writes: they keep what the spawn
    /// left, in slot order.
    pub unused: [u16; 4],
}

/// A species' AI state (D14): one variant per species slice 0 runs.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyBody {
    /// One of the Morph Ball Room eye's two parts.
    MorphBallEye(MorphBallEye),
    /// The elevator platform.
    Elevator(Elevator),
    /// A Blue Brinstar face block.
    FaceBlock(FaceBlock),
}

/// The enemies the main enemy routine queued to draw, and its frame count.
///
/// Bank $A0 log: the main enemy routine (`$A0:8FD4`) adds each enemy it processed that is
/// on screen and visible to its layer's drawing queue (`$A0:9423`); the drawing loop
/// (`$A0:884D`) empties the queues. The queues are full only between the two, within a
/// frame's gameplay pass.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct EnemyDrawing {
    /// Each layer's queue, first queued first.
    pub queues: [Vec<SlotIndex<Enemy>>; ENEMY_LAYERS],
    /// The main enemy routine's runs, wrapping: the hurt flash shows on frames with its bit 1
    /// set (`$A0:949C`).
    pub routine_count: u16,
    /// What runs once the enemies are drawn.
    pub hook: GraphicsDrawnHook,
}

/// The enemy graphics drawn hook (bank $A0 log): the routine the Draw phase runs after the
/// enemies' layers (`$A0:88BD`, through `$A0:88C4`). Loading the enemies sets an `RTL`
/// (`$A0:8A37`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GraphicsDrawnHook {
    /// An `RTL`: nothing runs.
    #[default]
    None,
    /// The Blue Brinstar face block's glow, its palette cycle (bank $A8 log, `$A8:E86E`).
    FaceBlockGlow,
}

/// A sprite palette, 0 to 7: CGRAM's second half, sixteen colours each.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpritePalette(pub u8);

impl SpritePalette {
    /// The sprite palettes' first colour in CGRAM.
    const FIRST_COLOUR: usize = 0x80;
    /// Colours in a palette.
    const COLOURS: usize = 16;

    /// The palette an enemy's palette index ORs into its sprites: bits 9 to 11.
    #[must_use]
    pub const fn of(index: PaletteIndex) -> Self {
        let [_, high] = index.0.to_le_bytes();
        Self((high >> 1) & 7)
    }

    /// The CGRAM index of the palette's colour `colour`.
    #[must_use]
    pub fn colour(self, colour: usize) -> usize {
        Self::FIRST_COLOUR
            .wrapping_add(usize::from(self.0).wrapping_mul(Self::COLOURS))
            .wrapping_add(colour)
    }
}

/// Which of the face block's eight glow colour sets its palette cycle shows next (bank $A8
/// log, `$A8:E8A3`): it counts 0 to 7 and round again.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlowSet(pub u8);

impl GlowSet {
    /// The set after this one.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1) & 7)
    }

    /// The set's place in the glow colours.
    #[must_use]
    pub fn index(self) -> usize {
        usize::from(self.0)
    }
}

/// The enemy palette cycle the face block's glow runs (bank $A8 log, `$A8:E86E`).
///
/// They are the enemy graphics drawn hook's variables. Loading the enemies doesn't reset
/// them, so the colour set carries over from room to room.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnemyPaletteCycle {
    /// The sprite palette it writes, the face block's own.
    pub palette: SpritePalette,
    /// The colour set it writes next.
    pub set: GlowSet,
    /// Frames until it does.
    pub timer: Frames,
}

/// A room's width in blocks, 16 a screen.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockColumns(pub u16);

/// A layer blending configuration (`FX_LAYER_BLENDING_CONFIG_A`, the bank logs' default layer
/// blending configuration): the room's FX sets it as the room loads.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayerBlending(pub u16);

impl LayerBlending {
    /// The configurations whose colour math adds the backdrop colour to the backgrounds
    /// (28h and 2Ah), in which Samus's visor cycles (bank $91 log, `$91:D83F`).
    const BACKDROP_COLOUR_MATH: [Self; 2] = [Self(0x28), Self(0x2a)];

    /// Whether colour math adds the backdrop colour.
    #[must_use]
    pub fn adds_backdrop(self) -> bool {
        Self::BACKDROP_COLOUR_MATH.contains(&self)
    }
}

/// A byte of the scroll buffer past the room's screens: what an earlier room, or the RAM
/// before any room filled the buffer, left there. Loading a room writes only its own screens.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaleScroll {
    /// A byte an area has.
    Area(ScrollArea),
    /// Another byte. The scroll zones test only for red and blue, so they let the camera
    /// through it as they do a green screen (docs/LEGACY.md).
    Unknown(u8),
}

impl StaleScroll {
    /// The area the scroll zones read it as.
    #[must_use]
    pub const fn area(self) -> ScrollArea {
        match self {
            Self::Area(area) => area,
            Self::Unknown(_) => ScrollArea::Green,
        }
    }
}

impl From<u8> for StaleScroll {
    fn from(value: u8) -> Self {
        ScrollArea::try_from(value).map_or(Self::Unknown(value), Self::Area)
    }
}

impl From<StaleScroll> for u8 {
    fn from(stale: StaleScroll) -> Self {
        match stale {
            StaleScroll::Area(area) => area.into(),
            StaleScroll::Unknown(value) => value,
        }
    }
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
    /// How many blocks across the room is: the level's row length.
    pub width: BlockColumns,
    /// The room's default layer blending configuration.
    pub layer_blending: LayerBlending,
    /// One per screen, row-major.
    pub scroll_areas: Vec<ScrollArea>,
    /// The scroll buffer past the room's screens, which the scroll zones read when the screen
    /// below the camera is past the room's last row.
    pub stale_scrolls: Vec<StaleScroll>,
    /// The camera.
    pub camera: Camera,
    /// The PLMs.
    pub plms: SlotStore<PlmCommon, PlmBody, PLM_SLOTS>,
    /// The enemies.
    pub enemies: SlotStore<EnemyCommon, EnemyBody, ENEMY_SLOTS>,
    /// The enemies to draw this frame.
    // open: the routine count lives with the room until a reader shows whether a room
    // transition resets it.
    pub drawing: EnemyDrawing,
}

impl RoomScoped {
    /// Room `room`, as it loads in `state`: its level's layer 1 blocks, its width in blocks,
    /// its scroll areas and the camera's scroll settings, its PLMs spawned and set up, with no
    /// enemies yet and the camera at the top left.
    // open: the door's arrival, the enemy population, the FX's layer blending, the camera's
    // start and the scroll buffer past the room's screens (`stale_scrolls`, what the room
    // before left) join with the room transition (bank $82 log, `$82:E7D3`).
    #[must_use]
    pub fn load(room: RoomId, header: &Room, state: &RoomState) -> Self {
        let [x, y] = state.layer2_scroll.map(Layer2Scroll::from);
        let mut loaded = Self {
            room,
            blocks: state.layers.layer1.clone(),
            width: BlockColumns(u16::from(header.size.width).wrapping_mul(16)),
            scroll_areas: state.scroll_areas.clone(),
            camera: Camera {
                scrollers: Scrollers {
                    up: u16::from(header.scrollers.up),
                    down: u16::from(header.scrollers.down),
                },
                layer2_scroll: Layer2Scrolls { x, y },
                ..Camera::default()
            },
            ..Self::default()
        };
        crate::plm::spawn_population(&mut loaded, &state.plms);
        loaded
    }
}
