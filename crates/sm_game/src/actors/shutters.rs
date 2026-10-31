//! Shutters and up/down movers (bank $A2:E978-$A2:F497): the growing shutter, the up/down
//! movers (shootable and destroyable shutters, the up/down mover platform) and the horizontal
//! shutter. Was `moving_platforms`. Each frame runs on one actor lease.
use crate::actors::api::{self, Axis, ShotHandling};
use crate::actors::combat::{self, current_slot};
use crate::actors::host::{ActorLease, LeaseActors};
use crate::actors::layouts::part_view;
use crate::enemy_projectiles::host::Effects as _;
use crate::player_position::PlayerPosition as _;
use crate::player_projectiles::content::table;
use crate::ram::words as W;
use crate::session_access::SessionAccess as _;
use crate::session_slot;
use sm_data::enemy_speeds::SpeedTable::Linear;

part_view! {
    /// An up/down mover's or horizontal shutter's slot. "Positive" is down or right.
    Mover: EnemyLayout72 {
        words: [function, function_timer, positive_subvelocity, positive_velocity, negative_subvelocity,
            negative_velocity, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [speed_index, primary_direction, negative_rest_parameter, positive_rest_parameter, trigger_mode,
            distance, proximity, initial_function_index, negative_rest_time, positive_rest_time, moving_samus,
            var_0b, triggered, previous_x, previous_y, minimum_y, maximum_y, minimum_x, maximum_x, pushing_samus,
            var_14, samus_x, samus_x_fraction, var_17, var_18, var_19, var_1a, var_1b, var_1c, var_1d, var_1e,
            var_1f],
        pad2: [u8; 1984],
        words: [reaction_direction, var_21, var_22, var_23, var_24, var_25, var_26, var_27, var_28, var_29,
            var_2a, var_2b, var_2c, var_2d, var_2e, var_2f, var_30, var_31, var_32, var_33, var_34, var_35, var_36,
            var_37, var_38, var_39, var_3a, var_3b, var_3c, var_3d, var_3e, var_3f],
    }
}

part_view! {
    /// The growing shutter's slot: the Y position of each growth level, the speed, and the Y
    /// position at the start of the frame.
    Growing: EnemyLayout72Late {
        words: [function, level_0_y, level_1_y, level_2_y, level_3_y, growth_level, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [speed, subspeed, var_02, var_03, var_04, var_05, var_06, var_07, var_08, var_09, var_0a, var_0b,
            var_0c, var_0d, var_0e, var_0f, var_10, var_11, var_12, var_13, var_14, var_15, var_16, var_17, var_18,
            var_19, var_1a, var_1b, var_1c, var_1d, var_1e, var_1f],
        pad3: [u8; 4032],
        words: [previous_y, var_41, var_42, var_43, var_44, var_45, var_46, var_47, var_48, var_49, var_4a,
            var_4b, var_4c, var_4d, var_4e, var_4f, var_50, var_51, var_52, var_53, var_54, var_55, var_56, var_57,
            var_58, var_59, var_5a, var_5b, var_5c, var_5d, var_5e, var_5f],
    }
}

/// Which enemy: the growing shutter ($D4FF), the up/down mover platform ($D5FF), the shootable
/// or destroyable shutter ($D53F, $D5BF) and the horizontal shutter ($D57F).
#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    GrowingShutter,
    Platform,
    VerticalShutter,
    HorizontalShutter,
}
/// The touch, shot and power bomb reactions.
#[derive(Clone, Copy)]
pub enum Event {
    Touch,
    Shot,
    PowerBomb,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Initial,
    WaitForSamusToGetNear,
    MovingNegative,
    MovingPositive,
    StoppedNegative,
    StoppedPositive,
    Nothing,
}
use Phase::*;

/// An axis's functions, by phase.
struct Motion {
    horizontal: bool,
    functions: [u16; 7],
}
const VERTICAL: Motion = Motion {
    horizontal: false,
    functions: [0xef09, 0xef28, 0xef68, 0xefd4, 0xf040, 0xf072, 0xf099],
};
const HORIZONTAL: Motion = Motion {
    horizontal: true,
    functions: [0xf224, 0xf243, 0xf272, 0xf2e4, 0xf38c, 0xf3b0, 0xf3d4],
};
impl Motion {
    fn function(&self, phase: Phase) -> u16 {
        self.functions[phase as usize]
    }
    fn phase(&self, function: u16) -> Option<Phase> {
        [
            Initial,
            WaitForSamusToGetNear,
            MovingNegative,
            MovingPositive,
            StoppedNegative,
            StoppedPositive,
            Nothing,
        ]
        .into_iter()
        .find(|&phase| self.function(phase) == function)
    }
}
impl Kind {
    fn motion(self) -> &'static Motion {
        if self == Self::HorizontalShutter {
            &HORIZONTAL
        } else {
            &VERTICAL
        }
    }
}

