//! The one decoder from an oracle record to the engine's State (D21).
//!
//! The decoder reads what a record shows directly: the NMI counters and timing, the in-game
//! time, the RNG word, the controller, the game state, the NMI request, Samus's position and
//! equipment, video memory, the room and its camera.
//! The rest of the State stays at its default; [`crate::coverage::FIELDS`] says which.

use std::fmt;

use data::enemy::SpeciesMap;
use data::enemy::program::EnemyPrograms;
use data::id::{Line, RoomId};
use data::map::{MAP_BIT_BYTES, MapCells};
use data::plm::{ItemPlm, PlmPopulation, ScrollLists};
use data::pose::tiles::SamusTiles;
use data::pose::{MovementType, POSE_COUNT};
use data::room::{Area, MapPosition};
use data::tileset::{BlockTiles, TilesetId};
use game::plm::item_gfx;
use game::samus_appearance_bridge::Clips;
use game::state::atmospheric::{
    AtmosphericFrame, AtmosphericGraphic, AtmosphericKind, AtmosphericTimer, Atmospherics,
};
use game::state::hud::{Drawn, Hud, HudItem, HudTilemap, MiniMap};
use game::state::room::{
    Block, BlockColumns, BlockPoint, BlockRows, Camera, CameraDistance, DrawnBlocks,
    ElevatorArming, ElevatorDirection, ElevatorRide, ElevatorStatus, EnemyDrawing,
    EnemyPaletteCycle, GlowSet, GraphicsDrawnHook, ITEM_GFX_SLOTS, ItemGfx, ItemGfxSlot,
    Layer2Block, Layer2Scroll, Layer2Scrolls, LayerBlending, RoomScoped, ScrollArea, Scrollers,
    SpritePalette,
};
use game::state::samus::{
    Animation, AnimationFrame, Boost, BoostStage, ClipStep, DrawingHandler, Equipment, Facing,
    Fixed, FrameBefore, Gauge, Item, Items, JumpHeld, Point, PoseId, PoseRecord, ReserveMode,
    Samus, Visor, VisorShade, VisorTimer,
};
use game::state::slots::{FlagSet, Frames};
use game::state::sram::{SAVE_RAM_BYTES, SaveRam};
use game::state::strand::{IrqStrand, MainStrand, Message};
use game::state::video::{
    BACKGROUNDS, BBusRegister, BgMode, Brightness, BusAddress, CGRAM_COLOURS, Colour, ColourMath,
    ColourMathLayers, ColourMathSelect, ColourWrite, ColumnUpload, DisplaySettings, DmaChannel,
    DmaControl, FixedColour, GraphicsRef, LayerUploads, Layers, LevelUploads, MessageBoxSaved,
    Mode7, Mode7Select, Mosaic, NmiBatch, OAM_HIGH_BYTES, Oam, OamEntry, ObjectSelect, Playfield,
    PlayfieldColourMath, Registers, RomWords, RowUpload, SamusTileUploads, Scroll, TileBases,
    TilemapBase, TilemapEntry, TilemapPart, UploadRequests, VRAM_WORDS, Video, VideoMemory,
    VramSource, VramStep, VramTransfer, VramWord, WindowSettings, Windows,
};
use game::state::{
    Button, Buttons, Clock, CollectedItems, Controller, Counters, FrameCount, GameMode, GameTime,
    NmiTiming, Rng, State, Timing,
};
use oracle_record::{Cut, FrameRecord, NmiKind, PpuRegisters};

use crate::enemy::EnemySources;
use crate::plm::PlmSources;
use crate::projectile::ProjectileSources;
use crate::source::{self, TilemapUpload, WramArray, WramBlock, WramByte, WramWord};
use crate::strand::{self, CALL_SITES, StrandRules, WAIT_SITES};

/// Bank $82 log: game state 8 is main gameplay.
const MAIN_GAMEPLAY: u16 = 8;

/// OAM's low table in bytes: four per sprite.
const OAM_LOW_BYTES: usize = 0x200;

/// A pointer to a room header in bank $8F, as `ROOM_PTR` holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RoomHeader(pub u16);

/// The rooms the decoder knows: each room header with the room registry's ID for it (from
/// `import::world`), and the species their enemies are (see
/// `lockstep::loaded_rooms`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoomMap {
    /// The known rooms.
    pub rooms: Vec<(RoomHeader, RoomId)>,
    /// The known species, by the header pointer an enemy slot holds.
    pub species: SpeciesMap,
    /// Their instruction lists, by the pointers an enemy slot holds.
    pub programs: EnemyPrograms,
    /// Samus's tiles definitions, by the pointers her tile uploads hold.
    pub samus_tiles: SamusTiles,
    /// Her clips, whose step maps her animation's frame is read through.
    pub samus_clips: Clips,
    /// Each room's PLM population, which names the PLMs its slots hold.
    pub plms: Vec<(RoomId, PlmPopulation)>,
    /// Where each room's scroll change lists are, which its scroll PLMs' room arguments name.
    pub scroll_lists: Vec<(RoomId, ScrollLists)>,
    /// What each known room state loads that RAM doesn't keep, by its header pointer.
    pub states: Vec<(StateHeader, StateLevel)>,
    /// The blocks' reactions, which name the PLMs their places hold.
    pub reactions: data::blocks::Reactions,
}

/// A pointer to a room state header in bank $8F, as `ROOMDEFROOMSTATE_PTR` holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StateHeader(pub u16);

/// What a room state loads that the original keeps only in its tile table and level buffer:
/// the decoder takes it from the content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateLevel {
    /// The state's tileset, whose tile table `$82:E7D3` decompresses.
    pub tileset: TilesetId,
    /// Whether its level data has layer 2 after the BTS.
    pub layer2: bool,
}

impl RoomMap {
    /// The map with `programs` as its species' instruction lists.
    #[must_use]
    pub fn with_programs(self, programs: EnemyPrograms) -> Self {
        Self { programs, ..self }
    }

    /// The map with `samus_tiles` as Samus's tiles definitions.
    #[must_use]
    pub fn with_samus_tiles(self, samus_tiles: SamusTiles) -> Self {
        Self {
            samus_tiles,
            ..self
        }
    }

    /// The map with `samus_clips` as her clips.
    #[must_use]
    pub fn with_samus_clips(self, samus_clips: Clips) -> Self {
        Self {
            samus_clips,
            ..self
        }
    }

    /// The map with `plms` as its rooms' PLM populations.
    #[must_use]
    pub fn with_plms(self, plms: Vec<(RoomId, PlmPopulation)>) -> Self {
        Self { plms, ..self }
    }

    /// The map with `scroll_lists` as where its rooms' scroll change lists are.
    #[must_use]
    pub fn with_scroll_lists(self, scroll_lists: Vec<(RoomId, ScrollLists)>) -> Self {
        Self {
            scroll_lists,
            ..self
        }
    }

    /// The map with `reactions` as the blocks' reactions.
    #[must_use]
    pub fn with_reactions(self, reactions: data::blocks::Reactions) -> Self {
        Self { reactions, ..self }
    }

    /// The map with `states` as its room states' levels.
    #[must_use]
    pub fn with_states(self, states: Vec<(StateHeader, StateLevel)>) -> Self {
        Self { states, ..self }
    }

    /// What the state whose header is at `header` loads, if the map knows it.
    #[must_use]
    pub fn state(&self, header: StateHeader) -> Option<StateLevel> {
        self.states
            .iter()
            .find_map(|&(known, level)| (known == header).then_some(level))
    }

    /// Room `room`'s PLM population: empty for a room the map has none for, so no PLM
    /// decodes there.
    #[must_use]
    pub fn population(&self, room: RoomId) -> &PlmPopulation {
        static EMPTY: PlmPopulation = PlmPopulation {
            entries: Vec::new(),
            scroll_changes: Vec::new(),
        };
        self.plms
            .iter()
            .find_map(|(known, population)| (*known == room).then_some(population))
            .unwrap_or(&EMPTY)
    }

    /// Where room `room`'s scroll change lists are: none for a room the map has none for, so
    /// no scroll PLM decodes there.
    #[must_use]
    pub fn scroll_lists(&self, room: RoomId) -> &ScrollLists {
        static NONE: ScrollLists = ScrollLists(Vec::new());
        self.scroll_lists
            .iter()
            .find_map(|(known, lists)| (*known == room).then_some(lists))
            .unwrap_or(&NONE)
    }

    /// The content's ID for the room whose header is at `header`.
    #[must_use]
    pub fn get(&self, header: RoomHeader) -> Option<RoomId> {
        self.rooms
            .iter()
            .find_map(|&(known, room)| (known == header).then_some(room))
    }
}

/// A State the decoder can't produce yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsupported {
    /// A game state other than main gameplay, with `GAME_STATE`'s value.
    GameState(u16),
    /// A cut that isn't an NMI's return, so no NMI kind to decode the timing from.
    NoNmi(Cut),
    /// An interrupt command the State has no value for yet, or a queued one.
    IrqCommand {
        /// `CUR_IRQ_HANDLER`'s value.
        current: u16,
        /// `IRQHANDLER_NEXT_HANDLER`'s value.
        next: u16,
    },
}

/// A memory the decoder reads whole.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Memory {
    /// VRAM, in words.
    Vram,
    /// CGRAM, in colours.
    Cgram,
    /// OAM, in bytes.
    Oam,
    /// The cartridge's save RAM, in bytes.
    Sram,
}

