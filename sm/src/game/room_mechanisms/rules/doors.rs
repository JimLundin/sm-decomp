//! Ordinary door placement and the temporary mechanism that delivers a shot.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;
use super::terrain_setup as terrain;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Grey,
    Colored,
    Blue,
    ShotSignal,
    MotherBrainEscape,
    CeresEscape,
}

fn make_solid<C: Host>(c: &mut C, block: u16) {
    let level = terrain::level(c, block) & 0xfff | 0x8000;
    terrain::set_level(c, block, level);
}

pub fn setup(c: &mut impl Host, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::Grey | Setup::Colored => {
            if kind == Setup::Grey {
                let selector = u16::from(mechanism_byte!(c, slot, room_argument, 1) & 0x7c) >> 1;
                mechanism!(c, slot, work0 = selector);
                let argument = mechanism!(c, slot, room_argument) & !0x7c00;
                mechanism!(c, slot, room_argument = argument);
            }
            let block = mechanism!(c, slot, block);
            terrain::install(c, block, 0xc044);
        }
        Setup::MotherBrainEscape => {
            let mut block = mechanism!(c, slot, block);
            terrain::install(c, block, 0x9001);
            for _ in 0..3 {
                block = (block).wrapping_add(c.session().room.width.wrapping_mul(2));
                terrain::install(c, block, 0xd0ff);
            }
        }
        // The original middle stride adds two consecutive immutable width reads.
        // Each transition still observes width after the preceding terrain write.
        Setup::CeresEscape => {
            let mut block = mechanism!(c, slot, block);
            for row in 0..4 {
                make_solid(c, block);
                if row != 3 {
                    block = (block).wrapping_add(c.session().room.width.wrapping_mul(2));
                }
            }
        }
        Setup::Blue => {
            let selected = c.session().projectiles.selected;
            if selected & 0x8000 == 0
                && projectile_kind(c, (selected >> 1) as isize, ByteOrder::Native) & 0xf00 == 0x300
            {
                mechanism!(c, slot, header = 0);
            } else {
                let block = mechanism!(c, slot, block);
                make_solid(c, block);
            }
        }
        Setup::ShotSignal => {
            let block = mechanism!(c, slot, block);
            mechanism!(c, slot, block = 0);
            if let Some(target) = (0..40).rev().find(|&i| mechanism!(c, i, block) == block) {
                let selected = c.session().projectiles.selected as i16;
                let signal = projectile_kind(c, (selected >> 1) as isize, ByteOrder::Native)
                    & 0x1fff
                    | 0x8000;
                mechanism!(c, target, counter = signal);
            }
        }
    }
    0
}