/// Play gate opening/closing sound effect if on-screen ($A2:EF5A).
fn movement_sound(runtime: &mut impl ActorLease) {
    if !api::current_outside_screen(runtime) {
        runtime.queue_sound(3, 6, 0xe);
    }
}

fn near(runtime: &mut impl ActorLease, distance: u16) -> bool {
    let key = combat::current(runtime);
    api::player_within(runtime, key, Axis::X, distance)
}

/// Initialisation AI ($A2:E9DA, $A2:EE05, $A2:EE12, $A2:F111).
pub fn initialize<H: LeaseActors + ?Sized>(kind: Kind, game: &mut H) {
    let runtime = &mut game.lease();
    let e = current_slot(runtime);
    if kind == Kind::GrowingShutter {
        let index = session_slot!(runtime, Growing[e].base.current_instruction) as i32
            + 2 * session_slot!(runtime, Growing[e].base.extra_properties) as i32;
        // $A2:EA4E: the initial functions.
        let function = table(runtime, 0xa2_ea4e, index as isize);
        session_slot!(runtime, Growing[e].function = function);
        let step = if session_slot!(runtime, Growing[e].base.extra_properties) != 0 {
            -8i16 as u16
        } else {
            8
        };
        let y = session_slot!(runtime, Growing[e].base.y_pos);
        let levels = [
            y,
            y.wrapping_add(step),
            y.wrapping_add(step).wrapping_add(step),
            y.wrapping_add(step.wrapping_mul(3)),
        ];
        session_slot!(runtime, Growing[e]; level_0_y = levels[0], level_1_y = levels[1], level_2_y = levels[2],
            level_3_y = levels[3]);
        session_slot!(runtime, Growing[e].base; extra_properties = 0, current_instruction = 0xe998);
        session_slot!(runtime, Growing[e].growth_level = 0);
        let index = 2 * isize::from(session_slot!(runtime, Growing[e].parameter_2) as u8);
        let mut timer =
            |index| runtime.foreign(|game| crate::gameplay_tables::shutter_timer(game, index));
        let (speed, subspeed) = (timer(index), timer(index + 1));
        session_slot!(runtime, Growing[e]; speed = speed, subspeed = subspeed);
        return;
    }
    let p = kind.motion();
    let instruction = session_slot!(runtime, Mover[e].base.current_instruction);
    let index = 4 * isize::from(instruction as u8);
    let [positive, positive_sub, negative, negative_sub] =
        [0, 1, 2, 3].map(|offset| runtime.lease_enemy_speed(Linear, index + offset));
    let [rest_negative, rest_positive] =
        session_slot!(runtime, Mover[e].base.extra_properties).to_le_bytes();
    let [trigger, distance] = session_slot!(runtime, Mover[e].parameter_1).to_le_bytes();
    let proximity = session_slot!(runtime, Mover[e].parameter_2);
    session_slot!(runtime, Mover[e]; speed_index = u16::from(instruction as u8), positive_velocity = positive,
        positive_subvelocity = positive_sub, negative_velocity = negative, negative_subvelocity = negative_sub,
        primary_direction = instruction >> 8, reaction_direction = (instruction >> 8) ^ u16::from(p.horizontal),
        negative_rest_parameter = u16::from(rest_negative), negative_rest_time = 16 * u16::from(rest_negative),
        positive_rest_parameter = u16::from(rest_positive), positive_rest_time = 16 * u16::from(rest_positive),
        trigger_mode = u16::from(trigger), initial_function_index = 2 * u16::from(trigger),
        distance = u16::from(distance), proximity = proximity, function_timer = proximity);
    let (position, upward) = if p.horizontal {
        (
            session_slot!(runtime, Mover[e].base.x_pos),
            session_slot!(runtime, Mover[e].primary_direction) == 0,
        )
    } else {
        (
            session_slot!(runtime, Mover[e].base.y_pos),
            session_slot!(runtime, Mover[e].primary_direction) == 0,
        )
    };
    let (low, high) = if upward {
        (position.wrapping_sub(u16::from(distance)), position)
    } else {
        (position, u16::from(distance).wrapping_add(position))
    };
    if p.horizontal {
        session_slot!(runtime, Mover[e]; minimum_x = low, maximum_x = high);
    } else {
        session_slot!(runtime, Mover[e]; minimum_y = low, maximum_y = high);
    }
    session_slot!(runtime, Mover[e].function = p.function(Initial));
    if p.horizontal {
        let x = runtime.player_x();
        session_slot!(runtime, Mover[e].samus_x = x);
    }
    session_slot!(runtime, Mover[e].base.extra_properties = 0);
    session_slot!(runtime, Mover[e].moving_samus = 0);
    if p.horizontal {
        session_slot!(runtime, Mover[e].var_0b = 0);
    }
    let list = match kind {
        Kind::Platform => 0xede7,
        Kind::VerticalShutter => 0xe9aa,
        _ => 0xe9d4,
    };
    session_slot!(runtime, Mover[e].base.current_instruction = list);
}