/// Why a record has no State.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// The record's work RAM ends before the named source.
    WramTooShort {
        /// The source's `ram_map.rs` name.
        source: &'static str,
    },
    /// A video memory isn't the size the PPU has.
    WrongSize {
        /// Which memory.
        memory: Memory,
        /// Its size on the PPU.
        expected: usize,
        /// Its size in the record.
        found: usize,
    },
    /// `ROOM_PTR` names a room the map doesn't hold.
    UnknownRoom(RoomHeader),
    /// `ROOMDEFROOMSTATE_PTR` names a room state the map doesn't hold.
    UnknownRoomState(StateHeader),
    /// The record is in a state the decoder doesn't cover yet.
    Unsupported(Unsupported),
    /// A source holds a value no routine of the original writes, which the State has no
    /// value for: unknown item bits, or an NMI request flag other than 0 or 1.
    Unrepresentable {
        /// The source's `ram_map.rs` name.
        source: &'static str,
        /// What the State can't hold: the unknown bits, or the whole value.
        value: u16,
    },
    /// A requested NMI found the main thread at a PC no wait site holds, or its return chain
    /// holds an address no call site knows.
    UnknownYieldSite {
        /// The interrupted PC, or the unknown return address, 24-bit.
        pc: u32,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WramTooShort { source } => write!(f, "work RAM ends before {source}"),
            Self::WrongSize {
                memory,
                expected,
                found,
            } => write!(f, "{memory:?} holds {found}, not {expected}"),
            Self::UnknownRoom(RoomHeader(header)) => {
                write!(f, "no room has its header at {header:04X}")
            }
            Self::UnknownRoomState(StateHeader(header)) => {
                write!(f, "no room state has its header at {header:04X}")
            }
            Self::Unsupported(Unsupported::GameState(state)) => {
                write!(f, "game state {state:#06X} isn't decoded yet")
            }
            Self::Unsupported(Unsupported::NoNmi(cut)) => {
                write!(f, "a {cut:?} cut isn't decoded yet")
            }
            Self::Unsupported(Unsupported::IrqCommand { current, next }) => {
                write!(
                    f,
                    "interrupt command {current:#06X}, then {next:#06X}, isn't decoded yet"
                )
            }
            Self::Unrepresentable { source, value } => {
                write!(
                    f,
                    "{source} holds {value:#06X}, which the State has no value for"
                )
            }
            Self::UnknownYieldSite { pc } => {
                write!(f, "the main thread waits at {pc:06X}, which no table knows")
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Whether a cut has an engine equivalent (D21).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quiescence {
    /// The main thread was waiting at a known wait site: the typed streams compare here.
    Quiet,
    /// The NMI overran the main thread mid-routine: no engine State corresponds, so only
    /// the logs and the picture compare, and the main strand keeps its default.
    Busy,
}

/// A decoded record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decoded {
    /// The State the record shows.
    pub state: State,
    /// Whether the typed streams compare at its cut.
    pub quiescence: Quiescence,
}

/// Where each decoded field comes from: one named source per rule.
///
/// [`Rules::ORIGINAL`] is the original's layout; a mutant is a copy with one source moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules<'t> {
    /// `clock.counters.nmi`.
    pub nmi: WramWord,
    /// `clock.counters.completed`.
    pub completed: WramWord,
    /// `clock.counters.lag`.
    pub lag: WramByte,
    /// `clock.counters.max_lag`.
    pub max_lag: WramByte,
    /// `clock.game_time.hours`.
    pub hours: WramWord,
    /// `clock.game_time.minutes`.
    pub minutes: WramWord,
    /// `clock.game_time.seconds`.
    pub seconds: WramWord,
    /// `clock.game_time.frames`.
    pub frames: WramWord,
    /// `rng.word`.
    pub rng: WramWord,
    /// `controller.held`.
    pub held: WramWord,
    /// `controller.pressed`.
    pub pressed: WramWord,
    /// `controller.previous`.
    pub previous: WramWord,
    /// `controller.repeat`.
    pub repeat: WramWord,
    /// `controller.repeat_delay`.
    pub repeat_delay: WramWord,
    /// `mode`.
    pub game_state: WramWord,
    /// `room.room`.
    pub room: WramWord,
    /// `video.nmi_requested`.
    pub nmi_requested: WramByte,
    /// `video.batch.oam`.
    pub oam_buffer: WramBlock,
    /// `video.batch.cgram`.
    pub palette_buffer: WramBlock,
    /// `video.batch.vram`: the queue.
    pub vram_queue: WramBlock,
    /// `video.batch.vram`: the queue's end.
    pub vram_queue_tail: WramWord,
    /// `video.batch.samus_tiles`.
    pub samus_tiles: SamusTileSources,
    /// `video.batch.registers.forced_blank`, `.brightness`.
    pub inidisp: WramByte,
    /// `video.batch.registers.main_screen`.
    pub main_screen: WramByte,
    /// `video.batch.registers.sub_screen`.
    pub sub_screen: WramByte,
    /// `video.batch.registers.scroll`.
    pub scroll: [[WramWord; 2]; BACKGROUNDS],
    /// The other register mirrors `$80:91EE` writes.
    pub mirrors: MirrorSources,
    /// The registers the HUD interrupt writes for the playfield.
    pub playfield: PlayfieldSources,
    /// Where the message box keeps what it puts back when it closes.
    pub message_box: MessageBoxSources,
    /// `samus.equipment.collected`.
    pub collected: WramWord,
    /// `samus.equipment.equipped`.
    pub equipped: WramWord,
    /// `samus.equipment.energy.current`.
    pub energy: WramWord,
    /// `samus.equipment.energy.max`.
    pub max_energy: WramWord,
    /// `samus.equipment.reserve.current`.
    pub reserve: WramWord,
    /// `samus.equipment.reserve.max`.
    pub max_reserve: WramWord,
    /// `samus.equipment.missiles.current`.
    pub missiles: WramWord,
    /// `samus.equipment.missiles.max`.
    pub max_missiles: WramWord,
    /// `samus.equipment.super_missiles.current`.
    pub super_missiles: WramWord,
    /// `samus.equipment.super_missiles.max`.
    pub max_super_missiles: WramWord,
    /// `samus.equipment.power_bombs.current`.
    pub power_bombs: WramWord,
    /// `samus.equipment.power_bombs.max`.
    pub max_power_bombs: WramWord,
    /// `samus.equipment.reserve_mode`.
    pub reserve_mode: WramWord,
    /// `clock.counters.requested_byte`.
    pub requested_byte: WramByte,
    /// `hud`.
    pub hud: HudSources,
    /// `room.camera.x.pixels`.
    pub camera_x: WramWord,
    /// `room.camera.x.subpixels`.
    pub camera_x_subpixels: WramWord,
    /// `room.camera.y.pixels`.
    pub camera_y: WramWord,
    /// `room.camera.y.subpixels`.
    pub camera_y_subpixels: WramWord,
    /// The rest of `room.camera`.
    pub scrolling: ScrollingSources,
    /// `samus.previous_position`: x, its subpixels, y, its subpixels.
    pub previous_position: [WramWord; 4],
    /// `main`: the wait sites, the return chains and the sources they resume from.
    pub strand: StrandRules<'t>,
    /// `irq.command`.
    pub irq_command: WramWord,
    /// The queued interrupt command, which the State has no place for yet.
    pub irq_next: WramWord,
    /// `samus.pose`, `samus.facing`, `samus.movement`.
    pub pose: PoseSources,
    /// `samus.previous`.
    pub previous_pose: PoseSources,
    /// `samus.last_different`.
    pub last_different_pose: PoseSources,
    /// `message`.
    pub message: WramWord,
    /// `collected_items`.
    pub collected_items: WramBlock,
    /// `room.plms`.
    pub plms: PlmSources,
    /// `projectiles`.
    pub projectiles: ProjectileSources,
    /// `room.enemies`.
    pub enemies: EnemySources,
    /// `samus.animation`.
    pub animation: AnimationSources,
    /// `samus.visor`: its timer and shade, a byte each.
    pub visor: WramWord,
    /// `room.layer_blending`.
    pub layer_blending: WramWord,
    /// `samus.drawing`.
    pub drawing: WramWord,
    /// `samus.y_radius`.
    pub y_radius: WramWord,
    /// `samus.slope_adjusted`.
    pub slope_adjusted: WramWord,
    /// `samus.jump_held`: its count and the pose input handler.
    pub jump_held: [WramWord; 2],
    /// `elevator`.
    pub elevator: ElevatorSources,
    /// `room.drawing.hook`.
    pub drawn_hook: DrawnHookSources,
    /// `enemy_palette_cycle`.
    pub palette_cycle: PaletteCycleSources,
    /// `atmospherics`.
    pub atmospherics: AtmosphericSources,
    /// `room.blocks` and `room.scroll_areas`.
    pub level: LevelSources,
    /// `room.area`.
    pub area: WramWord,
    /// `room.tilemap_gap`.
    pub tilemap_gap: WramWord,
    /// `room.item_gfx`.
    pub item_gfx: ItemGfxSources,
    /// `video.batch.level`: BG1's, then BG2's.
    pub uploads: [LayerUploadSources; 2],
}

/// Where the original keeps the item GFX slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemGfxSources {
    /// The slot the next load takes, times two.
    pub next: WramWord,
    /// Each slot's last load's arguments.
    pub pointers: WramBlock,
    /// The tile table entries the slots rewrite.
    pub entries: WramBlock,
}

/// The bank $84 pointer to an item's GFX load's arguments in its list (`$84:8764`'s `Y`).
const fn item_gfx_arguments(item: ItemPlm) -> u16 {
    match item {
        ItemPlm::MorphBall => 0xe3f1,
    }
}

/// The items whose GFX loads the decoder names.
const ITEM_GFX_ITEMS: [ItemPlm; 1] = [ItemPlm::MorphBall];

impl ItemGfxSources {
    /// The slots: a slot holds the item its pointer names while the tile table still holds
    /// the entries that item's load wrote there. A room's load leaves the pointers and
    /// restores the entries, so a stale pointer reads as nothing loaded.
    fn read(self, wram: &[u8]) -> Result<ItemGfx, DecodeError> {
        let doubled = self.next.read(wram)?;
        let next = u8::try_from(doubled / 2)
            .ok()
            .filter(|&slot| doubled % 2 == 0 && usize::from(slot) < ITEM_GFX_SLOTS)
            .ok_or(DecodeError::Unrepresentable {
                source: self.next.name,
                value: doubled,
            })?;
        let pointers = self.pointers.read(wram)?.as_chunks::<2>().0;
        let entries = self.entries.read(wram)?.as_chunks::<16>().0;
        let mut loaded = [None; ITEM_GFX_SLOTS];
        for (((slot, held), pointer), written) in
            (0..).zip(loaded.iter_mut()).zip(pointers).zip(entries)
        {
            let pointer = u16::from_le_bytes(*pointer);
            *held = ITEM_GFX_ITEMS.into_iter().find(|&item| {
                item_gfx_arguments(item) == pointer
                    && block_bytes(item_gfx::entries(ItemGfxSlot(slot), item)) == *written
            });
        }
        Ok(ItemGfx {
            next: ItemGfxSlot(next),
            loaded,
        })
    }
}

/// Two blocks' tile table entries as the table's bytes hold them.
fn block_bytes(blocks: [BlockTiles; 2]) -> [u8; 16] {
    let mut bytes = [0; 16];
    let words = blocks.iter().flat_map(|tiles| {
        [
            tiles.top_left,
            tiles.top_right,
            tiles.bottom_left,
            tiles.bottom_right,
        ]
    });
    for (pair, word) in bytes.as_chunks_mut::<2>().0.iter_mut().zip(words) {
        *pair = word.to_le_bytes();
    }
    bytes
}

/// Where the original keeps one background's level tilemap uploads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerUploadSources {
    /// `.column`.
    pub column: TilemapUpload,
    /// `.row`.
    pub row: TilemapUpload,
}

/// A tilemap upload part's bytes and VRAM word.
fn part(bytes: WramWord, to: WramWord, wram: &[u8]) -> Result<TilemapPart, DecodeError> {
    Ok(TilemapPart {
        to: VramWord(to.read(wram)?),
        bytes: bytes.read(wram)?,
    })
}

/// The tilemap words a buffer holds.
fn entries<const N: usize>(
    block: WramBlock,
    wram: &[u8],
) -> Result<[TilemapEntry; N], DecodeError> {
    let words = block.read(wram)?.as_chunks::<2>().0;
    let mut entries = [TilemapEntry::default(); N];
    if words.len() < N {
        return Err(DecodeError::WramTooShort { source: block.name });
    }
    for (entry, word) in entries.iter_mut().zip(words) {
        *entry = TilemapEntry::from_word(u16::from_le_bytes(*word));
    }
    Ok(entries)
}

