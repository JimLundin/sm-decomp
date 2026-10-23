//! Select and publish a Metroid room's persistent population-clear event.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Room {
    First,
    Second,
    Third,
    Fourth,
}
pub trait Context {
    type Callback;
    fn selected_callback(&self, slot: usize) -> Self::Callback;
    fn watch(&mut self, slot: usize, callback: Self::Callback);
    fn deaths(&self) -> u16;
    fn required(&self) -> u16;
    fn cleared(&mut self, room: Room);
}
pub fn begin(c: &mut impl Context, slot: usize) -> u8 {
    let callback = c.selected_callback(slot);
    c.watch(slot, callback);
    0
}
pub fn poll(c: &mut impl Context, room: Room) {
    if c.deaths() >= c.required() {
        c.cleared(room);
    }
}