/// Main AI ($A2:EAB6, $A2:EED1, $A2:F1DE).
pub fn main_ai<H: LeaseActors + ?Sized>(kind: Kind, game: &mut H) {
    let runtime = &mut game.lease();
    if kind == Kind::GrowingShutter {
        return grow(runtime);
    }
    let e = current_slot(runtime);
    // Both original entry points dispatch both axes' functions.
    let function = session_slot!(runtime, Mover[e].function);
    let p = if VERTICAL.functions.contains(&function) {
        &VERTICAL
    } else {
        &HORIZONTAL
    };
    let Some(phase) = p.phase(function) else {
        return runtime.unreachable();
    };
    match phase {
        Initial | WaitForSamusToGetNear => {
            let trigger = if phase == WaitForSamusToGetNear {
                1
            } else {
                session_slot!(runtime, Mover[e].initial_function_index) >> 1
            };
            let ready = match trigger {
                0 => {
                    let timer = session_slot!(runtime, Mover[e].function_timer);
                    session_slot!(runtime, Mover[e].function_timer = timer.wrapping_sub(1));
                    if timer == 1 {
                        let reload = session_slot!(runtime, Mover[e].proximity);
                        session_slot!(runtime, Mover[e].function_timer = reload);
                    }
                    timer == 1
                }
                1 => {
                    let distance = session_slot!(runtime, Mover[e].proximity);
                    near(runtime, distance)
                }
                2 => true,
                3 | 4 => false,
                _ => panic!("mechanism trigger index out of bounds"),
            };
            if ready {
                let positive = session_slot!(runtime, Mover[e].primary_direction) != 0;
                let phase = if positive {
                    MovingPositive
                } else {
                    MovingNegative
                };
                session_slot!(runtime, Mover[e].function = p.function(phase));
                if !p.horizontal {
                    movement_sound(runtime);
                }
            }
        }
        MovingNegative | MovingPositive => travel(runtime, p, phase == MovingPositive),
        StoppedNegative | StoppedPositive => {
            let timer = session_slot!(runtime, Mover[e].function_timer).wrapping_sub(1);
            session_slot!(runtime, Mover[e].function_timer = timer);
            if (timer & 0x8000) != 0 {
                if !p.horizontal && phase == StoppedPositive {
                    movement_sound(runtime);
                }
                let next = if phase == StoppedNegative {
                    MovingPositive
                } else {
                    MovingNegative
                };
                session_slot!(runtime, Mover[e].function = p.function(next));
                if session_slot!(runtime, Mover[e].trigger_mode) == 1
                    && (session_slot!(runtime, Mover[e].primary_direction) != 0)
                        == (phase == StoppedNegative)
                {
                    session_slot!(
                        runtime,
                        Mover[e].function = p.function(WaitForSamusToGetNear)
                    );
                }
                if !p.horizontal
                    && phase == StoppedNegative
                    && session_slot!(runtime, Mover[e].base.enemy_ptr) != 0xd83f
                {
                    movement_sound(runtime);
                }
            }
        }
        Nothing => {}
    }
    let p = kind.motion();
    // The enemy Samus collides with as a solid enemy, in all four directions.
    let collision = [3, 2, 1, 0]
        .map(|direction| {
            runtime.word(crate::ram::word_arrays::ENEMY_INDEX_COLLIDING_DIRS + 2 * direction)
        })
        .into_iter()
        .fold(0xffff, |all, index| all & index);
    let function = session_slot!(runtime, Mover[e].function);
    if function != p.function(MovingNegative)
        && function != p.function(MovingPositive)
        && collision != 0xffff
        && collision == runtime.word(W::CUR_ENEMY_INDEX)
        && runtime.session().player.contact.contact_attack != 0
    {
        activate(runtime, p);
    }
    if p.horizontal {
        let (x, fraction) = (runtime.player_x(), runtime.player_x_fraction());
        session_slot!(runtime, Mover[e]; samus_x = x, samus_x_fraction = fraction);
    }
}

