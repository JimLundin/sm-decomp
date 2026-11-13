//! The door passage (world-doors, part 1) as a harness domain: the model against the oracle.
//!
//! The model (`world_doors`) is checked frame by frame by the decode notes' harness note
//! (`docs/behaviours/world-doors.decode.md`, "The harness"). It runs on every frame from a
//! record in a passage (game state 9, Ah or Bh) to the next; a frame from a record of play is
//! skipped: play is the other domains', and her touch of a door is samus-movement's.
//!
//! The State is the passage decoded from record N: the phase from the game state and the door
//! transition function, `t` from the palette change numerator, the arrival once taken and the
//! slide's frames; the scene the passage moves (her position, where the camera last saw her,
//! the camera, the colours and the colours they head for); and what the harness carries beside
//! the model to compare it: her pose and speeds as the passage found them, the frame counter and
//! the HUD's frame count, and whether sounds are held. The inputs are record N's: the sound
//! queues' emptiness, the music's readiness from the music queue (never from the phase), the
//! timer, the CRE bits of both rooms, and the doorway's arrival, which the harness reads from the
//! door record her touch named (`078D`) and holds once taken, checking the record holds the same
//! (the World's doorway records don't hold the arrival yet, nor the rooms outside Ceres). The
//! work of a frame of work is done when the frame's cut is `Requested` (the decode notes'
//! contract with T1, until T1 exists), and the contents' fetch at once: the records bound its
//! cost only by the slide's end, which its wait holds.
//!
//! Before the slide and as the colours come up only the frame of work's clock runs, so a record
//! inside a frame of work there (a cut not `R`) is folded into it: it holds the work part-done,
//! the fade's colours part-faded. From the slide's first step until she steps clear every record
//! ends a frame of its own: the slide and the finished frames run on each.
//!
//! Compared every frame, an overrun's too (it ends no frame of work, so the model holds what one
//! would move): her position, where the camera last saw her, the camera, her pose and speeds
//! held, the frame counter (a finished frame steps it), the HUD's frame count, the phase and its
//! data, the colours, the silence queued on library 2, sounds held, her sound on library 1 as
//! the decode notes' G6 picks it for each request (samus-appearance's choice, until its spec owns
//! it; no request leaves library 1 as it was), and that no enemy's frame counter steps on a frame
//! the enemies don't run. The mini-map's turn is compared on the first frame of work only. The
//! enemies' run on a frame they do run is C1's (only those in view run), so it is masked on
//! those frames.
//!
//! Inputs another writer makes: the new room's target colours during `Brightening` (record N's,
//! the room's loading's), and the colours the loading (`$A0:8A1E`) writes to sprite palettes 6
//! and 7 on any frame in `Arriving`, not only the one it ends on, taken from record N+1
//! through the stand-in, so a frame on which they change the colours is unverified. Masked under
//! named writers: BG palette 7's and sprite palette 7's colours 1 to 14 in Kraid's room while the
//! enemies run (Kraid's corpse), and BG palette 2's colours 9 to 14 in the Ceres elevator room on
//! the `Brightening` frame of work at the fade's step 0 (the elevator beacon, `$A6:F850`). Not
//! compared: the touch (`TheTouch`: samus-movement's, on a frame of play, which only record N+1
//! shows), the cost event, the room's calls, the animated tiles' and the projectiles' runs, what
//! is drawn and the slide's screen (no lean record holds them).
//!
//! The null model is the passage left as record N has it.

use core::fmt;

use data::rom::{AddressError, Lorom, RomAddress};
use oracle_record::FrameRecord;
use types::{Colour, Fixed, Heading, Pair, Position, Px, fixed};
use world_doors::{
    Activity, Arrival, BehaviourId, COLOURS, ColourGroup, Doors, Exports, FadeTime, Fetching,
    Finishing, HerSound, Inputs, LAST_BEFORE_THE_SLIDE, LAST_LANDING, Palette, Passage, Phase,
    Report, RoomCall, RoomSound, Rules, Scene, WorkFrames, catalogue,
};

use super::{Domain, Entry, Field, FieldSet, Hides, Interval, Mask, Uncompared};
use crate::cache::CutKind;
use crate::decode::DecodeError;
use crate::domain_field;
use crate::source::{self, WramByte, WramWord};

/// A work RAM word the harness reads that `source` doesn't name.
const fn wram(name: &'static str, offset: u16) -> WramWord {
    WramWord { name, offset }
}

/// A work RAM byte the harness reads that `source` doesn't name.
const fn wram_byte(name: &'static str, offset: u16) -> WramByte {
    WramByte { name, offset }
}

