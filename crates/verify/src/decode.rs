//! The one decoder from an oracle record to the engine's State (D21).
//!
//! The decoder reads what a record shows directly: the NMI counters and timing, the in-game
//! time, the RNG word, the controller, the game state, the NMI request, Samus's position and
//! equipment, video memory, the room and its camera.
//! The rest of the State stays at its default; [`crate::coverage::FIELDS`] says which.

use std::fmt;

use data::id::RoomId;
use game::state::room::{Camera, RoomScoped};
use game::state::samus::{Equipment, Fixed, Gauge, Item, Items, Point, Samus};
use game::state::slots::{FlagSet, Frames};
use game::state::video::{
    CGRAM_COLOURS, Colour, OAM_HIGH_BYTES, Oam, OamEntry, VRAM_WORDS, Video, VideoMemory,
};
use game::state::{
    Button, Buttons, Clock, Controller, Counters, FrameCount, GameMode, GameTime, NmiTiming, Rng,
    State, Timing,
};
use oracle_record::{Cut, FrameRecord, Nmi, NmiKind};

use crate::source::{self, WramByte, WramWord};

/// Bank $82 log: game state 8 is main gameplay.
const MAIN_GAMEPLAY: u16 = 8;

/// OAM's low table in bytes: four per sprite.
const OAM_LOW_BYTES: usize = 0x200;

/// A pointer to a room header in bank $8F, as `ROOM_PTR` holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RoomHeader(pub u16);

/// The rooms the decoder knows: each room header with the content's ID for it.
// open: data's address map replaces this once the ROM decoder lists the rooms.
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
            Self::Unrepresentable { source, value } => {
                write!(
                    f,
                    "{source} holds {value:#06X}, which the State has no value for"
                )
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Where each decoded field comes from: one named source per rule.
///
/// [`Rules::ORIGINAL`] is the original's layout; a mutant is a copy with one source moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
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
}

impl Rules {
    /// The original's layout.
    pub const ORIGINAL: Self = Self {
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
    };

    /// The State `record` shows, under these rules.
    ///
    /// # Errors
    ///
    /// A [`DecodeError`] when the record is short, names an unknown room or is in a state
    /// not decoded yet.
    pub fn decode(&self, record: &FrameRecord, rooms: &RoomMap) -> Result<State, DecodeError> {
        let wram = &*record.wram;
        let game_state = self.game_state.read(wram)?;
        if game_state != MAIN_GAMEPLAY {
            return Err(DecodeError::Unsupported(Unsupported::GameState(game_state)));
        }
        let header = RoomHeader(self.room.read(wram)?);
        let room = rooms.get(header).ok_or(DecodeError::UnknownRoom(header))?;
        let last = nmi_timing(record.cut)?;

        Ok(State {
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
            video: Video {
                memory: video_memory(record)?,
                nmi_requested: flag(self.nmi_requested, wram)?,
                ..Video::default()
            },
            samus: Samus {
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
                ..Samus::default()
            },
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
            ..State::default()
        })
    }
}

/// The State `record` shows, under the original's layout.
///
/// # Errors
///
/// As [`Rules::decode`].
pub fn decode(record: &FrameRecord, rooms: &RoomMap) -> Result<State, DecodeError> {
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

/// How the cut's NMI found the main thread: waiting at a wait site, or still working.
const fn nmi_timing(cut: Cut) -> Result<NmiTiming, DecodeError> {
    match cut {
        Cut::NmiReturn(Nmi {
            kind: NmiKind::Requested,
            ..
        }) => Ok(NmiTiming::OnTime),
        Cut::NmiReturn(Nmi {
            kind: NmiKind::Overrun,
            ..
        }) => Ok(NmiTiming::Overrun),
        Cut::VblankWithoutNmi | Cut::NmiStillRunning { .. } => {
            Err(DecodeError::Unsupported(Unsupported::NoNmi(cut)))
        }
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
    let oam_size = DecodeError::WrongSize {
        memory: Memory::Oam,
        expected: OAM_LOW_BYTES.wrapping_add(OAM_HIGH_BYTES),
        found: record.oam.len(),
    };
    let (low, high) = record
        .oam
        .split_first_chunk::<OAM_LOW_BYTES>()
        .ok_or(oam_size)?;
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
    Ok(VideoMemory {
        vram: record.vram.to_vec(),
        cgram: cgram.map(Colour),
        oam,
    })
}
