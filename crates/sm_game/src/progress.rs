//! Persistent progression flags: events, defeated bosses per area, used room
//! objects, collected items and opened doors, with the original scratch-mask
//! protocol. Indices past an owned collection reach the host's legacy aliases.
use crate::gameplay_flag_state::{FlagCompatibility, GameplayFlags};

pub mod exploration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlagColumn {
    Events,
    Bosses,
    RoomObjects,
    Items,
    Doors,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossWordOrder {
    Little,
    Native,
}

/// The finite compatibility seam for indices outside the owned collections.
pub trait FlagAliases {
    /// Widened signed displacement from a finite collection base. Never clamp.
    fn flag_byte(&self, column: FlagColumn, index: isize) -> u8;
    fn set_flag_byte(&mut self, column: FlagColumn, index: isize, value: u8);
    /// Includes the byte after the owned door collection when index is 63.
    fn door_adjacent_word_le(&self, byte_index: isize) -> u16;
    /// Source-specific reads: native typed dereference vs defined unaligned LE.
    fn boss_word(&self, area: u16, order: BossWordOrder) -> u16;
    /// update_place!(unaligned_word) uses LE reads AND LE writes.
    fn set_boss_word_le(&mut self, area: u16, value: u16);
    /// Kraid's actual typed-pointer store remains native.
    fn set_boss_word_native(&mut self, area: u16, value: u16);
}

/// The Session state the flag rules read and write.
pub trait FlagHost: FlagAliases {
    fn flags(&self) -> &GameplayFlags;
    fn flags_mut(&mut self) -> &mut GameplayFlags;
    fn compatibility(&self) -> &FlagCompatibility;
    fn compatibility_mut(&mut self) -> &mut FlagCompatibility;
    /// The current room's area, which selects the boss byte.
    fn area(&self) -> u16;
}

fn collection(flags: &GameplayFlags, column: FlagColumn) -> &[u8] {
    match column {
        FlagColumn::Events => &flags.events,
        FlagColumn::Bosses => &flags.bosses_by_area,
        FlagColumn::RoomObjects => &flags.used_room_objects,
        FlagColumn::Items => &flags.collected_locations,
        FlagColumn::Doors => &flags.opened_doors,
    }
}
fn collection_mut(flags: &mut GameplayFlags, column: FlagColumn) -> &mut [u8] {
    match column {
        FlagColumn::Events => &mut flags.events,
        FlagColumn::Bosses => &mut flags.bosses_by_area,
        FlagColumn::RoomObjects => &mut flags.used_room_objects,
        FlagColumn::Items => &mut flags.collected_locations,
        FlagColumn::Doors => &mut flags.opened_doors,
    }
}
fn read(host: &(impl FlagHost + ?Sized), column: FlagColumn, index: u16) -> u8 {
    collection(host.flags(), column)
        .get(usize::from(index))
        .copied()
        .unwrap_or_else(|| host.flag_byte(column, index as isize))
}
fn write(host: &mut (impl FlagHost + ?Sized), column: FlagColumn, index: u16, value: u8) {
    match collection_mut(host.flags_mut(), column).get_mut(usize::from(index)) {
        Some(byte) => *byte = value,
        None => host.set_flag_byte(column, index as isize, value),
    }
}
/// Store the bit's mask in the scratch word and return its byte index.
pub fn prepare_flag_mask(compatibility: &mut FlagCompatibility, bit: u16) -> u16 {
    compatibility.bitmask = 1u16 << (bit & 7);
    bit >> 3
}
fn present(host: &mut (impl FlagHost + ?Sized), column: FlagColumn, bit: u16) -> bool {
    let index = prepare_flag_mask(host.compatibility_mut(), bit);
    read(host, column, index) & host.compatibility().bitmask as u8 != 0
}
fn mark(host: &mut (impl FlagHost + ?Sized), column: FlagColumn, bit: u16) -> u16 {
    let index = prepare_flag_mask(host.compatibility_mut(), bit);
    let value = read(host, column, index) | host.compatibility().bitmask as u8;
    write(host, column, index, value);
    index
}
pub fn event_happened(host: &mut (impl FlagHost + ?Sized), event: u16) -> bool {
    present(host, FlagColumn::Events, event)
}
pub fn set_event(host: &mut (impl FlagHost + ?Sized), event: u16) {
    mark(host, FlagColumn::Events, event);
}
pub fn clear_event(host: &mut (impl FlagHost + ?Sized), event: u16) {
    let index = prepare_flag_mask(host.compatibility_mut(), event);
    let value = read(host, FlagColumn::Events, index) & !(host.compatibility().bitmask as u8);
    write(host, FlagColumn::Events, index, value);
}
fn boss_mask(compatibility: &mut FlagCompatibility, mask: u16) -> u8 {
    let low = mask as u8;
    compatibility.bitmask = (compatibility.bitmask & 0xff00) | u16::from(low);
    low
}
pub fn boss_defeated(host: &mut (impl FlagHost + ?Sized), mask: u16) -> bool {
    let mask = boss_mask(host.compatibility_mut(), mask);
    read(host, FlagColumn::Bosses, host.area()) & mask != 0
}
pub fn mark_boss_defeated(host: &mut (impl FlagHost + ?Sized), mask: u16) {
    let mask = boss_mask(host.compatibility_mut(), mask);
    let area = host.area();
    let value = read(host, FlagColumn::Bosses, area) | mask;
    write(host, FlagColumn::Bosses, area, value);
}
pub fn door_opened(host: &mut (impl FlagHost + ?Sized), door: u16) -> bool {
    present(host, FlagColumn::Doors, door)
}
pub fn item_collected(host: &mut (impl FlagHost + ?Sized), item: u16) -> bool {
    present(host, FlagColumn::Items, item)
}
pub fn collect_item(host: &mut (impl FlagHost + ?Sized), item: u16) {
    mark(host, FlagColumn::Items, item);
}
pub fn room_object_used(host: &mut (impl FlagHost + ?Sized), object: u16) -> bool {
    present(host, FlagColumn::RoomObjects, object)
}
pub fn use_room_object(host: &mut (impl FlagHost + ?Sized), object: u16) {
    mark(host, FlagColumn::RoomObjects, object);
}
pub fn open_door_and_read_adjacent_flags(host: &mut (impl FlagHost + ?Sized), door: u16) -> u16 {
    let index = mark(host, FlagColumn::Doors, door);
    let bytes = &host.flags().opened_doors;
    match bytes.get(usize::from(index)..usize::from(index) + 2) {
        Some(pair) => u16::from_le_bytes([pair[0], pair[1]]),
        None => host.door_adjacent_word_le(index as isize),
    }
}
// Raw boss operations intentionally do not prepare or mutate scratch.
pub fn boss_byte_at(host: &(impl FlagHost + ?Sized), area: u16) -> u8 {
    read(host, FlagColumn::Bosses, area)
}
pub fn set_boss_byte_at(host: &mut (impl FlagHost + ?Sized), area: u16, value: u8) {
    write(host, FlagColumn::Bosses, area, value);
}
pub fn boss_word_at(host: &(impl FlagHost + ?Sized), area: u16, order: BossWordOrder) -> u16 {
    let bytes = &host.flags().bosses_by_area;
    match bytes.get(usize::from(area)..usize::from(area) + 2) {
        Some(pair) => match order {
            BossWordOrder::Little => u16::from_le_bytes([pair[0], pair[1]]),
            BossWordOrder::Native => u16::from_ne_bytes([pair[0], pair[1]]),
        },
        None => host.boss_word(area, order),
    }
}
fn set_boss_word(host: &mut (impl FlagHost + ?Sized), area: u16, value: u16, order: BossWordOrder) {
    let bytes = &mut host.flags_mut().bosses_by_area;
    match bytes.get_mut(usize::from(area)..usize::from(area) + 2) {
        Some(pair) => pair.copy_from_slice(&match order {
            BossWordOrder::Little => value.to_le_bytes(),
            BossWordOrder::Native => value.to_ne_bytes(),
        }),
        None => match order {
            BossWordOrder::Little => host.set_boss_word_le(area, value),
            BossWordOrder::Native => host.set_boss_word_native(area, value),
        },
    }
}
pub fn set_boss_word_le(host: &mut (impl FlagHost + ?Sized), area: u16, value: u16) {
    set_boss_word(host, area, value, BossWordOrder::Little);
}
pub fn set_boss_word_native(host: &mut (impl FlagHost + ?Sized), area: u16, value: u16) {
    set_boss_word(host, area, value, BossWordOrder::Native);
}
