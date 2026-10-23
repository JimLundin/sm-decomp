//! Exact pre-instruction identities, owned slot transitions and live external effects.
#![forbid(unsafe_code)]
use super::{
    fields::{Fields, Global, SoundChannel},
    port::Execution,
    rules,
    state::Column,
};
impl Execution<'_> {
    fn link(&mut self, slot: usize) {
        self.set_slot(Column::Counter, slot, 0);
        self.set_slot(
            Column::Program,
            slot,
            self.slot(Column::ReturnProgram, slot),
        );
        self.set_slot(Column::InstructionDelay, slot, 1);
    }
    fn dud(&mut self, slot: usize) {
        if self.slot(Column::Counter, slot) != 0 {
            self.sound(SoundChannel::Two, 6, 0x57);
        }
        self.set_slot(Column::Counter, slot, 0);
    }
    fn wake(&mut self, slot: usize, pre: u16) {
        self.advance(slot, 1);
        self.advance(slot, 1);
        self.set_slot(Column::InstructionDelay, slot, 1);
        self.set_slot(Column::PreAction, slot, pre);
    }
    pub(super) fn pre(&mut self, ea: u32, k: u16) {
        let slot = usize::from(k >> 1);
        match ea {
            8698151 | 8698184 => {
                let target = if ea == 8698151 { 2080 } else { 1344 };
                if 240 < self.global(Global::PlayerX) && target < self.global(Global::PlayerY) {
                    self.advance(slot, 2);
                    self.set_slot(Column::InstructionDelay, slot, 1);
                    if ea == 8698151 {
                        self.eproj(0xb4b1, 0);
                    } else {
                        self.set_global(Global::LiquidVelocity, (-104i16) as u16);
                        self.set_global(Global::LiquidDelay, 16);
                    }
                }
            }
            8698731 | 8698788 => {
                self.coordinates(slot);
                let delta =
                    (self.global(Global::PlayerY) >> 4).wrapping_sub(self.global(Global::BlockY));
                if ((self.global(Global::PlayerX) >> 4) == self.global(Global::BlockX)
                    && if ea == 8698731 {
                        delta < 5
                    } else {
                        delta >= 0xfffc
                    })
                    || self.slot(Column::Counter, slot) != 0
                {
                    self.wake(slot, if ea == 8698731 { 0xbba3 } else { 0xbbdc });
                }
            }
            8699216 => {
                let shot = self.slot(Column::Counter, slot);
                if shot != 0 {
                    match shot & 0xf00 {
                        0x200 => self.set_slot(Column::Work1, slot, 119),
                        0x100 => {}
                        _ => {
                            self.sound(SoundChannel::Two, 6, 0x57);
                            self.set_slot(Column::Counter, slot, 0);
                            return;
                        }
                    }
                    self.link(slot);
                    return;
                }
                self.set_slot(Column::Counter, slot, 0);
            }
            8699348 | 8699363 | 8699378 => {
                let mask = if ea == 8699348 {
                    1
                } else if ea == 8699363 {
                    2
                } else {
                    4
                };
                let boss = self.boss(mask);
                if boss & 1 != 0 {
                    self.link(slot);
                } else {
                    self.dud(slot);
                }
            }
            8699393 => {
                if self.global(Global::EnemyDeaths) < self.global(Global::EnemyQuota) {
                    self.dud(slot);
                } else {
                    self.mark_event(0);
                    self.link(slot);
                }
            }
            8699423 => {
                if self
                    .port()
                    .word(crate::ram_map::words::tourian_entrance_statue_finished)
                    & 0x8000
                    == 0
                {
                    self.dud(slot);
                } else {
                    self.link(slot);
                }
            }
            8706916 => {
                match self.slot(Column::Counter, slot) & 0xf00 {
                    0x200 => self.set_slot(Column::RoomArgument, slot, 119),
                    0x100 => {}
                    _ => return,
                }
                self.link(slot);
            }
            8699151 => {
                if self.slot(Column::Counter, slot) != 0 {
                    self.link(slot);
                }
            }
            8699174 | 8699272 => {
                let shot = self.slot(Column::Counter, slot);
                if shot != 0 {
                    if shot & 0xf00 == if ea == 8699174 { 0x300 } else { 0x200 } {
                        self.link(slot);
                        return;
                    }
                    self.sound(SoundChannel::Two, 6, 0x57);
                }
                self.set_slot(Column::Counter, slot, 0);
            }
            8699314 => self.link(slot),
            8699420 => self.dud(slot),
            8698706 => {
                if self.slot(Column::Counter, slot) != 0 {
                    self.wake(slot, 0xbb6a);
                }
            }
            8707977 => {
                if self.slot_byte(Column::Counter, slot, 0) == 255 {
                    self.set_slot(Column::PreAction, slot, 0xdfa8);
                    self.set_slot(
                        Column::Program,
                        slot,
                        self.slot(Column::ReturnProgram, slot),
                    );
                    self.set_slot(Column::InstructionDelay, slot, 1);
                }
            }
            8708070 => {
                let shot = self.slot(Column::Counter, slot);
                if shot != 768 && shot as u8 == 255 {
                    self.set_slot(Column::Counter, slot, 0);
                    self.advance(slot, 1);
                    self.advance(slot, 1);
                    self.set_slot(Column::InstructionDelay, slot, 1);
                }
            }
            8705215 => rules::noob_tube::poll(self, slot),
            8704827 => rules::torizo_statue::poll(self, slot),
            8704348 => rules::hands::monitor_norfair(self),
            8704486 => rules::mother_brain_glass::poll(self, slot),
            8694921 => rules::plants::hold(self, slot),
            8705875 => rules::eye_door::wake(self, slot),
            8706782 => rules::metroid_quota::poll(self, rules::metroid_quota::Room::First),
            8706798 => rules::metroid_quota::poll(self, rules::metroid_quota::Room::Second),
            8706814 => rules::metroid_quota::poll(self, rules::metroid_quota::Room::Third),
            8706830 => rules::metroid_quota::poll(self, rules::metroid_quota::Room::Fourth),
            8697839 => rules::booster_escape::poll(
                self,
                rules::booster_escape::Stage::BoosterCollected,
                slot,
            ),
            8697898 => {
                rules::booster_escape::poll(self, rules::booster_escape::Stage::PlayerFarLeft, slot)
            }
            8697926 => {
                rules::booster_escape::poll(self, rules::booster_escape::Stage::AdvanceLava, slot)
            }
            8699440 => rules::animal_rescue::poll_rescue_gate(self, slot),
            8698032 => rules::shaktool::poll(self, slot),
            8697821 => rules::sound_delay::poll(self, slot),
            8696028 => {
                rules::plants::begin(self, slot, rules::plants::Surface::Floor);
            }
            8696083 => {
                rules::plants::begin(self, slot, rules::plants::Surface::Ceiling);
            }
            8684649 | 8708008 | 8685264 | 8684774 | 8684861 | 8686246 | 8698787 | 8704854
            | 8698618 | 8698730 | 8706773 | 8706774 | 8706775 | 8706776 | 8706777 | 8706778
            | 8706779 | 8706780 | 8706781 | 8686304 => {}
            _ => {
                self.foreign(super::effects::unreachable);
            }
        }
    }
}