impl LayerUploadSources {
    /// The background's column and row as its builders last left them. The wrapped parts'
    /// source addresses aren't read: `$80:A9DE` and `$80:AB78` set them to the buffer's
    /// start plus the first part's bytes.
    fn read(self, wram: &[u8]) -> Result<LayerUploads, DecodeError> {
        let Self { column, row } = self;
        Ok(LayerUploads {
            column: ColumnUpload {
                left: entries(column.first_halves, wram)?,
                right: entries(column.second_halves, wram)?,
                first: part(column.unwrapped_size, column.unwrapped_dst, wram)?,
                wrapped: part(column.wrapped_size, column.wrapped_dst, wram)?,
                requests: UploadRequests(column.enable.read(wram)?),
            },
            row: RowUpload {
                top: entries(row.first_halves, wram)?,
                bottom: entries(row.second_halves, wram)?,
                first: part(row.unwrapped_size, row.unwrapped_dst, wram)?,
                wrapped: part(row.wrapped_size, row.wrapped_dst, wram)?,
                requests: UploadRequests(row.enable.read(wram)?),
            },
        })
    }
}

/// Where the original keeps the room's level and its scroll areas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelSources {
    /// The room's width in blocks.
    pub width: WramWord,
    /// The room's height in blocks.
    pub height: WramWord,
    /// Layer 1's words.
    pub blocks: WramBlock,
    /// The BTS.
    pub bts: WramBlock,
    /// The scroll areas.
    pub scrolls: WramBlock,
    /// The room state, which names its tileset and says whether it has layer 2.
    pub state: WramWord,
    /// Layer 2's words.
    pub layer2: WramBlock,
}

/// Blocks a screen across or down.
const SCREEN_BLOCKS: u16 = 16;

impl LevelSources {
    /// Layer 1, layer 2, the level buffer past layer 1 and the scroll areas: the width and
    /// height say how much of each buffer the room fills. The rest of the scroll buffer is
    /// what earlier rooms left.
    ///
    /// The level buffer past layer 1 still holds the BTS and layer 2 as decompression wrote
    /// them (`$82:E7D3`); it is read up to the buffer's end, past which a column or row never
    /// reads.
    fn read(self, wram: &[u8], rooms: &RoomMap) -> Result<Level, DecodeError> {
        let width = self.width.read(wram)?;
        let height = self.height.read(wram)?;
        let blocks = usize::from(width).saturating_mul(usize::from(height));
        let short = |block: WramBlock| DecodeError::WramTooShort { source: block.name };
        let layer1 = self.layer1(wram)?;
        let header = StateHeader(self.state.read(wram)?);
        let loaded = rooms
            .state(header)
            .ok_or(DecodeError::UnknownRoomState(header))?;
        let layer2 = loaded
            .layer2
            .then(|| {
                let words = self
                    .layer2
                    .read(wram)?
                    .get(..blocks.saturating_mul(2))
                    .ok_or_else(|| short(self.layer2))?;
                Ok(words
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|word| Layer2Block::from_word(u16::from_le_bytes(*word)))
                    .collect())
            })
            .transpose()?;
        let tail_len = if loaded.layer2 {
            blocks.saturating_mul(3)
        } else {
            blocks
        };
        let buffer = self.blocks.read(wram)?;
        let tail_start = blocks.saturating_mul(2).min(buffer.len());
        let tail_end = tail_start.saturating_add(tail_len).min(buffer.len());
        let level_tail = buffer
            .get(tail_start..tail_end)
            .unwrap_or_default()
            .to_vec();
        let screens =
            usize::from(width / SCREEN_BLOCKS).saturating_mul(usize::from(height / SCREEN_BLOCKS));
        let (areas, below) = self
            .scrolls
            .read(wram)?
            .split_at_checked(screens)
            .ok_or_else(|| short(self.scrolls))?;
        let scroll_areas = areas.iter().copied().map(ScrollArea::from).collect();
        Ok(Level {
            blocks: layer1,
            layer2,
            tail: level_tail,
            tileset: loaded.tileset,
            scroll_areas,
            scrolls_below: below.iter().copied().map(ScrollArea::from).collect(),
        })
    }
}

impl LevelSources {
    /// Layer 1's blocks, as many as the width and height say the room fills: the level any
    /// room's moves read, which needs no room map.
    ///
    /// # Errors
    ///
    /// [`DecodeError::WramTooShort`] when the buffers end before the room does.
    pub fn layer1(self, wram: &[u8]) -> Result<Vec<Block>, DecodeError> {
        let width = self.width.read(wram)?;
        let height = self.height.read(wram)?;
        let blocks = usize::from(width).saturating_mul(usize::from(height));
        let short = |block: WramBlock| DecodeError::WramTooShort { source: block.name };
        let words = self
            .blocks
            .read(wram)?
            .get(..blocks.saturating_mul(2))
            .ok_or_else(|| short(self.blocks))?;
        let bts = self
            .bts
            .read(wram)?
            .get(..blocks)
            .ok_or_else(|| short(self.bts))?;
        Ok(words
            .as_chunks::<2>()
            .0
            .iter()
            .zip(bts)
            .map(|(word, &bts)| Block::new(u16::from_le_bytes(*word), bts))
            .collect())
    }
}

/// The room's level as [`LevelSources::read`] decodes it.
struct Level {
    blocks: Vec<Block>,
    layer2: Option<Vec<Layer2Block>>,
    tail: Vec<u8>,
    tileset: TilesetId,
    scroll_areas: Vec<ScrollArea>,
    scrolls_below: Vec<ScrollArea>,
}

/// Where the original keeps the camera's scroll state beside layer 1's position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollingSources {
    /// `.layer2`: x, y.
    pub layer2: [WramWord; 2],
    /// `.bg1_offset`: x, y.
    pub bg1_offset: [WramWord; 2],
    /// `.bg2_offset`: x, y.
    pub bg2_offset: [WramWord; 2],
    /// `.distance`.
    pub distance: WramWord,
    /// `.scrollers`: up, down.
    pub scrollers: [WramWord; 2],
    /// `.layer2_scroll`: x, y.
    pub layer2_scroll: [WramByte; 2],
    /// `.drawn`: layer 1's x, y, then layer 2's.
    pub drawn: [[WramWord; 2]; 2],
}

impl ScrollingSources {
    /// The camera but for layer 1's position.
    fn read(self, wram: &[u8]) -> Result<Camera, DecodeError> {
        let scroll = |[x, y]: [WramWord; 2]| -> Result<Scroll, DecodeError> {
            Ok(Scroll {
                x: x.read(wram)?,
                y: y.read(wram)?,
            })
        };
        let index = self.distance.read(wram)?;
        let distance =
            CameraDistance::try_from(index).map_err(|value| DecodeError::Unrepresentable {
                source: self.distance.name,
                value,
            })?;
        let [up, down] = self.scrollers;
        let [x, y] = self.layer2_scroll;
        let block = |[x, y]: [WramWord; 2]| -> Result<BlockPoint, DecodeError> {
            Ok(BlockPoint {
                x: x.read(wram)?,
                y: y.read(wram)?,
            })
        };
        let [layer1, layer2] = self.drawn;
        Ok(Camera {
            layer2: scroll(self.layer2)?,
            bg1_offset: scroll(self.bg1_offset)?,
            bg2_offset: scroll(self.bg2_offset)?,
            distance,
            scrollers: Scrollers {
                up: up.read(wram)?,
                down: down.read(wram)?,
            },
            layer2_scroll: Layer2Scrolls {
                x: Layer2Scroll::from(x.read(wram)?),
                y: Layer2Scroll::from(y.read(wram)?),
            },
            drawn: DrawnBlocks {
                layer1: block(layer1)?,
                layer2: block(layer2)?,
            },
            ..Camera::default()
        })
    }
}

/// Where the original keeps the enemy graphics drawn hook: a long pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawnHookSources {
    /// Its address.
    pub pointer: WramWord,
    /// Its bank.
    pub bank: WramByte,
}

/// Where the original keeps the enemy palette cycle: the hook's variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteCycleSources {
    /// `.palette`.
    pub palette: WramWord,
    /// `.set`.
    pub set: WramWord,
    /// `.timer`.
    pub timer: WramWord,
}

/// Where the original keeps the atmospheric graphics: one word array per field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtmosphericSources {
    /// Whether a slot is live, `.kind` and `.frame`.
    pub frame_and_type: WramArray,
    /// `.timer`.
    pub timer: WramArray,
    /// `.x`.
    pub x: WramArray,
    /// `.y`.
    pub y: WramArray,
}

/// Where the original keeps the elevator ride.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElevatorSources {
    /// `.status`.
    pub status: WramWord,
    /// `.armed`.
    pub armed: WramWord,
    /// `.direction`.
    pub direction: WramWord,
}

/// Where the original keeps Samus's animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationSources {
    /// `.step`, as the original's frame in her pose's delay list.
    pub frame: WramWord,
    /// `.frames_left`.
    pub timer: WramWord,
    /// `.boost`: the stage in the high byte, the laps left in the low.
    pub boost: WramWord,
}

/// Where the original keeps the HUD.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HudSources {
    /// `.tilemap`.
    pub tilemap: WramBlock,
    /// `.selected`.
    pub selected: WramWord,
    /// `.auto_cancel`.
    pub auto_cancel: WramWord,
    /// `.drawn.energy`.
    pub energy: WramWord,
    /// `.drawn.missiles`.
    pub missiles: WramWord,
    /// `.drawn.super_missiles`.
    pub super_missiles: WramWord,
    /// `.drawn.power_bombs`.
    pub power_bombs: WramWord,
    /// `.drawn.selection`.
    pub selection: WramWord,
    /// `.mini_map`.
    pub mini_map: MiniMapSources,
}

/// Where the original keeps what the mini-map draws from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiniMapSources {
    /// `.explored`.
    pub explored: WramBlock,
    /// `.area`.
    pub area: WramWord,
    /// `.room.x`.
    pub room_x: WramWord,
    /// `.room.y`.
    pub room_y: WramWord,
    /// `.map_collected`.
    pub map_collected: WramWord,
    /// `.disabled`.
    pub disabled: WramWord,
}

/// Where the original keeps what the message box saves (bank $85 log, `$85:814B` and
/// `$85:81B5`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MessageBoxSources {
    /// `video.message_box.tilemap`.
    pub tilemap: WramBlock,
    /// `video.message_box.bg3_tilemap`.
    pub bg3_tilemap: WramByte,
}

/// Where the original keeps the HUD interrupt's playfield registers (bank $80 log, `$80:96A9`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayfieldSources {
    /// `video.playfield.main_screen`.
    pub main_screen: WramByte,
    /// `video.playfield.bg3_tilemap`.
    pub bg3_tilemap: WramByte,
    /// `video.playfield.colour_math`: CGWSEL, then CGADSUB.
    pub colour_math: [WramByte; 2],
    /// `video.batch.playfield_colour_math`: CGWSEL, then CGADSUB.
    pub next_colour_math: [WramByte; 2],
}

