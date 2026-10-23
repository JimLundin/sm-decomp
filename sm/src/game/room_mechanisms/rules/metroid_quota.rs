//! Select and publish a Metroid room's persistent population-clear event.
#![forbid(unsafe_code)]
use super::super::{
    fields::{ContentTable, Fields, Global},
    state::Column,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Room {
    First,
    Second,
    Third,
    Fourth,
}

pub fn begin(c: &mut impl Fields, slot: usize) -> u8 {
    let callback = c.content_word(
        ContentTable::MetroidWatcher,
        (c.slot(Column::RoomArgument, slot) >> 1) as isize,
    );
    c.set_slot(Column::PreAction, slot, callback);
    0
}
pub fn poll(c: &mut impl Fields, room: Room) {
    if c.global(Global::EnemyDeaths) >= c.global(Global::EnemyQuota) {
        c.mark_event(match room {
            Room::First => 0x10,
            Room::Second => 0x11,
            Room::Third => 0x12,
            Room::Fourth => 0x13,
        });
    }
}
