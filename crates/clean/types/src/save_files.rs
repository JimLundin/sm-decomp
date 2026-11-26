//! The save boundary's records and State: the cartridge with its three player slots, the run a
//! file holds, and the choice or write a caller may leave part-way.
//!
//! The State is everything the save-files model carries between requests. A caller may build
//! each part at any value its types admit, read it back, carry it forward and restore it,
//! pending actions included.

use std::collections::BTreeSet;

use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};

use crate::file_select::Slot;
use crate::options::Settings;
use crate::{
    Area, Beam, BossKind, Event, Item, PlayFrames, PlayHours, PlayMinutes, PlaySeconds, PlayTime,
    Set, StationId,
};

/// The words of a file's encoded run, 814.
pub const PAYLOAD_WORDS: usize = 814;
/// The bytes of the completion mark, 12.
pub const MARK_BYTES: usize = 12;
/// The cartridge's retained opaque bytes, 3,260.
pub const RETAINED_BYTES: usize = 3_260;
/// The cartridge's spare check words, 4.
pub const SPARE_WORDS: usize = 4;
/// The bytes of the whole battery image, 8,192.
pub const IMAGE_BYTES: usize = 8_192;
/// The bytes of one area's explored map, 256, each covering eight cells.
pub const MAP_BYTES: usize = 256;

/// A run of values of one exact length, kept at that length by its constructor.
macro_rules! fixed_length {
    ($name:ident, $value:ty, $vec:literal, $len:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = $vec, into = $vec)]
        pub struct $name(Vec<$value>);

        impl $name {
            /// These values, if there are exactly as many as it holds.
            #[must_use]
            pub fn new(values: Vec<$value>) -> Option<Self> {
                (values.len() == $len).then_some(Self(values))
            }

            /// Every value zero.
            #[must_use]
            pub fn zeroed() -> Self {
                Self(vec![0; $len])
            }

            /// Its values, in order.
            #[must_use]
            pub fn as_slice(&self) -> &[$value] {
                &self.0
            }

            /// Its values to change in place; their number stays as it is.
            pub fn as_mut_slice(&mut self) -> &mut [$value] {
                &mut self.0
            }
        }

        impl TryFrom<Vec<$value>> for $name {
            type Error = &'static str;

            fn try_from(values: Vec<$value>) -> Result<Self, Self::Error> {
                Self::new(values).ok_or(concat!(stringify!($name), " has the wrong length"))
            }
        }

        impl From<$name> for Vec<$value> {
            fn from(run: $name) -> Self {
                run.0
            }
        }
    };
}

fixed_length!(
    EncodedRun,
    u16,
    "Vec<u16>",
    PAYLOAD_WORDS,
    "A file's run as the format encodes it: all 814 unsigned 16-bit words, which the checks \
     sum. Zeroed, it is a cleared file."
);
fixed_length!(
    RetainedBytes,
    u8,
    "Vec<u8>",
    RETAINED_BYTES,
    "The cartridge's 3,260 opaque bytes, which no request changes."
);
fixed_length!(
    BatteryImage,
    u8,
    "Vec<u8>",
    IMAGE_BYTES,
    "The whole 8,192-byte battery image, as the format lays the cartridge out."
);
fixed_length!(
    ExploredCells,
    u8,
    "Vec<u8>",
    MAP_BYTES,
    "One area's explored cells, the saved grid: 256 bytes, each covering eight cells. Zeroed, \
     it is blank."
);

/// Spec: `ReadSlot`. A check pair: the payload's word sum mod 65,536 and its complement,
/// 65,535 less the sum. The cartridge keeps two for each file, a primary and a mirror.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckPair {
    /// The word sum.
    pub sum: u16,
    /// The sum's complement.
    pub complement: u16,
}

/// One file on the cartridge: its payload and its two independent check pairs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotRecord {
    /// The encoded run.
    pub payload: EncodedRun,
    /// The primary check pair.
    pub primary: CheckPair,
    /// The mirror check pair.
    pub mirror: CheckPair,
}

/// Spec: `MenuFiles`, `ChooseFile`. The remembered selection: the last chosen slot's number and
/// its check word, any two words, unrecognized values included.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RememberedSelection {
    /// The slot's number: A is 0, B is 1, C is 2.
    pub slot_word: u16,
    /// The selection check: the slot number's bitwise complement when a choice wrote it.
    pub check_word: u16,
}

/// Spec: `BootFiles`. The completion mark, 12 opaque bytes. The format names the completed and
/// incomplete marks; every other value is carried as it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionMark(pub [u8; MARK_BYTES]);