/// Up/down mover function - moving up and down ($A2:EF68, $A2:EFD4), horizontal shutter
/// function - moving left and right ($A2:F272, $A2:F2E4): carry Samus, stop at the end.
fn travel(runtime: &mut impl ActorLease, p: &Motion, positive: bool) {
    let e = current_slot(runtime);
    let speed = if positive {
        session_slot!(runtime, Mover[e](positive_velocity, positive_subvelocity))
    } else {
        session_slot!(runtime, Mover[e](negative_velocity, negative_subvelocity))
    };
    let key = combat::current(runtime);
    let (position, limit) = if p.horizontal {
        let x = session_slot!(runtime, Mover[e].base.x_pos);
        session_slot!(runtime, Mover[e]; previous_x = x, pushing_samus = 0);
        let samus_side = (runtime.player_x() as i16).wrapping_sub(x as i16) >= 0;
        if api::overlaps_player_with_padding(runtime, key) && samus_side == positive {
            session_slot!(runtime, Mover[e].pushing_samus = 1);
        }
        session_slot!(runtime, Mover[e].base(x_pos, x_subpos) += speed);
        if session_slot!(runtime, Mover[e].pushing_samus) != 0 {
            let contact = &mut runtime.session_mut().player.contact;
            (contact.external_x.fraction, contact.external_x.whole) =
                (speed as u16, (speed >> 16) as u16);
            eject_samus(runtime, positive);
        }
        let limit = if positive {
            session_slot!(runtime, Mover[e].maximum_x)
        } else {
            session_slot!(runtime, Mover[e].minimum_x)
        };
        (session_slot!(runtime, Mover[e].base.x_pos), limit)
    } else {
        let y = session_slot!(runtime, Mover[e].base.y_pos);
        session_slot!(runtime, Mover[e]; previous_y = y, moving_samus = 0);
        if api::touches_player_from_above(runtime, key) {
            session_slot!(runtime, Mover[e].moving_samus = 1);
        }
        session_slot!(runtime, Mover[e].base(y_pos, y_subpos) += speed);
        if session_slot!(runtime, Mover[e].moving_samus) != 0 {
            let moved = session_slot!(runtime, Mover[e].base.y_pos)
                .wrapping_sub(session_slot!(runtime, Mover[e].previous_y));
            runtime.session_mut().player.contact.external_y.whole = moved;
        }
        let limit = if positive {
            session_slot!(runtime, Mover[e].maximum_y)
        } else {
            session_slot!(runtime, Mover[e].minimum_y)
        };
        (session_slot!(runtime, Mover[e].base.y_pos), limit)
    };
    let past = if positive {
        position.wrapping_sub(limit)
    } else {
        limit.wrapping_sub(position)
    };
    if (past as i16) >= 0 {
        let rest = if positive {
            session_slot!(runtime, Mover[e].positive_rest_time)
        } else {
            session_slot!(runtime, Mover[e].negative_rest_time)
        };
        if rest == 4080 {
            session_slot!(runtime, Mover[e].function = p.function(Nothing));
        } else {
            let phase = if positive {
                StoppedPositive
            } else {
                StoppedNegative
            };
            session_slot!(runtime, Mover[e]; function_timer = rest, function = p.function(phase));
        }
    }
}