/// Where the original keeps the register mirrors beyond INIDISP, TM, TS and the scroll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MirrorSources {
    /// `.objects`.
    pub objects: WramByte,
    /// `.mode`.
    pub mode: WramByte,
    /// `.mosaic`.
    pub mosaic: WramByte,
    /// `.tilemaps`.
    pub tilemaps: [WramByte; BACKGROUNDS],
    /// `.tile_bases`.
    pub tile_bases: [WramByte; BACKGROUNDS / 2],
    /// `.mode7.select`.
    pub mode7_select: WramByte,
    /// `.mode7.matrix`, then `.mode7.centre`.
    pub mode7: [WramWord; 6],
    /// `.windows.select`.
    pub window_select: [WramByte; 3],
    /// `.windows.edges`.
    pub window_edges: [WramByte; 4],
    /// `.windows.background_logic`.
    pub background_logic: WramByte,
    /// `.windows.object_logic`.
    pub object_logic: WramByte,
    /// `.main_window_mask`.
    pub main_window_mask: WramByte,
    /// `.sub_window_mask`.
    pub sub_window_mask: WramByte,
    /// `.colour_math.select`.
    pub colour_math_select: WramByte,
    /// `.colour_math.layers`.
    pub colour_math_layers: WramByte,
    /// `video.batch.fixed_colour`.
    pub fixed_colour: [WramByte; 3],
    /// `.settings`.
    pub settings: WramByte,
}