/// Bank $82 log: the door transition function, and the game states of the passage.
const DOOR_FUNCTION: WramWord = wram("DOOR_TRANSITION_FUNCTION", 0x099C);
/// The touch (game state 9) and the first frame of work's routine (Ah), and the rest (Bh).
const TOUCHED: u16 = 0x09;
const LEAVING: u16 = 0x0A;
const PASSING: u16 = 0x0B;
/// Bank $82 log: the door her touch named, the door's direction, destination and speed, the
/// slide's frame counter and its finished flag.
const DOOR_POINTER: WramWord = wram("DOOR_POINTER", 0x078D);
const DOOR_DIRECTION: WramWord = wram("DOOR_DIRECTION", 0x0791);
const DOOR_DESTINATION_X: WramWord = wram("DOOR_DESTINATION_X", 0x0927);
const DOOR_DESTINATION_Y: WramWord = wram("DOOR_DESTINATION_Y", 0x0929);
const DOOR_SPEED_SUB: WramWord = wram("DOOR_SPEED_SUB", 0x092B);
const DOOR_SPEED: WramWord = wram("DOOR_SPEED", 0x092D);
const SLIDE_FRAMES: WramWord = wram("DOOR_TRANSITION_FRAME_COUNTER", 0x0925);
const SLIDE_FINISHED: WramWord = wram("DOOR_TRANSITION_FINISHED", 0x0931);
/// `0931` bit 15: the slide's NMI wait passed.
const SLIDE_DONE: u16 = 0x8000;
/// Bank $82 log: the palette change numerator, `t`.
const FADE_NUMERATOR: WramWord = wram("PALETTE_CHANGE_NUMERATOR", 0xC400);
/// The colours and the colours they head for.
const SCREEN_COLOURS: u16 = 0xC000;
const TARGET_COLOURS: u16 = 0xC200;
/// Her position's subpixels and the camera's.
const CAMERA_X_SUB: WramWord = wram("LAYER1_X_SUBPOS", 0x090F);
const CAMERA_Y_SUB: WramWord = wram("LAYER1_Y_SUBPOS", 0x0913);
/// Her motion as the touch left it: her pose, speeds and momentum.
const MOTION: [WramWord; 7] = [
    wram("POSE", 0x0A1C),
    wram("SAMUS_X_EXTRA_RUN_SPEED", 0x0B42),
    wram("SAMUS_X_EXTRA_RUN_SUBSPEED", 0x0B44),
    wram("SAMUS_X_BASE_SPEED", 0x0B46),
    wram("SAMUS_X_BASE_SUBSPEED", 0x0B48),
    wram("SAMUS_Y_SUBSPEED", 0x0B2C),
    wram("SAMUS_Y_SPEED", 0x0B2E),
];
/// Bank $80 log: sounds disabled, `FFFFh` while held.
const SOUNDS_DISABLED: WramWord = wram("DISABLE_SOUNDS", 0x05F5);
const HELD: u16 = 0xFFFF;
/// The mini-map's disable flag, 0 when on.
const MINIMAP_DISABLED: WramWord = source::DEBUG_DISABLE_MINIMAP;
/// Bank $80 log: the three sound libraries' queues: start and next indices, and the entries.
const QUEUE_START: u16 = 0x0643;
const QUEUE_NEXT: u16 = 0x0646;
const QUEUE_ENTRIES: u16 = 0x0656;
const QUEUE_LENGTH: u16 = 0x10;
/// Library 2, the silence's, by its place among the three.
const LIBRARY_2: u16 = 1;
/// The silence: sound 71h of library 2 (`$82:E26F`).
const SILENCE: u8 = 0x71;
/// Bank $80 log: the music queue's timers, its start index and the music timer.
const MUSIC_TIMERS: u16 = 0x0629;
const MUSIC_ENTRIES: u16 = 8;
const MUSIC_START: WramWord = wram("MUSIC_QUEUE_START", 0x063B);
const MUSIC_TIMER: WramWord = wram("MUSIC_TIMER", 0x063F);
/// The escape's timer status, not 0 while it runs.
const TIMER_STATUS: WramWord = wram("TIMER_STATUS", 0x0943);
/// The CRE bitsets of the previous room and the room (`07B3` the room's).
const PREVIOUS_CRE: WramWord = wram("PREVIOUS_CRE_BITSET", 0x07B1);
const CRE: WramWord = wram("CRE_BITSET", 0x07B3);
/// Bit 0 keeps the items dark.
const ITEMS_DARK: u16 = 1;
/// The enemy slots: the species and the frame counter, a slot 40h bytes on.
const ENEMY_SPECIES: u16 = 0x0F78;
const ENEMY_COUNTER: u16 = 0x0FA4;
const ENEMY_STRIDE: u16 = 0x40;
const ENEMY_SLOTS: usize = 32;
/// The rooms whose own writers move the colours during a passage: Kraid's room (his corpse
/// writes BG palette 7 as the enemies run) and Ceres's elevator room (its beacon,
/// `$A6:F850`, writes BG palette 2 while the numerator is 0).
const KRAID_ROOM: u16 = 0xA59F;
const CERES_ELEVATOR_ROOM: u16 = 0xDF45;
/// BG palette 7's and sprite palette 7's colours 1 to 14, and BG palette 2's colours 9 to 14, by
/// index.
const KRAID_COLOURS: core::ops::RangeInclusive<usize> = 113..=126;
const KRAID_SPRITE_COLOURS: core::ops::RangeInclusive<usize> = 241..=254;
const BEACON_COLOURS: core::ops::RangeInclusive<usize> = 41..=46;
/// Sprite palettes 6 and 7, which the loading writes (`$A0:8A1E`), by index.
const LOADED_COLOURS: core::ops::Range<usize> = 224..256;
/// The door records' bank and the room headers'; a door record's room pointer, direction,
/// destination screens and speed; a room header's CRE bitset.
const DOOR_BANK: u8 = 0x83;
const ROOM_BANK: u8 = 0x8F;
const DOOR_RECORD: usize = 10;
const ROOM_CRE: usize = 8;
/// A door record's speed word with bit 15 set gives none: the passage's own.
const NO_SPEED: u16 = 0x8000;
/// How much lower a way up's arrival is taken from `SettingOff` on (`$80:ADEB`).
const UP_LOWERED: Px = core::num::Wrapping(0x20);
/// One screen, in px.
const SCREEN: u16 = 0x100;

/// The HUD's colour groups (`TheFades`), by palette index, as the decode notes list them.
const LIT: [usize; 8] = [9, 10, 13, 14, 17, 18, 19, 29];
const ICONS: [usize; 5] = [20, 21, 22, 23, 28];
const TIMER: [usize; 4] = [209, 210, 212, 221];
const VISOR: usize = 196;

/// The colour group of each of the screen's colours.
fn groups() -> [ColourGroup; COLOURS] {
    let mut groups = [ColourGroup::Other; COLOURS];
    for (members, group) in [
        (&LIT[..], ColourGroup::Lit),
        (&ICONS[..], ColourGroup::Icon),
        (&TIMER[..], ColourGroup::Timer),
        (&[VISOR][..], ColourGroup::Visor),
    ] {
        for &index in members {
            if let Some(slot) = groups.get_mut(index) {
                *slot = group;
            }
        }
    }
    groups
}

/// The declared fields by their places in `FIELDS`.
const HER: FieldSet = FieldSet::single(0);
const SEEN: FieldSet = FieldSet::single(1);
const CAMERA: FieldSet = FieldSet::single(2);
const MOTION_FIELD: FieldSet = FieldSet::single(3);
const FINISHED: FieldSet = FieldSet::single(4);
const HUD_COUNT: FieldSet = FieldSet::single(5);
const PHASE: FieldSet = FieldSet::single(6);
const SCREEN_FIELD: FieldSet = FieldSet::single(7);
const BEACON_FIELD: FieldSet = FieldSet::single(8);
const KRAID_FIELD: FieldSet = FieldSet::single(9);
const SILENCE_FIELD: FieldSet = FieldSet::single(10);
const HELD_FIELD: FieldSet = FieldSet::single(11);
const MINIMAP: FieldSet = FieldSet::single(12);
const ENEMIES: FieldSet = FieldSet::single(13);
const HER_SOUND: FieldSet = FieldSet::single(14);
/// Every colour field.
const ALL_COLOURS: FieldSet = SCREEN_FIELD.union(BEACON_FIELD).union(KRAID_FIELD);

/// The declared fields a catalogue entry's output reaches.
const fn shown_by(id: BehaviourId) -> FieldSet {
    use BehaviourId as B;
    match id {
        B::TheTouch | B::PassageTime | B::Building | B::Landing => PHASE,
        B::FinishedFrames => FINISHED.union(HUD_COUNT),
        B::Quieting => PHASE
            .union(SILENCE_FIELD)
            .union(HELD_FIELD)
            .union(MINIMAP)
            .union(HER_SOUND),
        B::TheFades => ALL_COLOURS.union(HELD_FIELD).union(HER_SOUND),
        B::Fading => ALL_COLOURS,
        B::Squaring | B::SettingOff => PHASE.union(CAMERA),
        B::Sliding => CAMERA.union(SEEN),
        B::Carrying => HER.union(SEEN),
        B::Arriving => PHASE.union(ALL_COLOURS).union(HER).union(SEEN),
        B::WhereSheLands => HER,
        B::SheKeepsHerMotion => MOTION_FIELD,
        B::WhatRuns => ENEMIES,
    }
}

