//! Ordinary door placement and the temporary mechanism that delivers a shot.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Grey,
    Colored,
    Blue,
    ShotSignal,
    MotherBrainEscape,
    CeresEscape,
}

/// Field accesses retain the caller's captured slot. Block identities need not
/// be encoded room offsets; the boundary supplies the two distinct terrain views.
pub trait Context {
    type Block: Copy + Eq;
    fn block(&self, slot: usize) -> Self::Block;
    fn below(&self, block: Self::Block) -> Self::Block;
    fn clear_block(&mut self, slot: usize);
    fn release(&mut self, slot: usize);
    fn argument_high(&self, slot: usize) -> u8;
    fn argument(&self, slot: usize) -> u16;
    fn set_argument(&mut self, slot: usize, value: u16);
    fn set_grey_selector(&mut self, slot: usize, value: u16);
    fn set_signal(&mut self, slot: usize, value: u16);
    fn projectile_selection(&self) -> u16;
    fn projectile_word(&self, index: isize) -> u16;
    fn level_word(&self, block: Self::Block) -> u16;
    fn set_level_word(&mut self, block: Self::Block, value: u16);
    fn level_high(&self, block: Self::Block) -> u8;
    fn set_level_high(&mut self, block: Self::Block, value: u8);
    fn set_behavior(&mut self, block: Self::Block, value: u8);
}

fn install<C: Context>(c: &mut C, block: C::Block, kind: u16) {
    let high = c.level_high(block) & 15 | (kind >> 8) as u8;
    c.set_level_high(block, high);
    c.set_behavior(block, kind as u8);
}

fn make_solid<C: Context>(c: &mut C, block: C::Block) {
    let level = c.level_word(block) & 0xfff | 0x8000;
    c.set_level_word(block, level);
}

pub fn setup(c: &mut impl Context, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::Grey | Setup::Colored => {
            if kind == Setup::Grey {
                let selector = u16::from(c.argument_high(slot) & 0x7c) >> 1;
                c.set_grey_selector(slot, selector);
                let argument = c.argument(slot) & !0x7c00;
                c.set_argument(slot, argument);
            }
            let block = c.block(slot);
            install(c, block, 0xc044);
        }
        Setup::MotherBrainEscape => {
            let mut block = c.block(slot);
            install(c, block, 0x9001);
            for _ in 0..3 {
                block = c.below(block);
                install(c, block, 0xd0ff);
            }
        }
        // The original middle stride adds two consecutive immutable width reads.
        // Each transition still observes width after the preceding terrain write.
        Setup::CeresEscape => {
            let mut block = c.block(slot);
            for row in 0..4 {
                make_solid(c, block);
                if row != 3 {
                    block = c.below(block);
                }
            }
        }
        Setup::Blue => {
            let selected = c.projectile_selection();
            if selected & 0x8000 == 0
                && c.projectile_word((selected >> 1) as isize) & 0xf00 == 0x300
            {
                c.release(slot);
            } else {
                let block = c.block(slot);
                make_solid(c, block);
            }
        }
        Setup::ShotSignal => {
            let block = c.block(slot);
            c.clear_block(slot);
            if let Some(target) = (0..40).rev().find(|&i| c.block(i) == block) {
                let selected = c.projectile_selection() as i16;
                let signal = c.projectile_word((selected >> 1) as isize) & 0x1fff | 0x8000;
                c.set_signal(target, signal);
            }
        }
    }
    0
}
