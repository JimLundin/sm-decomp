//! The vectors' invented format: its marks, its thirteen typed words and its image layout.

use enum_map::{Enum, EnumMap};
use save_files::SaveFormat;
use types::file_select::Slot;
use types::options::Settings;
use types::save_files::{
    BatteryImage, Cartridge, CheckPair, CompletionMark, ContentFlags, Destination, EncodedRun,
    ExploredCells, Gear, MARK_BYTES, NO_PLAY_TIME, OpaqueValue, PadButton, PlayerProfile, Progress,
    RememberedSelection, RestartPoint, RetainedBytes, RetainedCartridgeData, RetainedWords,
    RunSnapshot, SPARE_WORDS, SelectedWeapon, SlotRecord, StoredArea, Supply, blank_maps,
};
use types::{Area, BossKind, Event, Set, StationId};

/// The completed mark's repeated byte.
pub const COMPLETED: u8 = 3;
/// The incomplete mark's repeated byte.
pub const INCOMPLETE: u8 = 4;
/// The other mark's repeated byte.
pub const OTHER: u8 = 5;

pub const ENERGY: usize = 0;
pub const CAPACITY: usize = 1;
pub const DESTINATION: usize = 2;
pub const AREA: usize = 3;
pub const STATION: usize = 4;
pub const KEPT_PROFILE: usize = 5;
pub const PLACEMENT_DISABLED: usize = 6;
pub const STARTUP_WORD: usize = 7;
pub const EVENT: usize = 8;
pub const BOSS: usize = 9;
pub const ITEM_COUNT: usize = 10;
pub const MAP: usize = 11;
pub const TAIL: usize = 12;

/// The one byte of Crateria's map the fixture keeps.
pub const MAP_BYTE: usize = 43;

pub const INTRO: u16 = 0;
pub const CERES_ARRIVAL: u16 = 7;
pub const ZEBES_FLIGHT: u16 = 8;
pub const CRATERIA: u16 = 4;
pub const CERES: u16 = 9;

const WORD_BITS: usize = 16;
/// The kinds of boss in each area.
const BOSS_KINDS: usize = 3;

/// The menu order the image lays slots out in.
const SLOTS: [Slot; 3] = [Slot::First, Slot::Second, Slot::Third];

/// The invented format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fixture;

pub const fn mark_of(byte: u8) -> CompletionMark {
    CompletionMark([byte; MARK_BYTES])
}

fn first_word(words: &RetainedWords) -> u16 {
    words.0.first().copied().unwrap_or(0)
}

fn opaque_word(value: OpaqueValue) -> u16 {
    match value {
        OpaqueValue::Byte(byte) => u16::from(byte),
        OpaqueValue::Word(word) => word,
    }
}

fn destination_code(destination: Destination) -> u16 {
    match destination {
        Destination::Intro => INTRO,
        Destination::CeresArrival => CERES_ARRIVAL,
        Destination::ZebesFlight => ZEBES_FLIGHT,
        Destination::Unrecognized(value) => opaque_word(value),
    }
}

fn destination_of(code: u16) -> Destination {
    match code {
        INTRO => Destination::Intro,
        CERES_ARRIVAL => Destination::CeresArrival,
        ZEBES_FLIGHT => Destination::ZebesFlight,
        _ => Destination::Unrecognized(OpaqueValue::Word(code)),
    }
}

fn area_code(area: StoredArea) -> u16 {
    match area {
        StoredArea::Known(Area::Crateria) => CRATERIA,
        StoredArea::Known(Area::Ceres) => CERES,
        StoredArea::Known(other) => panic!("the fixture has no code for {other:?}"),
        StoredArea::Unrecognized(value) => opaque_word(value),
    }
}

fn area_of(code: u16) -> StoredArea {
    match code {
        CRATERIA => StoredArea::Known(Area::Crateria),
        CERES => StoredArea::Known(Area::Ceres),
        _ => StoredArea::Unrecognized(OpaqueValue::Word(code)),
    }
}