impl Rules<'_> {
    /// The original's layout.
    pub const ORIGINAL: Rules<'static> = Rules {
        nmi: source::NMI_FRAME_COUNTER_INCLUDING_LAG,
        completed: source::NMI_FRAME_COUNTER_WORD,
        lag: source::NMI_FRAMES_MISSED,
        max_lag: source::NMI_FRAMES_MISSED_MAX,
        hours: source::GAME_TIME_HOURS,
        minutes: source::GAME_TIME_MINUTES,
        seconds: source::GAME_TIME_SECONDS,
        frames: source::GAME_TIME_FRAMES,
        rng: source::RANDOM_NUMBER,
        held: source::JOYPAD1_LASTKEYS,
        pressed: source::JOYPAD1_NEWKEYS,
        previous: source::JOYPAD1_PREV,
        repeat: source::JOYPAD1_NEWKEYS2_UNUSED,
        repeat_delay: source::JOYPAD1_KEYREPEAT_CTR,
        game_state: source::GAME_STATE,
        room: source::ROOM_PTR,
        nmi_requested: source::WAITING_FOR_NMI,
        oam_buffer: source::OAM,
        palette_buffer: source::PALETTE_BUFFER,
        vram_queue: source::VRAM_WRITE_QUEUE,
        vram_queue_tail: source::VRAM_WRITE_QUEUE_TAIL,
        samus_tiles: SamusTileSources {
            halves: source::NMI_COPY_SAMUS_HALVES,
            top: source::NMI_COPY_SAMUS_TOP_HALF_SRC,
            bottom: source::NMI_COPY_SAMUS_BOTTOM_HALF_SRC,
        },
        inidisp: source::REG_INIDISP,
        main_screen: source::REG_TM,
        sub_screen: source::REG_TS,
        scroll: source::REG_BG_SCROLL,
        mirrors: MirrorSources {
            objects: source::REG_OBSEL,
            mode: source::REG_BGMODE,
            mosaic: source::REG_MOSAIC,
            tilemaps: source::REG_BG_SC,
            tile_bases: source::REG_BG_NBA,
            mode7_select: source::REG_M7SEL,
            mode7: source::REG_M7,
            window_select: source::REG_WINDOW_SELECT,
            window_edges: source::REG_WINDOW_EDGES,
            background_logic: source::REG_WBGLOG,
            object_logic: source::REG_WOBJLOG,
            main_window_mask: source::REG_TMW,
            sub_window_mask: source::REG_TSW,
            colour_math_select: source::REG_CGWSEL,
            colour_math_layers: source::REG_CGADSUB,
            fixed_colour: source::REG_COLDATA,
            settings: source::REG_SETINI,
        },
        message_box: MessageBoxSources {
            tilemap: source::MESSAGE_BOX_TILEMAP_BACKUP,
            bg3_tilemap: source::MESSAGE_BOX_SAVED_BG3SC,
        },
        playfield: PlayfieldSources {
            main_screen: source::GAMEPLAY_TM,
            bg3_tilemap: source::GAMEPLAY_BG3SC,
            colour_math: source::GAMEPLAY_COLOUR_MATH,
            next_colour_math: source::NEXT_GAMEPLAY_COLOUR_MATH,
        },
        collected: source::COLLECTED_ITEMS,
        equipped: source::EQUIPPED_ITEMS,
        energy: source::SAMUS_HEALTH,
        max_energy: source::SAMUS_MAX_HEALTH,
        reserve: source::SAMUS_RESERVE_HEALTH,
        max_reserve: source::SAMUS_MAX_RESERVE_HEALTH,
        missiles: source::SAMUS_MISSILES,
        max_missiles: source::SAMUS_MAX_MISSILES,
        super_missiles: source::SAMUS_SUPER_MISSILES,
        max_super_missiles: source::SAMUS_MAX_SUPER_MISSILES,
        power_bombs: source::SAMUS_POWER_BOMBS,
        max_power_bombs: source::SAMUS_MAX_POWER_BOMBS,
        reserve_mode: source::RESERVE_HEALTH_MODE,
        requested_byte: source::NMI_FRAME_COUNTER_BYTE,
        hud: HudSources {
            tilemap: source::HUD_TILEMAP,
            selected: source::HUD_ITEM_INDEX,
            auto_cancel: source::SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX,
            energy: source::SAMUS_PREV_HEALTH,
            missiles: source::SAMUS_PREV_MISSILES,
            super_missiles: source::SAMUS_PREV_SUPER_MISSILES,
            power_bombs: source::SAMUS_PREV_POWER_BOMBS,
            selection: source::SAMUS_PREV_HUD_ITEM_INDEX,
            mini_map: MiniMapSources {
                explored: source::MAP_TILES_EXPLORED,
                area: source::AREA_INDEX,
                room_x: source::ROOM_X_COORDINATE_ON_MAP,
                room_y: source::ROOM_Y_COORDINATE_ON_MAP,
                map_collected: source::HAS_AREA_MAP,
                disabled: source::DEBUG_DISABLE_MINIMAP,
            },
        },
        camera_x: source::LAYER1_X_POS,
        camera_x_subpixels: source::LAYER1_X_SUBPOS,
        camera_y: source::LAYER1_Y_POS,
        camera_y_subpixels: source::LAYER1_Y_SUBPOS,
        scrolling: ScrollingSources {
            layer2: [source::LAYER2_X_POS, source::LAYER2_Y_POS],
            bg1_offset: [source::BG1_X_OFFSET, source::BG1_Y_OFFSET],
            bg2_offset: [source::BG2_X_OFFSET, source::BG2_Y_OFFSET],
            distance: source::CAMERA_DISTANCE_INDEX,
            scrollers: [source::UP_SCROLLER, source::DOWN_SCROLLER],
            layer2_scroll: [source::LAYER2_SCROLL_X, source::LAYER2_SCROLL_Y],
            drawn: [
                [
                    source::PREVIOUS_LAYER1_X_BLOCK,
                    source::PREVIOUS_LAYER1_Y_BLOCK,
                ],
                [
                    source::PREVIOUS_LAYER2_X_BLOCK,
                    source::PREVIOUS_LAYER2_Y_BLOCK,
                ],
            ],
        },
        previous_position: [
            source::SAMUS_PREV_X_POS,
            source::SAMUS_PREV_X_SUBPOS,
            source::SAMUS_PREV_Y_POS,
            source::SAMUS_PREV_Y_SUBPOS,
        ],
        strand: StrandRules {
            sites: WAIT_SITES,
            calls: CALL_SITES,
            plm_index: source::PLM_ID,
            radius: source::MESSAGE_BOX_ANIMATION_Y_RADIUS,
        },
        irq_command: source::CUR_IRQ_HANDLER,
        irq_next: source::IRQHANDLER_NEXT_HANDLER,
        message: source::MESSAGE_BOX_INDEX,
        collected_items: source::ITEM_BIT_ARRAY,
        plms: PlmSources::ORIGINAL,
        projectiles: ProjectileSources::ORIGINAL,
        enemies: EnemySources::ORIGINAL,
        animation: AnimationSources {
            frame: source::SAMUS_ANIM_FRAME,
            timer: source::SAMUS_ANIM_FRAME_TIMER,
            boost: source::SPEED_BOOSTER_LEVEL,
        },
        visor: source::SAMUS_VISOR_PALETTE_TIMER_INDEX,
        layer_blending: source::FX_LAYER_BLENDING_CONFIG_A,
        drawing: source::SAMUS_DRAW_HANDLER,
        y_radius: source::SAMUS_Y_RADIUS,
        slope_adjusted: source::SAMUS_SLOPE_ADJUSTED,
        jump_held: [
            source::SAMUS_JUMP_HELD_FRAMES,
            source::SAMUS_POSE_INPUT_HANDLER,
        ],
        elevator: ElevatorSources {
            status: source::ELEVATOR_STATUS,
            armed: source::ELEVATOR_FLAGS,
            direction: source::ELEVATOR_DIRECTION,
        },
        drawn_hook: DrawnHookSources {
            pointer: source::ENEMY_GFX_DRAWN_HOOK,
            bank: source::ENEMY_GFX_DRAWN_HOOK_BANK,
        },
        palette_cycle: PaletteCycleSources {
            palette: source::ENEMY_PALETTE_CYCLE_PALETTE,
            set: source::ENEMY_PALETTE_CYCLE_SET,
            timer: source::ENEMY_PALETTE_CYCLE_TIMER,
        },
        atmospherics: AtmosphericSources {
            frame_and_type: source::ATMOSPHERIC_GFX_FRAME_AND_TYPE,
            timer: source::ATMOSPHERIC_GFX_ANIM_TIMER,
            x: source::ATMOSPHERIC_GFX_X_POS,
            y: source::ATMOSPHERIC_GFX_Y_POS,
        },
        level: LevelSources {
            width: source::ROOM_WIDTH_IN_BLOCKS,
            height: source::ROOM_HEIGHT_IN_BLOCKS,
            blocks: source::LEVEL_DATA,
            bts: source::BTS,
            scrolls: source::SCROLLS,
            state: source::ROOMDEFROOMSTATE_PTR,
            layer2: source::CUSTOM_BACKGROUND,
        },
        area: source::AREA_INDEX,
        tilemap_gap: source::SIZE_OF_BG2,
        item_gfx: ItemGfxSources {
            next: source::PLM_ITEM_GFX_INDEX,
            pointers: source::PLM_ITEM_GFX_PTRS,
            entries: source::ITEM_GFX_TILE_TABLE,
        },
        uploads: [
            LayerUploadSources {
                column: source::BG1_COLUMN_UPDATE,
                row: source::BG1_ROW_UPDATE,
            },
            LayerUploadSources {
                column: source::BG2_COLUMN_UPDATE,
                row: source::BG2_ROW_UPDATE,
            },
        ],
        pose: PoseSources {
            pose: source::SAMUS_POSE,
            facing: source::SAMUS_POSE_X_DIR,
            movement: source::SAMUS_MOVEMENT_TYPE,
        },
        previous_pose: PoseSources {
            pose: source::SAMUS_PREV_POSE,
            facing: source::SAMUS_PREV_POSE_X_DIR,
            movement: source::SAMUS_PREV_MOVEMENT_TYPE2,
        },
        last_different_pose: PoseSources {
            pose: source::SAMUS_LAST_DIFFERENT_POSE,
            facing: source::SAMUS_LAST_DIFFERENT_POSE_X_DIR,
            movement: source::SAMUS_LAST_DIFFERENT_POSE_MOVEMENT_TYPE,
        },
    };

    /// The State `record` shows, under these rules.
    ///
    /// # Errors
    ///
    /// A [`DecodeError`] when the record is short, names an unknown room or is in a state
    /// not decoded yet, and [`DecodeError::UnknownYieldSite`] when a requested NMI found the
    /// main thread somewhere the tables don't know.
    pub fn decode(&self, record: &FrameRecord, rooms: &RoomMap) -> Result<Decoded, DecodeError> {
        let wram = &*record.wram;
        let game_state = self.game_state.read(wram)?;
        if game_state != MAIN_GAMEPLAY {
            return Err(DecodeError::Unsupported(Unsupported::GameState(game_state)));
        }
        let header = RoomHeader(self.room.read(wram)?);
        let room = rooms.get(header).ok_or(DecodeError::UnknownRoom(header))?;
        let nmi = match record.cut {
            Cut::NmiReturn(nmi) => nmi,
            Cut::VblankWithoutNmi | Cut::NmiStillRunning { .. } => {
                return Err(DecodeError::Unsupported(Unsupported::NoNmi(record.cut)));
            }
        };
        let (last, main, quiescence) = match nmi.kind {
            NmiKind::Requested => (
                NmiTiming::OnTime,
                self.strand
                    .main(nmi.interrupted_pc, record.cpu.s, record.cpu.x, wram)?,
                Quiescence::Quiet,
            ),
            NmiKind::Overrun => (NmiTiming::Overrun, MainStrand::default(), Quiescence::Busy),
        };
        let irq = self.irq(record)?;

        let state = State {
            main,
            irq,
            clock: Clock {
                counters: Counters {
                    nmi: FrameCount(self.nmi.read(wram)?),
                    completed: FrameCount(self.completed.read(wram)?),
                    lag: FrameCount(u16::from(self.lag.read(wram)?)),
                    max_lag: FrameCount(u16::from(self.max_lag.read(wram)?)),
                    requested_byte: self.requested_byte.read(wram)?,
                },
                game_time: GameTime {
                    hours: self.hours.read(wram)?,
                    minutes: self.minutes.read(wram)?,
                    seconds: self.seconds.read(wram)?,
                    frames: self.frames.read(wram)?,
                },
                ..Clock::default()
            },
            rng: Rng {
                word: self.rng.read(wram)?,
                ..Rng::default()
            },
            controller: Controller {
                auto_read: buttons(record.pad.0),
                held: buttons(self.held.read(wram)?),
                pressed: buttons(self.pressed.read(wram)?),
                previous: buttons(self.previous.read(wram)?),
                repeat: buttons(self.repeat.read(wram)?),
                repeat_delay: Frames(self.repeat_delay.read(wram)?),
            },
            mode: GameMode::Gameplay,
            timing: Timing {
                last,
                ..Timing::default()
            },
            video: self.video(record, rooms)?,
            samus: self.samus(wram, &rooms.samus_clips)?,
            atmospherics: self.atmospherics.read(wram)?,
            room: self.room(wram, rooms, room)?,
            elevator: self.elevator.read(wram)?,
            enemy_palette_cycle: self.palette_cycle.read(wram)?,
            message: Message(self.message.read(wram)?),
            collected_items: collected_items(self.collected_items, wram)?,
            hud: self.hud.read(wram)?,
            sram: save_ram(&record.sram)?,
            projectiles: self.projectiles.read(wram)?,
        };
        Ok(Decoded { state, quiescence })
    }

    /// The room: its level, its camera and its slots.
    fn room(&self, wram: &[u8], rooms: &RoomMap, room: RoomId) -> Result<RoomScoped, DecodeError> {
        let Level {
            blocks,
            layer2,
            tail,
            tileset,
            scroll_areas,
            scrolls_below,
        } = self.level.read(wram, rooms)?;
        Ok(RoomScoped {
            room,
            blocks,
            width: BlockColumns(self.level.width.read(wram)?),
            height: BlockRows(self.level.height.read(wram)?),
            layer2,
            level_tail: tail,
            area: area(self.area, wram)?,
            tileset,
            tilemap_gap: VramWord(self.tilemap_gap.read(wram)?),
            layer_blending: LayerBlending(self.layer_blending.read(wram)?),
            scroll_areas,
            scrolls_below,
            camera: Camera {
                x: Fixed {
                    pixels: self.camera_x.read(wram)?,
                    subpixels: self.camera_x_subpixels.read(wram)?,
                },
                y: Fixed {
                    pixels: self.camera_y.read(wram)?,
                    subpixels: self.camera_y_subpixels.read(wram)?,
                },
                ..self.scrolling.read(wram)?
            },
            plms: self.plms.read(
                wram,
                rooms.population(room),
                rooms.scroll_lists(room),
                &rooms.reactions,
            )?,
            item_gfx: self.item_gfx.read(wram)?,
            enemies: self.enemies.read(wram, &rooms.species, &rooms.programs)?,
            drawing: EnemyDrawing {
                hook: self.drawn_hook.read(wram)?,
                ..EnemyDrawing::default()
            },
            ..RoomScoped::default()
        })
    }

    /// Samus: her position, pose and equipment, her animation's step read through her pose's
    /// clip in `clips`.
    ///
    /// # Errors
    ///
    /// [`DecodeError`] when a word is outside `wram` or holds a code the State has no value
    /// for.
    pub fn samus(&self, wram: &[u8], clips: &Clips) -> Result<Samus, DecodeError> {
        let pose = self.pose.read(wram)?;
        Ok(Samus {
            motion: crate::harness::movement::decode(wram)?,
            equipment: Equipment {
                collected: items(self.collected, wram)?,
                equipped: items(self.equipped, wram)?,
                energy: gauge(self.energy, self.max_energy, wram)?,
                reserve: gauge(self.reserve, self.max_reserve, wram)?,
                missiles: gauge(self.missiles, self.max_missiles, wram)?,
                super_missiles: gauge(self.super_missiles, self.max_super_missiles, wram)?,
                power_bombs: gauge(self.power_bombs, self.max_power_bombs, wram)?,
                reserve_mode: reserve_mode(self.reserve_mode, wram)?,
            },
            previous_position: {
                let [x, x_subpixels, y, y_subpixels] = self.previous_position;
                Point {
                    x: Fixed {
                        pixels: x.read(wram)?,
                        subpixels: x_subpixels.read(wram)?,
                    },
                    y: Fixed {
                        pixels: y.read(wram)?,
                        subpixels: y_subpixels.read(wram)?,
                    },
                }
            },
            pose: pose.pose,
            previous: self.previous_pose.read(wram)?,
            last_different: self.last_different_pose.read(wram)?,
            facing: pose.facing,
            movement: pose.movement,
            animation: self.animation.read(wram, pose.pose, clips)?,
            visor: visor(self.visor, wram)?,
            drawing: drawing_handler(self.drawing, wram)?,
            y_radius: self.y_radius.read(wram)?,
            slope_adjusted: flag_word(self.slope_adjusted, wram)?,
            jump_held: jump_held(self.jump_held, wram)?,
            ..Samus::default()
        })
    }

    /// Video: the PPU's memories and registers, the NMI request and the batch.
    fn video(&self, record: &FrameRecord, rooms: &RoomMap) -> Result<Video, DecodeError> {
        let wram = &*record.wram;
        Ok(Video {
            memory: video_memory(record)?,
            committed: committed(&record.ppu)?,
            playfield: self.playfield(wram)?,
            message_box: self.message_box.read(wram)?,
            dma: record.ppu.dma.map(dma_channel),
            nmi_requested: flag(self.nmi_requested, wram)?,
            batch: NmiBatch {
                registers: self.mirrors(wram)?,
                fixed_colour: bytes(self.mirrors.fixed_colour, wram)?.map(ColourWrite),
                oam: oam(self.oam_buffer.read(wram)?)?,
                cgram: colours(self.palette_buffer.read(wram)?),
                vram: self.vram_queue(wram)?,
                samus_tiles: self.samus_tiles.read(wram, &rooms.samus_tiles)?,
                level: {
                    let [bg1, bg2] = self.uploads;
                    LevelUploads {
                        bg1: bg1.read(wram)?,
                        bg2: bg2.read(wram)?,
                    }
                },
                playfield_colour_math: playfield_colour_math(
                    self.playfield.next_colour_math,
                    wram,
                )?,
                ..NmiBatch::default()
            },
        })
    }

    /// The HUD interrupt's playfield registers.
    fn playfield(&self, wram: &[u8]) -> Result<Playfield, DecodeError> {
        let sources = self.playfield;
        Ok(Playfield {
            main_screen: layers(sources.main_screen.read(wram)?, sources.main_screen.name)?,
            bg3_tilemap: TilemapBase(sources.bg3_tilemap.read(wram)?),
            colour_math: playfield_colour_math(sources.colour_math, wram)?,
        })
    }

    /// The register mirrors `$80:91EE` writes to the PPU. The fixed colour is left at
    /// black: the mirrors hold COLDATA's writes, which `fixed_colour` decodes.
    fn mirrors(&self, wram: &[u8]) -> Result<Registers, DecodeError> {
        let (forced_blank, brightness) = display(self.inidisp.read(wram)?);
        let mut scroll = [Scroll::default(); BACKGROUNDS];
        for (layer, [x, y]) in scroll.iter_mut().zip(self.scroll) {
            *layer = Scroll {
                x: x.read(wram)?,
                y: y.read(wram)?,
            };
        }
        let sources = &self.mirrors;
        let mut mode7 = [0_i16; 6];
        for (value, word) in mode7.iter_mut().zip(sources.mode7) {
            *value = i16::from_le_bytes(word.read(wram)?.to_le_bytes());
        }
        let [m7a, m7b, m7c, m7d, m7x, m7y] = mode7;
        let layers_at = |source: WramByte| layers(source.read(wram)?, source.name);
        Ok(Registers {
            forced_blank,
            brightness,
            objects: ObjectSelect(sources.objects.read(wram)?),
            mode: BgMode(sources.mode.read(wram)?),
            mosaic: Mosaic(sources.mosaic.read(wram)?),
            tilemaps: bytes(sources.tilemaps, wram)?.map(TilemapBase),
            tile_bases: bytes(sources.tile_bases, wram)?.map(TileBases),
            scroll,
            mode7: Mode7 {
                select: Mode7Select(sources.mode7_select.read(wram)?),
                matrix: [m7a, m7b, m7c, m7d],
                centre: [m7x, m7y],
            },
            windows: Windows {
                select: bytes(sources.window_select, wram)?.map(WindowSettings),
                edges: bytes(sources.window_edges, wram)?,
                background_logic: WindowSettings(sources.background_logic.read(wram)?),
                object_logic: WindowSettings(sources.object_logic.read(wram)?),
            },
            main_screen: layers_at(self.main_screen)?,
            sub_screen: layers_at(self.sub_screen)?,
            main_window_mask: layers_at(sources.main_window_mask)?,
            sub_window_mask: layers_at(sources.sub_window_mask)?,
            colour_math: ColourMath {
                select: ColourMathSelect(sources.colour_math_select.read(wram)?),
                layers: ColourMathLayers(sources.colour_math_layers.read(wram)?),
                fixed: FixedColour::default(),
            },
            settings: DisplaySettings(sources.settings.read(wram)?),
        })
    }

    /// The VRAM write queue up to its end (bank $80 log, `$80:8C83`). An upload from work
    /// RAM carries the words the DMA reads; one from ROM is a content reference.
    fn vram_queue(&self, wram: &[u8]) -> Result<Vec<VramTransfer>, DecodeError> {
        let tail = self.vram_queue_tail.read(wram)?;
        let queue = self.vram_queue.read(wram)?.get(..usize::from(tail)).ok_or(
            DecodeError::Unrepresentable {
                source: self.vram_queue_tail.name,
                value: tail,
            },
        )?;
        let (entries, rest) = queue.as_chunks::<VRAM_QUEUE_ENTRY>();
        if !rest.is_empty() {
            return Err(DecodeError::Unrepresentable {
                source: self.vram_queue_tail.name,
                value: tail,
            });
        }
        entries
            .iter()
            .map(
                |&[
                    size_low,
                    size_high,
                    from_low,
                    from_high,
                    bank,
                    to_low,
                    to_high,
                ]| {
                    let size = u16::from_le_bytes([size_low, size_high]);
                    // A DMA of size 0 moves 64 KiB.
                    let bytes = if size == 0 {
                        0x1_0000
                    } else {
                        usize::from(size)
                    };
                    if bytes % 2 != 0 {
                        return Err(DecodeError::Unrepresentable {
                            source: self.vram_queue.name,
                            value: size,
                        });
                    }
                    let to = u16::from_le_bytes([to_low, to_high]);
                    let from = u16::from_le_bytes([from_low, from_high]);
                    Ok(VramTransfer {
                        to: VramWord(to & VRAM_ADDRESS_BITS),
                        step: if to & VRAM_COLUMN_BIT == 0 {
                            VramStep::Row
                        } else {
                            VramStep::Column
                        },
                        source: vram_source(wram, bank, from, bytes),
                    })
                },
            )
            .collect()
    }

    /// The interrupt strand: the IRQ handler's command, at the line VTIME holds.
    fn irq(&self, record: &FrameRecord) -> Result<IrqStrand, DecodeError> {
        let current = self.irq_command.read(&record.wram)?;
        let next = self.irq_next.read(&record.wram)?;
        let command =
            strand::irq_command(current)
                .filter(|_| next == 0)
                .ok_or(DecodeError::Unsupported(Unsupported::IrqCommand {
                    current,
                    next,
                }))?;
        Ok(IrqStrand {
            command,
            line: Line(record.ppu.vtime),
        })
    }
}