/// The cartridge's contents no request changes: its opaque bytes and spare check words.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedCartridgeData {
    /// The 3,260 opaque bytes.
    pub bytes: RetainedBytes,
    /// The four spare check words.
    pub spare_words: [u16; SPARE_WORDS],
}

/// The cartridge: the three player slots, the remembered selection, the completion mark and
/// the retained data. The format maps it to and from the battery image byte for byte.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cartridge {
    /// The three player slots.
    pub slots: EnumMap<Slot, SlotRecord>,
    /// The remembered selection.
    pub selection: RememberedSelection,
    /// The completion mark.
    pub mark: CompletionMark,
    /// The retained bytes and spare words.
    pub retained: RetainedCartridgeData,
}

/// A stored value the format doesn't recognize, kept at its exact width and contents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpaqueValue {
    /// An 8-bit value.
    Byte(u8),
    /// A 16-bit value.
    Word(u16),
}

/// Spec: `ChooseFile`, `WriteFile`. Where a run starts when it is resumed: the start route the
/// options receive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Destination {
    /// The intro: a new file.
    Intro,
    /// The arrival at Ceres, saved as the trip to Ceres ends.
    CeresArrival,
    /// The flight to Zebes, saved as the Ceres escape fades out.
    ZebesFlight,
    /// A route the format doesn't recognize; the caller decides which scenes it can start.
    Unrecognized(OpaqueValue),
}

/// The area a restart point names, as stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoredArea {
    /// An area the format recognizes.
    Known(Area),
    /// A stored area code the format doesn't recognize.
    Unrecognized(OpaqueValue),
}

/// Spec: `WriteFile`. A restart point: the destination, the area and the station number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestartPoint {
    /// Where the run starts when resumed.
    pub destination: Destination,
    /// The area.
    pub area: StoredArea,
    /// The station number within the area, any unsigned 16-bit value.
    pub station: StationId,
}

/// Opaque words the format carries through unchanged: a profile's other retained words,
/// pass-through progress data, or a run's retained tail.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedWords(pub Vec<u16>);

/// A supply with a capacity: energy, reserves or one kind of ammunition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Supply {
    /// What she has now.
    pub current: u16,
    /// The most she can hold.
    pub maximum: u16,
}

/// A kind of ammunition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum Ammunition {
    /// Missiles.
    Missiles,
    /// Super missiles.
    SuperMissiles,
    /// Power bombs.
    PowerBombs,
}

/// Equipment: items and beams.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gear {
    /// The items.
    pub items: Set<Item>,
    /// The beams.
    pub beams: Set<Beam>,
}

/// The weapon selected on the HUD.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectedWeapon {
    /// No weapon selected.
    Nothing,
    /// Missiles.
    Missiles,
    /// Super missiles.
    SuperMissiles,
    /// Power bombs.
    PowerBombs,
    /// The grapple beam.
    GrappleBeam,
    /// The x-ray scope.
    XRayScope,
    /// A selection the format doesn't recognize.
    Unrecognized(OpaqueValue),
}

/// An action the player binds to a pad button. Movement always uses the directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum Control {
    /// Shoot.
    Shoot,
    /// Jump.
    Jump,
    /// Run.
    Run,
    /// Cancel the selected item.
    ItemCancel,
    /// Select an item.
    ItemSelect,
    /// Aim up.
    AimUp,
    /// Aim down.
    AimDown,
}

/// A pad button an action is bound to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PadButton {
    /// The A button.
    A,
    /// The B button.
    B,
    /// The X button.
    X,
    /// The Y button.
    Y,
    /// The Select button.
    Select,
    /// The left shoulder button.
    L,
    /// The right shoulder button.
    R,
    /// A binding the format doesn't recognize.
    Unrecognized(OpaqueValue),
}

/// The control bindings: a button for each bindable action.
pub type ControlBindings = EnumMap<Control, PadButton>;

/// Spec: `ChooseFile`. Her profile: her inventory and preferences.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerProfile {
    /// The equipment she owns.
    pub owned: Gear,
    /// The equipment she has in use.
    pub in_use: Gear,
    /// The control bindings.
    pub controls: ControlBindings,
    /// The settings: language, moonwalk and weapon reset.
    pub settings: Settings,
    /// Her energy and its capacity.
    pub energy: Supply,
    /// Her reserve energy and its capacity.
    pub reserves: Supply,
    /// Each kind of ammunition and its capacity.
    pub ammunition: EnumMap<Ammunition, Supply>,
    /// The selected weapon.
    pub weapon: SelectedWeapon,
    /// The four components of play time.
    pub play_time: PlayTime,
    /// The placement-disabled value.
    pub placement_disabled: u16,
    /// The spare startup word.
    pub startup_word: u16,
    /// The profile's other retained words, which only a read replaces.
    pub retained: RetainedWords,
}

