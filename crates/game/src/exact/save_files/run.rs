use enum_map::{Enum, EnumMap};
use types::options::{Language, Settings};
use types::save_files::{
    Ammunition, ContentFlags, Control, Destination, EncodedRun, Gear, OpaqueValue, PadButton,
    PlayerProfile, Progress, RestartPoint, RetainedWords, RunSnapshot, SelectedWeapon, StoredArea,
    Supply, blank_maps,
};
use types::{
    Area, Beam, BossKind, Item, PlayFrames, PlayHours, PlayMinutes, PlaySeconds, PlayTime, Set,
    StationId,
};

use super::{Format, PACKED_BYTES};

const PROFILE: usize = 48;
const PROGRESS_BYTES: usize = 244;
const PROGRESS: std::ops::Range<usize> = 96..96 + PROGRESS_BYTES;
const MAP: std::ops::Range<usize> = 348..348 + PACKED_BYTES;
const TAIL: usize = MAP.end;
const AREAS: [Area; 7] = [
    Area::Crateria,
    Area::Brinstar,
    Area::Norfair,
    Area::WreckedShip,
    Area::Maridia,
    Area::Tourian,
    Area::Ceres,
];
const ITEMS: [(u16, Item); 11] = [
    (1, Item::VariaSuit),
    (2, Item::SpringBall),
    (4, Item::MorphBall),
    (8, Item::ScrewAttack),
    (0x20, Item::GravitySuit),
    (0x100, Item::HiJumpBoots),
    (0x200, Item::SpaceJump),
    (0x1000, Item::Bombs),
    (0x2000, Item::SpeedBooster),
    (0x4000, Item::GrappleBeam),
    (0x8000, Item::XRayScope),
];
const BEAMS: [(u16, Beam); 5] = [
    (1, Beam::Wave),
    (2, Beam::Ice),
    (4, Beam::Spazer),
    (8, Beam::Plasma),
    (0x1000, Beam::Charge),
];
const BOSSES: [(u16, BossKind); 3] = [
    (1, BossKind::MainBoss),
    (2, BossKind::MiniBoss),
    (4, BossKind::Torizo),
];
const BUTTONS: [(u16, PadButton); 7] = [
    (0x80, PadButton::A),
    (0x8000, PadButton::B),
    (0x40, PadButton::X),
    (0x4000, PadButton::Y),
    (0x2000, PadButton::Select),
    (0x20, PadButton::L),
    (0x10, PadButton::R),
];
const WEAPONS: [SelectedWeapon; 6] = [
    SelectedWeapon::Nothing,
    SelectedWeapon::Missiles,
    SelectedWeapon::SuperMissiles,
    SelectedWeapon::PowerBombs,
    SelectedWeapon::GrappleBeam,
    SelectedWeapon::XRayScope,
];

// The seven configurable controls are interleaved with four directional masks.
const fn binding(control: Control) -> usize {
    match control {
        Control::Shoot => 8,
        Control::Jump => 9,
        Control::Run => 10,
        Control::ItemCancel => 11,
        Control::ItemSelect => 12,
        Control::AimUp => 14,
        Control::AimDown => 13,
    }
}

const fn ammo(kind: Ammunition) -> usize {
    match kind {
        Ammunition::Missiles => 18,
        Ammunition::SuperMissiles => 20,
        Ammunition::PowerBombs => 22,
    }
}

fn word(value: OpaqueValue) -> u16 {
    match value {
        OpaqueValue::Byte(value) => u16::from(value),
        OpaqueValue::Word(value) => value,
    }
}

fn bits<K: Enum + Copy>(raw: u16, table: &[(u16, K)]) -> Set<K> {
    let mut set = Set::default();
    for &(mask, key) in table {
        set[key] = raw & mask != 0;
    }
    set
}

fn overlay<K: Enum + Copy>(raw: u16, table: &[(u16, K)], set: &Set<K>) -> u16 {
    table.iter().fold(raw, |raw, &(mask, key)| {
        (raw & !mask) | if set[key] { mask } else { 0 }
    })
}

fn read<T: Copy + Default>(raw: &[T], index: usize) -> T {
    raw.get(index).copied().unwrap_or_default()
}

fn write<T>(raw: &mut [T], index: usize, value: T) {
    if let Some(out) = raw.get_mut(index) {
        *out = value;
    }
}

