//! Speed Booster escape: arm the lavaquake, start motion, then follow the player.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;
use crate::progress as flags;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    BoosterCollected,
    PlayerFarLeft,
    AdvanceLava,
}

pub fn begin(c: &mut impl Host, slot: usize) -> u8 {
    if flags::event_happened(c, 0x15) {
        mechanism!(c, slot, header = 0)
    }
    0
}
fn wake(c: &mut impl Host, slot: usize) {
    mechanism!(c, slot, instruction_delay = 1);
    {
        let index = slot;
        let value = mechanism!(c, index, program).wrapping_add(1);
        mechanism!(c, index, program = value);
    };
    {
        let index = slot;
        let value = mechanism!(c, index, program).wrapping_add(1);
        mechanism!(c, index, program = value);
    };
    mechanism!(c, slot, counter = 0);
}
pub fn poll(c: &mut impl Host, stage: Stage, slot: usize) {
    match stage {
        Stage::BoosterCollected => {
            if c.session().inventory.equipment.owned & 0x2000 != 0 {
                if c.session().room.liquid_target & 0x8000 != 0 {
                    mechanism!(c, slot, header = 0)
                } else {
                    c.session_mut().room.liquid_velocity = 0xff80;
                    wake(c, slot)
                }
            } else {
                c.session_mut().room.liquid_target = 0xffff;
                c.session_mut().room.liquid_velocity = 0;
                c.session_mut().room.liquid_delay = 0;
                c.session_mut().feedback.earthquake_timer = 0;
                mechanism!(c, slot, header = 0)
            }
        }
        Stage::PlayerFarLeft => {
            if c.session().player.x <= 0xae0 {
                c.session_mut().room.liquid_delay = 1;
                wake(c, slot)
            }
        }
        Stage::AdvanceLava => {
            // The encoded progress also admits odd and non-row-aligned table windows.
            const STEPS: [u16; 10] = [
                0x72b, 0x1bf, 0xff50, 0x50a, 0x167, 0xff20, 0x244, 0x100, 0xff20, 0x8000,
            ];
            let progress = mechanism!(c, slot, counter);
            let index = usize::from(progress >> 1);
            let threshold = STEPS[index];
            if threshold & 0x8000 != 0 {
                flags::set_event(c, 0x15)
            } else if threshold >= c.session().player.x {
                if STEPS[index + 1] < c.session().room.liquid_height {
                    c.session_mut().room.liquid_height = STEPS[index + 1]
                }
                c.session_mut().room.liquid_velocity = STEPS[index + 2];
                mechanism!(c, slot, counter = progress.wrapping_add(6))
            }
        }
    }
}