/// Eject Samus if pressing right or left ($A2:F356, $A2:F371).
fn eject_samus(runtime: &mut impl ActorLease, right: bool) {
    let button = if right { 0x200 } else { 0x100 };
    if (runtime.word(W::JOYPAD1_LASTKEYS) & button) != 0 {
        let contact = &mut runtime.session_mut().player.contact;
        contact.external_x.whole =
            contact
                .external_x
                .whole
                .wrapping_add(if right { 4 } else { -4i16 as u16 });
        contact.external_y.fraction = 0;
    }
}

/// Activate up/down mover / horizontal shutter ($A2:EF44, $A2:F25F) and the mover reactions.
fn activate(runtime: &mut impl ActorLease, p: &Motion) {
    let e = current_slot(runtime);
    if (session_slot!(runtime, Mover[e].initial_function_index).wrapping_sub(6) & 0x8000) == 0 {
        if session_slot!(runtime, Mover[e].initial_function_index) != 8 {
            if session_slot!(runtime, Mover[e].triggered) != 0 {
                return;
            }
            session_slot!(runtime, Mover[e].triggered = 1);
        }
        let function = session_slot!(runtime, Mover[e].function);
        if function == p.function(MovingNegative) || function == p.function(MovingPositive) {
            return;
        }
        if p.horizontal {
            session_slot!(runtime, Mover[e].reaction_direction, |old| old ^ 1);
        }
        let positive = session_slot!(runtime, Mover[e].reaction_direction) != 0;
        let phase = if positive {
            MovingPositive
        } else {
            MovingNegative
        };
        session_slot!(runtime, Mover[e].function = p.function(phase));
        if !p.horizontal {
            session_slot!(runtime, Mover[e].reaction_direction, |old| old ^ 1);
        }
    }
    if !p.horizontal {
        movement_sound(runtime);
    }
}

/// Enemy touch, shot and power bomb reactions ($A2:F09D-$A2:F0B6, $A2:F3D8-$A2:F41A).
pub fn reaction<H: LeaseActors + ?Sized>(kind: Kind, event: Event, game: &mut H) {
    if let Event::Shot = event {
        match kind {
            Kind::VerticalShutter => {
                combat::shot_reaction(&mut game.lease(), ShotHandling::Finalize)
            }
            Kind::HorizontalShutter => {
                combat::shot_reaction(&mut game.lease(), ShotHandling::ReactionOnly)
            }
            _ => {}
        }
    }
    let runtime = &mut game.lease();
    if kind == Kind::HorizontalShutter && matches!(event, Event::Touch) {
        let e = current_slot(runtime);
        if session_slot!(runtime, Mover[e].function) == HORIZONTAL.function(Nothing) {
            let right = (runtime.player_x() as i16)
                .wrapping_sub(session_slot!(runtime, Mover[e].base.x_pos) as i16)
                >= 0;
            let button = if right { 0x200 } else { 0x100 };
            if (runtime.word(W::JOYPAD1_LASTKEYS) & button) != 0 {
                let contact = &mut runtime.session_mut().player.contact;
                (contact.external_x.whole, contact.external_y.fraction) =
                    (if right { 4 } else { -4i16 as u16 }, 0);
            }
        }
    } else {
        activate(runtime, kind.motion());
    }
}

