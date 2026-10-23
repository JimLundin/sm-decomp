//! Bomb Torizo statue release and the actor waiting for its removal.
#![forbid(unsafe_code)]

pub trait Plm {
    fn defeated(&mut self) -> bool;
    fn release(&mut self, slot: usize);
    fn bombs_collected(&self) -> bool;
    fn set_timer(&mut self, slot: usize, value: u16);
    fn instruction(&self, slot: usize) -> u16;
    fn set_instruction(&mut self, slot: usize, value: u16);
    fn stop_polling(&mut self, slot: usize);
}
pub fn setup(c: &mut impl Plm, slot: usize) -> u8 {
    if c.defeated() {
        c.release(slot);
    }
    0
}
pub fn poll(c: &mut impl Plm, slot: usize) {
    if c.bombs_collected() {
        c.set_timer(slot, 1);
        let instruction = c.instruction(slot).wrapping_add(2);
        c.set_instruction(slot, instruction);
        c.stop_polling(slot);
    }
}

pub trait Actor {
    fn set_tangible(&mut self, value: bool);
    fn statue_present(&self, slot: usize) -> bool;
    fn music(&mut self);
    fn instruction(&self) -> u16;
    fn set_instruction(&mut self, value: u16);
    fn set_timer(&mut self, value: u16);
}
pub fn wait(c: &mut impl Actor) {
    c.set_tangible(true);
    for slot in (0..40).rev() {
        if c.statue_present(slot) {
            return;
        }
    }
    c.music();
    c.set_tangible(false);
    let instruction = c.instruction().wrapping_add(2);
    c.set_instruction(instruction);
    c.set_timer(1);
}