/// Why a frame isn't compared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Skip {
    /// Record N is play: no passage runs.
    Play,
    /// A record's words couldn't be read.
    Unreadable(DecodeError),
    /// The door transition function is none of the passage's.
    UnknownFunction(u16),
    /// A colour word has bit 15 set.
    NotAColour(usize),
    /// The door record her touch named is outside the ROM.
    Door(u16),
    /// The arrival the record holds isn't the doorway's.
    ArrivalDiffers,
}

impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Play => write!(f, "play: no passage"),
            Self::Unreadable(error) => write!(f, "unreadable: {error:?}"),
            Self::UnknownFunction(function) => {
                write!(f, "door transition function {function:04X} is no phase's")
            }
            Self::NotAColour(index) => write!(f, "colour {index} has bit 15 set"),
            Self::Door(pointer) => write!(f, "door record {pointer:04X} is outside the ROM"),
            Self::ArrivalDiffers => write!(f, "the arrival held isn't the doorway's"),
        }
    }
}

impl From<DecodeError> for Skip {
    fn from(error: DecodeError) -> Self {
        Self::Unreadable(error)
    }
}

/// The frame counter and the HUD's frame count (`05B6`, `05B5`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counts {
    /// The frame counter: a finished frame steps it.
    pub frames: u16,
    /// The HUD's frame count.
    pub hud: u8,
}

impl Counts {
    /// The counts after a frame that finishes as `finishing`.
    const fn after(self, finishing: Finishing) -> Self {
        match finishing {
            Finishing::Overrun => self,
            Finishing::Finished => Self {
                frames: self.frames.wrapping_add(1),
                hud: self.hud.wrapping_add(1),
            },
            Finishing::Restarts(_) => Self {
                frames: self.frames.wrapping_add(1),
                hud: 1,
            },
        }
    }
}

/// What the frame found that the harness compares the frame's effects against: library 2's
/// next index, and each enemy slot's species and frame counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Found {
    /// Library 1's next index.
    pub library_1: u8,
    /// The library-1 sound each of the passage's requests to her sounds plays, by G6's rule.
    pub picks: Picks,
    /// Library 2's next index.
    pub library_2: u8,
    /// Each enemy slot's species and frame counter.
    pub enemies: [(u16, u16); ENEMY_SLOTS],
}

/// The passage and what the harness carries beside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Passing {
    /// The doors' state.
    pub doors: Doors,
    /// What the passage moves.
    pub scene: Scene,
    /// Her pose and speeds, which nothing in the passage writes.
    pub motion: [u16; 7],
    /// The frame counter and the HUD's frame count.
    pub counts: Counts,
    /// Whether sounds are held.
    pub sounds_held: bool,
    /// The last frame's report; play's before any.
    pub report: Report,
    /// What the frame found.
    pub found: Found,
}

/// What a frame brings: the model's inputs, the new room's target colours (record N's) and what
/// the frame found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Read {
    /// The model's inputs.
    pub inputs: Inputs,
    /// The colours headed for, as record N holds them: the new room's during `Brightening`.
    pub goal: [Colour; COLOURS],
    /// What the frame found.
    pub found: Found,
    /// The room's header pointer at record N.
    pub room: u16,
    /// The frame's cut is `R`: a frame of work ends on it.
    pub requested: bool,
}

/// The colours the loading wrote to sprite palettes 6 and 7 by record N+1, the stand-in's answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Loaded {
    /// The colours, or none for the stand-in that copies nothing.
    pub colours: Option<[Colour; 32]>,
}

/// The phase as the records show it: `Arriving`'s contents fetched and being fetched are one,
/// and whether she is placed is the model's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    /// Play.
    Play,
    /// A passage: its phase, and the arrival with the speed it moves her at.
    Passing(Phase, Option<(Heading, Pair<Px>, Fixed)>),
}

/// The silence on library 2, as the model asks for it or a record holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Silence {
    /// The model: library 2's next index as the frame found it, and whether the frame asks for
    /// the silence.
    Asked {
        /// Library 2's next index at the frame's start.
        from: u8,
        /// Whether the silence is asked for.
        asked: bool,
    },
    /// A record: library 2's next index, and whether its last entry is the silence.
    Queued {
        /// Library 2's next index.
        next: u8,
        /// Whether the last entry is the silence.
        last: bool,
    },
}

impl Silence {
    /// Whether two agree: the silence is asked for exactly when the record's library 2 moved on
    /// from the frame's start with the silence last.
    fn agrees(self, other: Self) -> bool {
        match (self, other) {
            (Self::Asked { from, asked }, Self::Queued { next, last })
            | (Self::Queued { next, last }, Self::Asked { from, asked }) => {
                asked == (last && next != from)
            }
            (Self::Asked { .. }, Self::Asked { .. })
            | (Self::Queued { .. }, Self::Queued { .. }) => self == other,
        }
    }
}

/// A library-1 sound, as the model's request picks it or a record holds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    /// The model: library 1's next index as the frame found it, and the sound its request to her
    /// sounds plays (none: no request, or one that plays nothing).
    Asked {
        /// Library 1's next index at the frame's start.
        from: u8,
        /// The sound queued.
        sound: Option<u8>,
    },
    /// A record: library 1's next index and its last entry.
    Queued {
        /// Library 1's next index.
        next: u8,
        /// Its last entry.
        last: u8,
    },
}

impl Sound {
    /// Whether two agree: a sound asked for is library 1's last entry, newly queued; none asked
    /// for leaves library 1 as it was.
    fn agrees(self, other: Self) -> bool {
        match (self, other) {
            (Self::Asked { from, sound }, Self::Queued { next, last })
            | (Self::Queued { next, last }, Self::Asked { from, sound }) => {
                sound.map_or(next == from, |sound| next != from && last == sound)
            }
            (Self::Asked { .. }, Self::Asked { .. })
            | (Self::Queued { .. }, Self::Queued { .. }) => self == other,
        }
    }
}

/// The enemies on a frame: whether they run (the model, none where not compared) and each slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Enemies {
    /// Whether the enemies run, by the model; none for the record.
    pub run: Option<bool>,
    /// Each slot's species and frame counter: the frame's start (the model) or end (the record).
    pub slots: [(u16, u16); ENEMY_SLOTS],
}

/// The declared fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fields {
    /// Her position.
    pub her: Position,
    /// Where the camera last saw her.
    pub seen: Pair<Px>,
    /// The camera.
    pub camera: Position,
    /// Her pose and speeds.
    pub motion: [u16; 7],
    /// The frame counter and the HUD's frame count.
    pub counts: Counts,
    /// The phase.
    pub phase: Shown,
    /// The colours.
    pub colours: [Colour; COLOURS],
    /// The silence.
    pub silence: Silence,
    /// Whether sounds are held.
    pub sounds_held: bool,
    /// Whether the mini-map is on: turned on this frame (the model), or on (the record).
    pub minimap_on: bool,
    /// The enemies.
    pub enemies: Enemies,
    /// Her sound on library 1.
    pub her_sound: Sound,
}