fn decode_profile(raw: &[u16]) -> PlayerProfile {
    let gear = |offset: usize| Gear {
        items: bits(read(raw, offset), &ITEMS),
        beams: bits(read(raw, offset.wrapping_add(2)), &BEAMS),
    };
    let supply = |current, maximum| Supply {
        current: read(raw, current),
        maximum: read(raw, maximum),
    };
    PlayerProfile {
        owned: gear(1),
        in_use: gear(0),
        controls: EnumMap::from_fn(|control| {
            let value = read(raw, binding(control));
            BUTTONS
                .iter()
                .find_map(|&(mask, button)| (value == mask).then_some(button))
                .unwrap_or(PadButton::Unrecognized(OpaqueValue::Word(value)))
        }),
        settings: Settings {
            language: if read(raw, 32) == 0 {
                Language::English
            } else {
                Language::Japanese
            },
            moonwalk: read(raw, 33) != 0,
            weapon_reset: read(raw, 36) != 0,
        },
        energy: supply(16, 17),
        reserves: supply(26, 25),
        ammunition: EnumMap::from_fn(|kind| supply(ammo(kind), ammo(kind).wrapping_add(1))),
        weapon: WEAPONS
            .get(usize::from(read(raw, 24)))
            .copied()
            .unwrap_or_else(|| SelectedWeapon::Unrecognized(OpaqueValue::Word(read(raw, 24)))),
        play_time: PlayTime {
            frames: PlayFrames::new(u8::try_from(read(raw, 28)).unwrap_or(u8::MAX))
                .unwrap_or(PlayFrames::ZERO),
            seconds: PlaySeconds::new(u8::try_from(read(raw, 29)).unwrap_or(u8::MAX))
                .unwrap_or(PlaySeconds::ZERO),
            minutes: PlayMinutes::new(u8::try_from(read(raw, 30)).unwrap_or(u8::MAX))
                .unwrap_or(PlayMinutes::ZERO),
            hours: PlayHours::new(u8::try_from(read(raw, 31)).unwrap_or(u8::MAX))
                .unwrap_or(PlayHours::ZERO),
        },
        placement_disabled: read(raw, 34),
        startup_word: read(raw, 35),
        retained: RetainedWords(raw.to_vec()),
    }
}

fn encode_profile(profile: &PlayerProfile, raw: &mut [u16]) {
    for (out, &value) in raw.iter_mut().zip(&profile.retained.0) {
        *out = value;
    }
    for (offset, gear) in [(0, profile.in_use), (1, profile.owned)] {
        write(raw, offset, overlay(read(raw, offset), &ITEMS, &gear.items));
        write(
            raw,
            offset.wrapping_add(2),
            overlay(read(raw, offset.wrapping_add(2)), &BEAMS, &gear.beams),
        );
    }
    for (control, &button) in &profile.controls {
        let value = if let PadButton::Unrecognized(value) = button {
            word(value)
        } else {
            BUTTONS
                .iter()
                .find_map(|&(mask, key)| (key == button).then_some(mask))
                .unwrap_or_default()
        };
        write(raw, binding(control), value);
    }
    let mut supply = |current, maximum, values: Supply| {
        write(raw, current, values.current);
        write(raw, maximum, values.maximum);
    };
    supply(16, 17, profile.energy);
    supply(26, 25, profile.reserves);
    for (kind, &values) in &profile.ammunition {
        supply(ammo(kind), ammo(kind).wrapping_add(1), values);
    }
    let weapon = if let SelectedWeapon::Unrecognized(value) = profile.weapon {
        word(value)
    } else {
        u16::try_from(
            WEAPONS
                .iter()
                .position(|&entry| entry == profile.weapon)
                .unwrap_or_default(),
        )
        .unwrap_or_default()
    };
    write(raw, 24, weapon);
    let previous = decode_profile(raw);
    let clock = profile.play_time;
    let old = previous.play_time;
    for (index, value, prev) in [
        (28, clock.frames.get(), old.frames.get()),
        (29, clock.seconds.get(), old.seconds.get()),
        (30, clock.minutes.get(), old.minutes.get()),
        (31, clock.hours.get(), old.hours.get()),
    ] {
        if value != prev {
            write(raw, index, u16::from(value));
        }
    }
    for (index, value) in [
        (32, profile.settings.language == Language::Japanese),
        (33, profile.settings.moonwalk),
        (36, profile.settings.weapon_reset),
    ] {
        if value != (read(raw, index) != 0) {
            write(raw, index, u16::from(value));
        }
    }
    write(raw, 34, profile.placement_disabled);
    write(raw, 35, profile.startup_word);
}

fn content(bytes: &[u8]) -> ContentFlags {
    ContentFlags(
        bytes
            .iter()
            .flat_map(|&byte| (0..8).map(move |bit| byte & (1 << bit) != 0))
            .zip(0..=u16::MAX)
            .filter_map(|(set, index)| set.then_some(index))
            .collect(),
    )
}