/// Growing shutter functions ($A2:EABD-$A2:ED32): wait for the timer or for Samus to get near,
/// then grow or shrink a growth level at a time, carrying Samus standing on it.
fn grow(runtime: &mut impl ActorLease) {
    let e = current_slot(runtime);
    let function = session_slot!(runtime, Growing[e].function);
    match function {
        0xeabd | 0xead1 | 0xeae7 | 0xeafd => {
            let ready = if function == 0xead1 || function == 0xeae7 {
                let distance = session_slot!(runtime, Growing[e].parameter_1);
                near(runtime, distance)
            } else if session_slot!(runtime, Growing[e].parameter_1) != 0 {
                session_slot!(runtime, Growing[e].parameter_1, |timer| timer
                    .wrapping_sub(1));
                false
            } else {
                true
            };
            if ready {
                movement_sound(runtime);
                let upwards = function == 0xeabd || function == 0xead1;
                session_slot!(
                    runtime,
                    Growing[e].function = if upwards { 0xec13 } else { 0xeb11 }
                );
            }
        }
        0xeb11 | 0xec13 => {
            let upwards = function == 0xec13;
            if upwards {
                let y = session_slot!(runtime, Growing[e].base.y_pos);
                session_slot!(runtime, Growing[e].previous_y = y);
            }
            let level = session_slot!(runtime, Growing[e].growth_level);
            let level = usize::from(if upwards { level & 0x7fff } else { level });
            assert!(level < 5, "timed shutter stage out of bounds");
            if level < 4 {
                let speed = session_slot!(runtime, Growing[e](speed, subspeed));
                session_slot!(
                    runtime,
                    Growing[e].base(y_pos, y_subpos) +=
                        if upwards { speed.wrapping_neg() } else { speed }
                );
                let stop = [
                    session_slot!(runtime, Growing[e].level_0_y),
                    session_slot!(runtime, Growing[e].level_1_y),
                    session_slot!(runtime, Growing[e].level_2_y),
                    session_slot!(runtime, Growing[e].level_3_y),
                ][level];
                let limit = stop.wrapping_add(if upwards { -16i16 as u16 } else { 16 });
                let past = (limit as i16)
                    .wrapping_sub(session_slot!(runtime, Growing[e].base.y_pos) as i16);
                if (upwards && past >= 0) || (!upwards && past < 0) {
                    let y = if level == 3 {
                        limit
                    } else if upwards {
                        limit.wrapping_add(7)
                    } else {
                        stop.wrapping_add(9)
                    };
                    session_slot!(runtime, Growing[e].base.y_pos = y);
                    session_slot!(runtime, Growing[e].growth_level, |old| old.wrapping_add(1));
                    if level != 3 {
                        let list = [0xe99e, 0xe9a4, 0xe9aa][level];
                        session_slot!(runtime, Growing[e].base; instruction_timer = 1, current_instruction = list,
                            y_height = [16, 24, 32][level]);
                    }
                }
            }
            let key = combat::current(runtime);
            if upwards && api::touches_player_from_above(runtime, key) {
                let moved = session_slot!(runtime, Growing[e].base.y_pos)
                    .wrapping_sub(session_slot!(runtime, Growing[e].previous_y));
                if (moved as i16) < 0 {
                    let carry = &mut runtime.session_mut().player.contact.external_y.whole;
                    *carry = carry.wrapping_add(moved);
                }
            }
        }
        _ => runtime.unreachable(),
    }
}