/// A hash of some colours, which a report shows in place of the colours, which are the ROM's.
fn hash(colours: impl Iterator<Item = Colour>) -> String {
    let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
    for colour in colours {
        for byte in colour.bits().to_le_bytes() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01B3);
        }
    }
    format!("colours #{hash:016x}")
}

/// Whether `index` is one of the colours compared on its own.
fn apart(index: usize) -> bool {
    KRAID_COLOURS.contains(&index)
        || KRAID_SPRITE_COLOURS.contains(&index)
        || BEACON_COLOURS.contains(&index)
}

/// The colours not compared on their own.
fn rest(fields: &Fields) -> impl Iterator<Item = Colour> + '_ {
    fields
        .colours
        .iter()
        .enumerate()
        .filter(|&(index, _)| !apart(index))
        .map(|(_, &colour)| colour)
}

/// BG palette 7's and sprite palette 7's colours 1 to 14.
fn kraid(fields: &Fields) -> impl Iterator<Item = Colour> + '_ {
    some(fields, KRAID_COLOURS).chain(some(fields, KRAID_SPRITE_COLOURS))
}

/// The colours in `range`.
fn some(fields: &Fields, range: core::ops::RangeInclusive<usize>) -> impl Iterator<Item = Colour> {
    fields
        .colours
        .get(range)
        .unwrap_or_default()
        .iter()
        .copied()
}

const SCREEN_COLOURS_FIELD: Field<Fields> = Field {
    name: "colours",
    same: |model, oracle| rest(model).eq(rest(oracle)),
    show: |fields| hash(rest(fields)),
};
const BEACON_COLOURS_FIELD: Field<Fields> = Field {
    name: "colours.bg2_9_14",
    same: |model, oracle| some(model, BEACON_COLOURS).eq(some(oracle, BEACON_COLOURS)),
    show: |fields| hash(some(fields, BEACON_COLOURS)),
};
const KRAID_COLOURS_FIELD: Field<Fields> = Field {
    name: "colours.bg7_sp7_1_14",
    same: |model, oracle| kraid(model).eq(kraid(oracle)),
    show: |fields| hash(kraid(fields)),
};
/// The silence: asked for exactly when library 2 moved on and its last entry is the silence.
const SILENCE_FIELD_DEF: Field<Fields> = Field {
    name: "silence",
    same: |model, oracle| model.silence.agrees(oracle.silence),
    show: |fields| format!("{:?}", fields.silence),
};
/// The enemies: on a frame of work they don't run, no enemy that stays in its slot steps.
const ENEMIES_FIELD: Field<Fields> = Field {
    name: "enemies",
    same: |model, oracle| match model.enemies.run {
        Some(false) => !model.enemies.slots.iter().zip(&oracle.enemies.slots).any(
            |(&(species, before), &(after_species, after))| {
                species != 0 && species == after_species && before != after
            },
        ),
        Some(true) | None => true,
    },
    show: |fields| format!("{:?}", fields.enemies.run),
};

/// The door passage's domain, with the ROM its doorway reader reads.
#[derive(Clone, Debug)]
pub struct DoorsDomain {
    image: Vec<u8>,
}

impl DoorsDomain {
    /// The domain over `image`.
    #[must_use]
    pub const fn new(image: Vec<u8>) -> Self {
        Self { image }
    }

    /// The door record at `pointer`: the doorway's arrival, and its room's CRE bits.
    fn doorway(&self, pointer: u16) -> Result<(Arrival, u16), AddressError> {
        let image = Lorom::new(&self.image);
        let door = image.read(RomAddress::new(DOOR_BANK, pointer), DOOR_RECORD)?;
        let byte = |at: usize| door.get(at).copied().unwrap_or_default();
        let header = u16::from_le_bytes([byte(0), byte(1)]);
        let speed = u16::from_le_bytes([byte(8), byte(9)]);
        let cre = image
            .read(RomAddress::new(ROOM_BANK, header), ROOM_CRE + 1)?
            .get(ROOM_CRE)
            .copied()
            .unwrap_or_default();
        let arrival = Arrival {
            way: way(u16::from(byte(3))),
            view: Pair {
                x: core::num::Wrapping(u16::from(byte(6)).wrapping_mul(SCREEN)),
                y: core::num::Wrapping(u16::from(byte(7)).wrapping_mul(SCREEN)),
            },
            speed: (speed & NO_SPEED == 0).then(|| speed_of(speed)),
        };
        Ok((arrival, u16::from(cre)))
    }
}

/// The way of a door's direction: its low two bits.
const fn way(direction: u16) -> Heading {
    match direction & 3 {
        0 => Heading::Right,
        1 => Heading::Left,
        2 => Heading::Down,
        _ => Heading::Up,
    }
}

/// A door record's speed word, in 1/256 px a frame, as `P.S`.
fn speed_of(word: u16) -> Fixed {
    let [sub, pixel] = word.to_le_bytes();
    fixed(core::num::Wrapping(u16::from(pixel)), u16::from(sub) << 8)
}

fn word(record: &FrameRecord, source: WramWord) -> Result<u16, DecodeError> {
    source.read(&record.wram)
}

fn at(record: &FrameRecord, name: &'static str, offset: u16) -> Result<u16, DecodeError> {
    wram(name, offset).read(&record.wram)
}

fn byte(record: &FrameRecord, name: &'static str, offset: u16) -> Result<u8, DecodeError> {
    wram_byte(name, offset).read(&record.wram)
}

/// 256 colours from `base`.
fn colours(record: &FrameRecord, base: u16) -> Result<[Colour; COLOURS], Skip> {
    let mut colours = [Colour::BLACK; COLOURS];
    for ((index, colour), offset) in colours.iter_mut().enumerate().zip((base..).step_by(2)) {
        *colour = Colour::new(at(record, "PALETTE", offset)?).ok_or(Skip::NotAColour(index))?;
    }
    Ok(colours)
}

/// The position of a pixel word and a subpixel word.
fn position(record: &FrameRecord, words: [WramWord; 4]) -> Result<Position, DecodeError> {
    let [x, x_sub, y, y_sub] = words;
    Ok(Pair {
        x: fixed(core::num::Wrapping(word(record, x)?), word(record, x_sub)?),
        y: fixed(core::num::Wrapping(word(record, y)?), word(record, y_sub)?),
    })
}

/// Her position, where the camera last saw her, and the camera.
fn scene_of(record: &FrameRecord) -> Result<Scene, Skip> {
    Ok(Scene {
        her: position(
            record,
            [
                source::SAMUS_X_POS,
                source::SAMUS_X_SUBPOS,
                source::SAMUS_Y_POS,
                source::SAMUS_Y_SUBPOS,
            ],
        )?,
        seen: Pair {
            x: core::num::Wrapping(word(record, source::SAMUS_PREV_X_POS)?),
            y: core::num::Wrapping(word(record, source::SAMUS_PREV_Y_POS)?),
        },
        camera: position(
            record,
            [
                source::LAYER1_X_POS,
                CAMERA_X_SUB,
                source::LAYER1_Y_POS,
                CAMERA_Y_SUB,
            ],
        )?,
        palette: Palette {
            screen: colours(record, SCREEN_COLOURS)?,
            goal: colours(record, TARGET_COLOURS)?,
        },
    })
}