fn write_content(bytes: &mut [u8], flags: &ContentFlags) {
    bytes.fill(0);
    for &index in &flags.0 {
        if let Some(byte) = bytes.get_mut(usize::from(index / 8)) {
            *byte |= 1 << (index % 8);
        }
    }
}

fn decode_progress(raw: &[u8], item_count: u16, packed: &[u8]) -> Progress {
    let mut events = Set::default();
    for number in 0..64 {
        if let Some(event) = data::room::event(number) {
            events[event] = read(raw, usize::from(number / 8)) & (1 << (number % 8)) != 0;
        }
    }
    let mut bosses = EnumMap::default();
    let mut maps_collected = Set::default();
    let mut stations = Vec::new();
    let mut elevators = Vec::new();
    for (index, area) in AREAS.into_iter().enumerate() {
        bosses[area] = bits(u16::from(read(raw, 8_usize.wrapping_add(index))), &BOSSES);
        maps_collected[area] = read(raw, 232_usize.wrapping_add(index)) != 0;
        stations.push(read(raw, 216_usize.wrapping_add(index.wrapping_mul(2))));
        elevators.push(read(raw, 217_usize.wrapping_add(index.wrapping_mul(2))));
    }
    Progress {
        events,
        bosses,
        item_count,
        items: content(raw.get(80..144).unwrap_or_default()),
        changed_locations: content(raw.get(16..80).unwrap_or_default()),
        doors: content(raw.get(144..208).unwrap_or_default()),
        stations: content(&stations),
        elevators: content(&elevators),
        maps_collected,
        pass_through: RetainedWords(raw.iter().chain(packed).copied().map(u16::from).collect()),
    }
}

fn encode_progress(progress: &Progress, raw: &mut [u8]) {
    for (out, &value) in raw.iter_mut().zip(&progress.pass_through.0) {
        *out = value.to_le_bytes()[0];
    }
    for number in 0..64 {
        if let Some(event) = data::room::event(number)
            && let Some(byte) = raw.get_mut(usize::from(number / 8))
        {
            let mask = 1 << (number % 8);
            *byte = (*byte & !mask) | if progress.events[event] { mask } else { 0 };
        }
    }
    let mut stations = [0; AREAS.len()];
    let mut elevators = [0; AREAS.len()];
    write_content(&mut stations, &progress.stations);
    write_content(&mut elevators, &progress.elevators);
    for ((index, area), (station, elevator)) in AREAS
        .into_iter()
        .enumerate()
        .zip(stations.into_iter().zip(elevators))
    {
        let boss = 8_usize.wrapping_add(index);
        write(
            raw,
            boss,
            overlay(u16::from(read(raw, boss)), &BOSSES, &progress.bosses[area]).to_le_bytes()[0],
        );
        if let Some(map) = raw.get_mut(232_usize.wrapping_add(index))
            && progress.maps_collected[area] != (*map != 0)
        {
            *map = if progress.maps_collected[area] {
                u8::MAX
            } else {
                0
            };
        }
        let slot = index.wrapping_mul(2);
        write(raw, 216_usize.wrapping_add(slot), station);
        write(raw, 217_usize.wrapping_add(slot), elevator);
    }
    if let Some(bytes) = raw.get_mut(16..80) {
        write_content(bytes, &progress.changed_locations);
    }
    if let Some(bytes) = raw.get_mut(80..144) {
        write_content(bytes, &progress.items);
    }
    if let Some(bytes) = raw.get_mut(144..208) {
        write_content(bytes, &progress.doors);
    }
}

impl Format {
    pub(super) fn decode_run(&self, payload: &EncodedRun) -> RunSnapshot {
        let words = payload.as_slice();
        let bytes: Vec<_> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        let mut maps = blank_maps();
        let mut packed = bytes.get(MAP).unwrap_or_default().iter().copied();
        for (area, indices) in AREAS.into_iter().zip(&self.projection) {
            for (&index, value) in indices.iter().zip(&mut packed) {
                write(maps[area].as_mut_slice(), usize::from(index), value);
            }
        }
        RunSnapshot {
            profile: decode_profile(words.get(..PROFILE).unwrap_or_default()),
            progress: decode_progress(
                bytes.get(PROGRESS).unwrap_or_default(),
                read(words, 173),
                bytes.get(MAP).unwrap_or_default(),
            ),
            maps,
            restart: RestartPoint {
                destination: match read(words, 170) {
                    0 => Destination::Intro,
                    0x1f => Destination::CeresArrival,
                    0x22 => Destination::ZebesFlight,
                    value => Destination::Unrecognized(OpaqueValue::Word(value)),
                },
                area: AREAS
                    .get(usize::from(read(words, 172)))
                    .copied()
                    .map_or_else(
                        || StoredArea::Unrecognized(OpaqueValue::Word(read(words, 172))),
                        StoredArea::Known,
                    ),
                station: StationId(read(words, 171)),
            },
            tail: RetainedWords(
                bytes
                    .get(TAIL..)
                    .unwrap_or_default()
                    .chunks(2)
                    .map(|bytes| u16::from_le_bytes([read(bytes, 0), read(bytes, 1)]))
                    .collect(),
            ),
        }
    }