/// The State `record` shows, under the original's layout.
///
/// # Errors
///
/// As [`Rules::decode`].
pub fn decode(record: &FrameRecord, rooms: &RoomMap) -> Result<Decoded, DecodeError> {
    Rules::ORIGINAL.decode(record, rooms)
}

/// The buttons in declaration order, which is `FlagSet`'s bit order.
const BUTTONS: [Button; 12] = [
    Button::B,
    Button::Y,
    Button::Select,
    Button::Start,
    Button::Up,
    Button::Down,
    Button::Left,
    Button::Right,
    Button::A,
    Button::X,
    Button::L,
    Button::R,
];

/// A button's bit in a JOY1-order controller word ($4218, and the game's copies of it).
const fn joy1_bit(button: Button) -> u16 {
    match button {
        Button::B => 0x8000,
        Button::Y => 0x4000,
        Button::Select => 0x2000,
        Button::Start => 0x1000,
        Button::Up => 0x0800,
        Button::Down => 0x0400,
        Button::Left => 0x0200,
        Button::Right => 0x0100,
        Button::A => 0x0080,
        Button::X => 0x0040,
        Button::L => 0x0020,
        Button::R => 0x0010,
    }
}

/// The buttons a JOY1-order word holds; its low four bits are the pad's signature, not
/// buttons.
#[must_use]
pub fn buttons(word: u16) -> Buttons {
    // Fold from the last button, so each shift moves the earlier ones toward their bits.
    let bits = BUTTONS.iter().rev().fold(0_u32, |bits, &button| {
        bits.wrapping_shl(1) | u32::from(word & joy1_bit(button) != 0)
    });
    FlagSet {
        bits,
        ..FlagSet::default()
    }
}

/// A flag the original keeps as a byte it only ever sets to 1 and clears to 0.
fn flag(source: WramByte, wram: &[u8]) -> Result<bool, DecodeError> {
    match source.read(wram)? {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(DecodeError::Unrepresentable {
            source: source.name,
            value: u16::from(value),
        }),
    }
}

/// A gauge from its current and ceiling words.
fn gauge(current: WramWord, max: WramWord, wram: &[u8]) -> Result<Gauge, DecodeError> {
    Ok(Gauge {
        current: current.read(wram)?,
        max: max.read(wram)?,
    })
}

/// The items in declaration order, which is `FlagSet`'s bit order.
const ITEMS: [Item; 11] = [
    Item::VariaSuit,
    Item::SpringBall,
    Item::MorphBall,
    Item::ScrewAttack,
    Item::GravitySuit,
    Item::HiJumpBoots,
    Item::SpaceJump,
    Item::Bombs,
    Item::SpeedBooster,
    Item::GrappleBeam,
    Item::XRayScope,
];

/// An item's bit in `EQUIPPED_ITEMS` and `COLLECTED_ITEMS` (bank $84 log: the item PLMs'
/// equipment instruction).
const fn item_bit(item: Item) -> u16 {
    match item {
        Item::VariaSuit => 0x0001,
        Item::SpringBall => 0x0002,
        Item::MorphBall => 0x0004,
        Item::ScrewAttack => 0x0008,
        Item::GravitySuit => 0x0020,
        Item::HiJumpBoots => 0x0100,
        Item::SpaceJump => 0x0200,
        Item::Bombs => 0x1000,
        Item::SpeedBooster => 0x2000,
        Item::GrappleBeam => 0x4000,
        Item::XRayScope => 0x8000,
    }
}

/// The reserve tanks' mode word.
fn reserve_mode(source: WramWord, wram: &[u8]) -> Result<ReserveMode, DecodeError> {
    match source.read(wram)? {
        0 => Ok(ReserveMode::None),
        1 => Ok(ReserveMode::Automatic),
        2 => Ok(ReserveMode::Manual),
        value => Err(DecodeError::Unrepresentable {
            source: source.name,
            value,
        }),
    }
}

/// A HUD item index: 0 for none, then the HUD's items left to right (`$80:9D6E`).
fn hud_item(source: WramWord, wram: &[u8]) -> Result<Option<HudItem>, DecodeError> {
    match source.read(wram)? {
        0 => Ok(None),
        1 => Ok(Some(HudItem::Missiles)),
        2 => Ok(Some(HudItem::SuperMissiles)),
        3 => Ok(Some(HudItem::PowerBombs)),
        4 => Ok(Some(HudItem::GrappleBeam)),
        5 => Ok(Some(HudItem::XRayScope)),
        value => Err(DecodeError::Unrepresentable {
            source: source.name,
            value,
        }),
    }
}

/// The collected item bits, a word per 16.
fn collected_items(block: WramBlock, wram: &[u8]) -> Result<CollectedItems, DecodeError> {
    let mut items = CollectedItems::default();
    let (words, _) = block.read(wram)?.as_chunks::<2>();
    for (item, word) in items.0.iter_mut().zip(words) {
        *item = u16::from_le_bytes(*word);
    }
    Ok(items)
}

impl MessageBoxSources {
    /// What the message box saved.
    fn read(&self, wram: &[u8]) -> Result<MessageBoxSaved, DecodeError> {
        let words = self.tilemap.read(wram)?.as_chunks::<2>().0;
        Ok(MessageBoxSaved {
            tilemap: words.iter().map(|&word| u16::from_le_bytes(word)).collect(),
            bg3_tilemap: TilemapBase(self.bg3_tilemap.read(wram)?),
        })
    }
}

impl HudSources {
    /// The HUD the record shows.
    fn read(&self, wram: &[u8]) -> Result<Hud, DecodeError> {
        let mut tilemap = HudTilemap::default();
        let (words, _) = self.tilemap.read(wram)?.as_chunks::<2>();
        for (cell, word) in tilemap.cells.iter_mut().zip(words) {
            *cell = TilemapEntry::from_word(u16::from_le_bytes(*word));
        }
        Ok(Hud {
            tilemap,
            selected: hud_item(self.selected, wram)?,
            auto_cancel: hud_item(self.auto_cancel, wram)?,
            drawn: Drawn {
                energy: self.energy.read(wram)?,
                missiles: self.missiles.read(wram)?,
                super_missiles: self.super_missiles.read(wram)?,
                power_bombs: self.power_bombs.read(wram)?,
                selection: hud_item(self.selection, wram)?,
            },
            mini_map: self.mini_map.read(wram)?,
        })
    }
}

/// A flag word the original sets to 0 or 1 only.
fn flag_word(source: WramWord, wram: &[u8]) -> Result<bool, DecodeError> {
    match source.read(wram)? {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(DecodeError::Unrepresentable {
            source: source.name,
            value,
        }),
    }
}

/// A map coordinate word: the area map is 64 cells by 32, so its high byte is clear.
fn map_coordinate(source: WramWord, wram: &[u8]) -> Result<u8, DecodeError> {
    let value = source.read(wram)?;
    u8::try_from(value).map_err(|_| DecodeError::Unrepresentable {
        source: source.name,
        value,
    })
}

/// The area an area index names.
fn area(source: WramWord, wram: &[u8]) -> Result<Area, DecodeError> {
    let index = source.read(wram)?;
    u8::try_from(index)
        .ok()
        .and_then(|index| Area::try_from(index).ok())
        .ok_or(DecodeError::Unrepresentable {
            source: source.name,
            value: index,
        })
}

impl MiniMapSources {
    /// What the record's mini-map draws from.
    fn read(&self, wram: &[u8]) -> Result<MiniMap, DecodeError> {
        let explored: &[u8; MAP_BIT_BYTES] =
            self.explored
                .read(wram)?
                .first_chunk()
                .ok_or(DecodeError::WramTooShort {
                    source: self.explored.name,
                })?;
        Ok(MiniMap {
            explored: MapCells::from_bitmap(explored),
            area: area(self.area, wram)?,
            room: MapPosition {
                x: map_coordinate(self.room_x, wram)?,
                y: map_coordinate(self.room_y, wram)?,
            },
            map_collected: flag_word(self.map_collected, wram)?,
            disabled: flag_word(self.disabled, wram)?,
        })
    }
}