/// The arrival the record holds once taken.
fn arrival_of(record: &FrameRecord) -> Result<Arrival, DecodeError> {
    Ok(Arrival {
        way: way(word(record, DOOR_DIRECTION)?),
        view: Pair {
            x: core::num::Wrapping(word(record, DOOR_DESTINATION_X)?),
            y: core::num::Wrapping(word(record, DOOR_DESTINATION_Y)?),
        },
        speed: Some(fixed(
            core::num::Wrapping(word(record, DOOR_SPEED)?),
            word(record, DOOR_SPEED_SUB)?,
        )),
    })
}

/// The slide frames counted by the end of `SettingOff`'s frame of work: its own, and for a way up
/// the one `BeforeTheSlide(0)` takes.
const fn set_off_by(way: Heading) -> u16 {
    match way {
        Heading::Up => 2,
        Heading::Down | Heading::Left | Heading::Right => 1,
    }
}

/// The speed the passage carries her at where the doorway gives none, as the record holds it
/// (`$82:DE12`): 0.C800 px a frame left or right, 1.8000 down or up. The harness's own, so a
/// model whose speed differs is caught where she moves, not in the decode.
const fn own_speed(way: Heading) -> Fixed {
    match way {
        Heading::Left | Heading::Right => core::num::Wrapping(0x0000_C800),
        Heading::Down | Heading::Up => core::num::Wrapping(0x0001_8000),
    }
}

/// The doors with the arrival they hold taken from the doorway's record, so the speed the
/// doorway leaves to the passage stays the passage's; the record's must be the same, a way up's
/// taken 32 px lower from `SettingOff` on.
fn held_doorway(doors: Doors, doorway: Arrival) -> Result<Doors, Skip> {
    let Doors::Passing(mut passage) = doors else {
        return Ok(doors);
    };
    let Some(held) = passage.arrival else {
        return Ok(doors);
    };
    let set_off = match passage.phase {
        Phase::Arriving { .. }
        | Phase::Landing(_)
        | Phase::Music { .. }
        | Phase::Brightening(_) => true,
        Phase::Quieting { .. }
        | Phase::Darkening(_)
        | Phase::Squaring
        | Phase::BeforeTheSlide(_) => false,
    };
    let mut arrival = doorway;
    if set_off && arrival.way == Heading::Up {
        arrival.view.y += UP_LOWERED;
    }
    let speed = |arrival: Arrival| arrival.speed.unwrap_or_else(|| own_speed(arrival.way));
    if (arrival.way, arrival.view, speed(arrival)) != (held.way, held.view, speed(held)) {
        return Err(Skip::ArrivalDiffers);
    }
    passage.arrival = Some(arrival);
    Ok(Doors::Passing(passage))
}

/// Whether she stands on the screen the camera shows, on both axes.
fn on_the_view(record: &FrameRecord) -> Result<bool, DecodeError> {
    let within = |her: WramWord, camera: WramWord| -> Result<bool, DecodeError> {
        Ok(word(record, her)?.wrapping_sub(word(record, camera)?) < SCREEN)
    };
    Ok(within(source::SAMUS_X_POS, source::LAYER1_X_POS)?
        && within(source::SAMUS_Y_POS, source::LAYER1_Y_POS)?)
}

/// The doors' state the record holds, by the decode notes' mapping.
fn doors_of(record: &FrameRecord) -> Result<Doors, Skip> {
    let state = word(record, source::GAME_STATE)?;
    if state == TOUCHED || state == LEAVING {
        return Ok(Doors::Passing(Passage::START));
    }
    if state != PASSING {
        return Err(Skip::Play);
    }
    let function = word(record, DOOR_FUNCTION)?;
    let slide_frames = word(record, SLIDE_FRAMES)?;
    let taken = || arrival_of(record).map(Some);
    let fade = || -> Result<FadeTime, DecodeError> { Ok(FadeTime(word(record, FADE_NUMERATOR)?)) };
    let first = |n: u8| -> WorkFrames<LAST_BEFORE_THE_SLIDE> {
        WorkFrames::new(n).unwrap_or(WorkFrames::FIRST)
    };
    let landing =
        |n: u8| -> WorkFrames<LAST_LANDING> { WorkFrames::new(n).unwrap_or(WorkFrames::FIRST) };
    let (phase, arrival) = match function {
        0xE29E => (Phase::Quieting { asked: true }, None),
        0xE2DB => (Phase::Darkening(fade()?), None),
        0xE2F7 => (Phase::Squaring, None),
        0xE310 => (Phase::Squaring, taken()?),
        0xE353 => (Phase::BeforeTheSlide(first(0)), taken()?),
        0xE36E => (Phase::BeforeTheSlide(first(1)), taken()?),
        0xE38E => (Phase::BeforeTheSlide(first(2)), taken()?),
        0xE3C0 => {
            let arrival = arrival_of(record)?;
            (
                Phase::Arriving {
                    // The slide counts on from the frame after the placing. On the placing's
                    // own record she is placed if the finished flag the last slide left set is
                    // clear (the placing clears it; a row's first passage has none) and she
                    // stands on the view's screen.
                    placed: slide_frames > set_off_by(arrival.way)
                        || (word(record, SLIDE_FINISHED)? & SLIDE_DONE == 0
                            && on_the_view(record)?),
                    fetching: Fetching::Tiles,
                },
                Some(arrival),
            )
        }
        0xE4A9 if word(record, SLIDE_FINISHED)? & SLIDE_DONE != 0 => {
            (Phase::Landing(landing(0)), taken()?)
        }
        0xE4A9 => (
            Phase::Arriving {
                placed: true,
                fetching: Fetching::Done,
            },
            taken()?,
        ),
        0xE659 => (Phase::Landing(landing(1)), taken()?),
        0xE664 => (Phase::Music { started: false }, taken()?),
        0xE6A2 => (Phase::Music { started: true }, taken()?),
        0xE737 => (Phase::Brightening(fade()?), taken()?),
        other => return Err(Skip::UnknownFunction(other)),
    };
    // The counter holds the last passage's slide until `SettingOff` starts this one's; the
    // model counts the still frame 0 a way up takes as it draws the top row.
    let slide_frames = match phase {
        Phase::Arriving { .. }
        | Phase::Landing(_)
        | Phase::Music { .. }
        | Phase::Brightening(_) => slide_frames,
        Phase::BeforeTheSlide(n)
            if n != WorkFrames::FIRST
                && arrival.is_some_and(|arrival| arrival.way == Heading::Up) =>
        {
            1
        }
        Phase::BeforeTheSlide(_)
        | Phase::Quieting { .. }
        | Phase::Darkening(_)
        | Phase::Squaring => 0,
    };
    Ok(Doors::Passing(Passage {
        phase,
        arrival,
        slide_frames,
    }))
}

