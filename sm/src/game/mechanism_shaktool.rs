//! Shaktool room: open the camera path during a Power Bomb and mark its exit.
#![forbid(unsafe_code)]
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
pub trait Context {
    fn set_scroll_pair(&mut self, pair: ScrollPair, access: Access);
    fn power_bomb_active(&self) -> bool;
    fn player_x(&self) -> u16;
    fn mark_path_open(&mut self);
    fn release(&mut self, slot: usize);
}
pub fn begin(c: &mut impl Context) -> u8 {
    c.set_scroll_pair(ScrollPair::Near, Access::EntranceOnly);
    c.set_scroll_pair(ScrollPair::Far, Access::Closed);
    0
}
pub fn poll(c: &mut impl Context, slot: usize) {
    if c.power_bomb_active() {
        c.set_scroll_pair(ScrollPair::Near, Access::Open);
        c.set_scroll_pair(ScrollPair::Far, Access::Open);
    }
    if c.player_x() > 0x348 {
        c.mark_path_open();
        c.release(slot);
    }
}
