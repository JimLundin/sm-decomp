//! The one decoder from an oracle record to the engine's State (D21).
//!
//! The decoder reads what a record shows directly: the NMI counters and timing, the in-game
//! time, the RNG word, the controller, the game state, the NMI request, Samus's position and
//! equipment, video memory, the room and its camera.
//! The rest of the State stays at its default; [`crate::coverage::FIELDS`] says which.

use std::fmt;

use data::id::{Line, RoomId};
use data::pose::{MovementType, POSE_COUNT};
use game::state::room::{Camera, RoomScoped};
use game::state::samus::{
    Equipment, Facing, Fixed, Gauge, Item, Items, Movement, Point, PoseId, PoseRecord, Samus,
};
use game::state::slots::{FlagSet, Frames};
use game::state::strand::{IrqStrand, MainStrand, Message};
use game::state::video::{
    BACKGROUNDS, BgMode, Brightness, CGRAM_COLOURS, Colour, ColourMath, ColourMathLayers,
    ColourMathSelect, ColourWrite, DisplaySettings, DmaChannel, FixedColour, Layers, Mode7,
    Mode7Select, Mosaic, NmiBatch, OAM_HIGH_BYTES, Oam, OamEntry, ObjectSelect, Registers, Scroll,
    TileBases, TilemapBase, VRAM_WORDS, Video, VideoMemory, VramSource, VramStep, VramTransfer,
    VramWord, WindowSettings, Windows,
};
use game::state::{
    Button, Buttons, Clock, Controller, Counters, FrameCount, GameMode, GameTime, NmiTiming, Rng,
    State, Timing,
};
use oracle_record::{Cut, FrameRecord, NmiKind, PpuRegisters};

use crate::source::{self, WramBlock, WramByte, WramWord};
use crate::strand::{self, CALL_SITES, StrandRules, WAIT_SITES};

/// Bank $82 log: game state 8 is main gameplay.
const MAIN_GAMEPLAY: u16 = 8;

/// OAM's low table in bytes: four per sprite.
const OAM_LOW_BYTES: usize = 0x200;

/// A pointer to a room header in bank $8F, as `ROOM_PTR` holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RoomHeader(pub u16);

/// The rooms the decoder knows: each room header with the content's ID for it, from
/// `data::room::RoomMap` (see `lockstep::slice_zero_rooms`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoomMap {
    /// The known rooms.
    pub rooms: Vec<(RoomHeader, RoomId)>,
}

