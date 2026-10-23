//! Mother Brain glass hit counting, threshold progression and ordered shard bursts.
#![forbid(unsafe_code)]
pub trait Context {
    type Block;
    fn argument(&self, slot: usize) -> u16;
    fn set_argument(&mut self, slot: usize, value: u16);
    fn signal(&self, slot: usize) -> u16;
    fn clear_signal(&mut self, slot: usize);
    fn block(&self, slot: usize) -> Self::Block;
    fn install(&mut self, block: Self::Block);
}
pub fn setup(c: &mut impl Context, slot: usize) -> u8 {
    c.set_argument(slot, 0);
    let block = c.block(slot);
    c.install(block);
    0
}
pub fn add_hits(c: &mut impl Context, slot: usize, amount: u16) {
    let hits = c.argument(slot).wrapping_add(amount);
    c.set_argument(slot, hits);
}
pub fn poll(c: &mut impl Context, slot: usize) {
    if matches!(c.signal(slot) & 0xf00, 0x100 | 0x200) {
        add_hits(c, slot, 1);
    }
    c.clear_signal(slot);
}
#[derive(Clone, Copy)]
pub enum Command {
    Threshold,
    Shatter,
}
pub trait Program {
    type Position;
    fn argument(&self, slot: usize) -> u16;
    fn word(&self, offset: isize) -> u16;
    fn jump(&self, address: u16) -> Self::Position;
    fn advance(&self, bytes: isize) -> Self::Position;
    fn sound(&mut self);
    fn shard(&mut self, parameter: u16);
}
pub fn command<P: Program>(c: &mut P, slot: usize, kind: Command) -> P::Position {
    match kind {
        Command::Threshold => {
            let hits = c.argument(slot);
            if hits >= c.word(0) {
                c.advance(4)
            } else {
                c.jump(c.word(2))
            }
        }
        Command::Shatter => {
            c.sound();
            for offset in [0, 2, 4, 6] {
                let parameter = c.word(offset);
                c.shard(parameter);
            }
            c.advance(8)
        }
    }
}