    pub(super) fn encode_run(&self, run: &RunSnapshot) -> EncodedRun {
        let mut payload = EncodedRun::zeroed();
        let words = payload.as_mut_slice();
        encode_profile(&run.profile, words.get_mut(..PROFILE).unwrap_or_default());
        let destination = match run.restart.destination {
            Destination::Intro => 0,
            Destination::CeresArrival => 0x1f,
            Destination::ZebesFlight => 0x22,
            Destination::Unrecognized(value) => word(value),
        };
        write(words, 170, destination);
        write(words, 171, run.restart.station.0);
        let area = match run.restart.area {
            StoredArea::Known(area) => u16::try_from(
                AREAS
                    .iter()
                    .position(|&entry| entry == area)
                    .unwrap_or_default(),
            )
            .unwrap_or_default(),
            StoredArea::Unrecognized(value) => word(value),
        };
        write(words, 172, area);
        write(words, 173, run.progress.item_count);
        let mut bytes: Vec<_> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        encode_progress(&run.progress, bytes.get_mut(PROGRESS).unwrap_or_default());
        let mut old = run
            .progress
            .pass_through
            .0
            .iter()
            .skip(PROGRESS.len())
            .copied()
            .chain(std::iter::repeat(0))
            .map(|value| value.to_le_bytes()[0]);
        let mut packed = bytes.get_mut(MAP).unwrap_or_default().iter_mut();
        for (area, indices) in AREAS.into_iter().zip(&self.projection) {
            let mut original = [0; types::save_files::MAP_BYTES];
            let values: Vec<_> = old.by_ref().take(indices.len()).collect();
            for (&index, &value) in indices.iter().zip(&values) {
                write(&mut original, usize::from(index), value);
            }
            for ((&index, previous), out) in indices.iter().zip(values).zip(&mut packed) {
                let index = usize::from(index);
                let current = read(run.maps[area].as_slice(), index);
                *out = if current == read(&original, index) {
                    previous
                } else {
                    current
                };
            }
        }
        for (out, value) in bytes
            .get_mut(TAIL..)
            .unwrap_or_default()
            .iter_mut()
            .zip(run.tail.0.iter().flat_map(|word| word.to_le_bytes()))
        {
            *out = value;
        }
        for (out, &pair) in words.iter_mut().zip(bytes.as_chunks::<2>().0) {
            *out = u16::from_le_bytes(pair);
        }
        payload
    }
}

/// Clear assigned format values that have no typed gameplay owner during fresh initialization.
pub fn fresh_words(profile: &mut PlayerProfile) {
    let assigned = [
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 0),
        (4, 0x0800),
        (5, 0x0400),
        (6, 0x0200),
        (7, 0x0100),
        (15, 0),
        (27, 0),
        (28, 0),
        (29, 0),
        (30, 0),
        (31, 0),
    ];
    for (index, value) in assigned {
        if let Some(word) = profile.retained.0.get_mut(index) {
            *word = value;
        }
    }
}

/// Read a working profile in the same representation the cartridge stores.
#[must_use]
pub fn profile_from_bytes(bytes: &[u8; PROFILE * 2]) -> PlayerProfile {
    let words: Vec<_> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .copied()
        .map(u16::from_le_bytes)
        .collect();
    decode_profile(&words)
}

/// Read the persistent progress region and item count a save will hold.
#[must_use]
pub fn progress_from_bytes(bytes: &[u8; PROGRESS_BYTES], item_count: u16) -> Progress {
    decode_progress(bytes, item_count, &[])
}

/// The working profile's projection to the platform's live memory.
#[must_use]
pub fn profile_bytes(profile: &PlayerProfile) -> [u8; PROFILE * 2] {
    let mut words = [0; PROFILE];
    encode_profile(profile, &mut words);
    let mut bytes = [0; PROFILE * 2];
    for (out, word) in bytes.as_chunks_mut::<2>().0.iter_mut().zip(words) {
        *out = word.to_le_bytes();
    }
    bytes
}