/// The phase as the records show it.
fn shown(doors: Doors) -> Shown {
    match doors {
        Doors::Play => Shown::Play,
        Doors::Passing(passage) => {
            let phase = match passage.phase {
                Phase::Arriving { fetching, .. } => Phase::Arriving {
                    placed: true,
                    fetching: match fetching {
                        Fetching::Done | Fetching::Contents => Fetching::Contents,
                        Fetching::Tiles => Fetching::Tiles,
                    },
                },
                other @ (Phase::BeforeTheSlide(_)
                | Phase::Brightening(_)
                | Phase::Darkening(_)
                | Phase::Landing(_)
                | Phase::Music { .. }
                | Phase::Quieting { .. }
                | Phase::Squaring) => other,
            };
            let rules = Rules::default();
            let arrival = passage.arrival.map(|arrival| {
                let speed = arrival
                    .speed
                    .unwrap_or_else(|| rules.carry_speed[world_doors::Axis::of(arrival.way)]);
                (arrival.way, arrival.view, speed)
            });
            Shown::Passing(phase, arrival)
        }
    }
}

/// What the frame found: library 2's next index and the enemy slots.
fn found(record: &FrameRecord) -> Result<Found, DecodeError> {
    let mut enemies = [(0, 0); ENEMY_SLOTS];
    for (slot, offset) in enemies
        .iter_mut()
        .zip((0_u16..).map(|n| n.wrapping_mul(ENEMY_STRIDE)))
    {
        *slot = (
            at(record, "ENEMY_ID", ENEMY_SPECIES.wrapping_add(offset))?,
            at(
                record,
                "ENEMY_FRAME_COUNTER",
                ENEMY_COUNTER.wrapping_add(offset),
            )?,
        );
    }
    Ok(Found {
        library_1: byte(record, "SOUND_QUEUE_NEXT", QUEUE_NEXT)?,
        picks: picks(record)?,
        library_2: byte(
            record,
            "SOUND_QUEUE_NEXT",
            QUEUE_NEXT.wrapping_add(LIBRARY_2),
        )?,
        enemies,
    })
}

/// The library-1 sound each request to her sounds plays, if any: samus-appearance's choice, by
/// the decode notes' G6 until its spec owns it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Picks {
    /// `AtTheDoor`'s.
    pub at_the_door: Option<u8>,
    /// `SpinAgain`'s.
    pub spin_again: Option<u8>,
}

/// Bank $90 log: her movement type, pose, the pad, the shot's binding, her charge and her
/// animation frame.
const MOVEMENT_TYPE: WramByte = wram_byte("MOVEMENT_TYPE", 0x0A1F);
const POSE: WramByte = wram_byte("POSE", 0x0A1C);
const PAD: WramWord = wram("CONTROLLER_1_INPUT", 0x008B);
const SHOOT_BINDING: WramWord = wram("BINDING_SHOOT", 0x09B2);
const CHARGE: WramWord = wram("CHARGE_COUNTER", 0x0CD0);
const ANIMATION_FRAME: WramWord = wram("ANIMATION_FRAME", 0x0A96);
/// The movement types of a spin jump and a wall jump.
const SPIN_JUMPING: u8 = 0x03;
const WALL_JUMPING: u8 = 0x14;
/// The screw attack's and the space jump's spin poses.
const SCREW_ATTACK_POSES: [u8; 2] = [0x81, 0x82];
const SPACE_JUMP_POSES: [u8; 2] = [0x1B, 0x1C];
/// A wall jump's animation frame from which its spin is the screw attack's, and the space jump's.
const WALL_JUMP_SCREW_FROM: u16 = 0x17;
const WALL_JUMP_SPACE_FROM: u16 = 0x0D;
/// The charge under which the door's silence plays.
const CHARGE_SILENCED_BELOW: u16 = 16;
/// Library 1's sounds: the spin's end, a silence, the spin, the space jump's and the screw
/// attack's.
const SPIN_END: u8 = 0x32;
const QUIET: u8 = 0x02;
const SPIN: u8 = 0x31;
const SPACE_JUMP: u8 = 0x3E;
const SCREW_ATTACK: u8 = 0x33;

/// G6's choices as `record` finds her.
fn picks(record: &FrameRecord) -> Result<Picks, DecodeError> {
    let movement = MOVEMENT_TYPE.read(&record.wram)?;
    let pose = POSE.read(&record.wram)?;
    let shooting = word(record, PAD)? & word(record, SHOOT_BINDING)? != 0;
    let at_the_door = if movement == SPIN_JUMPING || movement == WALL_JUMPING {
        Some(SPIN_END)
    } else if !shooting && word(record, CHARGE)? < CHARGE_SILENCED_BELOW {
        Some(QUIET)
    } else {
        None
    };
    let spin_again = if movement == WALL_JUMPING {
        let frame = word(record, ANIMATION_FRAME)?;
        Some(if frame >= WALL_JUMP_SCREW_FROM {
            SCREW_ATTACK
        } else if frame >= WALL_JUMP_SPACE_FROM {
            SPACE_JUMP
        } else {
            SPIN
        })
    } else if movement == SPIN_JUMPING {
        Some(if SCREW_ATTACK_POSES.contains(&pose) {
            SCREW_ATTACK
        } else if SPACE_JUMP_POSES.contains(&pose) {
            SPACE_JUMP
        } else {
            SPIN
        })
    } else {
        None
    };
    Ok(Picks {
        at_the_door,
        spin_again,
    })
}

/// Library `library`'s next index and its last entry in `record`.
fn last_queued(record: &FrameRecord, library: u16) -> Result<(u8, u8), DecodeError> {
    let next = byte(record, "SOUND_QUEUE_NEXT", QUEUE_NEXT.wrapping_add(library))?;
    let last = byte(
        record,
        "SOUND_QUEUE",
        QUEUE_ENTRIES
            .wrapping_add(QUEUE_LENGTH.wrapping_mul(library))
            .wrapping_add(u16::from(next.wrapping_sub(1) & 0xF)),
    )?;
    Ok((next, last))
}

