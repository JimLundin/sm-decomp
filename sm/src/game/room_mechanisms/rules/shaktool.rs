//! Shaktool room: open the camera path during a Power Bomb and mark its exit.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global},
    state::Column,
};
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

pub fn begin(c: &mut impl Fields) -> u8 {
    set_scroll_pair(c, ScrollPair::Near, Access::EntranceOnly);
    set_scroll_pair(c, ScrollPair::Far, Access::Closed);
    0
}
pub fn poll(c: &mut impl Fields, slot: usize) {
    if c.global(Global::PowerBombActive) != 0 {
        set_scroll_pair(c, ScrollPair::Near, Access::Open);
        set_scroll_pair(c, ScrollPair::Far, Access::Open);
    }
    if c.global(Global::PlayerX) > 0x348 {
        c.mark_event(0xd);
        c.release(slot);
    }
}

fn set_scroll_pair(c: &mut impl Fields, pair: ScrollPair, access: Access) {
    c.native_scroll_pair(
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
