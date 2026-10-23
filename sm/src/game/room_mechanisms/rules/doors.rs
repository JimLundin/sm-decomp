//! Ordinary door placement and the temporary mechanism that delivers a shot.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global},
    state::Column,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Grey,
    Colored,
    Blue,
    ShotSignal,
    MotherBrainEscape,
    CeresEscape,
}

fn make_solid<C: Fields>(c: &mut C, block: u16) {
    let level = c.level(block) & 0xfff | 0x8000;
    c.set_level(block, level);
}

pub fn setup(c: &mut impl Fields, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::Grey | Setup::Colored => {
            if kind == Setup::Grey {
                let selector = u16::from(c.slot_byte(Column::RoomArgument, slot, 1) & 0x7c) >> 1;
                c.set_slot(Column::Work0, slot, selector);
                let argument = c.slot(Column::RoomArgument, slot) & !0x7c00;
                c.set_slot(Column::RoomArgument, slot, argument);
            }
            let block = c.slot(Column::Block, slot);
            c.install(block, 0xc044);
        }
        Setup::MotherBrainEscape => {
            let mut block = c.slot(Column::Block, slot);
            c.install(block, 0x9001);
            for _ in 0..3 {
                block = c.below(block);
                c.install(block, 0xd0ff);
            }
        }
        // The original middle stride adds two consecutive immutable width reads.
        // Each transition still observes width after the preceding terrain write.
        Setup::CeresEscape => {
            let mut block = c.slot(Column::Block, slot);
            for row in 0..4 {
                make_solid(c, block);
                if row != 3 {
                    block = c.below(block);
                }
            }
        }
        Setup::Blue => {
            let selected = c.global(Global::ProjectileSelection);
            if selected & 0x8000 == 0
                && c.projectile_word((selected >> 1) as isize) & 0xf00 == 0x300
            {
                c.release(slot);
            } else {
                let block = c.slot(Column::Block, slot);
                make_solid(c, block);
            }
        }
        Setup::ShotSignal => {
            let block = c.slot(Column::Block, slot);
            c.set_slot(Column::Block, slot, 0);
            if let Some(target) = (0..40).rev().find(|&i| c.slot(Column::Block, i) == block) {
                let selected = c.global(Global::ProjectileSelection) as i16;
                let signal = c.projectile_word((selected >> 1) as isize) & 0x1fff | 0x8000;
                c.set_slot(Column::Counter, target, signal);
            }
        }
    }
    0
}
