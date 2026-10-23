//! Delay sound effects after room entry, then retire the gate's mechanism.
#![forbid(unsafe_code)]
pub trait Context {
    fn in_ceres(&self) -> bool;
    fn timer(&self, slot: usize) -> u16;
    fn set_timer(&mut self, slot: usize, timer: u16);
    fn watch(&mut self, slot: usize);
    fn enable_sounds(&mut self);
    fn release(&mut self, slot: usize);
}
pub fn begin(c: &mut impl Context, slot: usize) -> u8 {
    let delay = if c.in_ceres() { 32 } else { 240 };
    c.set_timer(slot, delay);
    c.watch(slot);
    0
}
pub fn poll(c: &mut impl Context, slot: usize) {
    let old = c.timer(slot);
    let remaining = c.timer(slot).wrapping_sub(1);
    c.set_timer(slot, remaining);
    if old == 1 {
        c.enable_sounds();
        c.release(slot);
    }
}
