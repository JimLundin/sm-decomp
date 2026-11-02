//! Original save preparation and checksum order with an explicit storage handoff.
#![forbid(unsafe_code)]
use crate::session_access::SessionAccess;

/// Existing save workspaces have one owner. The live inventory and progress
/// flags are elsewhere; this type holds only their serialized destinations.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub player_words: [u16; 48],
    pub area_maps: Box<[u8; 2048]>,
    pub packed_map: Box<[u8; 1280]>,
    pub unused: [u8; 8],
    pub initialized: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            player_words: [0; 48],
            area_maps: Box::new([0; 2048]),
            packed_map: Box::new([0; 1280]),
            unused: [0; 8],
            initialized: false,
        }
    }
}
impl State {
    pub fn import(read: impl Fn(usize) -> u8) -> Self {
        let mut state = Self::default();
        for (index, word) in state.player_words.iter_mut().enumerate() {
            *word = u16::from_le_bytes([read(0xd7c0 + 2 * index), read(0xd7c1 + 2 * index)]);
        }
        for (index, byte) in state.area_maps.iter_mut().enumerate() {
            *byte = read(0xcd52 + index);
        }
        for (index, byte) in state.packed_map.iter_mut().enumerate() {
            *byte = read(0xd91c + index);
        }
        for (index, byte) in state.unused.iter_mut().enumerate() {
            *byte = read(0xd8f0 + index);
        }
        state.initialized = true;
        state
    }
    pub fn byte(&self, at: usize) -> Option<u8> {
        match at {
            0xd7c0..=0xd81f => {
                let offset = at - 0xd7c0;
                Some(self.player_words[offset / 2].to_le_bytes()[offset & 1])
            }
            0xcd52..=0xd551 => Some(self.area_maps[at - 0xcd52]),
            0xd91c..=0xde1b => Some(self.packed_map[at - 0xd91c]),
            0xd8f0..=0xd8f7 => Some(self.unused[at - 0xd8f0]),
            _ => None,
        }
    }
    pub fn set_byte(&mut self, at: usize, value: u8) -> bool {
        match at {
            0xd7c0..=0xd81f => {
                let offset = at - 0xd7c0;
                let word = &mut self.player_words[offset / 2];
                let mut bytes = word.to_le_bytes();
                bytes[offset & 1] = value;
                *word = u16::from_le_bytes(bytes);
            }
            0xcd52..=0xd551 => self.area_maps[at - 0xcd52] = value,
            0xd91c..=0xde1b => self.packed_map[at - 0xd91c] = value,
            0xd8f0..=0xd8f7 => self.unused[at - 0xd8f0] = value,
            _ => return false,
        }
        true
    }
    pub fn export(&self, mut span: impl FnMut(usize, &[u8])) {
        for (index, word) in self.player_words.iter().enumerate() {
            span(0xd7c0 + 2 * index, &word.to_le_bytes());
        }
        span(0xcd52, self.area_maps.as_ref());
        span(0xd91c, self.packed_map.as_ref());
        span(0xd8f0, &self.unused);
    }
}

/// Each lane denotes one original save interaction; no arbitrary memory access
/// is available to the algorithm. Exceptional selectors retain finite aliases.
#[derive(Clone, Copy)]
pub enum Lane {
    PlayerSource,
    SavedPlayer,
    CurrentMap,
    AreaMap,
    PackedMap,
    Payload,
}
/// A request owns its bytes. Pending means physical storage has not acknowledged
/// durability; reset/load must fence accepted requests outside this core.
pub struct SaveRequest {
    pub slot: u16,
    pub sram: Box<[u8]>,
    pub checksum: u16,
}
pub enum Delivery {
    Pending(SaveRequest),
    Durable,
}
pub trait Host: SessionAccess {
    fn save_byte(&self, lane: Lane, index: usize) -> u8;
    fn set_save_byte(&mut self, lane: Lane, index: usize, value: u8);
    fn save_native_word(&self, lane: Lane, index: usize) -> u16;
    fn save_prefix_word(&mut self, lane: Lane, index: usize, value: u16);
    fn area_map_count(&self, area: usize) -> u8;
    type Map;
    fn area_map_record(&mut self, area: usize) -> Self::Map;
    fn area_map_index(&self, record: &Self::Map, cursor: usize) -> u8;
    fn area_map_bit_offset(&self, area: usize) -> u16;
    fn save_slot_offset(&self, slot: u16) -> u16;
    fn sram_native_word(&self, byte: usize) -> u16;
    fn set_sram_native_word(&mut self, byte: usize, value: u16);
    fn sram_snapshot(&self) -> Box<[u8]>;
    fn deliver_save(&mut self, request: SaveRequest) -> Delivery;
}
pub fn prepare(h: &mut impl Host, slot: u16) -> Delivery {
    assert!(
        h.session().save_state.initialized,
        "save source must be supplied"
    );
    // Both endpoints descend: first source is the word at the end of the
    // player's saved data, first destination is SavedPlayer[47].
    for source in 0..48 {
        let word = h.save_native_word(Lane::PlayerSource, source);
        h.save_prefix_word(Lane::SavedPlayer, 47 - source, word);
    }
    let area_word = h.session().room.area.wrapping_mul(256) >> 1;
    for source in 0..128 {
        let word = h.save_native_word(Lane::CurrentMap, source);
        h.save_prefix_word(Lane::AreaMap, usize::from(area_word) + source, word);
    }
    for area in 0..6 {
        let mut remaining = i32::from(h.area_map_count(area));
        let map = h.area_map_record(area);
        let mut packed = usize::from(h.area_map_bit_offset(area));
        let mut record = 0;
        loop {
            let source = area * 256 + usize::from(h.area_map_index(&map, record));
            let byte = h.save_byte(Lane::AreaMap, source);
            h.set_save_byte(Lane::PackedMap, packed, byte);
            record += 1;
            packed += 1;
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
    }
    let area = h.session().room.area;
    let selection = &mut h.session_mut().menu_screens.save_selection;
    selection.save_station = selection.load_station;
    selection.save_area = area;
    let mut cursor = h.save_slot_offset(slot & 3);
    let mut checksum = 0u16;
    for index in 0..814 {
        let word = h.save_native_word(Lane::Payload, index);
        h.set_sram_native_word(usize::from(cursor & !1), word);
        checksum = checksum.wrapping_add(word);
        cursor = cursor.wrapping_add(2);
    }
    let slot_word = usize::from(slot & 3) * 2;
    for (base, value) in [
        (0, checksum),
        (0x1ff0, checksum),
        (8, !checksum),
        (0x1ff8, !checksum),
    ] {
        h.set_sram_native_word(base + slot_word, value);
    }
    let request = SaveRequest {
        slot,
        sram: h.sram_snapshot(),
        checksum,
    };
    h.deliver_save(request)
}