/// The word's bits, bit `i` the `i`th member of `K`, for as many members as a word holds.
fn members_word<K: Enum + Copy>(set: &Set<K>) -> u16 {
    set.iter()
        .filter(|&(member, &is_in)| is_in && member.into_usize() < WORD_BITS)
        .fold(0, |word, (member, _)| word | 1 << member.into_usize())
}

fn word_members<K: Enum>(word: u16) -> Set<K> {
    EnumMap::from_fn(|member: K| {
        let bit = member.into_usize();
        bit < WORD_BITS && word >> bit & 1 == 1
    })
}

/// A defeated boss's defeat flag: its bit in the boss word, area by area in the order listed,
/// three kinds each.
fn defeat_flag(area: Area, kind: BossKind) -> usize {
    area.into_usize()
        .wrapping_mul(BOSS_KINDS)
        .wrapping_add(kind.into_usize())
}

fn bosses_word(bosses: &EnumMap<Area, Set<BossKind>>) -> u16 {
    bosses
        .iter()
        .flat_map(|(area, kinds)| kinds.iter().map(move |(kind, &is_in)| (area, kind, is_in)))
        .filter(|&(area, kind, is_in)| is_in && defeat_flag(area, kind) < WORD_BITS)
        .fold(0, |word, (area, kind, _)| {
            word | 1 << defeat_flag(area, kind)
        })
}

fn word_bosses(word: u16) -> EnumMap<Area, Set<BossKind>> {
    EnumMap::from_fn(|area| {
        EnumMap::from_fn(|kind| {
            let flag = defeat_flag(area, kind);
            flag < WORD_BITS && word >> flag & 1 == 1
        })
    })
}

fn no_gear() -> Gear {
    Gear {
        items: EnumMap::default(),
        beams: EnumMap::default(),
    }
}

const NO_SUPPLY: Supply = Supply {
    current: 0,
    maximum: 0,
};

fn profile_of(words: &[u16]) -> PlayerProfile {
    PlayerProfile {
        owned: no_gear(),
        in_use: no_gear(),
        controls: EnumMap::from_fn(|_| PadButton::Unrecognized(OpaqueValue::Word(0))),
        settings: Settings::default(),
        energy: Supply {
            current: words[ENERGY],
            maximum: words[CAPACITY],
        },
        reserves: NO_SUPPLY,
        ammunition: EnumMap::from_fn(|_| NO_SUPPLY),
        weapon: SelectedWeapon::Nothing,
        play_time: NO_PLAY_TIME,
        placement_disabled: words[PLACEMENT_DISABLED],
        startup_word: words[STARTUP_WORD],
        retained: RetainedWords(vec![words[KEPT_PROFILE]]),
    }
}

fn progress_of(words: &[u16]) -> Progress {
    Progress {
        events: word_members::<Event>(words[EVENT]),
        bosses: word_bosses(words[BOSS]),
        item_count: words[ITEM_COUNT],
        items: ContentFlags::default(),
        changed_locations: ContentFlags::default(),
        doors: ContentFlags::default(),
        stations: ContentFlags::default(),
        elevators: ContentFlags::default(),
        maps_collected: EnumMap::default(),
        pass_through: RetainedWords(Vec::new()),
    }
}

/// Crateria's kept map byte.
pub fn kept_map_byte(maps: &EnumMap<Area, ExploredCells>) -> u8 {
    maps[Area::Crateria].as_slice()[MAP_BYTE]
}

/// A blank map with the kept byte set: the fixture's live or remembered map.
pub fn map_with(byte: u8) -> ExploredCells {
    let mut map = ExploredCells::zeroed();
    map.as_mut_slice()[MAP_BYTE] = byte;
    map
}

impl SaveFormat for Fixture {
    fn completed_mark(&self) -> CompletionMark {
        mark_of(COMPLETED)
    }

    fn incomplete_mark(&self) -> CompletionMark {
        mark_of(INCOMPLETE)
    }

