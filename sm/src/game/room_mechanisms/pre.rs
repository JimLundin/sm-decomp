//! Exact pre-instruction identities over shared Session state.
#![forbid(unsafe_code)]
use super::access::*;
use super::creation;
use super::host::*;
use super::rules;

fn link(c: &mut impl Host, slot: usize) {
    mechanism!(c, slot, counter = 0);
    mechanism!(c, slot, program = mechanism!(c, slot, return_program));
    mechanism!(c, slot, instruction_delay = 1);
}
fn dud(c: &mut impl Host, slot: usize) {
    if mechanism!(c, slot, counter) != 0 {
        c.sound(SoundChannel::Two, 6, 0x57);
    }
    mechanism!(c, slot, counter = 0);
}
fn wake(c: &mut impl Host, slot: usize, pre: u16) {
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
    mechanism!(c, slot, instruction_delay = 1);
    mechanism!(c, slot, pre_action = pre);
}
pub(super) fn run(c: &mut impl Host, ea: u32, k: u16) {
    let slot = usize::from(k >> 1);
    match ea {
        8698151 | 8698184 => {
            let target = if ea == 8698151 { 2080 } else { 1344 };
            if 240 < c.session().player.x && target < c.session().player.y {
                {
                    let index = slot;
                    let value = mechanism!(c, index, program).wrapping_add(2);
                    mechanism!(c, index, program = value);
                };
                mechanism!(c, slot, instruction_delay = 1);
                if ea == 8698151 {
                    c.enemy_projectile(0xb4b1, 0);
                } else {
                    c.session_mut().room.liquid_velocity = (-104i16) as u16;
                    c.session_mut().room.liquid_delay = 16;
                }
            }
        }
        8698731 | 8698788 => {
            creation::coordinates(c, slot);
            let delta = (c.session().player.y >> 4).wrapping_sub(c.session().mechanisms.block_y);
            if ((c.session().player.x >> 4) == c.session().mechanisms.block_x
                && if ea == 8698731 {
                    delta < 5
                } else {
                    delta >= 0xfffc
                })
                || mechanism!(c, slot, counter) != 0
            {
                wake(c, slot, if ea == 8698731 { 0xbba3 } else { 0xbbdc });
            }
        }
        8699216 => {
            let shot = mechanism!(c, slot, counter);
            if shot != 0 {
                match shot & 0xf00 {
                    0x200 => mechanism!(c, slot, work1 = 119),
                    0x100 => {}
                    _ => {
                        c.sound(SoundChannel::Two, 6, 0x57);
                        mechanism!(c, slot, counter = 0);
                        return;
                    }
                }
                link(c, slot);
                return;
            }
            mechanism!(c, slot, counter = 0);
        }
        8699348 | 8699363 | 8699378 => {
            let mask = if ea == 8699348 {
                1
            } else if ea == 8699363 {
                2
            } else {
                4
            };
            let boss = c.boss(mask);
            if boss & 1 != 0 {
                link(c, slot);
            } else {
                dud(c, slot);
            }
        }
        8699393 => {
            if c.session().room.enemy_deaths < c.session().room.enemy_quota {
                dud(c, slot);
            } else {
                c.mark_event(0);
                link(c, slot);
            }
        }
        8699423 => {
            if c.inputs().tourian_statue_finished & 0x8000 == 0 {
                dud(c, slot);
            } else {
                link(c, slot);
            }
        }
        8706916 => {
            match mechanism!(c, slot, counter) & 0xf00 {
                0x200 => mechanism!(c, slot, room_argument = 119),
                0x100 => {}
                _ => return,
            }
            link(c, slot);
        }
        8699151 => {
            if mechanism!(c, slot, counter) != 0 {
                link(c, slot);
            }
        }
        8699174 | 8699272 => {
            let shot = mechanism!(c, slot, counter);
            if shot != 0 {
                if shot & 0xf00 == if ea == 8699174 { 0x300 } else { 0x200 } {
                    link(c, slot);
                    return;
                }
                c.sound(SoundChannel::Two, 6, 0x57);
            }
            mechanism!(c, slot, counter = 0);
        }
        8699314 => link(c, slot),
        8699420 => dud(c, slot),
        8698706 => {
            if mechanism!(c, slot, counter) != 0 {
                wake(c, slot, 0xbb6a);
            }
        }
        8707977 => {
            if mechanism_byte!(c, slot, counter, 0) == 255 {
                mechanism!(c, slot, pre_action = 0xdfa8);
                mechanism!(c, slot, program = mechanism!(c, slot, return_program));
                mechanism!(c, slot, instruction_delay = 1);
            }
        }
        8708070 => {
            let shot = mechanism!(c, slot, counter);
            if shot != 768 && shot as u8 == 255 {
                mechanism!(c, slot, counter = 0);
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
                mechanism!(c, slot, instruction_delay = 1);
            }
        }
        8705215 => rules::noob_tube::poll(c, slot),
        8704827 => rules::torizo_statue::poll(c, slot),
        8704348 => rules::hands::monitor_norfair(c),
        8704486 => rules::mother_brain_glass::poll(c, slot),
        8694921 => rules::plants::hold(c, slot),
        8705875 => rules::eye_door::wake(c, slot),
        8706782 => rules::metroid_quota::poll(c, rules::metroid_quota::Room::First),
        8706798 => rules::metroid_quota::poll(c, rules::metroid_quota::Room::Second),
        8706814 => rules::metroid_quota::poll(c, rules::metroid_quota::Room::Third),
        8706830 => rules::metroid_quota::poll(c, rules::metroid_quota::Room::Fourth),
        8697839 => {
            rules::booster_escape::poll(c, rules::booster_escape::Stage::BoosterCollected, slot)
        }
        8697898 => {
            rules::booster_escape::poll(c, rules::booster_escape::Stage::PlayerFarLeft, slot)
        }
        8697926 => rules::booster_escape::poll(c, rules::booster_escape::Stage::AdvanceLava, slot),
        8699440 => rules::animal_rescue::poll_rescue_gate(c, slot),
        8698032 => rules::shaktool::poll(c, slot),
        8697821 => rules::sound_delay::poll(c, slot),
        8696028 => {
            rules::plants::begin(c, slot, rules::plants::Surface::Floor);
        }
        8696083 => {
            rules::plants::begin(c, slot, rules::plants::Surface::Ceiling);
        }
        8684649 | 8708008 | 8685264 | 8684774 | 8684861 | 8686246 | 8698787 | 8704854 | 8698618
        | 8698730 | 8706773 | 8706774 | 8706775 | 8706776 | 8706777 | 8706778 | 8706779
        | 8706780 | 8706781 | 8686304 => {}
        _ => {
            c.diagnostic(Diagnostic::Unreachable);
        }
    }
}
