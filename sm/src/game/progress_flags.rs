//! Gameplay flags shared by room, visual, projectile and legacy callers.
#![forbid(unsafe_code)]
use crate::game::{
    gameplay_flag_host::{BossWordOrder, Host, LegacyFlagColumn as Column},
    gameplay_flag_state::GameplayFlags,
    session::Session,
};
fn collection(flags: &GameplayFlags, column: Column) -> &[u8] {
    match column {
        Column::Events => &flags.events,
        Column::Bosses => &flags.bosses_by_area,
        Column::RoomObjects => &flags.used_room_objects,
        Column::Items => &flags.collected_locations,
        Column::Doors => &flags.opened_doors,
    }
}
fn collection_mut(flags: &mut GameplayFlags, column: Column) -> &mut [u8] {
    match column {
        Column::Events => &mut flags.events,
        Column::Bosses => &mut flags.bosses_by_area,
        Column::RoomObjects => &mut flags.used_room_objects,
        Column::Items => &mut flags.collected_locations,
        Column::Doors => &mut flags.opened_doors,
    }
}
fn read(host: &impl Host, column: Column, index: u16) -> u8 {
    collection(&host.session().gameplay_flags, column)
        .get(usize::from(index))
        .copied()
        .unwrap_or_else(|| host.flag_byte(column, index as isize))
}
fn write(host: &mut impl Host, column: Column, index: u16, value: u8) {
    match collection_mut(&mut host.session_mut().gameplay_flags, column).get_mut(usize::from(index))
    {
        Some(byte) => *byte = value,
        None => host.set_flag_byte(column, index as isize, value),
    }
}
pub(crate) fn prepare_flag_mask(session: &mut Session, bit: u16) -> u16 {
    session.flag_compatibility.bitmask = 1u16 << (bit & 7);
    bit >> 3
}
fn present(host: &mut impl Host, column: Column, bit: u16) -> bool {
    let index = prepare_flag_mask(host.session_mut(), bit);
    read(host, column, index) & host.session().flag_compatibility.bitmask as u8 != 0
}
fn mark(host: &mut impl Host, column: Column, bit: u16) -> u16 {
    let index = prepare_flag_mask(host.session_mut(), bit);
    let value = read(host, column, index) | host.session().flag_compatibility.bitmask as u8;
    write(host, column, index, value);
    index
}
pub(crate) fn event_happened(host: &mut impl Host, event: u16) -> bool {
    present(host, Column::Events, event)
}
pub(crate) fn set_event(host: &mut impl Host, event: u16) {
    mark(host, Column::Events, event);
}
pub(crate) fn clear_event(host: &mut impl Host, event: u16) {
    let index = prepare_flag_mask(host.session_mut(), event);
    let value =
        read(host, Column::Events, index) & !(host.session().flag_compatibility.bitmask as u8);
    write(host, Column::Events, index, value);
}
fn boss_mask(session: &mut Session, mask: u16) -> u8 {
    let low = mask as u8;
    session.flag_compatibility.bitmask =
        (session.flag_compatibility.bitmask & 0xff00) | u16::from(low);
    low
}
pub(crate) fn boss_defeated(host: &mut impl Host, mask: u16) -> bool {
    let mask = boss_mask(host.session_mut(), mask);
    read(host, Column::Bosses, host.session().room.area) & mask != 0
}
pub(crate) fn mark_boss_defeated(host: &mut impl Host, mask: u16) {
    let mask = boss_mask(host.session_mut(), mask);
    let area = host.session().room.area;
    let value = read(host, Column::Bosses, area) | mask;
    write(host, Column::Bosses, area, value);
}
pub(crate) fn door_opened(host: &mut impl Host, door: u16) -> bool {
    present(host, Column::Doors, door)
}
pub(crate) fn open_door(host: &mut impl Host, door: u16) {
    mark(host, Column::Doors, door);
}
pub(crate) fn item_collected(host: &mut impl Host, item: u16) -> bool {
    present(host, Column::Items, item)
}
pub(crate) fn collect_item(host: &mut impl Host, item: u16) {
    mark(host, Column::Items, item);
}
pub(crate) fn room_object_used(host: &mut impl Host, object: u16) -> bool {
    present(host, Column::RoomObjects, object)
}
pub(crate) fn use_room_object(host: &mut impl Host, object: u16) {
    mark(host, Column::RoomObjects, object);
}
pub(crate) fn open_door_and_read_adjacent_flags(host: &mut impl Host, door: u16) -> u16 {
    let index = mark(host, Column::Doors, door);
    let bytes = &host.session().gameplay_flags.opened_doors;
    match bytes.get(usize::from(index)..usize::from(index) + 2) {
        Some(pair) => u16::from_le_bytes([pair[0], pair[1]]),
        None => host.door_adjacent_word_le(index as isize),
    }
}
// Raw boss operations intentionally do not prepare or mutate scratch.
pub(crate) fn boss_byte_at(host: &impl Host, area: u16) -> u8 {
    read(host, Column::Bosses, area)
}
pub(crate) fn set_boss_byte_at(host: &mut impl Host, area: u16, value: u8) {
    write(host, Column::Bosses, area, value);
}
pub(crate) fn boss_word_at(host: &impl Host, area: u16, order: BossWordOrder) -> u16 {
    let bytes = &host.session().gameplay_flags.bosses_by_area;
    match bytes.get(usize::from(area)..usize::from(area) + 2) {
        Some(pair) => match order {
            BossWordOrder::Little => u16::from_le_bytes([pair[0], pair[1]]),
            BossWordOrder::Native => u16::from_ne_bytes([pair[0], pair[1]]),
        },
        None => host.boss_word(area, order),
    }
}
fn set_boss_word(host: &mut impl Host, area: u16, value: u16, order: BossWordOrder) {
    let bytes = &mut host.session_mut().gameplay_flags.bosses_by_area;
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
pub(crate) fn set_boss_word_le(host: &mut impl Host, area: u16, value: u16) {
    set_boss_word(host, area, value, BossWordOrder::Little);
}
pub(crate) fn set_boss_word_native(host: &mut impl Host, area: u16, value: u16) {
    set_boss_word(host, area, value, BossWordOrder::Native);
}