/// Whether every sound request so far was sent: each library's queue is empty (`$82:E2B0`).
fn sounds_sent(record: &FrameRecord) -> Result<bool, DecodeError> {
    for library in 0..3 {
        let start = byte(
            record,
            "SOUND_QUEUE_START",
            QUEUE_START.wrapping_add(library),
        )?;
        let next = byte(record, "SOUND_QUEUE_NEXT", QUEUE_NEXT.wrapping_add(library))?;
        if next.wrapping_sub(start) & 0xF != 0 {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether the music is ready as the frame of work finds it: the music queue takes its step
/// first (`$80:8F0C`), the entry at the start leaving when the music timer is 1, and the music
/// is ready when no timer is left (`$80:8EF4`).
fn music_ready(record: &FrameRecord) -> Result<bool, DecodeError> {
    let leaves = (word(record, MUSIC_TIMER)? == 1).then_some(word(record, MUSIC_START)? / 2);
    for (entry, offset) in (0..MUSIC_ENTRIES).zip((MUSIC_TIMERS..).step_by(2)) {
        if Some(entry) != leaves && at(record, "MUSIC_QUEUE_TIMER", offset)? != 0 {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether the frame is in the original's one frame of work that fetches the contents and then
/// waits for the slide's end (`$E4A9`), which the model splits into `Fetching::Contents`, whose
/// cost T1 answers, and the wait. The records bound the fetch's cost only by the slide's end
/// (the decode notes' contract with T1), so until T1 exists the harness takes it spent on the
/// first frame: the wait, which ends with the slide whatever the platform says, holds the rest.
const fn fetching_contents(doors: Doors) -> bool {
    matches!(
        phase(doors),
        Some(Phase::Arriving {
            fetching: Fetching::Contents | Fetching::Done,
            ..
        })
    )
}

/// The phase before the frame, if a passage.
const fn phase(doors: Doors) -> Option<Phase> {
    match doors {
        Doors::Play => None,
        Doors::Passing(passage) => Some(passage.phase),
    }
}

/// Whether the enemies run on a frame of work of `phase` (`WhatRuns`).
fn enemies_run(phase: Phase) -> bool {
    match phase {
        Phase::Quieting { .. } | Phase::Brightening(_) => true,
        Phase::Darkening(t) => !t.is_last(Rules::default().fade_steps),
        Phase::Squaring
        | Phase::BeforeTheSlide(_)
        | Phase::Arriving { .. }
        | Phase::Landing(_)
        | Phase::Music { .. } => false,
    }
}

/// Whether only the frame of work's clock runs in `phase`: before the slide, and as the colours
/// come up.
const fn no_clock_but_work(phase: Phase) -> bool {
    match phase {
        Phase::Quieting { .. }
        | Phase::Darkening(_)
        | Phase::Squaring
        | Phase::BeforeTheSlide(_)
        | Phase::Brightening(_) => true,
        Phase::Arriving { .. } | Phase::Landing(_) | Phase::Music { .. } => false,
    }
}

impl Domain for DoorsDomain {
    const NAME: &'static str = "world-doors";
    type State = Passing;
    type Inputs = Read;
    type World = Loaded;
    type Fields = Fields;
    type Skip = Skip;
    const FIELDS: &'static [Field<Fields>] = &[
        domain_field!("her", her),
        domain_field!("seen", seen),
        domain_field!("camera", camera),
        domain_field!("motion", motion),
        domain_field!("finished", counts.frames),
        domain_field!("hud count", counts.hud),
        domain_field!("phase", phase),
        SCREEN_COLOURS_FIELD,
        BEACON_COLOURS_FIELD,
        KRAID_COLOURS_FIELD,
        SILENCE_FIELD_DEF,
        domain_field!("sounds held", sounds_held),
        domain_field!("mini-map", minimap_on),
        ENEMIES_FIELD,
        Field {
            name: "her sound",
            same: |model, oracle| model.her_sound.agrees(oracle.her_sound),
            show: |fields| format!("{:?}", fields.her_sound),
        },
    ];

    fn behaviours(&self) -> Vec<Entry> {
        catalogue()
            .values()
            .map(|entry| Entry {
                name: format!("{:?}", entry.id),
                fields: shown_by(entry.id),
            })
            .collect()
    }

    fn decode(&self, from: &FrameRecord, interval: Interval) -> Result<(Passing, Read), Skip> {
        let doors = doors_of(from)?;
        let pointer = word(from, DOOR_POINTER)?;
        let (doorway, destination_cre) = self.doorway(pointer).map_err(|_| Skip::Door(pointer))?;
        let doors = held_doorway(doors, doorway)?;
        // The touch's record holds the rooms before the passage reads them (`$82:DDF1`): the
        // room she leaves and the doorway's; every later one holds them as read.
        let cre = if word(from, source::GAME_STATE)? == TOUCHED {
            word(from, CRE)? | destination_cre
        } else {
            word(from, CRE)? | word(from, PREVIOUS_CRE)?
        };
        let found = found(from)?;
        let scene = scene_of(from)?;
        let mut motion = [0; 7];
        for (value, source) in motion.iter_mut().zip(MOTION) {
            *value = word(from, source)?;
        }
        let inputs = Inputs {
            touched: false,
            work_done: interval.cut == CutKind::Requested || fetching_contents(doors),
            sounds_sent: sounds_sent(from)?,
            music_ready: music_ready(from)?,
            timer_running: word(from, TIMER_STATUS)? != 0,
            items_dark: cre & ITEMS_DARK != 0,
            doorway: Some(doorway),
            groups: groups(),
        };
        let passing = Passing {
            doors,
            scene,
            motion,
            counts: Counts {
                frames: word(from, source::NMI_FRAME_COUNTER_WORD)?,
                hud: source::NMI_FRAME_COUNTER_BYTE.read(&from.wram)?,
            },
            sounds_held: word(from, SOUNDS_DISABLED)? == HELD,
            report: Report::play(),
            found,
        };
        Ok((
            passing,
            Read {
                inputs,
                goal: scene.palette.goal,
                found,
                room: word(from, source::ROOM_PTR)?,
                requested: interval.cut == CutKind::Requested,
            },
        ))
    }

    fn world(&self, _from: &FrameRecord, next: &FrameRecord) -> Result<Loaded, Skip> {
        let all = colours(next, SCREEN_COLOURS)?;
        let mut loaded = [Colour::BLACK; 32];
        for (colour, &written) in loaded
            .iter_mut()
            .zip(all.get(LOADED_COLOURS).unwrap_or_default())
        {
            *colour = written;
        }
        Ok(Loaded {
            colours: Some(loaded),
        })
    }

    fn neutral(&self, _world: &Loaded) -> Option<Loaded> {
        Some(Loaded { colours: None })
    }

    fn masks(
        &self,
        from: &FrameRecord,
        _next: &FrameRecord,
        read: &Read,
    ) -> Result<Vec<Mask>, Skip> {
        let mut masks = Vec::new();
        let state = word(from, source::GAME_STATE)?;
        if !read.requested || (state != TOUCHED && state != LEAVING) {
            masks.push(Mask {
                reason: "the mini-map is turned on on the first frame of work only",
                fields: MINIMAP,
                hides: Hides::NoWriter,
            });
        }
        let before = phase(doors_of(from)?);
        if before.is_some_and(enemies_run) {
            masks.push(Mask {
                reason: "the enemies in view run (C1's)",
                fields: ENEMIES,
                hides: Hides::NoWriter,
            });
        }
        if read.room == KRAID_ROOM && before.is_some_and(enemies_run) {
            masks.push(Mask::another(
                "Kraid's corpse writes BG and sprite palette 7 as the enemies run",
                KRAID_FIELD,
            ));
        }
        // The beacon writes while the fade's step is 0, the first frame of work of the colours
        // coming up.
        if read.room == CERES_ELEVATOR_ROOM
            && matches!(before, Some(Phase::Brightening(FadeTime::START)))
        {
            masks.push(Mask::another(
                "the Ceres elevator beacon writes BG palette 2 at the fade's step 0 ($A6:F850)",
                BEACON_FIELD,
            ));
        }
        Ok(masks)
    }

    fn step(&self, state: &mut Passing, read: &Read, loaded: &mut Loaded) {
        let before = state.doors;
        if matches!(phase(before), Some(Phase::Brightening(_))) {
            state.scene.palette.goal = read.goal;
        }
        let Exports {
            doors,
            mut scene,
            report,
        } = before.step(state.scene, &read.inputs, &Rules::default());
        let arriving = matches!(phase(before), Some(Phase::Arriving { .. }));
        if let (true, Some(written)) = (arriving, loaded.colours) {
            for (colour, &written) in scene
                .palette
                .screen
                .get_mut(LOADED_COLOURS)
                .unwrap_or_default()
                .iter_mut()
                .zip(&written)
            {
                *colour = written;
            }
        }
        state.doors = doors;
        state.scene = scene;
        state.counts = state.counts.after(report.finishing);
        if report.room_sounds[RoomSound::Held] {
            state.sounds_held = true;
        }
        if report.room_sounds[RoomSound::LetThrough] {
            state.sounds_held = false;
        }
        state.report = report;
        state.found = read.found;
    }

    fn exercised(&self, state: &Passing, read: &Read, _loaded: &Loaded) -> Vec<usize> {
        use BehaviourId as B;
        let mut ids = vec![B::PassageTime, B::FinishedFrames, B::SheKeepsHerMotion];
        let work = read.requested;
        if let Doors::Passing(passage) = state.doors {
            let sliding = passage.arrival.is_some_and(|arrival| {
                passage.slide_frames
                    < Rules::default().slide_length[world_doors::Axis::of(arrival.way)]
            });
            match passage.phase {
                Phase::Quieting { .. } if work => ids.extend([B::Quieting, B::TheFades]),
                Phase::Darkening(_) | Phase::Brightening(_) if work => {
                    ids.extend([B::TheFades, B::Fading, B::WhatRuns]);
                }
                Phase::Squaring if work => ids.push(B::Squaring),
                Phase::BeforeTheSlide(n) if work => {
                    ids.push(B::Building);
                    if n.next().is_none() {
                        ids.extend([B::SettingOff, B::Sliding, B::Carrying]);
                    }
                }
                Phase::Arriving { placed, .. } => {
                    ids.push(B::Arriving);
                    if !placed {
                        ids.push(B::WhereSheLands);
                    } else if sliding {
                        ids.extend([B::Sliding, B::Carrying]);
                    }
                }
                Phase::Landing(_) | Phase::Music { .. } if work => {
                    ids.extend([B::Landing, B::WhereSheLands]);
                }
                Phase::Quieting { .. }
                | Phase::Darkening(_)
                | Phase::Brightening(_)
                | Phase::Squaring
                | Phase::BeforeTheSlide(_)
                | Phase::Landing(_)
                | Phase::Music { .. } => {}
            }
        }
        let entries: Vec<BehaviourId> = catalogue().values().map(|entry| entry.id).collect();
        let mut places: Vec<usize> = ids
            .into_iter()
            .filter_map(|id| entries.iter().position(|&entry| entry == id))
            .collect();
        places.sort_unstable();
        places.dedup();
        places
    }

    fn fields(&self, state: &Passing) -> Fields {
        let report = state.report;
        Fields {
            her: state.scene.her,
            seen: state.scene.seen,
            camera: state.scene.camera,
            motion: state.motion,
            counts: state.counts,
            phase: shown(state.doors),
            colours: state.scene.palette.screen,
            silence: Silence::Asked {
                from: state.found.library_2,
                asked: report.room_sounds[RoomSound::Silence],
            },
            sounds_held: state.sounds_held,
            minimap_on: report.minimap_on,
            enemies: Enemies {
                run: (report.call != Some(RoomCall::FetchContents))
                    .then_some(report.runs[Activity::Enemies]),
                slots: state.found.enemies,
            },
            her_sound: Sound::Asked {
                from: state.found.library_1,
                sound: match report.her_sound {
                    Some(HerSound::AtTheDoor) => state.found.picks.at_the_door,
                    Some(HerSound::SpinAgain) => state.found.picks.spin_again,
                    None => None,
                },
            },
        }
    }

    fn expected(&self, next: &FrameRecord) -> Result<Fields, Skip> {
        let doors = match doors_of(next) {
            Ok(doors) => doors,
            Err(Skip::Play) => Doors::Play,
            Err(skip) => return Err(skip),
        };
        let scene = scene_of(next)?;
        let mut motion = [0; 7];
        for (value, source) in motion.iter_mut().zip(MOTION) {
            *value = word(next, source)?;
        }
        let found = found(next)?;
        let (_, last) = last_queued(next, LIBRARY_2)?;
        let (library_1, last_1) = last_queued(next, 0)?;
        Ok(Fields {
            her: scene.her,
            seen: scene.seen,
            camera: scene.camera,
            motion,
            counts: Counts {
                frames: word(next, source::NMI_FRAME_COUNTER_WORD)?,
                hud: source::NMI_FRAME_COUNTER_BYTE.read(&next.wram)?,
            },
            phase: shown(doors),
            colours: scene.palette.screen,
            silence: Silence::Queued {
                next: found.library_2,
                last: last == SILENCE,
            },
            sounds_held: word(next, SOUNDS_DISABLED)? == HELD,
            minimap_on: word(next, MINIMAP_DISABLED)? == 0,
            enemies: Enemies {
                run: None,
                slots: found.enemies,
            },
            her_sound: Sound::Queued {
                next: library_1,
                last: last_1,
            },
        })
    }

    /// Before the slide and as the colours come up, no clock runs but the frame of work's: a
    /// record that isn't `R` falls inside it, holding its work part-done (the fade's colours
    /// part-faded), so the frame runs on to the next `R`. From the slide's first step to her
    /// stepping clear the slide and the finished frames run every frame, so every record ends
    /// one.
    fn folds(&self, from: &FrameRecord, next: &FrameRecord) -> bool {
        CutKind::of(next.cut) != CutKind::Requested
            && doors_of(from).is_ok_and(|doors| phase(doors).is_some_and(no_clock_but_work))
    }

    fn uncompared(&self, skip: &Skip) -> Uncompared {
        match skip {
            Skip::Play => Uncompared::OutOfScope(skip.to_string()),
            Skip::Unreadable(_)
            | Skip::UnknownFunction(_)
            | Skip::NotAColour(_)
            | Skip::Door(_)
            | Skip::ArrivalDiffers => Uncompared::Defect(skip.to_string()),
        }
    }
}
