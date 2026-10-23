//! Select and publish a Metroid room's persistent population-clear event.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;
use crate::game::progress_flags as flags;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Room {
    First,
    Second,
    Third,
    Fourth,
}

pub fn begin(c: &mut impl Host, slot: usize) -> u8 {
    let callback = c.table_word(
        ContentTable::MetroidWatcher,
        (mechanism!(c, slot, room_argument) >> 1) as isize,
    );
    mechanism!(c, slot, pre_action = callback);
    0
}
pub fn poll(c: &mut impl Host, room: Room) {
    if c.session().room.enemy_deaths >= c.session().room.enemy_quota {
        flags::set_event(
            c,
            match room {
                Room::First => 0x10,
                Room::Second => 0x11,
                Room::Third => 0x12,
                Room::Fourth => 0x13,
            },
        );
    }
}
