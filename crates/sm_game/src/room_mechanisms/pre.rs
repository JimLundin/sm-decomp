//! Exact pre-instruction identities over shared Session state.
use super::access::*;
use super::addresses;
use super::creation;
use super::host::*;
use super::rules;
use crate::progress_flags as flags;

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
    let index = slot;
    let value = mechanism!(c, index, program).wrapping_add(1);
    mechanism!(c, index, program = value);
    mechanism!(c, slot, instruction_delay = 1);
    mechanism!(c, slot, pre_action = pre);
}
pub fn run(c: &mut impl Host, ea: u32, k: u16) {
    let slot = usize::from(k >> 1);
    match ea {
        addresses::pre::TOURIAN_SHAFT_WALL_EXPLODE | addresses::pre::TOURIAN_ESCAPE_ACID_RISE => {
            let target = if ea == addresses::pre::TOURIAN_SHAFT_WALL_EXPLODE {
                2080
            } else {
                1344
            };
            if 240 < c.session().player.x && target < c.session().player.y {
                let index = slot;
                let value = mechanism!(c, index, program).wrapping_add(2);
                mechanism!(c, index, program = value);
                mechanism!(c, slot, instruction_delay = 1);
                if ea == addresses::pre::TOURIAN_SHAFT_WALL_EXPLODE {
                    // The spawn's initialisation parameter is A, still the 1
                    // just stored as the instruction delay.
                    c.enemy_projectile(0xb4b1, 1);
                } else {
                    c.session_mut().room.liquid_velocity = (-104i16) as u16;
                    c.session_mut().room.liquid_delay = 16;
                }
            }
        }
        addresses::pre::WAKE_IF_TRIGGERED_OR_SAMUS_BELOW
        | addresses::pre::WAKE_IF_TRIGGERED_OR_SAMUS_ABOVE => {
            creation::coordinates(c, slot);
            let delta = (c.session().player.y >> 4).wrapping_sub(c.session().mechanisms.block_y);
            if ((c.session().player.x >> 4) == c.session().mechanisms.block_x
                && if ea == addresses::pre::WAKE_IF_TRIGGERED_OR_SAMUS_BELOW {
                    delta < 5
                } else {
                    delta >= 0xfffc
                })
                || mechanism!(c, slot, counter) != 0
            {
                wake(
                    c,
                    slot,
                    if ea == addresses::pre::WAKE_IF_TRIGGERED_OR_SAMUS_BELOW {
                        0xbba3
                    } else {
                        0xbbdc
                    },
                );
            }
        }
        addresses::pre::LINK_IF_MISSILED => {
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
        addresses::pre::LINK_IF_AREA_BOSS_DEAD
        | addresses::pre::LINK_IF_AREA_MINI_BOSS_DEAD
        | addresses::pre::LINK_IF_AREA_TORIZO_DEAD => {
            let mask = if ea == addresses::pre::LINK_IF_AREA_BOSS_DEAD {
                1
            } else if ea == addresses::pre::LINK_IF_AREA_MINI_BOSS_DEAD {
                2
            } else {
                4
            };
            if flags::boss_defeated(c, mask) {
                link(c, slot);
            } else {
                dud(c, slot);
            }
        }
        addresses::pre::LINK_IF_ENEMY_QUOTA_MET => {
            if c.session().room.enemy_deaths < c.session().room.enemy_quota {
                dud(c, slot);
            } else {
                flags::set_event(c, 0);
                link(c, slot);
            }
        }
        addresses::pre::LINK_IF_TOURIAN_STATUE_DONE => {
            if c.session().mechanisms.statue_finished & 0x8000 == 0 {
                dud(c, slot);
            } else {
                link(c, slot);
            }
        }
        addresses::pre::SHOT_REACTION_DB64 => {
            match mechanism!(c, slot, counter) & 0xf00 {
                0x200 => mechanism!(c, slot, room_argument = 119),
                0x100 => {}
                _ => return,
            }
            link(c, slot);
        }
        addresses::pre::LINK_IF_SHOT => {
            if mechanism!(c, slot, counter) != 0 {
                link(c, slot);
            }
        }
        addresses::pre::LINK_IF_POWER_BOMBED | addresses::pre::LINK_IF_SUPER_MISSILED => {
            let shot = mechanism!(c, slot, counter);
            if shot != 0 {
                if shot & 0xf00
                    == if ea == addresses::pre::LINK_IF_POWER_BOMBED {
                        0x300
                    } else {
                        0x200
                    }
                {
                    link(c, slot);
                    return;
                }
                c.sound(SoundChannel::Two, 6, 0x57);
            }
            mechanism!(c, slot, counter = 0);
        }
        addresses::pre::LINK => link(c, slot),
        addresses::pre::DUD_IF_SHOT => dud(c, slot),
        addresses::pre::WAKE_IF_TRIGGERED => {
            if mechanism!(c, slot, counter) != 0 {
                wake(c, slot, 0xbb6a);
            }
        }
        addresses::pre::LINK_IF_TRIGGERED => {
            if mechanism_byte!(c, slot, counter, 0) == 255 {
                mechanism!(c, slot, pre_action = 0xdfa8);
                mechanism!(c, slot, program = mechanism!(c, slot, return_program));
                mechanism!(c, slot, instruction_delay = 1);
            }
        }
        addresses::pre::WAKE_IF_TRIGGERED_DFE6 => {
            let shot = mechanism!(c, slot, counter);
            if shot != 768 && shot as u8 == 255 {
                mechanism!(c, slot, counter = 0);
                {
                    let index = slot;
                    let value = mechanism!(c, index, program).wrapping_add(1);
                    mechanism!(c, index, program = value);
                };
                let index = slot;
                let value = mechanism!(c, index, program).wrapping_add(1);
                mechanism!(c, index, program = value);
                mechanism!(c, slot, instruction_delay = 1);
            }
        }
        addresses::pre::WAKE_ON_BUTTON => rules::noob_tube::poll(c, slot),
        addresses::pre::WAKE_IF_SAMUS_HAS_BOMBS => rules::torizo_statue::poll(c, slot),
        addresses::pre::NORFAIR_HAND_TRIGGER => rules::hands::monitor_norfair(c),
        addresses::pre::MOTHER_BRAIN_GLASS_SHOT => rules::mother_brain_glass::poll(c, slot),
        addresses::pre::PLANT_HOLD_SAMUS => rules::plants::hold(c, slot),
        addresses::pre::WAKE_IF_DOOR_SET => rules::eye_door::wake(c, slot),
        addresses::pre::METROIDS_CLEARED_12 => {
            rules::metroid_quota::poll(c, rules::metroid_quota::Room::First)
        }
        addresses::pre::METROIDS_CLEARED_14 => {
            rules::metroid_quota::poll(c, rules::metroid_quota::Room::Second)
        }
        addresses::pre::METROIDS_CLEARED_16 => {
            rules::metroid_quota::poll(c, rules::metroid_quota::Room::Third)
        }
        addresses::pre::METROIDS_CLEARED_18 => {
            rules::metroid_quota::poll(c, rules::metroid_quota::Room::Fourth)
        }
        addresses::pre::WAKE_ON_SPEED_BOOSTER => {
            rules::booster_escape::poll(c, rules::booster_escape::Stage::BoosterCollected, slot)
        }
        addresses::pre::WAKE_WHEN_SAMUS_FAR_LEFT => {
            rules::booster_escape::poll(c, rules::booster_escape::Stage::PlayerFarLeft, slot)
        }
        addresses::pre::ADVANCE_LAVA => {
            rules::booster_escape::poll(c, rules::booster_escape::Stage::AdvanceLava, slot)
        }
        addresses::pre::LINK_IF_CRITTERS_ESCAPED => rules::animal_rescue::poll_rescue_gate(c, slot),
        addresses::pre::SHAKTOOL_ROOM => rules::shaktool::poll(c, slot),
        addresses::pre::SOUND_DELAY => rules::sound_delay::poll(c, slot),
        addresses::pre::PLANT_BEGIN_FLOOR => {
            rules::plants::begin(c, slot, rules::plants::Surface::Floor);
        }
        addresses::pre::PLANT_BEGIN_CEILING => {
            rules::plants::begin(c, slot, rules::plants::Surface::Ceiling);
        }
        addresses::pre::NONE_8469
        | addresses::pre::NONE_DFA8
        | addresses::pre::NONE_86D0
        | addresses::pre::NONE_84E6
        | addresses::pre::NONE_853D
        | addresses::pre::NONE_8AA6
        | addresses::pre::NONE_BBA3
        | addresses::pre::NONE_D356
        | addresses::pre::NONE_BAFA
        | addresses::pre::NONE_BB6A
        | addresses::pre::NONE_DAD5
        | addresses::pre::NONE_DAD6
        | addresses::pre::NONE_DAD7
        | addresses::pre::NONE_DAD8
        | addresses::pre::NONE_DAD9
        | addresses::pre::NONE_DADA
        | addresses::pre::NONE_DADB
        | addresses::pre::NONE_DADC
        | addresses::pre::NONE_DADD
        | addresses::pre::NONE_8AE0 => {}
        _ => {
            c.diagnostic(Diagnostic::Unreachable);
        }
    }
}
