//! Speed Booster escape: arm the lavaquake, start motion, then follow the player.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global},
    state::Column,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    BoosterCollected,
    PlayerFarLeft,
    AdvanceLava,
}

pub fn begin(c: &mut impl Fields, slot: usize) -> u8 {
    if c.event(0x15) {
        c.release(slot)
    }
    0
}
fn wake(c: &mut impl Fields, slot: usize) {
    c.set_slot(Column::InstructionDelay, slot, 1);
    c.advance(slot, 1);
    c.advance(slot, 1);
    c.set_slot(Column::Counter, slot, 0);
}
pub fn poll(c: &mut impl Fields, stage: Stage, slot: usize) {
    match stage {
        Stage::BoosterCollected => {
            if c.global(Global::CollectedItems) & 0x2000 != 0 {
                if c.global(Global::LiquidTarget) & 0x8000 != 0 {
                    c.release(slot)
                } else {
                    c.set_global(Global::LiquidVelocity, 0xff80);
                    wake(c, slot)
                }
            } else {
                c.set_global(Global::LiquidTarget, 0xffff);
                c.set_global(Global::LiquidVelocity, 0);
                c.set_global(Global::LiquidDelay, 0);
                c.set_global(Global::QuakeTimer, 0);
                c.release(slot)
            }
        }
        Stage::PlayerFarLeft => {
            if c.global(Global::PlayerX) <= 0xae0 {
                c.set_global(Global::LiquidDelay, 1);
                wake(c, slot)
            }
        }
        Stage::AdvanceLava => {
            // The encoded progress also admits odd and non-row-aligned table windows.
            const STEPS: [u16; 10] = [
                0x72b, 0x1bf, 0xff50, 0x50a, 0x167, 0xff20, 0x244, 0x100, 0xff20, 0x8000,
            ];
            let progress = c.slot(Column::Counter, slot);
            let index = usize::from(progress >> 1);
            let threshold = STEPS[index];
            if threshold & 0x8000 != 0 {
                c.mark_event(0x15)
            } else if threshold >= c.global(Global::PlayerX) {
                if STEPS[index + 1] < c.global(Global::LiquidHeight) {
                    c.set_global(Global::LiquidHeight, STEPS[index + 1])
                }
                c.set_global(Global::LiquidVelocity, STEPS[index + 2]);
                c.set_slot(Column::Counter, slot, progress.wrapping_add(6))
            }
        }
    }
}