    fn encode(&self, run: &RunSnapshot) -> EncodedRun {
        let mut payload = EncodedRun::zeroed();
        let words = payload.as_mut_slice();
        words[ENERGY] = run.profile.energy.current;
        words[CAPACITY] = run.profile.energy.maximum;
        words[DESTINATION] = destination_code(run.restart.destination);
        words[AREA] = area_code(run.restart.area);
        words[STATION] = run.restart.station.0;
        words[KEPT_PROFILE] = first_word(&run.profile.retained);
        words[PLACEMENT_DISABLED] = run.profile.placement_disabled;
        words[STARTUP_WORD] = run.profile.startup_word;
        words[EVENT] = members_word(&run.progress.events);
        words[BOSS] = bosses_word(&run.progress.bosses);
        words[ITEM_COUNT] = run.progress.item_count;
        words[MAP] = u16::from(kept_map_byte(&run.maps));
        words[TAIL] = first_word(&run.tail);
        payload
    }

    fn decode(&self, payload: &EncodedRun) -> RunSnapshot {
        let words = payload.as_slice();
        let mut maps = blank_maps();
        maps[Area::Crateria] = map_with(words[MAP].to_le_bytes()[0]);
        RunSnapshot {
            profile: profile_of(words),
            progress: progress_of(words),
            maps,
            restart: RestartPoint {
                destination: destination_of(words[DESTINATION]),
                area: area_of(words[AREA]),
                station: StationId(words[STATION]),
            },
            tail: RetainedWords(vec![words[TAIL]]),
        }
    }

    fn cartridge(&self, image: &BatteryImage) -> Cartridge {
        let mut reader = Reader {
            bytes: image.as_slice(),
        };
        let mark = CompletionMark(reader.take(MARK_BYTES).try_into().unwrap());
        let selection = RememberedSelection {
            slot_word: reader.word(),
            check_word: reader.word(),
        };
        let bytes = RetainedBytes::new(reader.take(types::save_files::RETAINED_BYTES).to_vec());
        let spare_words = [(); SPARE_WORDS].map(|()| reader.word());
        let records = SLOTS.map(|_| reader.record());
        Cartridge {
            slots: EnumMap::from_array(records),
            selection,
            mark,
            retained: RetainedCartridgeData {
                bytes: bytes.unwrap(),
                spare_words,
            },
        }
    }

    fn image(&self, cartridge: &Cartridge) -> BatteryImage {
        let mut bytes = cartridge.mark.0.to_vec();
        let mut words = vec![
            cartridge.selection.slot_word,
            cartridge.selection.check_word,
        ];
        push_words(&mut bytes, &words);
        bytes.extend(cartridge.retained.bytes.as_slice());
        push_words(&mut bytes, &cartridge.retained.spare_words);
        for slot in SLOTS {
            let record = &cartridge.slots[slot];
            words.clear();
            words.extend(record.payload.as_slice());
            words.extend([
                record.primary.sum,
                record.primary.complement,
                record.mirror.sum,
                record.mirror.complement,
            ]);
            push_words(&mut bytes, &words);
        }
        BatteryImage::new(bytes).unwrap()
    }
}

fn push_words(bytes: &mut Vec<u8>, words: &[u16]) {
    bytes.extend(words.iter().flat_map(|word| word.to_le_bytes()));
}

/// Reads the image front to back.
struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> &'a [u8] {
        let (taken, rest) = self.bytes.split_at(count);
        self.bytes = rest;
        taken
    }

    fn word(&mut self) -> u16 {
        u16::from_le_bytes(self.take(2).try_into().unwrap())
    }

    fn pair(&mut self) -> CheckPair {
        CheckPair {
            sum: self.word(),
            complement: self.word(),
        }
    }

    fn record(&mut self) -> SlotRecord {
        let words = (0..types::save_files::PAYLOAD_WORDS)
            .map(|_| self.word())
            .collect();
        SlotRecord {
            payload: EncodedRun::new(words).unwrap(),
            primary: self.pair(),
            mirror: self.pair(),
        }
    }
}