impl RoomMap {
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

/// A video memory the decoder reads whole.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Memory {
    /// VRAM, in words.
    Vram,
    /// CGRAM, in colours.
    Cgram,
    /// OAM, in bytes.
    Oam,
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
    /// `samus.position.x.pixels`.
    pub samus_x: WramWord,
    /// `samus.position.x.subpixels`.
    pub samus_x_subpixels: WramWord,
    /// `samus.position.y.pixels`.
    pub samus_y: WramWord,
    /// `samus.position.y.subpixels`.
    pub samus_y_subpixels: WramWord,
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
    /// `room.camera.x.pixels`.
    pub camera_x: WramWord,
    /// `room.camera.x.subpixels`.
    pub camera_x_subpixels: WramWord,
    /// `room.camera.y.pixels`.
    pub camera_y: WramWord,
    /// `room.camera.y.subpixels`.
    pub camera_y_subpixels: WramWord,
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
        samus_x: source::SAMUS_X_POS,
        samus_x_subpixels: source::SAMUS_X_SUBPOS,
        samus_y: source::SAMUS_Y_POS,
        samus_y_subpixels: source::SAMUS_Y_SUBPOS,
        room: source::ROOM_PTR,
        nmi_requested: source::WAITING_FOR_NMI,
        oam_buffer: source::OAM,
        palette_buffer: source::PALETTE_BUFFER,
        vram_queue: source::VRAM_WRITE_QUEUE,
        vram_queue_tail: source::VRAM_WRITE_QUEUE_TAIL,
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
        collected: source::COLLECTED_ITEMS,
        equipped: source::EQUIPPED_ITEMS,
        energy: source::SAMUS_HEALTH,
        max_energy: source::SAMUS_MAX_HEALTH,
        reserve: source::SAMUS_RESERVE_HEALTH,
        max_reserve: source::SAMUS_MAX_RESERVE_HEALTH,
        missiles: source::SAMUS_MISSILES,
        max_missiles: source::SAMUS_MAX_MISSILES,
        camera_x: source::LAYER1_X_POS,
        camera_x_subpixels: source::LAYER1_X_SUBPOS,
        camera_y: source::LAYER1_Y_POS,
        camera_y_subpixels: source::LAYER1_Y_SUBPOS,
        strand: StrandRules {
            sites: WAIT_SITES,
            calls: CALL_SITES,
            plm_index: source::PLM_ID,
            radius: source::MESSAGE_BOX_ANIMATION_Y_RADIUS,
        },
        irq_command: source::CUR_IRQ_HANDLER,
        irq_next: source::IRQHANDLER_NEXT_HANDLER,
        message: source::MESSAGE_BOX_INDEX,
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
            video: self.video(record)?,
            samus: self.samus(wram)?,
            room: RoomScoped {
                room,
                camera: Camera {
                    x: Fixed {
                        pixels: self.camera_x.read(wram)?,
                        subpixels: self.camera_x_subpixels.read(wram)?,
                    },
                    y: Fixed {
                        pixels: self.camera_y.read(wram)?,
                        subpixels: self.camera_y_subpixels.read(wram)?,
                    },
                },
                ..RoomScoped::default()
            },
            message: Message(self.message.read(wram)?),
        };
        Ok(Decoded { state, quiescence })
    }

    /// Samus: her position, pose and equipment.
    fn samus(&self, wram: &[u8]) -> Result<Samus, DecodeError> {
        let pose = self.pose.read(wram)?;
        Ok(Samus {
            position: Point {
                x: Fixed {
                    pixels: self.samus_x.read(wram)?,
                    subpixels: self.samus_x_subpixels.read(wram)?,
                },
                y: Fixed {
                    pixels: self.samus_y.read(wram)?,
                    subpixels: self.samus_y_subpixels.read(wram)?,
                },
            },
            equipment: Equipment {
                collected: items(self.collected, wram)?,
                equipped: items(self.equipped, wram)?,
                energy: gauge(self.energy, self.max_energy, wram)?,
                reserve: gauge(self.reserve, self.max_reserve, wram)?,
                missiles: gauge(self.missiles, self.max_missiles, wram)?,
            },
            pose: pose.pose,
            previous: self.previous_pose.read(wram)?,
            last_different: self.last_different_pose.read(wram)?,
            facing: pose.facing,
            movement: Movement::from(pose.movement),
            ..Samus::default()
        })
    }

    /// Video: the PPU's memories and registers, the NMI request and the batch.
    fn video(&self, record: &FrameRecord) -> Result<Video, DecodeError> {
        let wram = &*record.wram;
        Ok(Video {
            memory: video_memory(record)?,
            committed: committed(&record.ppu)?,
            dma: record.ppu.dma.map(dma_channel),
            nmi_requested: flag(self.nmi_requested, wram)?,
            batch: NmiBatch {
                registers: self.mirrors(wram)?,
                fixed_colour: bytes(self.mirrors.fixed_colour, wram)?.map(ColourWrite),
                oam: oam(self.oam_buffer.read(wram)?)?,
                cgram: colours(self.palette_buffer.read(wram)?),
                vram: self.vram_queue(wram)?,
                ..NmiBatch::default()
            },
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
fn buttons(word: u16) -> Buttons {
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
        _ => return VramSource::Content(u16::try_from(bytes / 2).unwrap_or(u16::MAX)),
    };
    let words = (0..bytes / 2)
        .map(|index| {
            let at = start.wrapping_add(index.wrapping_mul(2));
            let byte = |offset: usize| wram.get(at.wrapping_add(offset)).copied().unwrap_or(0);
            u16::from_le_bytes([byte(0), byte(1)])
        })
        .collect();
    VramSource::Words(words)
}

/// The bytes at `sources`.
fn bytes<const N: usize>(sources: [WramByte; N], wram: &[u8]) -> Result<[u8; N], DecodeError> {
    let mut values = [0; N];
    for (value, source) in values.iter_mut().zip(sources) {
        *value = source.read(wram)?;
    }
    Ok(values)
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
        control,
        destination,
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