/// The items an equipment word holds.
///
/// No routine of the original sets the word's five unused bits, and the State has no place to
/// keep them, so a word with any of them set is [`DecodeError::Unrepresentable`] rather than
/// decoded without them.
fn items(source: WramWord, wram: &[u8]) -> Result<Items, DecodeError> {
    let word = source.read(wram)?;
    let unknown = ITEMS
        .iter()
        .fold(word, |unknown, &item| unknown & !item_bit(item));
    if unknown != 0 {
        return Err(DecodeError::Unrepresentable {
            source: source.name,
            value: unknown,
        });
    }
    // Fold from the last item, so each shift moves the earlier ones toward their bits.
    let bits = ITEMS.iter().rev().fold(0_u32, |bits, &item| {
        bits.wrapping_shl(1) | u32::from((word & item_bit(item)) != 0)
    });
    Ok(FlagSet {
        bits,
        ..FlagSet::default()
    })
}

/// VRAM, CGRAM and OAM, straight from the record.
fn video_memory(record: &FrameRecord) -> Result<VideoMemory, DecodeError> {
    if record.vram.len() != VRAM_WORDS {
        return Err(DecodeError::WrongSize {
            memory: Memory::Vram,
            expected: VRAM_WORDS,
            found: record.vram.len(),
        });
    }
    let cgram =
        <&[u16; CGRAM_COLOURS]>::try_from(&*record.cgram).map_err(|_| DecodeError::WrongSize {
            memory: Memory::Cgram,
            expected: CGRAM_COLOURS,
            found: record.cgram.len(),
        })?;
    Ok(VideoMemory {
        vram: record.vram.to_vec(),
        cgram: cgram.map(Colour),
        oam: oam(&record.oam)?,
    })
}

/// The save RAM, straight from the record.
fn save_ram(bytes: &[u8]) -> Result<SaveRam, DecodeError> {
    let image = <&[u8; SAVE_RAM_BYTES]>::try_from(bytes).map_err(|_| DecodeError::WrongSize {
        memory: Memory::Sram,
        expected: SAVE_RAM_BYTES,
        found: bytes.len(),
    })?;
    Ok(SaveRam::from_bytes(image))
}

/// OAM in the PPU's layout, from the record's OAM or the original's buffer for it.
fn oam(bytes: &[u8]) -> Result<Oam, DecodeError> {
    let oam_size = DecodeError::WrongSize {
        memory: Memory::Oam,
        expected: OAM_LOW_BYTES.wrapping_add(OAM_HIGH_BYTES),
        found: bytes.len(),
    };
    let (low, high) = bytes.split_first_chunk::<OAM_LOW_BYTES>().ok_or(oam_size)?;
    let high = <[u8; OAM_HIGH_BYTES]>::try_from(high).map_err(|_| oam_size)?;
    let mut oam = Oam {
        high,
        ..Oam::default()
    };
    for (entry, &[x, y, tile, attributes]) in oam.low.iter_mut().zip(low.as_chunks::<4>().0) {
        *entry = OamEntry {
            x,
            y,
            tile,
            attributes,
        };
    }
    Ok(oam)
}

/// A VRAM write queue entry's size in bytes.
const VRAM_QUEUE_ENTRY: usize = 7;
/// The queue's destination bit for the 32-word step.
const VRAM_COLUMN_BIT: u16 = 0x8000;
/// The destination's word address.
const VRAM_ADDRESS_BITS: u16 = 0x7FFF;
/// INIDISP's forced blank bit.
const FORCED_BLANK: u8 = 0x80;
/// INIDISP's brightness bits.
const BRIGHTNESS: u8 = 0x0F;
/// TM's and TS's layer bits, BG1 to BG4 then sprites: `Layer`'s declaration order.
const LAYER_BITS: u8 = 0x1F;

/// Forced blank and brightness from INIDISP's value.
const fn display(inidisp: u8) -> (bool, Brightness) {
    (
        inidisp & FORCED_BLANK != 0,
        Brightness(inidisp & BRIGHTNESS),
    )
}

/// The layers a TM or TS value from `source` enables.
///
/// # Errors
///
/// [`DecodeError::Unrepresentable`] when it sets a bit the PPU has no layer for.
fn layers(screen: u8, source: &'static str) -> Result<Layers, DecodeError> {
    if screen & !LAYER_BITS != 0 {
        return Err(DecodeError::Unrepresentable {
            source,
            value: u16::from(screen),
        });
    }
    Ok(FlagSet {
        bits: u32::from(screen),
        ..FlagSet::default()
    })
}

/// Little-endian BGR555 colours.
fn colours(bytes: &[u8]) -> [Colour; CGRAM_COLOURS] {
    let mut colours = [Colour::default(); CGRAM_COLOURS];
    for (colour, &pair) in colours.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *colour = Colour(u16::from_le_bytes(pair));
    }
    colours
}

/// Where an upload's bytes come from: work RAM in banks $7E and $7F, or ROM.
fn vram_source(wram: &[u8], bank: u8, from: u16, bytes: usize) -> VramSource {
    let start = match bank {
        0x7E => usize::from(from),
        0x7F => usize::from(from) | 0x1_0000,
        _ => {
            return VramSource::Content(RomWords {
                from: GraphicsRef { bank, offset: from },
                words: u16::try_from(bytes / 2).unwrap_or(u16::MAX),
            });
        }
    };
    let words = (0..bytes / 2)
        .map(|index| {
            let at = start.wrapping_add(index.wrapping_mul(2));
            let byte = |offset: usize| wram.get(at.wrapping_add(offset)).copied().unwrap_or(0);
            u16::from_le_bytes([byte(0), byte(1)])
        })
        .collect();
    VramSource::Words {
        from: BusAddress { bank, offset: from },
        words,
    }
}

/// The bytes at `sources`.
fn bytes<const N: usize>(sources: [WramByte; N], wram: &[u8]) -> Result<[u8; N], DecodeError> {
    let mut values = [0; N];
    for (value, source) in values.iter_mut().zip(sources) {
        *value = source.read(wram)?;
    }
    Ok(values)
}

/// CGWSEL and CGADSUB from their two bytes.
fn playfield_colour_math(
    sources: [WramByte; 2],
    wram: &[u8],
) -> Result<PlayfieldColourMath, DecodeError> {
    let [select, layers] = bytes(sources, wram)?;
    Ok(PlayfieldColourMath {
        select: ColourMathSelect(select),
        layers: ColourMathLayers(layers),
    })
}

/// A DMA channel's registers, from the record's `$43x0`-`$43xA` bytes.
const fn dma_channel(bytes: [u8; 11]) -> DmaChannel {
    let [
        control,
        destination,
        source_low,
        source_high,
        source_bank,
        count_low,
        count_high,
        indirect_bank,
        table_low,
        table_high,
        line_counter,
    ] = bytes;
    DmaChannel {
        control: DmaControl::from_byte(control),
        destination: BBusRegister::from_byte(destination),
        source: u16::from_le_bytes([source_low, source_high]),
        source_bank,
        count: u16::from_le_bytes([count_low, count_high]),
        indirect_bank,
        table: u16::from_le_bytes([table_low, table_high]),
        line_counter,
    }
}

/// The registers the PPU holds at the cut, from the record's register struct.
fn committed(ppu: &PpuRegisters) -> Result<Registers, DecodeError> {
    let [main, sub, main_mask, sub_mask] = ppu.screens;
    let (forced_blank, brightness) = display(ppu.inidisp);
    let scroll = ppu.bg_scroll.map(|[x, y]| Scroll { x, y });
    let [m7a, m7b, m7c, m7d, m7x, m7y] = ppu.m7;
    let [red, green, blue] = ppu.fixed_color;
    Ok(Registers {
        forced_blank,
        brightness,
        objects: ObjectSelect(ppu.obsel),
        mode: BgMode(ppu.bgmode),
        mosaic: Mosaic(ppu.mosaic),
        tilemaps: ppu.bg_sc.map(TilemapBase),
        tile_bases: [TileBases(ppu.bg12nba), TileBases(ppu.bg34nba)],
        scroll,
        mode7: Mode7 {
            select: Mode7Select(ppu.m7sel),
            matrix: [m7a, m7b, m7c, m7d],
            centre: [m7x, m7y],
        },
        windows: Windows {
            select: ppu.window_select.map(WindowSettings),
            edges: ppu.window_positions,
            background_logic: WindowSettings(ppu.wbglog),
            object_logic: WindowSettings(ppu.wobjlog),
        },
        main_screen: layers(main, "ppu.screens[0]")?,
        sub_screen: layers(sub, "ppu.screens[1]")?,
        main_window_mask: layers(main_mask, "ppu.screens[2]")?,
        sub_window_mask: layers(sub_mask, "ppu.screens[3]")?,
        colour_math: ColourMath {
            select: ColourMathSelect(ppu.cgwsel),
            layers: ColourMathLayers(ppu.cgadsub),
            fixed: FixedColour { red, green, blue },
        },
        settings: DisplaySettings(ppu.setini),
    })
}

/// Where one pose record is: the pose word and the X direction and movement type bytes the
/// original keeps beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoseSources {
    /// The pose.
    pub pose: WramWord,
    /// Its X direction.
    pub facing: WramByte,
    /// Its movement type.
    pub movement: WramByte,
}

impl AnimationSources {
    /// The animation in `wram`, her step through the step map of `pose`'s clip in `clips`.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Unrepresentable`] for a frame on no step of her pose's clip (an operand
    /// byte, or a pose outside her domain, which has no clip), a time left past 255, or a boost
    /// stage past 4.
    fn read(self, wram: &[u8], pose: PoseId, clips: &Clips) -> Result<Animation, DecodeError> {
        let unrepresentable = |source: WramWord, value| DecodeError::Unrepresentable {
            source: source.name,
            value,
        };
        let frame = self.frame.read(wram)?;
        let step = clips
            .of(pose)
            .and_then(|imported| imported.step_of(AnimationFrame(frame)))
            .ok_or_else(|| unrepresentable(self.frame, frame))?;
        let timer = self.timer.read(wram)?;
        let frames_left = u8::try_from(timer).map_err(|_| unrepresentable(self.timer, timer))?;
        let boost = self.boost.read(wram)?;
        let [laps_left, stage] = boost.to_le_bytes();
        let stage = BOOST_STAGES
            .get(usize::from(stage))
            .copied()
            .ok_or_else(|| unrepresentable(self.boost, boost))?;
        Ok(Animation {
            step: ClipStep(step.0),
            frames_left,
            boost: Boost { stage, laps_left },
        })
    }
}

/// Her run's boost stages, by the number the speed booster's counter holds.
const BOOST_STAGES: [BoostStage; 5] = [
    BoostStage::Stage0,
    BoostStage::Stage1,
    BoostStage::Stage2,
    BoostStage::Stage3,
    BoostStage::Boosting,
];

