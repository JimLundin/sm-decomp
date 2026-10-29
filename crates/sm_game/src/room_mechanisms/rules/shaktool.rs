//! Shaktool room: open the camera path during a Power Bomb and mark its exit.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;
use crate::progress as flags;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollPair {
    Near,
    Far,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    EntranceOnly,
    Closed,
    Open,
}

pub fn begin(c: &mut impl Host) -> u8 {
    set_scroll_pair(c, ScrollPair::Near, Access::EntranceOnly);
    set_scroll_pair(c, ScrollPair::Far, Access::Closed);
    0
}
pub fn poll(c: &mut impl Host, slot: usize) {
    if c.inputs().power_bomb_active != 0 {
        set_scroll_pair(c, ScrollPair::Near, Access::Open);
        set_scroll_pair(c, ScrollPair::Far, Access::Open);
    }
    if c.session().player.x > 0x348 {
        flags::set_event(c, 0xd);
        mechanism!(c, slot, header = 0);
    }
}

fn set_scroll_pair(c: &mut impl Host, pair: ScrollPair, access: Access) {
    c.scroll_pair(
        match pair {
            ScrollPair::Near => 0,
            ScrollPair::Far => 2,
        },
        match access {
            Access::EntranceOnly => 1,
            Access::Closed => 0,
            Access::Open => 257,
        },
    );
}