/// Numbered flags of room content (item places, doors, stations, elevators): the format and
/// the room content give each number its meaning.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ContentFlags(pub BTreeSet<u16>);

/// Spec: `ChooseFile`. Her persistent progress.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    /// The events that have happened.
    pub events: Set<Event>,
    /// The bosses defeated, by area.
    pub bosses: EnumMap<Area, Set<BossKind>>,
    /// The acquired item count.
    pub item_count: u16,
    /// The item places whose items she acquired.
    pub items: ContentFlags,
    /// The other changed locations.
    pub changed_locations: ContentFlags,
    /// The doors opened.
    pub doors: ContentFlags,
    /// The save station progress.
    pub stations: ContentFlags,
    /// The elevator progress.
    pub elevators: ContentFlags,
    /// The areas whose map station she visited: the map collection.
    pub maps_collected: Set<Area>,
    /// Progress data the format passes through as it is.
    pub pass_through: RetainedWords,
}

/// The remembered explored maps, one per area.
pub type RememberedMaps = EnumMap<Area, ExploredCells>;

/// All explored grids start empty.
#[must_use]
pub fn blank_maps() -> RememberedMaps {
    EnumMap::from_fn(|_| ExploredCells::zeroed())
}

/// Spec: `ChooseFile`, `WriteFile`. A run in typed form: what a file holds once decoded, and
/// what a save captures.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSnapshot {
    /// Her profile.
    pub profile: PlayerProfile,
    /// Her progress.
    pub progress: Progress,
    /// The remembered explored maps.
    pub maps: RememberedMaps,
    /// Where the run restarts.
    pub restart: RestartPoint,
    /// The run's retained tail, which the format passes through.
    pub tail: RetainedWords,
}

/// Spec: `BootFiles`. How many attract sequences, the recorded scenes on the title screen, are
/// available.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DemoCount {
    /// Three attract sequences: the game not marked completed.
    Three,
    /// Four attract sequences: the game marked completed.
    Four,
}

impl DemoCount {
    /// The number of attract sequences.
    #[must_use]
    pub const fn sets(self) -> u8 {
        match self {
            Self::Three => 3,
            Self::Four => 4,
        }
    }
}

/// Spec: `MenuFiles`, `ChooseFile`. The working profile and maps: carried from the last
/// successful read or the current run, and kept through a failed read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingRun {
    /// The working profile.
    pub profile: PlayerProfile,
    /// The working remembered maps.
    pub maps: RememberedMaps,
}

/// Spec: `ChooseFile`. Whether this cartridge session's file menu has finished its scan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuInspection {
    /// No completed menu scan since boot: no choice is admitted.
    Uninspected,
    /// The menu scan finished: a choice is admitted.
    Inspected,
}

/// A choice or a write that has reached its first milestone and not yet its second. While one
/// stands, no new choice, read or save runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingAction {
    /// Nothing in progress.
    Idle,
    /// Spec: `ChooseFile`. The selection is written for this slot; its load is to come.
    Choice(Slot),
    /// Spec: `WriteFile`. The payload is written to this slot; its new check pair is to come.
    Write {
        /// The slot written.
        slot: Slot,
        /// The pair of the payload written, for both checks.
        pair: CheckPair,
    },
}

/// The save-files model's complete State.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveFilesState {
    /// The cartridge.
    pub cartridge: Cartridge,
    /// The active slot: the file chosen, if one is.
    pub active: Option<Slot>,
    /// The working profile and maps.
    pub working: WorkingRun,
    /// Whether the menu scan has finished this session.
    pub menu: MenuInspection,
    /// The choice or write in progress.
    pub pending: PendingAction,
}

impl SaveFilesState {
    /// The State at power-on: the cartridge supplied, no file chosen, nothing inspected or
    /// pending, and the working profile and maps the caller supplies.
    #[must_use]
    pub const fn power_on(cartridge: Cartridge, working: WorkingRun) -> Self {
        Self {
            cartridge,
            active: None,
            working,
            menu: MenuInspection::Uninspected,
            pending: PendingAction::Idle,
        }
    }
}

/// Play time at zero: a fresh file's.
pub const NO_PLAY_TIME: PlayTime = PlayTime {
    hours: PlayHours::ZERO,
    minutes: PlayMinutes::ZERO,
    seconds: PlaySeconds::ZERO,
    frames: PlayFrames::ZERO,
};