/// Samus's visor cycle from its word: the timer in the low byte, in the high byte the byte
/// offset of the next colour in the visor colours (`$9B:A3C0`), whose cycle is the last three.
///
/// # Errors
///
/// [`DecodeError::Unrepresentable`] for a timer of 0, which `$91:D83F` never leaves, or an
/// offset outside the cycle.
fn visor(source: WramWord, wram: &[u8]) -> Result<Visor, DecodeError> {
    let word = source.read(wram)?;
    let [timer, offset] = word.to_le_bytes();
    let shade = match offset {
        6 => VisorShade::Bright,
        8 => VisorShade::Dim,
        0x0a => VisorShade::Dark,
        _ => {
            return Err(DecodeError::Unrepresentable {
                source: source.name,
                value: word,
            });
        }
    };
    if timer == 0 {
        return Err(DecodeError::Unrepresentable {
            source: source.name,
            value: word,
        });
    }
    Ok(Visor {
        shade,
        timer: VisorTimer(timer),
    })
}

/// Where Samus's tile uploads are: the two halves' flags, a byte each, then each half's
/// tiles definition pointer in bank $92 (bank $92 log, `$92:8000`; bank $80 log, `$80:9376`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SamusTileSources {
    /// The top half's flag, then the bottom half's.
    pub halves: WramWord,
    /// The top half's definition.
    pub top: WramWord,
    /// The bottom half's definition.
    pub bottom: WramWord,
}

impl SamusTileSources {
    /// The uploads: a half whose flag is 1 uploads the definition its pointer names; one
    /// whose flag and pointer are 0 (as the main loop leaves them, `$82:8948`) none.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Unrepresentable`] for a flag other than 0 or 1, a pointer beside a
    /// clear flag, or a pointer that names no definition the frames reach.
    fn read(self, wram: &[u8], tiles: &SamusTiles) -> Result<SamusTileUploads, DecodeError> {
        let halves = self.halves.read(wram)?;
        let [top_flag, bottom_flag] = halves.to_le_bytes();
        let half = |flag: u8, pointer: WramWord| {
            let value = pointer.read(wram)?;
            let unrepresentable =
                |source: &'static str, value| DecodeError::Unrepresentable { source, value };
            match flag {
                0 if value == 0 => Ok(None),
                0 => Err(unrepresentable(pointer.name, value)),
                1 => tiles
                    .at_pointer(value)
                    .map(Some)
                    .ok_or_else(|| unrepresentable(pointer.name, value)),
                _ => Err(unrepresentable(self.halves.name, halves)),
            }
        };
        Ok(SamusTileUploads {
            top: half(top_flag, self.top)?,
            bottom: half(bottom_flag, self.bottom)?,
        })
    }
}

/// Samus's drawing handler from its bank $90 routine pointer (bank $90 log, `$90:EB4B`).
///
/// # Errors
///
/// [`DecodeError::Unrepresentable`] for any other word, the RTS `$90:EBF2` included: only
/// unused code (`$90:E35A`) stores it.
fn drawing_handler(source: WramWord, wram: &[u8]) -> Result<DrawingHandler, DecodeError> {
    Ok(match source.read(wram)? {
        0xEB52 => DrawingHandler::Default,
        0xEB86 => DrawingHandler::FiringGrapple,
        0xEBF3 => DrawingHandler::ShinesparkCrashEchoCircle,
        0xEC14 => DrawingHandler::UsingElevator,
        0xEC1D => DrawingHandler::Inanimate,
        0xE90E => DrawingHandler::Hidden,
        value => {
            return Err(DecodeError::Unrepresentable {
                source: source.name,
                value,
            });
        }
    })
}

/// Bank $90 log: the pose input handler the frame after a landing or an animation's end.
const JUMP_HELD_THROUGH_INPUT: u16 = 0xE926;

/// Jump held through: the count, and whether the pose input handler is the one a landing or an
/// animation's end installs; every other handler is the same to jump held through.
fn jump_held([frames, input]: [WramWord; 2], wram: &[u8]) -> Result<JumpHeld, DecodeError> {
    Ok(JumpHeld {
        frames: frames.read(wram)?,
        before: if input.read(wram)? == JUMP_HELD_THROUGH_INPUT {
            FrameBefore::LandedOrEnded
        } else {
            FrameBefore::Other
        },
    })
}

/// The elevator properties word for Samus in the elevator's doorway (bank $82 log).
const IN_DOORWAY: u16 = 1;

/// The elevator properties word after an elevator door: the door's elevator property bit
/// (bank $82 log, door loading).
const ELEVATOR_DOOR: u16 = 0x80;

/// The elevator direction word for a departure upwards (bank $A3 log, `$A3:9581`).
const UPWARDS: u16 = 0x8000;

/// The address of an `RTL` in each enemy AI bank, which loading the enemies and the face
/// block's init without the morph ball set as the hook (`$A0:8A37`, `$A8:E842`).
const RTL: u16 = 0x804C;
/// The face block's glow (bank $A8 log, `$A8:E86E`).
const FACE_BLOCK_GLOW: u16 = 0xE86E;
/// The face block's AI bank.
const FACE_BLOCK_BANK: u8 = 0xA8;
/// The banks whose `RTL` the rows set: loading the enemies' and the face block's.
const RTL_BANKS: [u8; 2] = [0xA0, 0xA8];

impl DrawnHookSources {
    /// The hook in `wram`.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Unrepresentable`] for a hook other than those banks' `RTL` or the
    /// face block's glow.
    fn read(self, wram: &[u8]) -> Result<GraphicsDrawnHook, DecodeError> {
        let pointer = self.pointer.read(wram)?;
        let bank = self.bank.read(wram)?;
        match (bank, pointer) {
            (bank, RTL) if RTL_BANKS.contains(&bank) => Ok(GraphicsDrawnHook::None),
            (FACE_BLOCK_BANK, FACE_BLOCK_GLOW) => Ok(GraphicsDrawnHook::FaceBlockGlow),
            (bank, RTL | FACE_BLOCK_GLOW) => Err(DecodeError::Unrepresentable {
                source: self.bank.name,
                value: u16::from(bank),
            }),
            _ => Err(DecodeError::Unrepresentable {
                source: self.pointer.name,
                value: pointer,
            }),
        }
    }
}

/// The bytes between sprite palettes in the cycle's palette word.
const SPRITE_PALETTE_BYTES: u16 = 0x20;
/// Sprite palettes, and the glow's colour sets.
const EIGHT: u8 = 8;

impl AtmosphericSources {
    /// The slots in `wram`.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Unrepresentable`] for a type byte nothing spawns (2, past 7) or a
    /// frame in a free slot, which the handler would dispatch through its null entry
    /// (bank $90 log, `$90:8AB5`).
    fn read(self, wram: &[u8]) -> Result<Atmospherics, DecodeError> {
        let mut atmospherics = Atmospherics::default();
        for (index, slot) in (0_u16..).zip(&mut atmospherics.slots) {
            let field = |array: WramArray| array.word(index).and_then(|word| word.read(wram));
            let word = field(self.frame_and_type)?;
            if word == 0 {
                continue;
            }
            let [frame, kind] = word.to_le_bytes();
            let kind = match kind {
                1 => AtmosphericKind::FootstepSplash,
                3 => AtmosphericKind::DivingSplash,
                4 => AtmosphericKind::LavaSurfaceDamage,
                5 => AtmosphericKind::Bubbles,
                6 => AtmosphericKind::LandingDust,
                7 => AtmosphericKind::SpeedBoostDust,
                _ => {
                    return Err(DecodeError::Unrepresentable {
                        source: self.frame_and_type.name,
                        value: word,
                    });
                }
            };
            *slot = Some(AtmosphericGraphic {
                kind,
                frame: AtmosphericFrame(frame),
                timer: AtmosphericTimer(field(self.timer)?),
                x: field(self.x)?,
                y: field(self.y)?,
            });
        }
        Ok(atmospherics)
    }
}

impl PaletteCycleSources {
    /// The cycle in `wram`.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Unrepresentable`] for a palette offset that isn't a sprite palette's
    /// first colour, or a colour set past 7 (a magdollite's cycle leaves one, bank $A8 log,
    /// `$A8:E86E`).
    fn read(self, wram: &[u8]) -> Result<EnemyPaletteCycle, DecodeError> {
        let offset = self.palette.read(wram)?;
        let palette = u8::try_from(offset / SPRITE_PALETTE_BYTES)
            .ok()
            .filter(|&palette| offset % SPRITE_PALETTE_BYTES == 0 && palette < EIGHT)
            .ok_or(DecodeError::Unrepresentable {
                source: self.palette.name,
                value: offset,
            })?;
        let set = self.set.read(wram)?;
        let set_index = u8::try_from(set)
            .ok()
            .filter(|&index| index < EIGHT)
            .ok_or(DecodeError::Unrepresentable {
                source: self.set.name,
                value: set,
            })?;
        Ok(EnemyPaletteCycle {
            palette: SpritePalette(palette),
            set: GlowSet(set_index),
            timer: Frames(self.timer.read(wram)?),
        })
    }
}

impl ElevatorSources {
    /// The ride in `wram`.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Unrepresentable`] for a status past 3 (`$A3:952A` jumps through a
    /// table of four), a flags word other than 0, 1 or $80, or a direction other than 0 or
    /// $8000.
    pub fn read(self, wram: &[u8]) -> Result<ElevatorRide, DecodeError> {
        let unrepresentable = |source: WramWord, value| DecodeError::Unrepresentable {
            source: source.name,
            value,
        };
        let status = match self.status.read(wram)? {
            0 => ElevatorStatus::Inactive,
            1 => ElevatorStatus::LeavingRoom,
            2 => ElevatorStatus::DoorTransition,
            3 => ElevatorStatus::EnteringRoom,
            value => return Err(unrepresentable(self.status, value)),
        };
        let armed = match self.armed.read(wram)? {
            0 => ElevatorArming::Disarmed,
            IN_DOORWAY => ElevatorArming::InDoorway,
            ELEVATOR_DOOR => ElevatorArming::ElevatorDoor,
            value => return Err(unrepresentable(self.armed, value)),
        };
        let direction = match self.direction.read(wram)? {
            0 => ElevatorDirection::Down,
            UPWARDS => ElevatorDirection::Up,
            value => return Err(unrepresentable(self.direction, value)),
        };
        Ok(ElevatorRide {
            status,
            armed,
            direction,
        })
    }
}

impl PoseSources {
    /// The record in `wram`.
    ///
    /// # Errors
    ///
    /// [`DecodeError::Unrepresentable`] for a pose past the definitions table (the
    /// original would read another table as its definition) or a movement type past the
    /// movement handler's table.
    fn read(self, wram: &[u8]) -> Result<PoseRecord, DecodeError> {
        let pose = self.pose.read(wram)?;
        if usize::from(pose) >= POSE_COUNT {
            return Err(DecodeError::Unrepresentable {
                source: self.pose.name,
                value: pose,
            });
        }
        let movement = self.movement.read(wram)?;
        Ok(PoseRecord {
            pose: PoseId(pose),
            facing: Facing::from(self.facing.read(wram)?),
            movement: MovementType::try_from(movement).map_err(|value| {
                DecodeError::Unrepresentable {
                    source: self.movement.name,
                    value: u16::from(value),
                }
            })?,
        })
    }
}
